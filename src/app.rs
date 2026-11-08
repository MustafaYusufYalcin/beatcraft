//! Main window: menu, transport toolbar, browser, and window management.

use crate::engine::{self, Cmd, SharedRef};
use crate::model::*;
use crate::widgets::{self, col};
use egui::{vec2, Align2, Color32, FontId, Key, RichText, Sense, Stroke};
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq)]
pub enum NoteDrag {
    Move { idx: usize, grab_ticks: i64, grab_key: i32 },
    Resize { idx: usize },
}

#[derive(Clone, Copy, PartialEq)]
pub enum ClipDrag {
    Move { idx: usize, grab_ticks: i64 },
    Resize { idx: usize },
}

pub struct App {
    pub project: Project,
    pub shared: SharedRef,
    _audio: engine::AudioOut,
    pub dirty: bool,
    // view state
    pub show_rack: bool,
    pub show_piano: bool,
    pub show_mixer: bool,
    pub show_browser: bool,
    pub show_channel: bool,
    pub show_help: bool,
    pub selected_channel: usize,
    pub selected_insert: usize,
    pub current_pattern: usize,
    pub song_mode: bool,
    pub playing: bool,
    // read back from the engine
    pub pos_ticks: f64,
    pub meters: Vec<[f32; 2]>,
    pub scope: Vec<f32>,
    pub load: f32,
    pub voices: usize,
    pub audio_ok: bool,
    pub device: String,
    // editors
    pub note_drag: Option<NoteDrag>,
    pub last_note_len: u32,
    pub piano_zoom: f32,
    pub clip_drag: Option<ClipDrag>,
    pub bar_px: f32,
    pub hint: String,
    pub held_keys: Vec<(Key, u8)>,
    pub octave: i32,
    tap_times: Vec<f64>,
    pub rename: Option<(usize, String)>,
    // screenshot
    frame: u64,
    screenshot: Option<PathBuf>,
    shot_requested: bool,
    first_frame: bool,
}

impl App {
    pub fn new(cc: &eframe::CreationContext, screenshot: Option<PathBuf>, open: Option<PathBuf>) -> Self {
        setup_style(&cc.egui_ctx);
        let project = Project::demo();
        let shared = std::sync::Arc::new(std::sync::Mutex::new(engine::Shared::new(project.clone())));
        let audio = engine::start_audio(shared.clone(), screenshot.is_none());
        let mut app = Self {
            project,
            shared,
            _audio: audio,
            dirty: false,
            show_rack: true,
            show_piano: true,
            show_mixer: true,
            show_browser: true,
            show_channel: false,
            show_help: false,
            selected_channel: 4,
            selected_insert: 5,
            current_pattern: 0,
            song_mode: true,
            playing: false,
            pos_ticks: 0.0,
            meters: vec![[0.0; 2]; MIXER_TRACKS],
            scope: vec![0.0; 1024],
            load: 0.0,
            voices: 0,
            audio_ok: false,
            device: String::new(),
            note_drag: None,
            last_note_len: STEP_TICKS * 2,
            piano_zoom: 1.0,
            clip_drag: None,
            bar_px: 64.0,
            hint: "Welcome! Press Space to play the demo song.".into(),
            held_keys: Vec::new(),
            octave: 5,
            tap_times: Vec::new(),
            rename: None,
            frame: 0,
            shot_requested: false,
            first_frame: true,
            screenshot,
        };
        if let Some(path) = &open {
            app.load(path);
        }
        if app.screenshot.is_some() {
            // show something alive in the check picture
            app.playing = true;
            let mut s = app.shared.lock().unwrap();
            s.playing = true;
            s.song_mode = true;
            s.pos_ticks = (STEPS_PER_BAR * STEP_TICKS * 6) as f64;
        } else {
            let mut s = app.shared.lock().unwrap();
            s.song_mode = true;
        }
        app
    }

    pub fn mark(&mut self) {
        self.dirty = true;
    }

    pub fn send(&self, cmd: Cmd) {
        if let Ok(mut s) = self.shared.lock() {
            s.cmds.push(cmd);
        }
    }

    fn sync_from_engine(&mut self) {
        if let Ok(s) = self.shared.lock() {
            self.pos_ticks = s.pos_ticks;
            self.meters.clone_from(&s.meters);
            let n = s.scope.len();
            for i in 0..n {
                self.scope[i] = s.scope[(s.scope_pos + i) % n];
            }
            self.load = s.load;
            self.voices = s.voices;
            self.audio_ok = s.audio_ok;
            if self.device.is_empty() {
                self.device = s.device_name.clone();
            }
        }
    }

