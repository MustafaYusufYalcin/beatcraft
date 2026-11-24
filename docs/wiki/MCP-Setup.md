# MCP Setup

`beatcraft-cli mcp` is a [Model Context Protocol](https://modelcontextprotocol.io) server. It lets
an AI assistant build, inspect and render BeatCraft projects with the same commands as the
[Command Reference](Command-Reference).

## What the server is

- **Transport:** JSON-RPC 2.0 over stdio. The client writes one JSON object per line to the
  server's stdin and reads one JSON object per line from its stdout. Blank lines are skipped. The
  server exits when stdin closes. It writes nothing to stderr.
- **Methods:** `initialize`, `ping`, `tools/list` and `tools/call`. The only capability it
  announces is `tools`. Any other method gets the error `-32601`.
- **Notifications:** a message without an `id` (such as `notifications/initialized`) gets no reply.
- **`initialize`:** the reply echoes the client's `protocolVersion` (or `2025-06-18` if it sent
  none), names the server `beatcraft` with the crate version, and includes short usage
  instructions for the assistant.
- **Session:** the server holds one project in memory. It starts as an empty project with no
  file. Nothing is written to disk until `open_project` or `save_project` gives the session a file.

## Tools

Tool results are `{"content": [{"type": "text", "text": …}], "isError": false}`. The text is the
JSON result, pretty-printed. When a command or tool fails, the result has `"isError": true` and the
text is the plain error message. JSON-RPC errors are only used for protocol problems:

| Code | When | `id` |
|---|---|---|
| `-32700` | A line is not valid JSON (`parse error`) | `null` |
| `-32601` | The method is not supported (`method not found: x`) | the request's |

| Tool | Input | What it does |
|---|---|---|
| `list_commands` | none | The 19 commands with their parameters and help text |
| `execute` | `command` (string, required), `params` (object) | Runs one command on the open project and returns its result. If the session has a file, the project is saved after the command succeeds |
| `batch` | `steps` (array of `{command, params}`, required) | Runs the steps in order and returns an array with each result. Saves once at the end if the session has a file |
| `inspect_project` | none | The `project.info` summary of the open project |
| `open_project` | `path` (string, required) | Loads the project file, or starts an empty project if the file does not exist yet, and binds the session to that path. Returns the summary |
| `save_project` | `path` (string, optional) | Saves the project. With `path` the session is bound to the new path first. Returns `{"saved": path}` |
| `export_wav` | `path` (string, required) | Renders the song to a WAV file, like `song.export_wav`. It needs at least one clip on the playlist |

The input schemas are the usual JSON Schema objects, for example:

```json
{"type": "object", "properties": {"command": {"type": "string"}, "params": {"type": "object"}}, "required": ["command"]}
```

Tool-specific errors: `unknown tool `x``, ``missing `command` ``, ``missing `steps` ``,
``missing `path` ``, `no file yet: pass `path`` (from `save_project`) and
`not a BeatCraft project: …` (from `open_project` on a file that is not a project).

### Saving, and what a failed batch leaves behind

- `open_project` binds the session to a file. From then on every successful `execute` and `batch`
  saves the whole project to that file.
- `export_wav` does not save the project.
- `execute project.new` replaces the project in memory, so on a bound session it overwrites the
  file with a new empty (or demo) project.
- A `batch` stops at the first step that fails and returns `isError: true` with the text
  `step N (command): message`, where N counts from 0 (`step 1` is the second step). **The steps
  before the failing one stay applied in memory, but that call does not save, so the file on disk
  does not change.** The next successful `execute`, `batch` or `save_project` writes the project
  including those earlier steps. If you need all-or-nothing behaviour, check the parameters first
  or use `inspect_project` after a failure to see what was applied.

## Connect a client

Use an absolute path to `beatcraft-cli` (for example `/home/you/beatcraft/target/release/beatcraft-cli`)
in place of `beatcraft-cli` if it is not on your `PATH`. Where each client keeps its configuration
can change between versions, so the client's own documentation wins over the locations below.

**Claude Code**

```sh
claude mcp add beatcraft -- beatcraft-cli mcp
```

**Claude Desktop** (`claude_desktop_config.json`) and **Cursor** (`~/.cursor/mcp.json`, or
`.cursor/mcp.json` inside a project):

```json
{"mcpServers": {"beatcraft": {"command": "beatcraft-cli", "args": ["mcp"]}}}
```

**Codex CLI** (`~/.codex/config.toml`):

```toml
[mcp_servers.beatcraft]
command = "beatcraft-cli"
args = ["mcp"]
```

**VS Code** (`.vscode/mcp.json`):

```json
{"servers": {"beatcraft": {"type": "stdio", "command": "beatcraft-cli", "args": ["mcp"]}}}
```

