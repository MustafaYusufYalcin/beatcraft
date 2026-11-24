# BeatCraft

BeatCraft is an open-source beat maker written in Rust. You light up steps in a channel rack,
draw melodies in a piano roll, arrange patterns into a song on a playlist and balance it all in a
mixer. Every sound is synthesised by BeatCraft itself, so there are no sample files to install.

Every change to a project is also a plain command with JSON parameters. The `beatcraft-cli`
program runs those commands from a terminal, and its MCP server lets an AI assistant build and
render a beat for you. You can use BeatCraft as an app, from the command line, or both on the same
project file.

## Status

BeatCraft is **early alpha**. Today it can:

- sequence drums with 16, 32 or 64 steps per pattern, per-step velocity and swing;
- write melodies in a piano roll;
- arrange pattern clips on a playlist, in pattern mode or song mode;
- mix on Master, 125 inserts and a Current track, with ten effect slots each (low-pass filter,
  ping-pong delay, room reverb, soft limiter);
- play eleven built-in instruments;
- save projects as `.beatcraft.json` files and render the song to a 44.1 kHz WAV file;
- run 19 commands from the CLI and an MCP server;
- run natively on macOS, Windows and Linux, and in the browser (WebAssembly).

Not yet: audio recording, sample loading, automation clips, undo, MIDI input, plugins. The app's
buttons do not go through the command layer yet, and there is no way to drive the running window
from outside.

## Pages

| Page | What is in it |
|---|---|
| [Getting Started](Getting-Started) | Build, run, window layout, shortcuts, where files are saved |
| [Your First Beat](Your-First-Beat) | The same house beat built in the app, from the terminal and as one MCP call |
| [Command Reference](Command-Reference) | All 19 commands and the `beatcraft-cli` program |
| [MCP Setup](MCP-Setup) | Connect an AI assistant to BeatCraft |
| [Project File Format](Project-File-Format) | The `.beatcraft.json` fields and what is repaired on load |
| [Architecture](Architecture) | Binaries, modules and how the window and the audio thread talk |
| [Sound Engine](Sound-Engine) | Timing, swing, gain, instruments, effects and WAV export |
| [Testing](Testing) | Running the tests, what they cover, writing an audio test |
| [Releasing](Releasing) | Version bump, packaging and the release workflows |
| [FAQ](FAQ) | Troubleshooting |

BeatCraft is dual-licensed under MIT or Apache-2.0. See the repository README for downloads and
the roadmap.
