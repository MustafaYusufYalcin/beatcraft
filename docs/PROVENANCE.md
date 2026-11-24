# Where BeatCraft's rules and numbers come from

BeatCraft's code, sounds, icon and demo song are our own work. This page records where its design
numbers come from, including something we did at the start and have since undone.

## Our own design

These rules are ours and are explained in the wiki's [Sound Engine](wiki/Sound-Engine.md) page:

- the cubic volume taper, the default channel level and the top of the mixer fader;
- swing in whole percent, delaying off-beat steps by up to half a step;
- the tempo range (20 to 400 BPM), tap tempo and the tempo read back from an export;
- the 1000-bar song limit;
- every instrument and effect in `src/synth.rs`, the icon in `src/icon.rs` and the demo song.

## Conventions we share with other pattern-based sequencers

Some limits follow what a widely used commercial pattern-based DAW documents in its public manual, so
that people who know that way of working find what they expect:

| Convention | Public source |
|---|---|
| 96 ticks per quarter note | [Project settings, "Timebase (PPQ)"](https://www.image-line.com/fl-studio-learning-content/fl-studio-online-manual/html/songsettings_settings.htm) |
| Up to 999 patterns | [Getting Started manual, "The Playlist"](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/boxinstaller/WebContent/FLStudio20_GettingStartedManual.pdf) |
| A mixer of one master, 125 insert tracks and one "current" track | [Mixer](https://www.image-line.com/fl-studio-learning-content/fl-studio-online-manual/html/mixer.htm) and [What's new](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/basics_new_25.htm) ("up from 125") |
| 10 effect slots per mixer track | [Mixer, "Effects slots"](https://www.image-line.com/fl-studio-learning-content/fl-studio-online-manual/html/mixer.htm) |

The following also match that program. We have not looked for a public source for each of them, and
we kept them so that projects saved with 0.1.0 still open and play in the same place: 500 playlist
tracks, 0 dB at 80 % of the fader's travel, the 140 BPM default tempo, the name C5 for MIDI key 60,
and the four drums of an empty project.

BeatCraft is not affiliated with that program or its publisher, and uses none of its code, sounds,
presets, artwork or text.

## What changed in 0.2.0

Before 0.2.0, BeatCraft used a volume-curve formula, several parameter and timing values, and a
tap-tempo error message obtained by inspecting a commercial DAW's binary and bundled project
templates. In 0.2.0, we replaced the formula, values and message with our own.

We first learned the conventions in the section above the same way. We kept only those listed there,
and nothing in the code depends on that program's files. No code was copied at any point.

If you find something in BeatCraft that looks taken from another product, please open an issue.
