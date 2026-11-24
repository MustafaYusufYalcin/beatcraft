# Attribution

## Assets
BeatCraft ships no third-party artwork, fonts, samples or sounds of its own choosing:

| Asset | Source | License |
|---|---|---|
| App icon (`packaging/icon.png`, `packaging/icon.ico`, `packaging/icons/hicolor/`) | Drawn in code by `src/icon.rs` | Same as BeatCraft |
| All instrument sounds and the demo song | Synthesised at run time by `src/synth.rs` | Same as BeatCraft |
| Interface font | egui's bundled default fonts (Ubuntu-Light, Hack, Noto Emoji, emoji-icon-font) | Their own open licenses (OFL / Apache-2.0 / MIT), as shipped by the `epaint_default_fonts` crate |

## Rust dependencies
The main ones are eframe/egui (MIT OR Apache-2.0), cpal (Apache-2.0), serde and serde_json
(MIT OR Apache-2.0) and image (MIT OR Apache-2.0). The complete list with licenses can be
produced with `cargo tree` or `cargo about`.
