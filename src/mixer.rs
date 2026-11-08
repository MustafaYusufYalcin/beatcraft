//! Mixer: Master + 125 inserts + Current, with an effect-slot rack for the selected track.

use crate::app::App;
use crate::model::*;
use crate::widgets::{self, col};
use egui::{pos2, vec2, Align2, Color32, FontId, Rect, RichText, Sense, Stroke, StrokeKind};

const STRIP_W: f32 = 50.0;

impl App {
    pub fn mixer_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("mixer")
            .resizable(true)
            .default_height(268.0)
            .min_height(200.0)
            .frame(egui::Frame::new().fill(col::PANEL).inner_margin(egui::Margin::same(0)))
            .show(ctx, |ui| {
                widgets::panel_header(ui, &format!("  MIXER  ·  Master + {MIXER_INSERTS} inserts  ·  selected: {}", self.project.mixer[self.selected_insert].name));
                ui.horizontal_top(|ui| {
                    let fx_w = 250.0;
                    let strips_w = ui.available_width() - fx_w - 8.0;
                    ui.allocate_ui(vec2(strips_w, ui.available_height()), |ui| {
                        egui::ScrollArea::horizontal().id_salt("mix_scroll").auto_shrink([false, false]).show(ui, |ui| {
                            ui.horizontal_top(|ui| {
                                ui.spacing_mut().item_spacing.x = 2.0;
                                for i in 0..MIXER_TRACKS {
                                    self.strip(ui, i);
                                    if i == 0 {
                                        ui.add_space(6.0);
                                    }
                                }
                            });
                        });
                    });
                    ui.separator();
                    self.fx_rack(ui);
                });
            });
    }

    fn strip(&mut self, ui: &mut egui::Ui, i: usize) {
        let h = ui.available_height() - 4.0;
        let master = i == 0;
        let w = if master { STRIP_W + 14.0 } else { STRIP_W };
        let (rect, resp) = ui.allocate_exact_size(vec2(w, h), Sense::click());
        let sel = self.selected_insert == i;
        // channels routed here
        let routed: Vec<Color32> = self.project.channels.iter().filter(|c| c.insert == i).map(|c| widgets::rgb(c.color)).collect();
        let p = ui.painter().clone();
        p.rect_filled(rect, 3.0, if sel { col::PANEL_HI } else if master { Color32::from_rgb(0x3b, 0x45, 0x4c) } else { col::DARK });
        if sel {
            p.rect_stroke(rect, 3.0, Stroke::new(1.0, col::ACCENT), StrokeKind::Inside);
        }
        if resp.clicked() {
            self.selected_insert = i;
        }
        let ins = &mut self.project.mixer[i];
        // number + name
        let label = if master { "M".to_string() } else if i == MIXER_TRACKS - 1 { "C".to_string() } else { i.to_string() };
        p.text(pos2(rect.center().x, rect.top() + 8.0), Align2::CENTER_CENTER, label, FontId::monospace(10.0), col::DIM);
        let name_r = Rect::from_min_size(pos2(rect.left() + 2.0, rect.top() + 15.0), vec2(rect.width() - 4.0, 14.0));
        p.with_clip_rect(name_r).text(name_r.center(), Align2::CENTER_CENTER, &ins.name, FontId::proportional(10.0), if routed.is_empty() && !master { col::DIM } else { col::TEXT });
        // routing colours
        for (k, c) in routed.iter().take(4).enumerate() {
            p.rect_filled(Rect::from_min_size(pos2(rect.left() + 4.0 + k as f32 * 6.0, rect.top() + 31.0), vec2(5.0, 3.0)), 0.0, *c);
        }
        // fx count dots
        let nfx = ins.slots.iter().filter(|s| s.is_some()).count();
        if nfx > 0 {
            p.text(pos2(rect.right() - 4.0, rect.top() + 33.0), Align2::RIGHT_CENTER, format!("fx{nfx}"), FontId::proportional(8.0), col::ACCENT);
        }
        // pan knob
        let knob_rect = Rect::from_center_size(pos2(rect.center().x, rect.top() + 50.0), vec2(22.0, 22.0));
        let mut kui = ui.new_child(egui::UiBuilder::new().max_rect(knob_rect));
        if widgets::knob(&mut kui, &mut ins.pan, -1.0, 1.0, 0.0, 22.0, col::ACCENT, "Pan") {
            self.dirty = true;
        }
        // fader + meter
        let top = rect.top() + 66.0;
        let bottom = rect.bottom() - 26.0;
        let fr = Rect::from_min_max(pos2(rect.left() + 3.0, top), pos2(rect.left() + 3.0 + 20.0, bottom));
        if widgets::fader(ui, &mut ins.volume, FADER_UNITY, fr) {
            self.dirty = true;
        }
        let mr = Rect::from_min_max(pos2(fr.right() + 4.0, top + 4.0), pos2(rect.right() - 5.0, bottom - 4.0));
        widgets::meter(ui, mr, self.meters.get(i).copied().unwrap_or([0.0; 2]));
        // mute / solo
        let mbr = Rect::from_min_size(pos2(rect.left() + 3.0, rect.bottom() - 22.0), vec2(w / 2.0 - 4.0, 18.0));
        let sbr = Rect::from_min_size(pos2(rect.center().x + 1.0, rect.bottom() - 22.0), vec2(w / 2.0 - 4.0, 18.0));
        let mresp = ui.interact(mbr, ui.id().with(("mute", i)), Sense::click());
        let sresp = ui.interact(sbr, ui.id().with(("solo", i)), Sense::click());
        p.rect_filled(mbr, 2.0, if !ins.mute { widgets::shade(col::GREEN, 0.6) } else { col::DARKER });
        p.text(mbr.center(), Align2::CENTER_CENTER, "ON", FontId::proportional(9.0), if !ins.mute { Color32::WHITE } else { col::DIM });
        p.rect_filled(sbr, 2.0, if ins.solo { widgets::shade(col::ACCENT, 0.7) } else { col::DARKER });
        p.text(sbr.center(), Align2::CENTER_CENTER, "S", FontId::proportional(9.0), if ins.solo { Color32::WHITE } else { col::DIM });
        if mresp.on_hover_text("Track on / off").clicked() {
            ins.mute = !ins.mute;
            self.dirty = true;
        }
        if !master && sresp.on_hover_text("Solo").clicked() {
            ins.solo = !ins.solo;
            self.dirty = true;
        }
    }

    fn fx_rack(&mut self, ui: &mut egui::Ui) {
        let i = self.selected_insert;
        ui.vertical(|ui| {
            ui.set_width(240.0);
            ui.add_space(4.0);
            let ins = &mut self.project.mixer[i];
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("{}", if i == 0 { "MASTER".to_string() } else { format!("INSERT {i}") })).color(col::ACCENT).strong());
                if i != 0 && i != MIXER_TRACKS - 1 && ui.add(egui::TextEdit::singleline(&mut ins.name).desired_width(120.0)).changed() {
                    self.dirty = true;
                }
            });
            ui.label(RichText::new(format!("{:+.1} dB · {} effect slots", fader_db(ins.volume), FX_SLOTS)).size(10.0).color(col::DIM));
            ui.add_space(2.0);
            egui::ScrollArea::vertical().id_salt("fxslots").max_height(112.0).show(ui, |ui| {
                for s in 0..FX_SLOTS {
                    ui.horizontal(|ui| {
                        let cur = ins.slots[s];
                        let (r, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
                        ui.painter().circle_filled(r.center(), 3.5, if cur.is_some() { col::GREEN } else { col::DARKER });
                        egui::ComboBox::from_id_salt(("slot", i, s))
                            .width(170.0)
                            .selected_text(RichText::new(cur.map(|f| f.label()).unwrap_or("(empty)")).size(11.0).color(if cur.is_some() { col::TEXT } else { col::DIM }))
                            .show_ui(ui, |ui| {
                                if ui.selectable_label(cur.is_none(), "(empty)").clicked() {
                                    ins.slots[s] = None;
                                    self.dirty = true;
                                }
                                for f in [FxKind::Filter, FxKind::Delay, FxKind::Reverb, FxKind::Limiter] {
                                    if ui.selectable_label(cur == Some(f), f.label()).clicked() {
                                        ins.slots[s] = Some(f);
                                        self.dirty = true;
                                    }
                                }
                            });
                    });
                }
            });
            ui.add_space(4.0);
            let has = |k: FxKind| ins.slots.iter().any(|x| *x == Some(k));
            let (hf, hd, hr) = (has(FxKind::Filter), has(FxKind::Delay), has(FxKind::Reverb));
            ui.horizontal(|ui| {
                for (label, v, on, c) in [
                    ("CUTOFF", &mut ins.lowpass, hf, Color32::from_rgb(0x7a, 0xb8, 0xe8)),
                    ("DELAY", &mut ins.delay, hd, col::GREEN),
                    ("REVERB", &mut ins.reverb, hr, Color32::from_rgb(0xb0, 0x8a, 0xe0)),
                ] {
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 1.0;
                        let def = if label == "CUTOFF" { 1.0 } else { 0.0 };
                        ui.add_enabled_ui(on, |ui| {
                            if widgets::knob(ui, v, 0.0, 1.0, def, 30.0, if on { c } else { col::DIM }, label) {
                                self.dirty = true;
                            }
                        });
                        ui.label(RichText::new(label).size(9.0).color(if on { col::TEXT } else { col::DIM }));
                    });
                    ui.add_space(6.0);
                }
            });
            let _ = Rect::NOTHING;
        });
    }
}
