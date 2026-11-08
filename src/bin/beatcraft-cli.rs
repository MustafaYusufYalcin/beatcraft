//! beatcraft-cli: run BeatCraft commands on a project file, render it, or serve them over MCP.

#[allow(dead_code)]
#[path = "../model.rs"]
mod model;
#[allow(dead_code)]
#[path = "../synth.rs"]
mod synth;
#[allow(dead_code)]
#[path = "../engine.rs"]
mod engine;
#[path = "../commands.rs"]
mod commands;

use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

const USAGE: &str = "beatcraft-cli — drive BeatCraft from scripts and AI agents

  beatcraft-cli list                                   list every command and its parameters
  beatcraft-cli run <project.json> <command> [json]    run one command (the file is created if missing)
  beatcraft-cli info <project.json>                    print a project summary
  beatcraft-cli render <project.json> <out.wav>        render the song to a WAV file
  beatcraft-cli mcp                                    serve the commands as an MCP server on stdio
  beatcraft-cli --version";

fn load(path: &Path) -> Result<model::Project, String> {
    if !path.exists() {
        return Ok(model::Project::empty());
    }
    let t = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&t).map_err(|e| format!("not a BeatCraft project: {e}"))
}

fn save(path: &Path, p: &model::Project) -> Result<(), String> {
    let t = serde_json::to_string_pretty(p).map_err(|e| e.to_string())?;
    std::fs::write(path, t).map_err(|e| e.to_string())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let out = match args.first().map(String::as_str) {
        Some("list") => Ok(commands::list_json()),
        Some("--version") => {
            println!("beatcraft-cli {}", env!("CARGO_PKG_VERSION"));
            return;
        }
        Some("info") if args.len() == 2 => load(Path::new(&args[1])).map(|p| commands::info(&p)),
        Some("run") if args.len() >= 3 => {
            let path = PathBuf::from(&args[1]);
            let params: Result<Value, String> = match args.get(3) {
                Some(s) => serde_json::from_str(s).map_err(|e| format!("bad JSON parameters: {e}")),
                None => Ok(json!({})),
            };
            params.and_then(|params| {
                let mut p = load(&path)?;
                let r = commands::run(&mut p, &args[2], &params)?;
                save(&path, &p)?;
                Ok(r)
            })
        }
        Some("render") if args.len() == 3 => load(Path::new(&args[1])).and_then(|mut p| commands::run(&mut p, "song.export_wav", &json!({"path": args[2]}))),
        Some("mcp") => {
            mcp();
            return;
        }
        _ => {
            eprintln!("{USAGE}");
            std::process::exit(2);
        }
    };
    match out {
        Ok(v) => println!("{}", serde_json::to_string_pretty(&v).unwrap()),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}

// ------------------------------------------------------------------ MCP (JSON-RPC 2.0 over stdio)

struct Session {
    project: model::Project,
    path: Option<PathBuf>,
}

fn tools() -> Value {
    let obj = |props: Value, req: &[&str]| json!({"type": "object", "properties": props, "required": req});
    json!([
        {"name": "list_commands", "description": "Every BeatCraft command with its parameters.", "inputSchema": obj(json!({}), &[])},
        {"name": "execute", "description": "Run one command (ids from list_commands) on the open project. The project is saved after each change when it has a file.",
         "inputSchema": obj(json!({"command": {"type": "string"}, "params": {"type": "object"}}), &["command"])},
        {"name": "batch", "description": "Run several commands in order; stops at the first error.",
         "inputSchema": obj(json!({"steps": {"type": "array", "items": {"type": "object", "properties": {"command": {"type": "string"}, "params": {"type": "object"}}, "required": ["command"]}}}), &["steps"])},
        {"name": "inspect_project", "description": "Summary of the open project: tempo, channels, patterns, clips.", "inputSchema": obj(json!({}), &[])},
        {"name": "open_project", "description": "Open (or create) a project file; later changes are saved to it.", "inputSchema": obj(json!({"path": {"type": "string"}}), &["path"])},
        {"name": "save_project", "description": "Save the open project, optionally to a new path.", "inputSchema": obj(json!({"path": {"type": "string"}}), &[])},
        {"name": "export_wav", "description": "Render the song to a WAV file.", "inputSchema": obj(json!({"path": {"type": "string"}}), &["path"])}
    ])
}

impl Session {
    fn autosave(&self) -> Result<(), String> {
        match &self.path {
            Some(p) => save(p, &self.project),
            None => Ok(()),
        }
    }
    fn call(&mut self, name: &str, a: &Value) -> Result<Value, String> {
        match name {
            "list_commands" => Ok(commands::list_json()),
            "inspect_project" => Ok(commands::info(&self.project)),
            "execute" => {
                let id = a.get("command").and_then(Value::as_str).ok_or("missing `command`")?;
                let r = commands::run(&mut self.project, id, a.get("params").unwrap_or(&Value::Null))?;
                self.autosave()?;
                Ok(r)
            }
            "batch" => {
                let steps = a.get("steps").and_then(Value::as_array).ok_or("missing `steps`")?;
                let mut results = Vec::new();
                for (i, s) in steps.iter().enumerate() {
                    let id = s.get("command").and_then(Value::as_str).ok_or(format!("step {i}: missing `command`"))?;
                    let r = commands::run(&mut self.project, id, s.get("params").unwrap_or(&Value::Null)).map_err(|e| format!("step {i} ({id}): {e}"))?;
                    results.push(r);
                }
                self.autosave()?;
                Ok(Value::Array(results))
            }
            "open_project" => {
                let p = PathBuf::from(a.get("path").and_then(Value::as_str).ok_or("missing `path`")?);
                self.project = load(&p)?;
                self.path = Some(p);
                Ok(commands::info(&self.project))
            }
            "save_project" => {
                if let Some(p) = a.get("path").and_then(Value::as_str) {
                    self.path = Some(PathBuf::from(p));
                }
                let p = self.path.clone().ok_or("no file yet: pass `path`")?;
                save(&p, &self.project)?;
                Ok(json!({"saved": p}))
            }
            "export_wav" => commands::run(&mut self.project, "song.export_wav", a),
            _ => Err(format!("unknown tool `{name}`")),
        }
    }
}

fn mcp() {
    let mut s = Session { project: model::Project::empty(), path: None };
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let Ok(msg) = serde_json::from_str::<Value>(&line) else {
            let _ = writeln!(stdout, "{}", json!({"jsonrpc": "2.0", "id": null, "error": {"code": -32700, "message": "parse error"}}));
            continue;
        };
        let Some(id) = msg.get("id").cloned() else { continue }; // notifications get no reply
        let method = msg.get("method").and_then(Value::as_str).unwrap_or("");
        let params = msg.get("params").cloned().unwrap_or(json!({}));
        let reply = match method {
            "initialize" => json!({"result": {
                "protocolVersion": params.get("protocolVersion").cloned().unwrap_or(json!("2025-06-18")),
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "beatcraft", "version": env!("CARGO_PKG_VERSION")},
                "instructions": "BeatCraft is a pattern-based beat maker. Steps are 1/16 notes (16 per bar). Typical flow: open_project or execute project.new, add channels (channel.add), light steps (step.set / step.fill) and add notes (note.add, key 60 = C5), place patterns on the playlist (clip.add), then export_wav. Use batch for several changes."
            }}),
            "ping" => json!({"result": {}}),
            "tools/list" => json!({"result": {"tools": tools()}}),
            "tools/call" => {
                let name = params.get("name").and_then(Value::as_str).unwrap_or("");
                let args = params.get("arguments").cloned().unwrap_or(json!({}));
                match s.call(name, &args) {
                    Ok(v) => json!({"result": {"content": [{"type": "text", "text": serde_json::to_string_pretty(&v).unwrap()}], "isError": false}}),
                    Err(e) => json!({"result": {"content": [{"type": "text", "text": e}], "isError": true}}),
                }
            }
            _ => json!({"error": {"code": -32601, "message": format!("method not found: {method}")}}),
        };
        let mut r = reply;
        r["jsonrpc"] = json!("2.0");
        r["id"] = id;
        let _ = writeln!(stdout, "{r}");
        let _ = stdout.flush();
    }
}
