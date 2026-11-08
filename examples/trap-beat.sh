#!/bin/bash
# Builds a short trap beat with beatcraft-cli, the same commands an AI agent sends over MCP.
# Usage: examples/trap-beat.sh [project.json] [out.wav]
set -euo pipefail
P="${1:-midnight-trap.beatcraft.json}"
OUT="${2:-midnight-trap.wav}"
CLI="${BEATCRAFT_CLI:-beatcraft-cli}"
run() { "$CLI" run "$P" "$@" >/dev/null; }

rm -f "$P"
run project.new '{"template":"empty"}'          # Kick, Clap, HiHat, Snare
run project.rename '{"name":"Midnight Trap"}'
run project.set_tempo '{"bpm":140}'
run project.set_swing '{"amount":0.1}'
# drums (pattern 0, 16 steps)
for s in 0 7 10; do run step.set "{\"pattern\":0,\"channel\":0,\"step\":$s}"; done
for s in 4 12;   do run step.set "{\"pattern\":0,\"channel\":1,\"step\":$s}"; done
run step.fill '{"pattern":0,"channel":2,"every":1,"velocity":0.55}'
run step.set  '{"pattern":0,"channel":3,"step":15,"velocity":0.4}'
# 808-style bass and a pad
run channel.add '{"kind":"Bass"}'
run note.add '{"pattern":0,"channel":4,"key":36,"start_step":0,"length_steps":6}'
run note.add '{"pattern":0,"channel":4,"key":36,"start_step":7,"length_steps":3}'
run note.add '{"pattern":0,"channel":4,"key":39,"start_step":10,"length_steps":6}'
run channel.add '{"kind":"Pad"}'
for k in 60 63 67; do run note.add "{\"pattern\":0,\"channel\":5,\"key\":$k,\"start_step\":0,\"length_steps\":16,\"velocity\":0.5}"; done
run channel.set '{"channel":5,"insert":2}'
run mixer.set '{"insert":2,"reverb":0.4}'
# arrange: four bars on the playlist, then render
run clip.add '{"pattern":0,"track":0,"bar":0,"bars":4}'
"$CLI" render "$P" "$OUT"
