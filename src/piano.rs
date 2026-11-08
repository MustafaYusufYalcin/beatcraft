//! Piano roll editor for the selected channel in the current pattern.

use crate::app::{App, NoteDrag};
use crate::engine::Cmd;
use crate::model::*;
use crate::widgets::{self, col};
use egui::{pos2, vec2, Align2, Color32, FontId, PointerButton, Rect, RichText, Sense, Stroke, StrokeKind};

const KEY_H: f32 = 11.0;
const LOW_KEY: u8 = 24;
const HIGH_KEY: u8 = 108;
const KEYB_W: f32 = 46.0;
const VEL_H: f32 = 56.0;

impl App {
    pub fn piano_roll(&mut self, ctx: &egui::Context) {
        let ci = self.selected_channel;
        if ci >= self.project.channels.len() {
            return;
        }
        let mut open = self.show_piano;
        let title = format!("Piano roll  ·  {}  ·  {}", self.project.channels[ci].name, self.project.patterns[self.current_pattern].name);
        egui::Window::new(title)
            .id(egui::Id::new("piano"))
            .open(&mut open)
            .default_pos(pos2(860.0, 120.0))
            .default_size(vec2(640.0, 400.0))
            .resizable(true)
            .show(ctx, |ui| self.piano_contents(ui));
        self.show_piano = open;
    }