    fn push_to_engine(&mut self) {
        if let Ok(mut s) = self.shared.lock() {
            if self.dirty {
                s.project = self.project.clone();
            }
            s.playing = self.playing;
            s.song_mode = self.song_mode;
            s.current_pattern = self.current_pattern;
        }
        self.dirty = false;
    }

    pub fn toggle_play(&mut self) {
        self._audio.resume();
        self.playing = !self.playing;
        if let Ok(mut s) = self.shared.lock() {
            s.playing = self.playing;
            if !self.playing {
                s.pos_ticks = 0.0;
            }
        }
        self.hint = if self.playing { "Playing".into() } else { "Stopped".into() };
    }

    pub fn stop(&mut self) {
        self.playing = false;
        if let Ok(mut s) = self.shared.lock() {
            s.playing = false;
            s.pos_ticks = 0.0;
            s.cmds.push(Cmd::Panic);
        }
    }

    pub fn set_position(&mut self, ticks: f64) {
        if let Ok(mut s) = self.shared.lock() {
            s.pos_ticks = ticks.max(0.0);
        }
    }

    fn projects_dir() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(home).join("Music").join("BeatCraft")
    }

    fn save(&mut self) {
        let dir = Self::projects_dir();
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join(format!("{}.beatcraft.json", sanitize(&self.project.name)));
        match serde_json::to_string_pretty(&self.project).map(|j| std::fs::write(&file, j)) {
            Ok(Ok(())) => self.hint = format!("Saved to {}", file.display()),
            _ => self.hint = "Could not save the project".into(),
        }
    }

    fn load(&mut self, path: &std::path::Path) {
        match std::fs::read_to_string(path).ok().and_then(|t| serde_json::from_str::<Project>(&t).ok()) {
            Some(p) => {
                self.set_project(p);
                self.hint = format!("Opened {}", path.display());
            }
            None => self.hint = "Could not open that file".into(),
        }
    }

    pub fn set_project(&mut self, p: Project) {
        self.stop();
        self.project = p;
        self.current_pattern = 0;
        self.selected_channel = 0;
        self.selected_insert = 1;
        self.mark();
    }

    fn export(&mut self) {
        let dir = Self::projects_dir();
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join(format!("{}.wav", sanitize(&self.project.name)));
        match engine::export_wav(&self.project, &file) {
            Ok(info) => {
                let bpm = info.measured_bpm.map(|b| format!("{b:.0} BPM")).unwrap_or_else(|| "unrealistic tempo".into());
                self.hint = format!("Exported {:.1} s ({}) to {}", info.seconds, bpm, file.display());
            }
            Err(e) => self.hint = format!("Export failed: {e}"),
        }
    }

    fn saved_projects() -> Vec<PathBuf> {
        let mut v: Vec<PathBuf> = std::fs::read_dir(Self::projects_dir())
            .map(|r| r.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.to_string_lossy().ends_with(".beatcraft.json")).collect())
            .unwrap_or_default();
        v.sort();
        v
    }

    fn tap_tempo(&mut self, now: f64) {
        self.tap_times.retain(|t| now - t < 2.5);
        self.tap_times.push(now);
        if self.tap_times.len() >= 3 {
            let n = self.tap_times.len();
            let avg = (self.tap_times[n - 1] - self.tap_times[0]) / (n - 1) as f64;
            let bpm = (60.0 / avg) as f32;
            // results outside the realistic 60–240 BPM window are refused
            if (REALISTIC_TEMPO_MIN..=REALISTIC_TEMPO_MAX).contains(&bpm) {
                self.project.tempo = bpm.round();
                self.mark();
                self.hint = format!("Tap tempo: {:.0} BPM", bpm);
            } else {
                self.hint = "Unrealistic tempo".into();
            }
        } else {
            self.hint = "Keep tapping…".into();
        }
    }

    // ------------------------------------------------------------------ keyboard
    fn keyboard(&mut self, ctx: &egui::Context) {
        let typing = ctx.memory(|m| m.focused().is_some());
        let events = ctx.input(|i| i.events.clone());
        for e in events {
            if let egui::Event::Key { key, pressed, repeat, modifiers, .. } = e {
                if repeat {
                    continue;
                }
                if pressed && modifiers.command {
                    if key == Key::S {
                        self.save();
                    }
                    continue;
                }
                if typing {
                    continue;
                }
                if pressed {
                    match key {
                        Key::Space => self.toggle_play(),
                        Key::F5 => self.show_rack = !self.show_rack,
                        Key::F6 => self.show_rack = !self.show_rack,
                        Key::F7 => self.show_piano = !self.show_piano,
                        Key::F8 => self.show_browser = !self.show_browser,
                        Key::F9 => self.show_mixer = !self.show_mixer,
                        Key::L => {
                            self.song_mode = !self.song_mode;
                            self.set_position(0.0);
                        }
                        Key::Minus => self.octave = (self.octave - 1).max(1),
                        Key::Equals => self.octave = (self.octave + 1).min(8),
                        _ => {}
                    }
                }
                if let Some(semi) = qwerty_semitone(key) {
                    let k = (self.octave * 12 + semi).clamp(0, 127) as u8;
                    let ch = self.selected_channel;
                    if pressed {
                        if !self.held_keys.iter().any(|(hk, _)| *hk == key) {
                            self.held_keys.push((key, k));
                            self.send(Cmd::NoteOn { ch, key: k, vel: 0.85 });
                        }
                    } else if let Some(i) = self.held_keys.iter().position(|(hk, _)| *hk == key) {
                        let (_, k) = self.held_keys.remove(i);
                        self.send(Cmd::NoteOff { ch, key: k });
                    }
                }
            }
        }
    }

    // ------------------------------------------------------------------ top bar
    fn top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu").exact_height(24.0).frame(egui::Frame::new().fill(col::DARK).inner_margin(egui::Margin::symmetric(6, 2))).show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(RichText::new("◆ BeatCraft").color(col::ACCENT).strong());
                ui.add_space(8.0);
                ui.menu_button("FILE", |ui| {
                    if ui.button("New (basic drums)").clicked() {
                        self.set_project(Project::empty());
                        ui.close();
                    }
                    if ui.button("Open demo song").clicked() {
                        self.set_project(Project::demo());
                        ui.close();
                    }
                    if ui.button("Save   ⌘S").clicked() {
                        self.save();
                        ui.close();
                    }
                    ui.menu_button("Open saved", |ui| {
                        let list = Self::saved_projects();
                        if list.is_empty() {
                            ui.label("Nothing saved yet");
                        }
                        for p in list {
                            let name = p.file_name().map(|n| n.to_string_lossy().replace(".beatcraft.json", "")).unwrap_or_default();
                            if ui.button(name).clicked() {
                                self.load(&p);
                                ui.close();
                            }
                        }
                    });
                    ui.separator();
                    if ui.button("Export song to WAV").clicked() {
                        self.export();
                        ui.close();
                    }
                });
                ui.menu_button("ADD", |ui| {
                    for k in InstKind::ALL {
                        if ui.button(k.label()).clicked() {
                            self.add_channel(k);
                            ui.close();
                        }
                    }
                });
                ui.menu_button("PATTERNS", |ui| {
                    if ui.button("New pattern").clicked() {
                        self.current_pattern = self.project.add_pattern();
                        self.mark();
                        ui.close();
                    }
                    if ui.button("Clone current pattern").clicked() {
                        let mut p = self.project.patterns[self.current_pattern].clone();
                        p.name = format!("{} copy", p.name);
                        self.project.patterns.push(p);
                        self.current_pattern = self.project.patterns.len() - 1;
                        self.mark();
                        ui.close();
                    }
                    if ui.button("Clear current pattern").clicked() {
                        let n = self.project.channels.len();
                        let p = &mut self.project.patterns[self.current_pattern];
                        p.data.clear();
                        p.ensure_channels(n);
                        self.mark();
                        ui.close();
                    }
                });
                ui.menu_button("VIEW", |ui| {
                    ui.checkbox(&mut self.show_rack, "Channel rack  (F6)");
                    ui.checkbox(&mut self.show_piano, "Piano roll  (F7)");
                    ui.checkbox(&mut self.show_browser, "Browser  (F8)");
                    ui.checkbox(&mut self.show_mixer, "Mixer  (F9)");
                    ui.checkbox(&mut self.show_channel, "Channel settings");
                });
                ui.menu_button("HELP", |ui| {
                    if ui.button("Shortcuts").clicked() {
                        self.show_help = true;
                        ui.close();
                    }
                });
                ui.add_space(16.0);
                let (r, _) = ui.allocate_exact_size(vec2(420.0, 18.0), Sense::hover());
                ui.painter().rect_filled(r, 2.0, col::DARKER);
                ui.painter().text(r.left_center() + vec2(6.0, 0.0), Align2::LEFT_CENTER, &self.hint, FontId::proportional(11.5), col::DIM);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(&self.project.name).color(col::TEXT));
                });
            });
        });

        egui::TopBottomPanel::top("transport").exact_height(52.0).frame(egui::Frame::new().fill(col::PANEL).inner_margin(egui::Margin::symmetric(8, 6))).show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                // PAT / SONG
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    if widgets::flat_button(ui, "PAT", !self.song_mode, 44.0, 18.0, col::ACCENT).on_hover_text("Pattern mode (L)").clicked() {
                        self.song_mode = false;
                        self.set_position(0.0);
                    }
                    if widgets::flat_button(ui, "SONG", self.song_mode, 44.0, 18.0, col::GREEN).on_hover_text("Song mode (L)").clicked() {
                        self.song_mode = true;
                        self.set_position(0.0);
                    }
                });
                if transport_button(ui, if self.playing { "⏸" } else { "▶" }, self.playing, col::GREEN).on_hover_text("Play / pause (Space)").clicked() {
                    self.toggle_play();
                }
                if transport_button(ui, "■", false, col::ACCENT).on_hover_text("Stop").clicked() {
                    self.stop();
                }
                transport_button(ui, "●", false, col::RED).on_hover_text("Record (not in this version)");
                ui.add_space(6.0);
                // tempo LCD
                let tresp = widgets::lcd(ui, &format!("{:7.3}", self.project.tempo), 92.0, 34.0, 18.0).on_hover_text("Tempo (BPM) – drag up/down, scroll, double-click for 140");
                if tresp.dragged() {
                    let fine = if ui.input(|i| i.modifiers.shift) { 0.05 } else { 0.5 };
                    self.project.tempo = (self.project.tempo - tresp.drag_delta().y * fine).clamp(TEMPO_MIN, TEMPO_MAX);
                    if !ui.input(|i| i.modifiers.shift) {
                        self.project.tempo = self.project.tempo.round();
                    }
                    self.mark();
                }
                if tresp.hovered() {
                    let s = ui.input(|i| i.raw_scroll_delta.y);
                    if s != 0.0 {
                        self.project.tempo = (self.project.tempo + s.signum()).clamp(TEMPO_MIN, TEMPO_MAX).round();
                        self.mark();
                    }
                }
                if tresp.double_clicked() {
                    self.project.tempo = DEFAULT_TEMPO;
                    self.mark();
                }
                if widgets::flat_button(ui, "TAP", false, 34.0, 34.0, col::ACCENT).on_hover_text("Tap tempo").clicked() {
                    let now = ui.input(|i| i.time);
                    self.tap_tempo(now);
                }
                // time display
                let t = self.pos_ticks as u32;
                let bar = t / (STEPS_PER_BAR * STEP_TICKS) + 1;
                let beat = (t / PPQ) % BEATS_PER_BAR + 1;
                let step = (t / STEP_TICKS) % 4 + 1;
                let tr = widgets::lcd(ui, &format!("{:3}:{:02}:{:02}", bar, beat, step), 120.0, 34.0, 20.0).on_hover_text("Song position  bar:beat:step");
                if tr.clicked() {
                    self.set_position(0.0);
                }
                ui.add_space(6.0);
                // pattern selector
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    ui.label(RichText::new("PATTERN").size(9.0).color(col::DIM));
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;
                        if widgets::flat_button(ui, "◀", false, 18.0, 20.0, col::ACCENT).clicked() && self.current_pattern > 0 {
                            self.current_pattern -= 1;
                        }
                        let name = self.project.patterns[self.current_pattern].name.clone();
                        let c = widgets::rgb(self.project.patterns[self.current_pattern].color);
                        let (r, resp) = ui.allocate_exact_size(vec2(120.0, 20.0), Sense::click());
                        ui.painter().rect_filled(r, 3.0, col::DARK);
                        ui.painter().rect_filled(egui::Rect::from_min_size(r.min, vec2(5.0, r.height())), 2.0, c);
                        ui.painter().text(r.left_center() + vec2(10.0, 0.0), Align2::LEFT_CENTER, &name, FontId::proportional(12.0), col::TEXT);
                        resp.context_menu(|ui| {
                            for (i, p) in self.project.patterns.iter().enumerate() {
                                if ui.button(&p.name).clicked() {
                                    self.current_pattern = i;
                                    ui.close();
                                }
                            }
                        });
                        if widgets::flat_button(ui, "▶", false, 18.0, 20.0, col::ACCENT).clicked() && self.current_pattern + 1 < self.project.patterns.len() {
                            self.current_pattern += 1;
                        }
                        if widgets::flat_button(ui, "+", false, 20.0, 20.0, col::ACCENT).on_hover_text("New pattern").clicked() {
                            self.current_pattern = self.project.add_pattern();
                            self.mark();
                        }
                    });
                });
                ui.add_space(8.0);
                // master knobs
                for (label, v, min, max, def, c) in [
                    ("VOL", 0usize, 0.0f32, 1.0f32, FADER_UNITY, col::ACCENT),
                    ("PITCH", 1, -12.0, 12.0, 0.0, col::GREEN),
                    ("SWING", 2, 0.0, 1.0, 0.0, Color32::from_rgb(0x7a, 0xb8, 0xe8)),
                ] {
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        let val = match v {
                            0 => &mut self.project.master_volume,
                            1 => &mut self.project.master_pitch,
                            _ => &mut self.project.swing,
                        };
                        if widgets::knob(ui, val, min, max, def, 26.0, c, label) {
                            if v == 2 {
                                *val = snap_swing(*val);
                            }
                            self.dirty = true;
                        }
                        ui.label(RichText::new(label).size(9.0).color(col::DIM));
                    });
                }
                ui.add_space(8.0);
                // scope
                let (r, _) = ui.allocate_exact_size(vec2(130.0, 36.0), Sense::hover());
                let p = ui.painter();
                p.rect_filled(r, 3.0, col::LCD);
                let n = self.scope.len();
                let pts: Vec<_> = (0..130)
                    .map(|i| {
                        let v = self.scope[n - 520 + i * 4].clamp(-1.0, 1.0);
                        egui::pos2(r.left() + i as f32, r.center().y - v * 16.0)
                    })
                    .collect();
                p.add(egui::Shape::line(pts, Stroke::new(1.2, col::LCD_TEXT)));
                // cpu / voices
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    ui.label(RichText::new(format!("CPU {:>3.0}%", self.load * 100.0)).size(10.0).monospace().color(col::TEXT));
                    ui.label(RichText::new(format!("POLY {:>3}", self.voices)).size(10.0).monospace().color(col::TEXT));
                    ui.label(RichText::new(if self.audio_ok { "AUDIO ON" } else { "NO AUDIO" }).size(9.0).color(if self.audio_ok { col::GREEN } else { col::DIM }));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 3.0;
                    for (txt, idx, tip) in [("MIX", 3, "Mixer (F9)"), ("PNO", 2, "Piano roll (F7)"), ("RACK", 1, "Channel rack (F6)"), ("BRWS", 0, "Browser (F8)")] {
                        let flag = match idx {
                            0 => &mut self.show_browser,
                            1 => &mut self.show_rack,
                            2 => &mut self.show_piano,
                            _ => &mut self.show_mixer,
                        };
                        if widgets::flat_button(ui, txt, *flag, 42.0, 30.0, col::ACCENT).on_hover_text(tip).clicked() {
                            *flag = !*flag;
                        }
                    }
                });
            });
        });
    }

    pub fn add_channel(&mut self, k: InstKind) {
        self.selected_channel = self.project.add_channel(k);
        self.selected_insert = self.project.channels[self.selected_channel].insert;
        self.hint = format!("Added {}", k.label());
        self.mark();
    }

    // ------------------------------------------------------------------ browser
    fn browser(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("browser")
            .default_width(190.0)
            .resizable(true)
            .frame(egui::Frame::new().fill(col::PANEL).inner_margin(egui::Margin::same(0)))
            .show(ctx, |ui| {
                widgets::panel_header(ui, "  BROWSER");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    ui.add_space(4.0);
                    egui::CollapsingHeader::new(RichText::new("📁 Current project").color(col::TEXT)).default_open(true).show(ui, |ui| {
                        egui::CollapsingHeader::new(RichText::new("Patterns").color(col::DIM)).default_open(true).show(ui, |ui| {
                            for i in 0..self.project.patterns.len() {
                                let p = &self.project.patterns[i];
                                let sel = i == self.current_pattern;
                                let txt = RichText::new(format!("▦ {}", p.name)).color(if sel { col::ACCENT } else { col::TEXT });
                                if ui.selectable_label(sel, txt).clicked() {
                                    self.current_pattern = i;
                                }
                            }
                        });
                        egui::CollapsingHeader::new(RichText::new("Channels").color(col::DIM)).default_open(true).show(ui, |ui| {
                            for i in 0..self.project.channels.len() {
                                let c = &self.project.channels[i];
                                let sel = i == self.selected_channel;
                                if ui.selectable_label(sel, RichText::new(format!("♪ {}", c.name)).color(widgets::rgb(c.color))).clicked() {
                                    self.selected_channel = i;
                                    self.selected_insert = c.insert;
                                }
                            }
                        });
                    });
                    egui::CollapsingHeader::new(RichText::new("📁 Instruments").color(col::TEXT)).default_open(true).show(ui, |ui| {
                        for k in InstKind::ALL {
                            let r = ui.selectable_label(false, RichText::new(format!("{} {}", if k.is_drum() { "◉" } else { "♫" }, k.label())).color(col::TEXT));
                            if r.on_hover_text("Click to add to the channel rack").clicked() {
                                self.add_channel(k);
                            }
                        }
                    });
                    egui::CollapsingHeader::new(RichText::new("📁 Effects").color(col::TEXT)).default_open(true).show(ui, |ui| {
                        for fx in [FxKind::Filter, FxKind::Delay, FxKind::Reverb, FxKind::Limiter] {
                            let r = ui.selectable_label(false, RichText::new(format!("⚙ {}", fx.label())).color(col::TEXT));
                            if r.on_hover_text("Click to load into the selected mixer track").clicked() {
                                let ins = &mut self.project.mixer[self.selected_insert];
                                if let Some(slot) = ins.slots.iter_mut().find(|s| s.is_none()) {
                                    *slot = Some(fx);
                                    self.hint = format!("{} loaded on {}", fx.label(), ins.name);
                                    self.mark();
                                }
                            }
                        }
                    });
                    egui::CollapsingHeader::new(RichText::new("📁 Projects").color(col::TEXT)).default_open(false).show(ui, |ui| {
                        if ui.selectable_label(false, "★ Demo song").clicked() {
                            self.set_project(Project::demo());
                        }
                        for p in Self::saved_projects() {
                            let name = p.file_name().map(|n| n.to_string_lossy().replace(".beatcraft.json", "")).unwrap_or_default();
                            if ui.selectable_label(false, format!("▤ {name}")).clicked() {
                                self.load(&p);
                            }
                        }
                    });
                });
            });
    }

    fn help(&mut self, ctx: &egui::Context) {
        egui::Window::new("Shortcuts").open(&mut self.show_help).collapsible(false).resizable(false).show(ctx, |ui| {
            for (k, v) in [
                ("Space", "Play / pause"),
                ("L", "Switch pattern / song mode"),
                ("F6 / F7 / F8 / F9", "Channel rack / piano roll / browser / mixer"),
                ("Z S X D C … / Q 2 W 3 E …", "Play the selected channel from the computer keyboard"),
                ("- / =", "Keyboard octave down / up"),
                ("⌘S", "Save"),
                ("Piano roll", "Click: add note · drag: move · drag end: length · right click: delete"),
                ("Playlist", "Click: place pattern · drag: move · drag end: length · right click: delete"),
                ("Knobs", "Drag up/down · Shift = fine · double click = reset"),
            ] {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(k).color(col::ACCENT).monospace());
                    ui.label(v);
                });
            }
        });
    }

    fn handle_screenshot(&mut self, ctx: &egui::Context) {
        let Some(path) = self.screenshot.clone() else { return };
        self.frame += 1;
        ctx.request_repaint();
        if self.frame >= 120 && !self.shot_requested {
            self.shot_requested = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        }
        let shot = ctx.input(|i| {
            i.raw.events.iter().find_map(|e| if let egui::Event::Screenshot { image, .. } = e { Some(image.clone()) } else { None })
        });
        if let Some(img) = shot {
            let [w, h] = img.size;
            let mut buf = Vec::with_capacity(w * h * 4);
            for p in &img.pixels {
                buf.extend_from_slice(&[p.r(), p.g(), p.b(), 255]);
            }
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            match image::save_buffer(&path, &buf, w as u32, h as u32, image::ColorType::Rgba8) {
                Ok(()) => eprintln!("screenshot saved to {}", path.display()),
                Err(e) => eprintln!("screenshot failed: {e}"),
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.first_frame {
            self.first_frame = false;
            self.mark();
        }
        self.sync_from_engine();
        if self.screenshot.is_none() {
            self.keyboard(ctx);
        } else {
            // the check picture always shows the song playing
            self.song_mode = true;
            self.playing = true;
        }
        self.top_bar(ctx);
        if self.show_browser {
            self.browser(ctx);
        }
        if self.show_mixer {
            self.mixer_panel(ctx);
        }
        self.playlist(ctx);
        if self.show_rack {
            self.channel_rack(ctx);
        }
        if self.show_piano {
            self.piano_roll(ctx);
        }
        if self.show_channel {
            self.channel_settings(ctx);
        }
        if self.show_help {
            self.help(ctx);
        }
        // keep pattern data sized to channels
        let n = self.project.channels.len();
        for p in &mut self.project.patterns {
            if p.data.len() != n || p.data.iter().any(|d| d.steps.len() != p.steps) {
                p.ensure_channels(n);
                p.data.truncate(n);
                self.dirty = true;
            }
        }
        self.selected_channel = self.selected_channel.min(n.saturating_sub(1));
        self.current_pattern = self.current_pattern.min(self.project.patterns.len().saturating_sub(1));
        self.push_to_engine();
        if self.playing || self.screenshot.is_some() {
            ctx.request_repaint();
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(60));
        }
        self.handle_screenshot(ctx);
    }
}

