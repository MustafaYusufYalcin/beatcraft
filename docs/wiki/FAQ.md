# FAQ

## I hear nothing

- **The transport says "NO AUDIO".** BeatCraft could not open an output device: there is no default
  output device, or it does not offer 32-bit float samples, or it could not be started. The app
  still runs (the playhead moves and the meters work) but stays silent. Check that your system has a
  working default output device, then restart BeatCraft. On Linux the ALSA library is needed.
- **It plays but there is no sound in the browser.** Browsers keep audio suspended until you
  interact with the page. Click the page once, then press play.
- **Song mode is silent.** Song mode plays the playlist, and a project made with FILE, New has no
  clips. Press **L** to play the current pattern, or place a pattern on the playlist (see
  [Your First Beat](Your-First-Beat)).
- **A channel is silent.** Check its mute light in the rack, the mute and solo buttons of its mixer
  track (if any track is soloed, all others are silent), and the Master mute.

## An effect knob does nothing

The CUTOFF, DELAY and REVERB knobs of a mixer track only work while the matching effect is loaded
in one of the track's ten slots. In the app they are greyed out until you load it (choose it in a
slot, or click it in the browser's Effects list). Over the CLI use `mixer.load_fx`, then `mixer.set`.
The soft limiter only does something on Master.

## Files

- **Where are my projects and WAV files?** In the `BeatCraft` folder inside your Music folder:
  `~/Music/BeatCraft` (macOS, Linux) or `Music\BeatCraft` under `%USERPROFILE%` (Windows).
  Files made with the CLI or MCP are wherever you pointed them.
- **"Could not open that file".** The file could not be read, is not JSON, or is not a BeatCraft
  project (a field is missing). Run `beatcraft-cli info that-file.json`: it prints the reason, such
  as ``not a BeatCraft project: missing field `tempo` ``. Out-of-range values are repaired on load,
  see [Project File Format](Project-File-Format).
- **`info` and `render` say "no such project file".** They never create a file. Check the path and
  the folder you are in. Only `run` creates a missing file.
- **The exported WAV is silent or has the wrong music.** Export plays the playlist, not the current
  pattern. With no clips the app refuses to export and says "The playlist is empty: add clips first"
  in the status line, like the CLI.
- **The web build cannot save or export.** The browser has no folder to write to, so you get "Could
  not save the project" or "Export failed". Saving in the browser is on the roadmap.
- **The app will not let me rename the project.** There is no rename control yet. Use the
  `project.rename` command or edit `name` in the file.

## First start

- **macOS says the app can't be checked.** BeatCraft is not notarised. Right-click the app and
  choose Open.
- **Windows SmartScreen warns.** The build is not signed. Choose More info, then Run anyway.
- You can check what you downloaded against `SHA256SUMS.txt` (see [Releasing](Releasing)).

## Commands and MCP

- **A note or step command says something "doesn't exist".** Indexes start at 0 and the step must
  be inside the pattern (0 to 15 for a 16-step pattern). See the
  [Command Reference](Command-Reference).
- **`note.add` says the note must end before bar 1000.** Songs and notes stop at bar 1000.
- **My MCP client does not show the tools.** Run `beatcraft-cli mcp` by hand as in
  [MCP Setup](MCP-Setup), use an absolute path to the program in the client's configuration, and
  restart the client.
- **A batch failed halfway.** The steps before the failing one are applied in memory but the file was
  not saved; see [MCP Setup](MCP-Setup).
- **My changes through the CLI do not show in the open window.** The window does not watch files.
  Open the file again (start BeatCraft with the file name, or FILE, Open saved).
