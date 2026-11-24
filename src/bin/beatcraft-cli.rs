//! beatcraft-cli: run BeatCraft commands on a project file, render it, or serve them over MCP.

#[path = "../commands.rs"]
mod commands;
#[allow(dead_code)]
#[path = "../engine.rs"]
mod engine;
#[allow(dead_code)]
#[path = "../model.rs"]
mod model;
#[allow(dead_code)]
#[path = "../synth.rs"]
mod synth;

use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::path::{Component, Path, PathBuf};

const USAGE: &str = "beatcraft-cli — drive BeatCraft from scripts and AI agents

  beatcraft-cli list                                   list every command and its parameters
  beatcraft-cli run <project.json> <command> [json]    run one command (the file is created if missing)
  beatcraft-cli info <project.json>                    print a project summary
  beatcraft-cli render <project.json> <out.wav>        render the song to a WAV file
  beatcraft-cli mcp [--automation-read-root DIR]... [--automation-write-root DIR]...
                                                       serve the commands as an MCP server on stdio,
                                                       optionally confined to folders
  beatcraft-cli --version";

fn load(path: &Path, create_if_missing: bool) -> Result<model::Project, String> {
    if !path.exists() {
        return if create_if_missing { Ok(model::Project::empty()) } else { Err(format!("no such project file: {}", path.display())) };
    }
    let t = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    model::Project::from_json(&t)
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
        Some("info") if args.len() == 2 => load(Path::new(&args[1]), false).map(|p| commands::info(&p)),
        Some("run") if args.len() >= 3 => {
            let path = PathBuf::from(&args[1]);
            let params: Result<Value, String> = match args.get(3) {
                Some(s) => serde_json::from_str(s).map_err(|e| format!("bad JSON parameters: {e}")),
                None => Ok(json!({})),
            };
            params.and_then(|params| {
                let mut p = load(&path, true)?;
                let r = commands::run(&mut p, &args[2], &params)?;
                save(&path, &p)?;
                Ok(r)
            })
        }
        Some("render") if args.len() == 3 => load(Path::new(&args[1]), false).and_then(|mut p| commands::run(&mut p, "song.export_wav", &json!({"path": args[2]}))),
        Some("mcp") => {
            let roots = match parse_mcp_args(&args[1..]) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("error: {e}\n{USAGE}");
                    std::process::exit(2);
                }
            };
            mcp(roots);
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

/// Folders the MCP server may read from and write to. Without any root flag the server is unconfined.
struct Roots {
    read: Vec<PathBuf>,
    write: Vec<PathBuf>,
}

impl Roots {
    fn check(&self, path: &Path, write: bool) -> Result<PathBuf, String> {
        let (roots, kind) = if write { (&self.write, "write") } else { (&self.read, "read") };
        if roots.is_empty() {
            return Err(format!("automation filesystem access is not granted: {kind} authority is absent: {}", path.display()));
        }
        let rejected = || {
            let list: Vec<String> = roots.iter().map(|r| r.display().to_string()).collect();
            format!("automation path rejected: outside the {kind} root{} {}: {}", if roots.len() > 1 { "s" } else { "" }, list.join(", "), path.display())
        };
        let abs = std::env::current_dir().map_err(|e| e.to_string())?.join(path);
        let comps: Vec<Component> = abs.components().collect();
        let mut split = comps.len();
        let mut base = abs.clone();
        while split > 0 {
            base = comps[..split].iter().collect();
            if std::fs::symlink_metadata(&base).is_ok() {
                break;
            }
            split -= 1;
        }
        if split == 0 {
            return Err(rejected());
        }
        let mut resolved = std::fs::canonicalize(&base).map_err(|_| rejected())?;
        for c in &comps[split..] {
            match c {
                Component::Normal(n) => resolved.push(n),
                _ => return Err(rejected()),
            }
        }
        if roots.iter().any(|r| resolved.starts_with(r)) {
            Ok(resolved)
        } else {
            Err(rejected())
        }
    }
}

