//! Channel rack (step sequencer) and channel settings window.

use crate::app::App;
use crate::engine::Cmd;
use crate::model::*;
use crate::widgets::{self, col};
use egui::{pos2, vec2, Align2, Color32, FontId, Rect, RichText, Sense, Stroke, StrokeKind};

impl App {
    pub fn channel_rack(&mut self, ctx: &egui::Context) {
        let mut open = self.show_rack;
        let pat_name = self.project.patterns[self.current_pattern].name.clone();
        egui::Window::new(format!("Channel rack  ·  {pat_name}"))
            .id(egui::Id::new("rack"))
            .open(&mut open)
            .default_pos(pos2(215.0, 120.0))
            .default_width(640.0)
            .resizable(false)
            .collapsible(true)
            .show(ctx, |ui| self.rack_contents(ui));
        self.show_rack = open;
    }

    fn rack_contents(&mut self, ui: &mut egui::Ui) {
        let pi = self.current_pattern;
        let steps = self.project.patterns[pi].steps;
        let pat_len = self.project.patterns[pi].length_ticks();
        let play_step = if self.playing && !self.song_mode { Some((self.pos_ticks as u32 % pat_len) / STEP_TICKS) } else { None };
        // header: swing + steps
        ui.horizontal(|ui| {
            ui.label(RichText::new("Swing").size(11.0).color(col::DIM));
            if widgets::knob(ui, &mut self.project.swing, 0.0, 1.0, 0.0, 20.0, Color32::from_rgb(0x7a, 0xb8, 0xe8), "Swing") {
                self.project.swing = snap_swing(self.project.swing);
                self.mark();
            }
            ui.add_space(10.0);
            ui.label(RichText::new("Steps").size(11.0).color(col::DIM));
            for n in [16usize, 32, 64] {
                if widgets::flat_button(ui, &n.to_string(), steps == n, 28.0, 18.0, col::ACCENT).clicked() {
                    let nch = self.project.channels.len();
                    let p = &mut self.project.patterns[pi];
                    p.steps = n;
                    p.ensure_channels(nch);
                    self.mark();
                }
            }
            ui.add_space(10.0);
            ui.label(RichText::new("Click steps to toggle · right click clears · scroll a step for velocity").size(10.0).color(col::DIM));
        });
        ui.add_space(2.0);
        let step_w = if steps > 32 {
            11.0
        } else if steps > 16 {
            15.0
        } else {
            22.0
        };
        let mut remove: Option<usize> = None;
        let mut preview: Option<usize> = None;
        let mut grid_w = 0.0;
        egui::ScrollArea::both().max_height(360.0).show(ui, |ui| {
            for ci in 0..self.project.channels.len() {
                let sel = ci == self.selected_channel;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 3.0;
                    let ch = &mut self.project.channels[ci];
                    // mute LED
                    if widgets::led(ui, !ch.mute, col::GREEN, 14.0).on_hover_text("Mute / unmute").clicked() {
                        ch.mute = !ch.mute;
                        self.dirty = true;
                    }
                    let c = widgets::rgb(ch.color);
                    if widgets::knob(ui, &mut ch.pan, -1.0, 1.0, 0.0, 18.0, c, "Pan") {
                        self.dirty = true;
                    }
                    if widgets::knob(ui, &mut ch.volume, 0.0, 1.0, CHANNEL_DEFAULT_VOLUME, 18.0, c, "Volume") {
                        self.dirty = true;
                    }
                    // mixer route
                    let (r, resp) = ui.allocate_exact_size(vec2(24.0, 18.0), Sense::click_and_drag());
                    ui.painter().rect_filled(r, 2.0, col::LCD);
                    ui.painter().text(
                        r.center(),
                        Align2::CENTER_CENTER,
                        if ch.insert == 0 { "M".to_string() } else { ch.insert.to_string() },
                        FontId::monospace(11.0),
                        col::LCD_TEXT,
                    );
                    if resp.dragged() {
                        let d = (-resp.drag_delta().y / 4.0).round() as i32;
                        ch.insert = (ch.insert as i32 + d).clamp(0, MIXER_INSERTS as i32) as usize;
                        self.dirty = true;
                    }
                    if resp.clicked() {
                        self.selected_insert = ch.insert;
                        self.show_mixer = true;
                    }
                    resp.on_hover_text("Mixer track (drag to change, click to show)");
                    // name
                    let (r, resp) = ui.allocate_exact_size(vec2(108.0, 22.0), Sense::click());
                    let p = ui.painter();
                    p.rect_filled(
                        r,
                        3.0,
                        if sel {
                            widgets::shade(c, 0.55)
                        } else if resp.hovered() {
                            col::PANEL_HI
                        } else {
                            col::STEP_A
                        },
                    );
                    p.rect_filled(Rect::from_min_size(r.min, vec2(4.0, r.height())), 2.0, c);
                    if sel {
                        p.rect_stroke(r, 3.0, Stroke::new(1.0_f32, c), StrokeKind::Inside);
                    }
                    p.text(r.left_center() + vec2(9.0, 0.0), Align2::LEFT_CENTER, &ch.name, FontId::proportional(12.0), Color32::WHITE);
                    if resp.clicked() {
                        self.selected_channel = ci;
                        self.selected_insert = self.project.channels[ci].insert;
                        self.send(Cmd::Preview { ch: ci, key: ROOT_KEY });
                    }
                    if resp.double_clicked() {
                        self.show_channel = true;
                    }
                    resp.context_menu(|ui| {
                        if ui.button("Open in piano roll").clicked() {
                            self.selected_channel = ci;
                            self.show_piano = true;
                            ui.close();
                        }
                        if ui.button("Channel settings").clicked() {
                            self.selected_channel = ci;
                            self.show_channel = true;
                            ui.close();
                        }
                        if ui.button("Fill every 4 steps").clicked() {
                            let p = &mut self.project.patterns[pi];
                            for (i, s) in p.data[ci].steps.iter_mut().enumerate() {
                                *s = if i % 4 == 0 { 0.8 } else { 0.0 };
                            }
                            self.dirty = true;
                            ui.close();
                        }
                        if ui.button("Fill every 2 steps").clicked() {
                            let p = &mut self.project.patterns[pi];
                            for (i, s) in p.data[ci].steps.iter_mut().enumerate() {
                                *s = if i % 2 == 0 { 0.8 } else { 0.0 };
                            }
                            self.dirty = true;
                            ui.close();
                        }
                        if ui.button("Clear").clicked() {
                            let p = &mut self.project.patterns[pi];
                            p.data[ci].steps.iter_mut().for_each(|s| *s = 0.0);
                            p.data[ci].notes.clear();
                            self.dirty = true;
                            ui.close();
                        }
                        if ui.button("Delete channel").clicked() {
                            remove = Some(ci);
                            ui.close();
                        }
                    });
                    // steps or note preview
                    let has_notes = !self.project.patterns[pi].data[ci].notes.is_empty();
                    let w = step_w * steps as f32 + (steps / 4) as f32 * 2.0;
                    grid_w = w;
                    if has_notes {
                        let (r, resp) = ui.allocate_exact_size(vec2(w, 22.0), Sense::click());
                        let p = ui.painter();
                        p.rect_filled(r, 3.0, col::DARK);
                        let d = &self.project.patterns[pi].data[ci];
                        let total = (steps as u32 * STEP_TICKS).max(1) as f32;
                        let (lo, hi) = d.notes.iter().fold((127u8, 0u8), |(a, b), n| (a.min(n.key), b.max(n.key)));
                        let span = (hi - lo).max(6) as f32;
                        for n in &d.notes {
                            let x0 = r.left() + n.start as f32 / total * r.width();
                            let x1 = r.left() + (n.start + n.len) as f32 / total * r.width();
                            let y = r.bottom() - 4.0 - (n.key - lo) as f32 / span * (r.height() - 8.0);
                            p.rect_filled(Rect::from_min_max(pos2(x0, y - 1.5), pos2(x1.min(r.right()) - 1.0, y + 1.5)), 1.0, c);
                        }
                        if let Some(s) = play_step {
                            let x = r.left() + (s as f32 + 0.5) / steps as f32 * r.width();
                            p.line_segment([pos2(x, r.top()), pos2(x, r.bottom())], Stroke::new(1.0_f32, col::ACCENT));
                        }
                        if resp.on_hover_text("This channel uses the piano roll — click to open it").clicked() {
                            self.selected_channel = ci;
                            self.show_piano = true;
                        }
                    } else {
                        for si in 0..steps {
                            if si > 0 && si % 4 == 0 {
                                ui.add_space(-1.0);
                            }
                            let (r, resp) = ui.allocate_exact_size(vec2(step_w - 2.0, 22.0), Sense::click());
                            let vel = self.project.patterns[pi].data[ci].steps[si];
                            let base = if (si / 4) % 2 == 0 { col::STEP_A } else { col::STEP_B };
                            let p = ui.painter();
                            let fill = if vel > 0.0 {
                                widgets::shade(col::STEP_ON, 0.65 + 0.35 * vel)
                            } else if resp.hovered() {
                                widgets::shade(base, 1.25)
                            } else {
                                base
                            };
                            p.rect_filled(r, 2.0, fill);
                            if vel > 0.0 {
                                p.rect_filled(Rect::from_min_size(pos2(r.left() + 2.0, r.top() + 2.0), vec2(r.width() - 4.0, 2.0)), 0.0, c);
                            }
                            if play_step == Some(si as u32) {
                                p.rect_stroke(r, 2.0, Stroke::new(1.5_f32, col::ACCENT), StrokeKind::Inside);
                            }
                            let d = &mut self.project.patterns[pi].data[ci];
                            if resp.clicked() {
                                d.steps[si] = if vel > 0.0 { 0.0 } else { 0.8 };
                                if d.steps[si] > 0.0 && !self.playing {
                                    preview = Some(ci);
                                }
                                self.dirty = true;
                            }
                            if resp.secondary_clicked() {
                                d.steps[si] = 0.0;
                                self.dirty = true;
                            }
                            if resp.hovered() && vel > 0.0 {
                                let s = ui.input(|i| i.raw_scroll_delta.y);
                                if s != 0.0 {
                                    d.steps[si] = (vel + s.signum() * 0.05).clamp(0.05, 1.0);
                                    self.dirty = true;
                                }
                            }
                        }
                    }
                });
            }
        });
        ui.add_space(3.0);
        ui.horizontal(|ui| {
            ui.menu_button(RichText::new(" + Add channel ").color(col::ACCENT), |ui| {
                for k in InstKind::ALL {
                    if ui.button(k.label()).clicked() {
                        self.add_channel(k);
                        ui.close();
                    }
                }
            });
            ui.label(RichText::new(format!("{} channels · {} steps · keyboard octave {}", self.project.channels.len(), steps, self.octave)).size(10.0).color(col::DIM));
            let _ = grid_w;
        });
        if let Some(ci) = preview {
            self.send(Cmd::Preview { ch: ci, key: ROOT_KEY });
        }
        if let Some(ci) = remove {
            self.project.remove_channel(ci);
            self.selected_channel = self.selected_channel.min(self.project.channels.len() - 1);
            self.send(Cmd::Panic);
            self.mark();
        }
    }

    pub fn channel_settings(&mut self, ctx: &egui::Context) {
        let ci = self.selected_channel;
        if ci >= self.project.channels.len() {
            return;
        }
        let mut open = self.show_channel;
        let title = format!("Channel settings · {}", self.project.channels[ci].name);
        egui::Window::new(title).id(egui::Id::new("chset")).open(&mut open).default_pos(pos2(880.0, 140.0)).resizable(false).show(ctx, |ui| {
            let ch = &mut self.project.channels[ci];
            let c = widgets::rgb(ch.color);
            ui.horizontal(|ui| {
                ui.label("Name");
                if ui.text_edit_singleline(&mut ch.name).changed() {
                    self.dirty = true;
                }
            });
            ui.horizontal(|ui| {
                ui.label("Instrument");
                egui::ComboBox::from_id_salt("kind").selected_text(ch.kind.label()).show_ui(ui, |ui| {
                    for k in InstKind::ALL {
                        if ui.selectable_label(ch.kind == k, k.label()).clicked() {
                            ch.kind = k;
                            ch.params = k.default_params();
                            self.dirty = true;
                        }
                    }
                });
            });
            ui.add_space(4.0);
            let drum = ch.kind.is_drum();
            ui.horizontal(|ui| {
                let mut knob = |ui: &mut egui::Ui, label: &str, v: &mut f32, min: f32, max: f32, def: f32| {
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 1.0;
                        if widgets::knob(ui, v, min, max, def, 34.0, c, label) {
                            self.dirty = true;
                        }
                        ui.label(RichText::new(label).size(10.0).color(col::DIM));
                    });
                };
                knob(ui, "VOL", &mut ch.volume, 0.0, 1.0, CHANNEL_DEFAULT_VOLUME);
                knob(ui, "PAN", &mut ch.pan, -1.0, 1.0, 0.0);
                knob(ui, "PITCH", &mut ch.pitch, -24.0, 24.0, 0.0);
                knob(ui, "TONE", &mut ch.params.tone, 0.0, 1.0, 0.5);
                if !drum {
                    knob(ui, "ATT", &mut ch.params.attack, 0.001, 2.0, 0.001);
                    knob(ui, "DEC", &mut ch.params.decay, 0.01, 3.0, 0.3);
                    knob(ui, "SUS", &mut ch.params.sustain, 0.0, 1.0, 0.0);
                    knob(ui, "REL", &mut ch.params.release, 0.01, 3.0, 0.1);
                    knob(ui, "CUT", &mut ch.params.cutoff, 0.0, 1.0, 1.0);
                    knob(ui, "RES", &mut ch.params.reso, 0.0, 0.95, 0.0);
                }
            });
            // little piano to audition
            ui.add_space(6.0);
            let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width().max(360.0), 40.0), Sense::click());
            let p = ui.painter();
            let keys = 24;
            let w = r.width() / keys as f32;
            for i in 0..keys {
                let k = 48 + i as u8;
                let black = matches!(k % 12, 1 | 3 | 6 | 8 | 10);
                let kr = Rect::from_min_size(pos2(r.left() + i as f32 * w, r.top()), vec2(w - 1.0, r.height()));
                p.rect_filled(kr, 2.0, if black { col::DARKER } else { Color32::from_rgb(0xd8, 0xdc, 0xde) });
            }
            if resp.clicked() {
                if let Some(pos) = resp.interact_pointer_pos() {
                    let i = ((pos.x - r.left()) / w) as u8;
                    self.send(Cmd::Preview { ch: ci, key: 48 + i });
                }
            }
        });
        self.show_channel = open;
    }
}
