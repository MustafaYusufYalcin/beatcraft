# Sound Engine

This page describes what BeatCraft does with a project to turn it into sound. The code is in
`src/engine.rs` (sequencer, mixer, export), `src/synth.rs` (voices and effects) and `src/model.rs`
(the timing and gain rules).

## Timing

Time inside a project is counted in ticks: 96 per quarter note, 24 per 1/16 step, 384 per bar.

```
samples per tick = sample rate * 15 / (tempo * 24)
```

At 44.1 kHz and 140 BPM that is 196.875 samples per tick, 4725 per step and 75600 per bar. The
tempo is clamped to 20 to 400 BPM before it is used.

The engine renders in chunks of **64 frames**. For each chunk it works out which ticks the chunk
covers and starts every note or step that falls inside it at the beginning of the chunk. A sound can
therefore start up to 64 frames (about 1.5 ms at 44.1 kHz) early, and never late.

A lit step plays for one step (24 ticks); a note plays for its length. Drums ignore the length and
end by themselves.

### Swing

The main swing is stored in whole percent (0.18 is 18%). Every odd 1/16 step (the 2nd, 4th, 6th, … of the pattern)
is delayed by

```
swing * half a step   (swing 1.0 = 12 ticks)
```

Piano-roll notes are delayed the same way if they start exactly on an odd step. Notes that start
between steps are not moved.

### Pattern mode and song mode

- **Pattern mode** loops the current pattern. Its length is its step count or the end of its last
  note, whichever is longer, rounded up to whole beats and at least one bar.
- **Song mode** plays every clip on the playlist and loops at the end of the last clip, rounded up
  to whole bars (at least one bar). Tracks make no difference to the sound: all clips play.
- A clip **loops** its pattern while the clip is longer than the pattern, and **cuts** it when the
  clip is shorter: anything that would start at or after the clip's end is not played.

## Gain

### Volume taper

Channel volumes and mixer faders share one cubic taper from a knob position `x` to a linear gain:

```
gain(x) = x^3          (0 for x <= 0)
```

A cube gives fine control at low levels: `gain(1) = 1` exactly (0 dB) and `gain(0.5)` is 0.125 (-18 dB).

- **Channel volume** `v` (0 to 1): `gain(v)`. The default is 0.8, which is about -5.8 dB.
- **Mixer fader** `v` (0 to 1): `gain(v / 0.8)`, so **0.8 is 0 dB** and the top of the fader (1.0) is
  `1.25^3`, about +5.8 dB of headroom. Below a gain of 0.00001 the fader reads minus infinity.

### Pan law

Pan `p` from -1 to 1 uses a constant-power law:

```
angle = (p + 1) * pi / 4
left = cos(angle)    right = sin(angle)
```

Centre is 0.7071 on each side, hard left is (1, 0), and left squared plus right squared is always 1.
Inserts and Master multiply the result by the square root of 2, so a centred insert passes the
signal at unity gain.

### The chain

```
voice -> channel volume * 0.8 -> channel pan -> insert bus
insert bus -> insert effects -> fader -> pan * sqrt(2) -> Master bus     (if not muted / soloed out)
Master bus -> Master effects -> Master fader * master VOL knob -> pan * sqrt(2) -> mute -> limiter or clip -> output
```

- The fixed factor **0.8** after the channel volume is headroom, so a few channels at their
  default volume do not clip the Master.
- A channel's `insert` chooses its bus (0 plays straight into Master).
- An insert is silent if it is muted, or if any insert is soloed and it is not. Solo on Master
  does nothing.
- Master has two volume controls that multiply: its fader in the mixer and the VOL knob in the
  transport (the project's `master_volume`). Both are 0 dB at 0.8.
- Each channel's `pitch` plus the project's `master_pitch` transposes melodic instruments and the
  tom, in semitones.

### Limiter and clipping

If Master has the **soft limiter** in one of its slots (it does by default), the final signal goes
through `tanh`, which rounds off peaks smoothly. Without it the signal is cut hard at -1 and 1. The
limiter has no effect when loaded on any track other than Master.

## Voices

- Every note or step starts a voice. When about **160** voices are sounding, the oldest one is
  dropped as each new one starts.
- **Drum choke:** a new hit on a drum channel ends the previous hit of the same channel, like a
  single sample that retriggers. A closed hat also ends any open-hat voice, on any channel.
- Melodic instruments are not choked. They use an envelope: attack is linear, decay and release
  are exponential, and a voice ends when its level falls below 0.0005.
- Voices that play from the computer keyboard are held until you let go of the key.
- Noise comes from a small fixed-seed generator, so rendering the same project twice gives the same
  audio.

### Instruments

| Instrument | How it is made |
|---|---|
| Kick | A sine whose pitch falls quickly from a high start, with a short click of noise, saturated with `tanh`; ends after 0.9 s |
| Snare | A sine body plus high-passed noise, both decaying; ends after 0.6 s |
| Clap | High-passed and low-passed noise with a few quick re-attacks in the first 30 ms, then a decay; ends after 0.6 s |
| Hat | High-passed noise with a very fast decay; ends after 1.2 s at most |
| Open hat | The same noise with a slow decay |
| Tom | A sine whose pitch falls slightly, decaying; follows the key and channel pitch |
| Sub bass | A sine an octave below the note plus a little band-limited saw, through a low-pass that closes quickly after the start |
| Saw lead | Three slightly detuned band-limited saws through a low-pass |
| Warm pad | The same three saws, more detuned, with a 0.45 s attack and long release |
| Pluck | A band-limited square and saw mix with a fast filter sweep and no sustain |
| E-keys | Two-operator FM: a modulator at twice the note's frequency with a decaying amount |

The `tone` setting changes the character of the drums and the detune of the saws. The `cutoff`,
`reso` and envelope settings shape the melodic instruments.

## Effects

Each mixer track runs its effects in this order: low-pass, then delay, then reverb. An effect works
only while it is loaded in a slot of that track; its amount is the track's `lowpass`, `delay` or
`reverb` value.

| Effect | What it does |
|---|---|
| Low-pass filter | A state-variable filter with a cutoff of `60 Hz * 2^(8.5 * amount)` (limited to 22% of the sample rate) and a little resonance. Amount 1 is open |
| Echo delay | A ping-pong delay with a feedback of 0.45. The delay time is a **dotted eighth note at the project tempo** (0.75 beats), at most 2 seconds. The amount is how loud the echoes are |
| Room reverb | A small reverb made of six comb filters and two all-pass filters. The amount is the wet level |
| Soft limiter | Only on Master; see above |

## WAV export

`song.export_wav`, `beatcraft-cli render` and FILE, Export song to WAV all call the same code:

- The song is played **once** from the start in song mode, without looping, followed by a
  **2-second tail** so reverbs and echoes can die away. Nothing restarts at the end of the song.
- The file is 44.1 kHz, 16-bit, stereo PCM with a standard 44-byte header, rendered offline in
  512-frame blocks (faster than real time).
- The length is the song length (whole bars up to the end of the last clip) plus 2 seconds, and the
  result's `seconds` is that length.
- `measured_bpm` is the project's tempo measured back from the number of rendered frames, rounded to
  a whole number. It is `null` when that is outside 20 to 400, or when there is nothing to
  measure, so a 128.4 BPM project reports 128.0.
- A song too long for the 4 GiB WAV size limit is refused with `the song is too long for a WAV
  file`.