    fn piano_contents(&mut self, ui: &mut egui::Ui) {
        let ci = self.selected_channel;
        let pi = self.current_pattern;
        let ccol = widgets::rgb(self.project.channels[ci].color);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Channel").size(11.0).color(col::DIM));
            egui::ComboBox::from_id_salt("pr_ch").selected_text(self.project.channels[ci].name.clone()).width(110.0).show_ui(ui, |ui| {
                for (i, c) in self.project.channels.iter().enumerate() {
                    if ui.selectable_label(i == ci, &c.name).clicked() {
                        self.selected_channel = i;
                    }
                }
            });
            ui.label(RichText::new("Zoom").size(11.0).color(col::DIM));
            ui.add(egui::Slider::new(&mut self.piano_zoom, 0.5..=2.5).show_value(false));
            ui.label(RichText::new("Snap: 1/16 step").size(11.0).color(col::DIM));
            if ui.button("Quantize").on_hover_text("Snap all note starts to the step grid").clicked() {
                for n in &mut self.project.patterns[pi].data[ci].notes {
                    n.start = (n.start + STEP_TICKS / 2) / STEP_TICKS * STEP_TICKS;
                }
                self.mark();
            }
            if ui.button("Clear").clicked() {
                self.project.patterns[pi].data[ci].notes.clear();
                self.mark();
            }
        });
        let step_px = 18.0 * self.piano_zoom;
        let tick_px = step_px / STEP_TICKS as f32;
        let pat = &self.project.patterns[pi];
        let pat_end = pat.length_ticks();
        let len_ticks = pat_end.max(pat.steps as u32 * STEP_TICKS) + STEPS_PER_BAR * STEP_TICKS;
        let grid_w = len_ticks as f32 * tick_px;
        let keys = (HIGH_KEY - LOW_KEY) as f32;
        let grid_h = keys * KEY_H;
        let avail_h = ui.available_height() - VEL_H - 6.0;
        let scroll = egui::ScrollArea::both().id_salt("pr_scroll").max_height(avail_h.max(120.0)).auto_shrink([false, false]);
        // centre the view on this channel's notes whenever the channel or pattern changes
        let init_id = egui::Id::new("pr_scroll_for");
        let want = (ci, pi);
        let seen = ui.ctx().data(|d| d.get_temp::<(usize, usize)>(init_id));
        let scroll = if seen != Some(want) {
            ui.ctx().data_mut(|d| d.insert_temp(init_id, want));
            let notes = &self.project.patterns[pi].data[ci].notes;
            let centre = if notes.is_empty() { 66.0 } else { notes.iter().map(|n| n.key as f32).sum::<f32>() / notes.len() as f32 };
            let y = (HIGH_KEY as f32 - 1.0 - centre) * KEY_H - avail_h.max(120.0) * 0.5;
            scroll.vertical_scroll_offset(y.max(0.0))
        } else {
            scroll
        };
        let mut hscroll = 0.0;
        let out = scroll.show(ui, |ui| {
            let (rect, resp) = ui.allocate_exact_size(vec2(KEYB_W + grid_w, grid_h), Sense::click_and_drag());
            let p = ui.painter().clone();
            let grid = Rect::from_min_max(pos2(rect.left() + KEYB_W, rect.top()), rect.max);
            let key_y = |k: u8| rect.top() + (HIGH_KEY - 1 - k) as f32 * KEY_H;
            // rows
            for k in LOW_KEY..HIGH_KEY {
                let y = key_y(k);
                let black = matches!(k % 12, 1 | 3 | 6 | 8 | 10);
                let row = Rect::from_min_size(pos2(grid.left(), y), vec2(grid.width(), KEY_H));
                p.rect_filled(row, 0.0, if black { col::DARK } else { Color32::from_rgb(0x2f, 0x37, 0x3c) });
                if k % 12 == 0 {
                    p.line_segment([pos2(grid.left(), y + KEY_H), pos2(grid.right(), y + KEY_H)], Stroke::new(1.0, col::GRID_BAR));
                }
                // keyboard
                let kr = Rect::from_min_size(pos2(rect.left(), y), vec2(KEYB_W - 2.0, KEY_H - 0.5));
                p.rect_filled(kr, 1.0, if black { col::DARKER } else { Color32::from_rgb(0xcf, 0xd5, 0xd8) });
                if k % 12 == 0 {
                    p.text(kr.right_center() - vec2(3.0, 0.0), Align2::RIGHT_CENTER, key_name(k), FontId::proportional(8.5), col::DARKER);
                }
            }
            // columns
            let total_steps = len_ticks / STEP_TICKS;
            for s in 0..=total_steps {
                let x = grid.left() + s as f32 * step_px;
                let c = if s % STEPS_PER_BAR == 0 { col::GRID_BAR } else if s % 4 == 0 { Color32::from_rgb(0x44, 0x4e, 0x55) } else { Color32::from_rgb(0x36, 0x3f, 0x45) };
                p.line_segment([pos2(x, grid.top()), pos2(x, grid.bottom())], Stroke::new(1.0, c));
            }
            // pattern end marker
            let pend = grid.left() + pat_end as f32 * tick_px;
            p.rect_filled(Rect::from_min_max(pos2(pend, grid.top()), grid.max), 0.0, Color32::from_black_alpha(60));
            // ghost notes from other channels
            for (oc, d) in self.project.patterns[pi].data.iter().enumerate() {
                if oc == ci {
                    continue;
                }
                for n in &d.notes {
                    if n.key < LOW_KEY || n.key >= HIGH_KEY {
                        continue;
                    }
                    let r = Rect::from_min_size(pos2(grid.left() + n.start as f32 * tick_px, key_y(n.key) + 1.0), vec2(n.len as f32 * tick_px - 1.0, KEY_H - 2.0));
                    p.rect_filled(r, 2.0, Color32::from_white_alpha(14));
                }
            }
            // notes
            let notes = &self.project.patterns[pi].data[ci].notes;
            let note_rect = |n: &Note| Rect::from_min_size(pos2(grid.left() + n.start as f32 * tick_px, key_y(n.key) + 0.5), vec2((n.len as f32 * tick_px - 1.0).max(3.0), KEY_H - 1.0));
            for (i, n) in notes.iter().enumerate() {
                if n.key < LOW_KEY || n.key >= HIGH_KEY {
                    continue;
                }
                let r = note_rect(n);
                let active = matches!(self.note_drag, Some(NoteDrag::Move { idx, .. }) | Some(NoteDrag::Resize { idx }) if idx == i);
                p.rect_filled(r, 2.0, widgets::shade(ccol, 0.6 + 0.4 * n.vel));
                p.rect_stroke(r, 2.0, Stroke::new(1.0, if active { Color32::WHITE } else { widgets::shade(ccol, 0.4) }), StrokeKind::Inside);
                if r.width() > 26.0 {
                    p.text(r.left_center() + vec2(3.0, 0.0), Align2::LEFT_CENTER, key_name(n.key), FontId::proportional(8.5), col::DARKER);
                }
            }
            // playhead
            if self.playing && !self.song_mode {
                let x = grid.left() + (self.pos_ticks as f32 % self.project.patterns[pi].length_ticks() as f32) * tick_px;
                p.line_segment([pos2(x, grid.top()), pos2(x, grid.bottom())], Stroke::new(1.5, col::ACCENT));
            }
            // interaction
            let hit = |notes: &[Note], pos: egui::Pos2| -> Option<(usize, bool)> {
                for (i, n) in notes.iter().enumerate().rev() {
                    let r = note_rect(n);
                    if r.contains(pos) {
                        return Some((i, pos.x > r.right() - 6.0));
                    }
                }
                None
            };
            let to_tick = |x: f32| ((x - grid.left()) / tick_px).max(0.0);
            let to_key = |y: f32| (HIGH_KEY as f32 - 1.0 - ((y - rect.top()) / KEY_H).floor()).clamp(LOW_KEY as f32, HIGH_KEY as f32 - 1.0) as u8;
            if let Some(hp) = resp.hover_pos() {
                if hp.x > grid.left() {
                    if let Some((_, edge)) = hit(&self.project.patterns[pi].data[ci].notes, hp) {
                        ui.ctx().set_cursor_icon(if edge { egui::CursorIcon::ResizeHorizontal } else { egui::CursorIcon::Grab });
                    }
                }
            }
            let snap = |t: f32| ((t / STEP_TICKS as f32).floor() as u32) * STEP_TICKS;
            if let Some(pos) = resp.interact_pointer_pos() {
                if pos.x < grid.left() {
                    if resp.clicked() {
                        self.send(Cmd::Preview { ch: ci, key: to_key(pos.y) });
                    }
                } else {
                    let start_drag = resp.drag_started_by(PointerButton::Primary);
                    if start_drag || resp.clicked() {
                        match hit(&self.project.patterns[pi].data[ci].notes, pos) {
                            Some((i, edge)) => {
                                if start_drag {
                                    let n = self.project.patterns[pi].data[ci].notes[i];
                                    self.note_drag = Some(if edge {
                                        NoteDrag::Resize { idx: i }
                                    } else {
                                        NoteDrag::Move { idx: i, grab_ticks: to_tick(pos.x) as i64 - n.start as i64, grab_key: to_key(pos.y) as i32 - n.key as i32 }
                                    });
                                }
                            }
                            None => {
                                let key = to_key(pos.y);
                                let start = snap(to_tick(pos.x));
                                let notes = &mut self.project.patterns[pi].data[ci].notes;
                                notes.push(Note { key, start, len: self.last_note_len, vel: 0.8 });
                                let i = notes.len() - 1;
                                if start_drag {
                                    self.note_drag = Some(NoteDrag::Move { idx: i, grab_ticks: (to_tick(pos.x) - start as f32) as i64, grab_key: 0 });
                                }
                                self.send(Cmd::Preview { ch: ci, key });
                                self.mark();
                            }
                        }
                    }
                    if resp.secondary_clicked() {
                        if let Some((i, _)) = hit(&self.project.patterns[pi].data[ci].notes, pos) {
                            self.project.patterns[pi].data[ci].notes.remove(i);
                            self.mark();
                        }
                    }
                    if resp.dragged_by(PointerButton::Primary) {
                        match self.note_drag {
                            Some(NoteDrag::Move { idx, grab_ticks, grab_key }) => {
                                let t = (to_tick(pos.x) as i64 - grab_ticks).max(0) as f32;
                                let start = ((t / STEP_TICKS as f32).round() as u32) * STEP_TICKS;
                                let key = (to_key(pos.y) as i32 - grab_key).clamp(LOW_KEY as i32, HIGH_KEY as i32 - 1) as u8;
                                let old_key = self.project.patterns[pi].data[ci].notes.get(idx).map(|n| n.key);
                                if let Some(ok) = old_key {
                                    if ok != key {
                                        self.send(Cmd::Preview { ch: ci, key });
                                    }
                                    let n = &mut self.project.patterns[pi].data[ci].notes[idx];
                                    n.start = start;
                                    n.key = key;
                                    self.mark();
                                }
                            }
                            Some(NoteDrag::Resize { idx }) => {
                                if let Some(n) = self.project.patterns[pi].data[ci].notes.get_mut(idx) {
                                    let end = ((to_tick(pos.x) / STEP_TICKS as f32).round() as u32) * STEP_TICKS;
                                    n.len = end.saturating_sub(n.start).max(STEP_TICKS / 2);
                                    self.last_note_len = n.len;
                                    self.mark();
                                }
                            }
                            None => {}
                        }
                    }
                }
            }
            if resp.drag_stopped() {
                self.note_drag = None;
            }
        });
        hscroll = out.state.offset.x + hscroll;
        // velocity lane, aligned with the grid
        let (vr, vresp) = ui.allocate_exact_size(vec2(ui.available_width(), VEL_H), Sense::click_and_drag());
        let p = ui.painter();
        p.rect_filled(vr, 2.0, col::DARKER);
        p.text(vr.left_top() + vec2(4.0, 3.0), Align2::LEFT_TOP, "VELOCITY", FontId::proportional(9.0), col::DIM);
        let gx = vr.left() + KEYB_W - hscroll;
        let clip = p.with_clip_rect(Rect::from_min_max(pos2(vr.left() + KEYB_W, vr.top()), vr.max));
        for n in &self.project.patterns[pi].data[ci].notes {
            let x = gx + n.start as f32 * tick_px;
            let h = (VEL_H - 8.0) * n.vel;
            clip.line_segment([pos2(x + 1.0, vr.bottom() - 2.0), pos2(x + 1.0, vr.bottom() - 2.0 - h)], Stroke::new(2.0, ccol));
            clip.circle_filled(pos2(x + 1.0, vr.bottom() - 2.0 - h), 2.5, ccol);
        }
        if vresp.dragged() || vresp.clicked() {
            if let Some(pos) = vresp.interact_pointer_pos() {
                let t = (pos.x - gx) / tick_px;
                let v = ((vr.bottom() - 2.0 - pos.y) / (VEL_H - 8.0)).clamp(0.05, 1.0);
                for n in &mut self.project.patterns[pi].data[ci].notes {
                    if (n.start as f32 - t).abs() < STEP_TICKS as f32 * 0.5 {
                        n.vel = v;
                        self.dirty = true;
                    }
                }
            }
        }
    }
}

