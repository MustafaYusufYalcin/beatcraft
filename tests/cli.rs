use serde_json::{json, Value};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

const BIN: &str = env!("CARGO_BIN_EXE_beatcraft-cli");

fn temp(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("beatcraft-test-{}-{name}", std::process::id()))
}

fn cli(args: &[&str]) -> Output {
    Command::new(BIN).args(args).output().unwrap()
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

#[test]
fn list_prints_every_command() {
    let o = cli(&["list"]);
    assert!(o.status.success());
    let v: Value = serde_json::from_str(&stdout(&o)).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 19, "update the command count in README.md and the wiki");
}

#[test]
fn run_creates_a_project_and_info_reads_it() {
    let f = temp("tempo.json");
    let fs = f.to_str().unwrap();
    let o = cli(&["run", fs, "project.set_tempo", r#"{"bpm":128}"#]);
    assert!(o.status.success(), "{}", stderr(&o));
    assert!(f.exists());
    let o = cli(&["info", fs]);
    let v: Value = serde_json::from_str(&stdout(&o)).unwrap();
    std::fs::remove_file(&f).unwrap();
    assert_eq!(v["tempo"], json!(128.0));
}

#[test]
fn info_and_render_refuse_a_missing_file() {
    let f = temp("missing.json");
    let w = temp("missing.wav");
    for args in [vec!["info", f.to_str().unwrap()], vec!["render", f.to_str().unwrap(), w.to_str().unwrap()]] {
        let o = cli(&args);
        assert_eq!(o.status.code(), Some(1));
        assert!(stderr(&o).contains("no such project file"), "{}", stderr(&o));
    }
    assert!(!f.exists() && !w.exists());
}

#[test]
fn bad_input_is_reported() {
    let f = temp("bad.json");
    let fs = f.to_str().unwrap();
    let o = cli(&["run", fs, "project.rename", "{not json"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(stderr(&o).contains("bad JSON parameters"));
    let o = cli(&["run", fs, "no.such.command"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(stderr(&o).contains("unknown command"));
    let _ = std::fs::remove_file(&f);

    let o = cli(&[]);
    assert_eq!(o.status.code(), Some(2));
    assert!(stderr(&o).contains("beatcraft-cli"));

    let o = cli(&["--version"]);
    assert!(stdout(&o).contains(env!("CARGO_PKG_VERSION")));
}

#[cfg(unix)]
#[test]
fn trap_beat_example_renders() {
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/trap-beat.sh");
    let (p, w) = (temp("trap.json"), temp("trap.wav"));
    let o = Command::new("bash").args([script, p.to_str().unwrap(), w.to_str().unwrap()]).env("BEATCRAFT_CLI", BIN).output().unwrap();
    assert!(o.status.success(), "{}", stderr(&o));
    let size = std::fs::metadata(&w).unwrap().len();
    let _ = std::fs::remove_file(&p);
    let _ = std::fs::remove_file(&w);
    assert!(size > 1_000_000, "wav is {size} bytes");
}

#[test]
fn hand_broken_project_still_works() {
    let f = temp("broken.json");
    let w = temp("broken.wav");
    let (fs, ws) = (f.to_str().unwrap(), w.to_str().unwrap());
    assert!(cli(&["run", fs, "project.new"]).status.success());
    let mut v: Value = serde_json::from_str(&std::fs::read_to_string(&f).unwrap()).unwrap();
    v["mixer"] = json!([]);
    std::fs::write(&f, v.to_string()).unwrap();
    assert!(cli(&["run", fs, "clip.add", r#"{"pattern":0,"track":0,"bar":0}"#]).status.success());
    assert!(cli(&["info", fs]).status.success());
    let o = cli(&["render", fs, ws]);
    let rendered = w.exists();
    let _ = std::fs::remove_file(&f);
    let _ = std::fs::remove_file(&w);
    assert!(o.status.success() && rendered, "{}", stderr(&o));
}

fn call(id: u32, tool: &str, args: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "method": "tools/call", "params": {"name": tool, "arguments": args}})
}

fn text_of(r: &Value) -> String {
    r["result"]["content"][0]["text"].as_str().unwrap_or("").to_string()
}

#[test]
fn mcp_session() {
    let project = temp("mcp.json");
    let wav = temp("mcp.wav");
    let (ps, ws) = (project.to_str().unwrap(), wav.to_str().unwrap());
    let lines = vec![
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "t", "version": "0"}}})
            .to_string(),
        json!({"jsonrpc": "2.0", "method": "notifications/initialized"}).to_string(),
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}).to_string(),
        call(3, "execute", json!({"command": "project.set_tempo", "params": {"bpm": 90}})).to_string(),
        call(
            4,
            "batch",
            json!({"steps": [
                {"command": "step.set", "params": {"pattern": 0, "channel": 0, "step": 0}},
                {"command": "step.set", "params": {"pattern": 0, "channel": 0, "step": 99}},
            ]}),
        )
        .to_string(),
        call(5, "no_such_tool", json!({})).to_string(),
        json!({"jsonrpc": "2.0", "id": 6, "method": "no/such/method"}).to_string(),
        "this is not json".to_string(),
        call(7, "open_project", json!({"path": ps})).to_string(),
        call(8, "execute", json!({"command": "step.set", "params": {"pattern": 0, "channel": 0, "step": 5, "velocity": 0.5}})).to_string(),
        call(9, "execute", json!({"command": "clip.add", "params": {"pattern": 0, "track": 0, "bar": 0}})).to_string(),
        call(10, "export_wav", json!({"path": ws})).to_string(),
        json!({"jsonrpc": "2.0", "id": 11, "method": "ping"}).to_string(),
    ];
    let mut child = Command::new(BIN).arg("mcp").stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let mut stdin = child.stdin.take().unwrap();
    for l in &lines {
        writeln!(stdin, "{l}").unwrap();
    }
    drop(stdin);
    let out = child.wait_with_output().unwrap();
    let replies: Vec<Value> = String::from_utf8_lossy(&out.stdout).lines().map(|l| serde_json::from_str(l).unwrap()).collect();
    let by_id = |id: u32| replies.iter().find(|r| r["id"] == json!(id)).unwrap_or_else(|| panic!("no reply for id {id}"));

    assert_eq!(replies.len(), 12, "{replies:?}");
    let parse_errors: Vec<&Value> = replies.iter().filter(|r| r["id"].is_null()).collect();
    assert_eq!(parse_errors.len(), 1);
    assert_eq!(parse_errors[0]["error"]["code"], json!(-32700));

    let r = by_id(1);
    assert_eq!(r["result"]["serverInfo"]["name"], json!("beatcraft"));
    assert_eq!(r["result"]["protocolVersion"], json!("2024-11-05"));
    let mut names: Vec<&str> = by_id(2)["result"]["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap()).collect();
    names.sort_unstable();
    assert_eq!(names, ["batch", "execute", "export_wav", "inspect_project", "list_commands", "open_project", "save_project"]);
    assert!(text_of(by_id(3)).contains("90"));
    assert_eq!(by_id(4)["result"]["isError"], json!(true));
    assert!(text_of(by_id(4)).contains("step 1"));
    assert_eq!(by_id(5)["result"]["isError"], json!(true));
    assert_eq!(by_id(6)["error"]["code"], json!(-32601));
    assert_eq!(by_id(8)["result"]["isError"], json!(false));
    assert_eq!(by_id(10)["result"]["isError"], json!(false), "{}", text_of(by_id(10)));
    assert_eq!(by_id(11)["result"], json!({}));

    let saved: Value = serde_json::from_str(&std::fs::read_to_string(&project).unwrap()).unwrap();
    let wav_exists = wav.exists();
    let _ = std::fs::remove_file(&project);
    let _ = std::fs::remove_file(&wav);
    assert_eq!(saved["patterns"][0]["data"][0]["steps"][5], json!(0.5));
    assert!(wav_exists);
}

fn mcp_run(args: &[&str], lines: &[Value]) -> Vec<Value> {
    let mut child = Command::new(BIN).arg("mcp").args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let mut stdin = child.stdin.take().unwrap();
    for l in lines {
        writeln!(stdin, "{l}").unwrap();
    }
    drop(stdin);
    let out = child.wait_with_output().unwrap();
    String::from_utf8_lossy(&out.stdout).lines().map(|l| serde_json::from_str(l).unwrap()).collect()
}

fn reply(replies: &[Value], id: u32) -> &Value {
    replies.iter().find(|r| r["id"] == json!(id)).unwrap_or_else(|| panic!("no reply for id {id}"))
}

fn is_error(r: &Value) -> bool {
    r["result"]["isError"] == json!(true)
}

fn temp_dir(name: &str) -> PathBuf {
    let d = temp(name);
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn confined_mcp_server_stays_inside_its_roots() {
    let root = temp_dir("root");
    let outside = temp_dir("outside");
    let inside_file = root.join("a.beatcraft.json");
    let saved = root.join("b.beatcraft.json");
    let wav = root.join("a.wav");
    let existing_outside = outside.join("existing.beatcraft.json");
    assert!(cli(&["run", existing_outside.to_str().unwrap(), "project.rename", r#"{"name":"x"}"#]).status.success());
    let (out_save, out_wav, out_wav2, out_wav3) = (outside.join("s.json"), outside.join("e.wav"), outside.join("x.wav"), outside.join("y.wav"));
    let p = |f: &PathBuf| f.to_str().unwrap().to_string();
    let lines = [
        call(1, "open_project", json!({"path": p(&inside_file)})),
        call(2, "execute", json!({"command": "step.set", "params": {"pattern": 0, "channel": 0, "step": 2}})),
        call(3, "save_project", json!({"path": p(&saved)})),
        call(4, "execute", json!({"command": "clip.add", "params": {"pattern": 0, "track": 0, "bar": 0}})),
        call(5, "export_wav", json!({"path": p(&wav)})),
        call(6, "open_project", json!({"path": p(&existing_outside)})),
        call(7, "save_project", json!({"path": p(&out_save)})),
        call(8, "export_wav", json!({"path": p(&out_wav)})),
        call(9, "execute", json!({"command": "song.export_wav", "params": {"path": p(&out_wav2)}})),
        call(10, "batch", json!({"steps": [{"command": "song.export_wav", "params": {"path": p(&out_wav3)}}]})),
    ];
    let args = ["--automation-read-root", root.to_str().unwrap(), "--automation-write-root", root.to_str().unwrap()];
    let r = mcp_run(&args, &lines);
    for id in 1..=5 {
        assert!(!is_error(reply(&r, id)), "id {id}: {}", text_of(reply(&r, id)));
    }
    for id in 6..=10 {
        assert!(is_error(reply(&r, id)), "id {id} should be refused");
        assert!(text_of(reply(&r, id)).contains("automation path rejected"), "id {id}: {}", text_of(reply(&r, id)));
    }
    assert!(inside_file.exists() && saved.exists() && wav.exists());
    assert!(!out_save.exists() && !out_wav.exists() && !out_wav2.exists() && !out_wav3.exists());
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&outside);
}

#[test]
fn confined_server_refuses_dotdot_and_symlink_escapes() {
    let root = temp_dir("root2");
    let outside = temp_dir("outside2");
    let dotdot = root.join("missing").join("..").join("..").join(format!("beatcraft-test-{}-escape.wav", std::process::id()));
    let mut lines =
        vec![call(1, "execute", json!({"command": "clip.add", "params": {"pattern": 0, "track": 0, "bar": 0}})), call(2, "export_wav", json!({"path": dotdot.to_str().unwrap()}))];
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&outside, root.join("link")).unwrap();
        lines.push(call(3, "export_wav", json!({"path": root.join("link").join("e.wav").to_str().unwrap()})));
    }
    let args = ["--automation-write-root", root.to_str().unwrap()];
    let r = mcp_run(&args, &lines);
    assert!(is_error(reply(&r, 2)));
    assert!(text_of(reply(&r, 2)).contains("automation path rejected"), "{}", text_of(reply(&r, 2)));
    assert!(!root.parent().unwrap().join(dotdot.file_name().unwrap()).exists());
    #[cfg(unix)]
    {
        assert!(is_error(reply(&r, 3)));
        assert!(text_of(reply(&r, 3)).contains("automation path rejected"));
        assert!(!outside.join("e.wav").exists());
    }
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&outside);
}

#[test]
fn read_only_root_opens_without_autosave() {
    let root = temp_dir("readonly");
    let file = root.join("p.beatcraft.json");
    let fs = file.to_str().unwrap();
    assert!(cli(&["run", fs, "project.rename", r#"{"name":"keep me"}"#]).status.success());
    let before = std::fs::read(&file).unwrap();
    let wav = root.join("o.wav");
    let lines = [
        call(1, "open_project", json!({"path": fs})),
        call(2, "execute", json!({"command": "project.set_tempo", "params": {"bpm": 99}})),
        call(3, "execute", json!({"command": "clip.add", "params": {"pattern": 0, "track": 0, "bar": 0}})),
        call(4, "export_wav", json!({"path": wav.to_str().unwrap()})),
    ];
    let r = mcp_run(&["--automation-read-root", root.to_str().unwrap()], &lines);
    let opened: Value = serde_json::from_str(&text_of(reply(&r, 1))).unwrap();
    assert_eq!(opened["autosave"], json!(false));
    assert!(opened["note"].as_str().unwrap().contains("outside the write roots"));
    assert!(!is_error(reply(&r, 2)));
    assert!(is_error(reply(&r, 4)));
    assert!(text_of(reply(&r, 4)).contains("write authority is absent"), "{}", text_of(reply(&r, 4)));
    assert_eq!(std::fs::read(&file).unwrap(), before);
    assert!(!wav.exists());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn bad_mcp_arguments_exit_with_usage_error() {
    for args in [vec!["mcp", "--automation-raed-root", "x"], vec!["mcp", "--automation-read-root", "/definitely/missing"], vec!["mcp", "stray"]] {
        let o = Command::new(BIN).args(&args).stdin(Stdio::null()).output().unwrap();
        assert_eq!(o.status.code(), Some(2), "{args:?}");
        assert!(o.stdout.is_empty(), "{args:?}");
        assert!(!stderr(&o).is_empty());
    }
}