Restart the client after changing its configuration, then ask it to list its tools. You should see
the seven BeatCraft tools.

## Talk to it by hand

You can try the server without a client. Save these lines as `hs.in`:

```
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"t","version":"0"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/list"}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"execute","arguments":{"command":"project.set_tempo","params":{"bpm":90}}}}
not json
{"jsonrpc":"2.0","id":4,"method":"nope"}
{"jsonrpc":"2.0","id":5,"method":"ping"}
```

and run `beatcraft-cli mcp < hs.in`. There are six replies (the notification gets none). Long
lines are cut here with `…`:

```
{"id":1,"jsonrpc":"2.0","result":{"capabilities":{"tools":{}},"instructions":"BeatCraft is a pattern-based beat maker. Steps are 1/16 notes (16 per bar). Typical flow: open_project or execute project.new, add channels (channel.add), light steps (step.set / step.fill) and add notes (note.add, key 60 = C5), place patterns on the p …
{"id":2,"jsonrpc":"2.0","result":{"tools":[{"description":"Every BeatCraft command with its parameters.","inputSchema":{"properties":{},"required":[],"type":"object"},"name":"list_commands"},{"description":"Run one command (ids from list_commands) on the open project. The project is saved after each change when it has a file."," …
{"id":3,"jsonrpc":"2.0","result":{"content":[{"text":"{\n  \"tempo\": 90.0\n}","type":"text"}],"isError":false}}
{"error":{"code":-32700,"message":"parse error"},"id":null,"jsonrpc":"2.0"}
{"error":{"code":-32601,"message":"method not found: nope"},"id":4,"jsonrpc":"2.0"}
{"id":5,"jsonrpc":"2.0","result":{}}
```

## Example prompts

- "Open `~/beats/night.beatcraft.json` in BeatCraft and make a 140 BPM trap beat: kick and snare,
  a busy hi-hat, a deep sub bass and a pad. Export it as `night.wav` and tell me the tempo
  BeatCraft measured."
- "Look at my current BeatCraft project and describe what is in it. Then add a second pattern with
  a half-time version of the drums and put it on the playlist after bar 8."
- "Turn the reverb up on my snare and add a delay to the lead. Remember the effect has to be
  loaded in a slot before its amount makes a sound."

## Confining file access

By default the server is **unconfined**: it reads and writes any path the client names, with the
permissions of the user who started it. To confine it, pass root folders when you start it:

```sh
beatcraft-cli mcp --automation-read-root ~/beats --automation-write-root ~/beats
```

Each flag may be repeated, and `--flag DIR` and `--flag=DIR` both work. Roots must be existing
folders; a missing folder, an unknown argument after `mcp` (including a misspelt flag) or a flag
without a folder prints an error on stderr and exits with code 2 without starting the server.

Once at least one root flag is given the server is confined:

- Reading a file needs a read root and writing a file needs a write root. A kind with no root grants
  none of that access: the error is `automation filesystem access is not granted: write authority is
  absent: <path>` (or `read authority`).
- A path is made absolute, links are followed, and the result must lie inside a root, compared by
  whole folder names (`/work2/x` is not inside `/work`). The part of a path that does not exist yet
  may only contain plain names, so `..` below a missing folder is refused. A refused path gives the
  tool error `automation path rejected: outside the write root <root>: <path>`.
- The rule is applied to `open_project`, `save_project`, every autosave, the `export_wav` tool, and
  `song.export_wav` whether sent through `execute` or as a `batch` step.
- `open_project` on an existing file needs the read check; on a new file it needs the write check.
  If a file passes the read check but not the write check, the project opens but is not saved:
  the result has `"autosave": false` and a `note` saying so.

Example client configuration with both flags:

```json
{"mcpServers": {"beatcraft": {"command": "beatcraft-cli", "args": ["mcp", "--automation-read-root", "/home/you/beats", "--automation-write-root", "/home/you/beats"]}}}
```

Not covered: commands that take no path still change the open project in memory, and the desktop app
is not confined. The one-shot subcommands (`run`, `info`, `render`) are not affected by these flags.

## Limits

- **Unconfined unless you pass the root flags.** Without `--automation-read-root` and
  `--automation-write-root` the server reads and writes any path the client asks for, with the
  permissions of the user who started it. Use the flags, and only connect clients you trust.
- **No undo.** Changes to a bound file are saved immediately and cannot be taken back, except by
  your own backups.
- **No control of the running window.** The server edits project files; it cannot drive an open
  BeatCraft window yet, and the window does not reload a file that changes underneath it.
- **One project per session**, no progress reports while a long song renders, and no resources or
  prompts, only tools.
