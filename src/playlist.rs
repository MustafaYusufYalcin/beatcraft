//! Playlist: arrange pattern clips on tracks.

use crate::app::{App, ClipDrag};
use crate::model::*;
use crate::widgets::{self, col};
use egui::{pos2, vec2, Align2, Color32, FontId, PointerButton, Rect, RichText, Sense, Stroke, StrokeKind};

const TRACK_H: f32 = 46.0;
const HEADER_W: f32 = 120.0;
const RULER_H: f32 = 20.0;
const SHOWN_TRACKS: usize = 24;

impl App {
    pub fn playlist(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().frame(egui::Frame::new().fill(col::BG)).show(ctx, |ui| {
            // toolbar
            let (bar, _) = ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::hover());
            ui.painter().rect_filled(bar, 0.0, col::DARK);
            let mut tb = ui.new_child(egui::UiBuilder::new().max_rect(bar.shrink2(vec2(6.0, 2.0))).layout(egui::Layout::left_to_right(egui::Align::Center)));
            tb.label(RichText::new("PLAYLIST  ·  Arrangement").color(col::TEXT).strong());
            tb.add_space(12.0);
            tb.label(RichText::new("Brush:").size(11.0).color(col::DIM));
            let pc = widgets::rgb(self.project.patterns[self.current_pattern].color);
            tb.label(RichText::new(format!("▦ {}", self.project.patterns[self.current_pattern].name)).color(pc));
            tb.add_space(12.0);
            tb.label(RichText::new("Zoom").size(11.0).color(col::DIM));
            tb.add(egui::Slider::new(&mut self.bar_px, 24.0..=160.0).show_value(false));
            tb.add_space(12.0);
            tb.label(
                RichText::new(format!(
                    "{} clips · {} bars · tracks 1–{} of {}",
                    self.project.clips.len(),
                    self.project.song_length_ticks() / (STEPS_PER_BAR * STEP_TICKS),
                    SHOWN_TRACKS,
                    PLAYLIST_TRACKS
                ))
                .size(11.0)
                .color(col::DIM),
            );

            let bar_ticks = (STEPS_PER_BAR * STEP_TICKS) as f32;
            let tick_px = self.bar_px / bar_ticks;
            let bars = (self.project.song_length_ticks() as f32 / bar_ticks) as usize + 16;
            let w = HEADER_W + bars as f32 * self.bar_px;
            let h = RULER_H + SHOWN_TRACKS as f32 * TRACK_H;
            egui::ScrollArea::both().id_salt("pl").auto_shrink([false, false]).show(ui, |ui| {
                let (rect, resp) = ui.allocate_exact_size(vec2(w, h), Sense::click_and_drag());
                let p = ui.painter().clone();
                let grid = Rect::from_min_max(pos2(rect.left() + HEADER_W, rect.top() + RULER_H), rect.max);
                // ruler
                let ruler = Rect::from_min_max(pos2(grid.left(), rect.top()), pos2(rect.right(), rect.top() + RULER_H));
                p.rect_filled(ruler, 0.0, col::DARK);
                for b in 0..bars {
                    let x = grid.left() + b as f32 * self.bar_px;
                    p.line_segment([pos2(x, ruler.top() + 8.0), pos2(x, ruler.bottom())], Stroke::new(1.0_f32, col::DIM));
                    if self.bar_px >= 30.0 || b % 2 == 0 {
                        p.text(pos2(x + 3.0, ruler.top() + 2.0), Align2::LEFT_TOP, (b + 1).to_string(), FontId::proportional(10.0), col::TEXT);
                    }
                }
                // song end marker
                let end_x = grid.left() + self.project.song_length_ticks() as f32 * tick_px;
                p.text(pos2(end_x + 2.0, ruler.bottom() - 2.0), Align2::LEFT_BOTTOM, "◣", FontId::proportional(10.0), col::ACCENT);
                // tracks
                for t in 0..SHOWN_TRACKS {
                    let y = grid.top() + t as f32 * TRACK_H;
                    let row = Rect::from_min_size(pos2(grid.left(), y), vec2(grid.width(), TRACK_H));
                    p.rect_filled(row, 0.0, if t % 2 == 0 { Color32::from_rgb(0x30, 0x38, 0x3d) } else { Color32::from_rgb(0x2c, 0x34, 0x39) });
                    for b in 0..bars * 4 {
                        let x = grid.left() + b as f32 * self.bar_px / 4.0;
                        let c = if b % 4 == 0 { col::GRID_BAR } else { col::GRID };
                        if b % 4 == 0 || self.bar_px >= 48.0 {
                            p.line_segment([pos2(x, y), pos2(x, y + TRACK_H)], Stroke::new(1.0_f32, c));
                        }
                    }
                    p.line_segment([pos2(grid.left(), y + TRACK_H), pos2(grid.right(), y + TRACK_H)], Stroke::new(1.0_f32, col::DARKER));
                    // header
                    let hr = Rect::from_min_size(pos2(rect.left(), y), vec2(HEADER_W - 2.0, TRACK_H - 1.0));
                    p.rect_filled(hr, 0.0, col::PANEL);
                    p.rect_filled(Rect::from_min_size(hr.min, vec2(3.0, hr.height())), 0.0, widgets::shade(col::GREEN, 0.7));
                    p.text(hr.left_center() + vec2(10.0, 0.0), Align2::LEFT_CENTER, &self.project.track_names[t], FontId::proportional(12.0), col::TEXT);
                }
                // clips
                let clip_rect = |c: &Clip| {
                    Rect::from_min_size(
                        pos2(grid.left() + c.start as f32 * tick_px, grid.top() + c.track as f32 * TRACK_H + 1.0),
                        vec2(c.len as f32 * tick_px - 1.0, TRACK_H - 3.0),
                    )
                };
                for (i, c) in self.project.clips.iter().enumerate() {
                    if c.track >= SHOWN_TRACKS {
                        continue;
                    }
                    let Some(pat) = self.project.patterns.get(c.pattern) else { continue };
                    let r = clip_rect(c);
                    let pc = widgets::rgb(pat.color);
                    let dragging = matches!(self.clip_drag, Some(ClipDrag::Move { idx, .. }) | Some(ClipDrag::Resize { idx }) if idx == i);
                    p.rect_filled(r, 3.0, widgets::shade(pc, 0.42));
                    let title = Rect::from_min_size(r.min, vec2(r.width(), 13.0));
                    p.rect_filled(title, 3.0, widgets::shade(pc, 0.8));
                    p.rect_stroke(r, 3.0, Stroke::new(1.0_f32, if dragging { Color32::WHITE } else { widgets::shade(pc, 0.95) }), StrokeKind::Inside);
                    let clip_p = p.with_clip_rect(r.shrink(1.0));
                    clip_p.text(title.left_center() + vec2(4.0, 0.0), Align2::LEFT_CENTER, &pat.name, FontId::proportional(10.0), col::DARKER);
                    // mini preview
                    let body = Rect::from_min_max(pos2(r.left() + 2.0, title.bottom() + 2.0), pos2(r.right() - 2.0, r.bottom() - 2.0));
                    let plen = pat.length_ticks().max(1);
                    let reps = c.len.div_ceil(plen);
                    let (lo, hi) = pat.data.iter().flat_map(|d| d.notes.iter()).fold((127u8, 0u8), |(a, b), n| (a.min(n.key), b.max(n.key)));
                    let span = (hi.saturating_sub(lo)).max(8) as f32;
                    let nch = pat.data.len().max(1) as f32;
                    for rep in 0..reps {
                        let off = rep * plen;
                        for (chi, d) in pat.data.iter().enumerate() {
                            for (si, v) in d.steps.iter().enumerate() {
                                if *v > 0.0 {
                                    let t = off + si as u32 * STEP_TICKS;
                                    if t >= c.len {
                                        continue;
                                    }
                                    let x = r.left() + t as f32 * tick_px;
                                    let y = body.top() + chi as f32 / nch * body.height();
                                    clip_p.rect_filled(
                                        Rect::from_min_size(pos2(x + 1.0, y), vec2((STEP_TICKS as f32 * tick_px - 1.0).max(1.0), (body.height() / nch - 1.0).max(1.0))),
                                        0.0,
                                        Color32::from_white_alpha(130),
                                    );
                                }
                            }
                            for n in &d.notes {
                                let t = off + n.start;
                                if t >= c.len {
                                    continue;
                                }
                                let x = r.left() + t as f32 * tick_px;
                                let y = body.bottom() - (n.key.saturating_sub(lo)) as f32 / span * body.height();
                                clip_p.line_segment([pos2(x, y), pos2(x + n.len as f32 * tick_px - 1.0, y)], Stroke::new(2.0_f32, Color32::from_white_alpha(150)));
                            }
                        }
                    }
                }
                // playhead
                if self.song_mode {
                    let x = grid.left() + self.pos_ticks as f32 * tick_px;
                    p.line_segment([pos2(x, rect.top()), pos2(x, rect.bottom())], Stroke::new(1.5_f32, col::ACCENT));
                    p.add(egui::Shape::convex_polygon(vec![pos2(x - 5.0, rect.top()), pos2(x + 5.0, rect.top()), pos2(x, rect.top() + 7.0)], col::ACCENT, Stroke::NONE));
                }
                // interaction
                let hit = |clips: &[Clip], pos: egui::Pos2| -> Option<(usize, bool)> {
                    for (i, c) in clips.iter().enumerate().rev() {
                        let r = clip_rect(c);
                        if r.contains(pos) {
                            return Some((i, pos.x > r.right() - 7.0));
                        }
                    }
                    None
                };
                let to_tick = |x: f32| ((x - grid.left()) / tick_px).max(0.0);
                let snap_bar = |t: f32| ((t / bar_ticks).floor() as u32) * bar_ticks as u32;
                if let Some(hp) = resp.hover_pos() {
                    if let Some((_, edge)) = hit(&self.project.clips, hp) {
                        ui.ctx().set_cursor_icon(if edge { egui::CursorIcon::ResizeHorizontal } else { egui::CursorIcon::Grab });
                    }
                }
                if let Some(pos) = resp.interact_pointer_pos() {
                    if pos.y < grid.top() && pos.x > grid.left() {
                        if resp.clicked() || resp.dragged() {
                            self.song_mode = true;
                            self.set_position(snap_bar(to_tick(pos.x)) as f64);
                        }
                    } else if pos.x < grid.left() {
                        // track header: nothing yet
                    } else {
                        let track = (((pos.y - grid.top()) / TRACK_H).floor() as usize).min(SHOWN_TRACKS - 1);
                        let start_drag = resp.drag_started_by(PointerButton::Primary);
                        if start_drag || resp.clicked() {
                            match hit(&self.project.clips, pos) {
                                Some((i, edge)) => {
                                    if start_drag {
                                        let c = self.project.clips[i];
                                        self.clip_drag =
                                            Some(if edge { ClipDrag::Resize { idx: i } } else { ClipDrag::Move { idx: i, grab_ticks: to_tick(pos.x) as i64 - c.start as i64 } });
                                    } else {
                                        // clicking a clip selects its pattern
                                        self.current_pattern = self.project.clips[i].pattern;
                                    }
                                }
                                None => {
                                    let pat = self.current_pattern;
                                    let len = self.project.patterns[pat].length_ticks();
                                    let start = snap_bar(to_tick(pos.x));
                                    self.project.clips.push(Clip { track, pattern: pat, start, len });
                                    if start_drag {
                                        self.clip_drag = Some(ClipDrag::Move { idx: self.project.clips.len() - 1, grab_ticks: (to_tick(pos.x) - start as f32) as i64 });
                                    }
                                    self.mark();
                                }
                            }
                        }
                        if resp.secondary_clicked() {
                            if let Some((i, _)) = hit(&self.project.clips, pos) {
                                self.project.clips.remove(i);
                                self.mark();
                            }
                        }
                        if resp.dragged_by(PointerButton::Primary) {
                            match self.clip_drag {
                                Some(ClipDrag::Move { idx, grab_ticks }) => {
                                    let t = (to_tick(pos.x) as i64 - grab_ticks).max(0) as f32;
                                    let start = ((t / bar_ticks).round() as u32) * bar_ticks as u32;
                                    if let Some(c) = self.project.clips.get_mut(idx) {
                                        if c.start != start || c.track != track {
                                            c.start = start;
                                            c.track = track;
                                            self.dirty = true;
                                        }
                                    }
                                }
                                Some(ClipDrag::Resize { idx }) => {
                                    if let Some(c) = self.project.clips.get_mut(idx) {
                                        let end = ((to_tick(pos.x) / (bar_ticks / 4.0)).round() as u32) * (bar_ticks as u32 / 4);
                                        let len = end.saturating_sub(c.start).max(bar_ticks as u32 / 4);
                                        if len != c.len {
                                            c.len = len;
                                            self.dirty = true;
                                        }
                                    }
                                }
                                None => {}
                            }
                        }
                    }
                }
                if resp.drag_stopped() {
                    self.clip_drag = None;
                }
            });
        });
    }
}