fn parse_mcp_args(args: &[String]) -> Result<Option<Roots>, String> {
    let mut roots = Roots { read: Vec::new(), write: Vec::new() };
    let mut confined = false;
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        let (flag, value) = match a.split_once('=') {
            Some((f, v)) => (f, Some(v.to_string())),
            None => (a.as_str(), None),
        };
        let list = match flag {
            "--automation-read-root" => &mut roots.read,
            "--automation-write-root" => &mut roots.write,
            _ => return Err(format!("unknown argument `{a}` after `mcp`")),
        };
        let dir = match value {
            Some(v) => v,
            None => {
                i += 1;
                args.get(i).cloned().ok_or_else(|| format!("`{flag}` needs a folder"))?
            }
        };
        let canon = std::fs::canonicalize(&dir).map_err(|e| format!("root `{dir}` is not usable: {e}"))?;
        if !canon.is_dir() {
            return Err(format!("root `{dir}` is not a folder"));
        }
        list.push(canon);
        confined = true;
        i += 1;
    }
    Ok(confined.then_some(roots))
}

struct Session {
    project: model::Project,
    path: Option<PathBuf>,
    roots: Option<Roots>,
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
    fn check(&self, path: &Path, write: bool) -> Result<PathBuf, String> {
        match &self.roots {
            Some(r) => r.check(path, write),
            None => Ok(path.to_path_buf()),
        }
    }
    fn guard(&self, id: &str, params: &Value) -> Result<(), String> {
        if id == "song.export_wav" {
            if let Some(p) = params.get("path").and_then(Value::as_str) {
                self.check(Path::new(p), true)?;
            }
        }
        Ok(())
    }
    fn autosave(&self) -> Result<(), String> {
        match &self.path {
            Some(p) => save(&self.check(p, true)?, &self.project),
            None => Ok(()),
        }
    }
    fn call(&mut self, name: &str, a: &Value) -> Result<Value, String> {
        match name {
            "list_commands" => Ok(commands::list_json()),
            "inspect_project" => Ok(commands::info(&self.project)),
            "execute" => {
                let id = a.get("command").and_then(Value::as_str).ok_or("missing `command`")?;
                self.guard(id, a.get("params").unwrap_or(&Value::Null))?;
                let r = commands::run(&mut self.project, id, a.get("params").unwrap_or(&Value::Null))?;
                self.autosave()?;
                Ok(r)
            }
            "batch" => {
                let steps = a.get("steps").and_then(Value::as_array).ok_or("missing `steps`")?;
                let mut results = Vec::new();
                for (i, s) in steps.iter().enumerate() {
                    let id = s.get("command").and_then(Value::as_str).ok_or(format!("step {i}: missing `command`"))?;
                    let params = s.get("params").unwrap_or(&Value::Null);
                    let r = self.guard(id, params).and_then(|()| commands::run(&mut self.project, id, params)).map_err(|e| format!("step {i} ({id}): {e}"))?;
                    results.push(r);
                }
                self.autosave()?;
                Ok(Value::Array(results))
            }
            "open_project" => {
                let p = PathBuf::from(a.get("path").and_then(Value::as_str).ok_or("missing `path`")?);
                if self.roots.is_none() {
                    self.project = load(&p, true)?;
                    self.path = Some(p);
                    return Ok(commands::info(&self.project));
                }
                if p.exists() {
                    let readable = self.check(&p, false)?;
                    let project = load(&readable, false)?;
                    let bound = self.check(&p, true);
                    self.project = project;
                    let mut info = commands::info(&self.project);
                    match bound {
                        Ok(w) => self.path = Some(w),
                        Err(_) => {
                            self.path = None;
                            info["autosave"] = json!(false);
                            info["note"] = json!("outside the write roots: changes are not saved to this file");
                        }
                    }
                    Ok(info)
                } else {
                    let w = self.check(&p, true)?;
                    self.project = model::Project::empty();
                    self.path = Some(w);
                    Ok(commands::info(&self.project))
                }
            }
            "save_project" => {
                let p = match a.get("path").and_then(Value::as_str) {
                    Some(p) => self.check(Path::new(p), true)?,
                    None => self.check(&self.path.clone().ok_or("no file yet: pass `path`")?, true)?,
                };
                self.path = Some(p.clone());
                save(&p, &self.project)?;
                Ok(json!({"saved": p}))
            }
            "export_wav" => {
                self.guard("song.export_wav", a)?;
                commands::run(&mut self.project, "song.export_wav", a)
            }
            _ => Err(format!("unknown tool `{name}`")),
        }
    }
}

fn mcp(roots: Option<Roots>) {
    let mut s = Session { project: model::Project::empty(), path: None, roots };
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