fn transport_button(ui: &mut egui::Ui, icon: &str, active: bool, c: Color32) -> egui::Response {
    let (r, resp) = ui.allocate_exact_size(vec2(34.0, 34.0), Sense::click());
    let p = ui.painter();
    p.rect_filled(r, 4.0, if active { widgets::shade(c, 0.45) } else if resp.hovered() { col::PANEL_HI } else { col::DARK });
    p.rect_stroke(r, 4.0, Stroke::new(1.0, if active { c } else { col::GRID }), egui::StrokeKind::Inside);
    p.text(r.center(), Align2::CENTER_CENTER, icon, FontId::proportional(16.0), c);
    resp
}

fn qwerty_semitone(k: Key) -> Option<i32> {
    use Key::*;
    Some(match k {
        Z => 0, S => 1, X => 2, D => 3, C => 4, V => 5, G => 6, B => 7, H => 8, N => 9, J => 10, M => 11, Comma => 12,
        Q => 12, Num2 => 13, W => 14, Num3 => 15, E => 16, R => 17, Num5 => 18, T => 19, Num6 => 20, Y => 21, Num7 => 22, U => 23, I => 24,
        _ => return None,
    })
}

fn sanitize(s: &str) -> String {
    s.chars().map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' { c } else { '_' }).collect::<String>().trim().to_string()
}

fn setup_style(ctx: &egui::Context) {
    let mut v = egui::Visuals::dark();
    v.panel_fill = col::PANEL;
    v.window_fill = col::PANEL;
    v.extreme_bg_color = col::DARKER;
    v.faint_bg_color = col::DARK;
    v.window_stroke = Stroke::new(1.0, col::GRID_BAR);
    v.window_corner_radius = egui::CornerRadius::same(4);
    v.selection.bg_fill = widgets::shade(col::ACCENT, 0.45);
    v.selection.stroke = Stroke::new(1.0, col::ACCENT);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, col::TEXT);
    v.widgets.inactive.bg_fill = col::DARK;
    v.widgets.inactive.weak_bg_fill = col::DARK;
    v.widgets.hovered.weak_bg_fill = col::PANEL_HI;
    v.override_text_color = Some(col::TEXT);
    ctx.set_visuals(v);
    ctx.style_mut(|s| {
        s.spacing.item_spacing = vec2(4.0, 3.0);
        s.spacing.window_margin = egui::Margin::same(6);
        s.text_styles.insert(egui::TextStyle::Heading, FontId::proportional(13.5));
    });
}
