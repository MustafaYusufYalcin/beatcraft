//! BeatCraft — a pattern-based music studio (channel rack, piano roll, playlist, mixer).

// No console window behind the app in Windows release builds.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod engine;
mod icon;
mod mixer;
mod model;
mod piano;
mod playlist;
mod rack;
mod synth;
mod widgets;

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut screenshot: Option<PathBuf> = None;
    let mut export: Option<PathBuf> = None;
    let mut icon_out: Option<(PathBuf, u32)> = None;
    let mut open: Option<PathBuf> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--screenshot" if i + 1 < args.len() => {
                screenshot = Some(PathBuf::from(&args[i + 1]));
                i += 1;
            }
            "--export-demo" if i + 1 < args.len() => {
                export = Some(PathBuf::from(&args[i + 1]));
                i += 1;
            }
            "--write-icon" if i + 1 < args.len() => {
                let size = args.get(i + 2).and_then(|s| s.parse().ok()).unwrap_or(1024);
                icon_out = Some((PathBuf::from(&args[i + 1]), size));
                i += 2;
            }
            "--open" if i + 1 < args.len() => {
                open = Some(PathBuf::from(&args[i + 1]));
                i += 1;
            }
            a if a.ends_with(".json") && !a.starts_with("--") => open = Some(PathBuf::from(a)),
            "--version" => {
                println!("BeatCraft {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            _ => {}
        }
        i += 1;
    }
    if let Some((path, size)) = icon_out {
        match icon::write_png(&path, size) {
            Ok(()) => println!("icon written to {}", path.display()),
            Err(e) => eprintln!("icon failed: {e}"),
        }
        return Ok(());
    }
    if let Some(path) = export {
        match engine::export_wav(&model::Project::demo(), &path) {
            Ok(info) => println!("exported {:.1} s to {} (measures as {:?} BPM)", info.seconds, path.display(), info.measured_bpm),
            Err(e) => eprintln!("export failed: {e}"),
        }
        return Ok(());
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1600.0, 960.0])
            .with_min_inner_size([1100.0, 700.0])
            .with_title("BeatCraft")
            .with_app_id("io.github.mustafayusufyalcin.beatcraft")
            .with_icon(std::sync::Arc::new(icon::icon_data())),
        ..Default::default()
    };
    eframe::run_native("BeatCraft", options, Box::new(move |cc| Ok(Box::new(app::App::new(cc, screenshot, open)))))
}

/// Web build: started by trunk from index.html (canvas id "beatcraft").
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast;
    let options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        let canvas = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("beatcraft"))
            .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
            .expect("canvas #beatcraft missing");
        let _ = eframe::WebRunner::new().start(canvas, options, Box::new(|cc| Ok(Box::new(app::App::new(cc, None, None))))).await;
        if let Some(l) = web_sys::window().and_then(|w| w.document()).and_then(|d| d.get_element_by_id("loading")) {
            l.remove();
        }
    });
}
