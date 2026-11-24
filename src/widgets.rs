//! Custom painted controls: knobs, LEDs, LCD readouts, faders, meters.

use egui::{pos2, vec2, Align2, Color32, FontId, Rect, Response, Sense, Stroke, StrokeKind, Ui};

pub mod col {
    use egui::Color32;
    pub const BG: Color32 = Color32::from_rgb(0x2a, 0x31, 0x36);
    pub const PANEL: Color32 = Color32::from_rgb(0x35, 0x3e, 0x44);
    pub const PANEL_HI: Color32 = Color32::from_rgb(0x41, 0x4b, 0x52);
    pub const DARK: Color32 = Color32::from_rgb(0x22, 0x28, 0x2c);
    pub const DARKER: Color32 = Color32::from_rgb(0x1b, 0x20, 0x23);
    pub const GRID: Color32 = Color32::from_rgb(0x3a, 0x43, 0x49);
    pub const GRID_BAR: Color32 = Color32::from_rgb(0x52, 0x5d, 0x64);
    pub const TEXT: Color32 = Color32::from_rgb(0xc6, 0xd0, 0xd5);
    pub const DIM: Color32 = Color32::from_rgb(0x80, 0x8d, 0x94);
    pub const ACCENT: Color32 = Color32::from_rgb(0xf2, 0x9b, 0x3e);
    pub const GREEN: Color32 = Color32::from_rgb(0x86, 0xc8, 0x55);
    pub const RED: Color32 = Color32::from_rgb(0xe0, 0x58, 0x5c);
    pub const LCD: Color32 = Color32::from_rgb(0x12, 0x17, 0x19);
    pub const LCD_TEXT: Color32 = Color32::from_rgb(0xa8, 0xe6, 0x7a);
    pub const STEP_A: Color32 = Color32::from_rgb(0x4a, 0x54, 0x5b);
    pub const STEP_B: Color32 = Color32::from_rgb(0x5c, 0x47, 0x4c);
    pub const STEP_ON: Color32 = Color32::from_rgb(0xdf, 0xe7, 0xeb);
}

pub fn rgb(c: [u8; 3]) -> Color32 {
    Color32::from_rgb(c[0], c[1], c[2])
}

pub fn shade(c: Color32, f: f32) -> Color32 {
    let m = |v: u8| ((v as f32 * f).clamp(0.0, 255.0)) as u8;
    Color32::from_rgb(m(c.r()), m(c.g()), m(c.b()))
}

/// Rotary knob. Vertical drag changes value; double click resets. Returns true if changed.
#[allow(clippy::too_many_arguments)]
pub fn knob(ui: &mut Ui, value: &mut f32, min: f32, max: f32, default: f32, size: f32, color: Color32, tip: &str) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(size, size), Sense::click_and_drag());
    let mut changed = false;
    if resp.dragged() {
        let fine = if ui.input(|i| i.modifiers.shift) { 0.2 } else { 1.0 };
        let d = -resp.drag_delta().y / 150.0 * (max - min) * fine;
        *value = (*value + d).clamp(min, max);
        changed = true;
    }
    if resp.hovered() {
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll != 0.0 {
            *value = (*value + scroll / 400.0 * (max - min)).clamp(min, max);
            changed = true;
        }
    }
    if resp.double_clicked() {
        *value = default;
        changed = true;
    }
    let p = ui.painter();
    let c = rect.center();
    let r = size * 0.5 - 1.5;
    p.circle_filled(c, r, col::DARK);
    p.circle_stroke(c, r, Stroke::new(1.0_f32, col::GRID_BAR));
    let t = (*value - min) / (max - min);
    let a0 = std::f32::consts::PI * 0.75;
    let sweep = std::f32::consts::PI * 1.5;
    let bipolar = min < 0.0 && max > 0.0;
    let from = if bipolar { 0.5 } else { 0.0 };
    let (ta, tb) = if t < from { (t, from) } else { (from, t) };
    let steps = 24;
    let pts: Vec<_> = (0..=steps)
        .map(|i| {
            let tt = ta + (tb - ta) * i as f32 / steps as f32;
            let a = a0 + sweep * tt;
            c + vec2(a.cos(), a.sin()) * (r - 1.5)
        })
        .collect();
    if pts.len() > 1 {
        p.add(egui::Shape::line(pts, Stroke::new(2.2_f32, color)));
    }
    let a = a0 + sweep * t;
    p.line_segment([c + vec2(a.cos(), a.sin()) * (r * 0.25), c + vec2(a.cos(), a.sin()) * (r - 2.5)], Stroke::new(1.6_f32, col::TEXT));
    if !tip.is_empty() {
        resp.on_hover_text(format!("{tip}: {}", fmt_val(*value, min, max)));
    }
    changed
}

fn fmt_val(v: f32, min: f32, max: f32) -> String {
    if min < 0.0 && max > 0.0 && max <= 1.0 {
        if v.abs() < 0.01 {
            "centred".into()
        } else if v < 0.0 {
            format!("{:.0}% left", -v * 100.0)
        } else {
            format!("{:.0}% right", v * 100.0)
        }
    } else if max <= 1.0 {
        format!("{:.0}%", v * 100.0)
    } else {
        format!("{v:.2}")
    }
}

/// Round LED toggle. Returns clicked response.
pub fn led(ui: &mut Ui, on: bool, on_color: Color32, size: f32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(size, size), Sense::click());
    let p = ui.painter();
    let c = rect.center();
    p.circle_filled(c, size * 0.5 - 1.0, col::DARKER);
    p.circle_filled(c, size * 0.5 - 2.5, if on { on_color } else { shade(on_color, 0.25) });
    resp
}

/// LCD style readout box.
pub fn lcd(ui: &mut Ui, text: &str, width: f32, height: f32, size: f32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(width, height), Sense::click_and_drag());
    let p = ui.painter();
    p.rect_filled(rect, 3.0, col::LCD);
    p.rect_stroke(rect, 3.0, Stroke::new(1.0_f32, col::DARKER), StrokeKind::Inside);
    p.text(rect.center(), Align2::CENTER_CENTER, text, FontId::monospace(size), col::LCD_TEXT);
    resp
}

/// Small flat button with optional active state.
pub fn flat_button(ui: &mut Ui, text: &str, active: bool, w: f32, h: f32, active_color: Color32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(w, h), Sense::click());
    let p = ui.painter();
    let bg = if active {
        shade(active_color, 0.55)
    } else if resp.hovered() {
        col::PANEL_HI
    } else {
        col::DARK
    };
    p.rect_filled(rect, 3.0, bg);
    p.rect_stroke(rect, 3.0, Stroke::new(1.0_f32, if active { active_color } else { col::GRID }), StrokeKind::Inside);
    p.text(rect.center(), Align2::CENTER_CENTER, text, FontId::proportional(12.0), if active { Color32::WHITE } else { col::TEXT });
    resp
}

/// Vertical fader (0..1). Returns true if changed.
pub fn fader(ui: &mut Ui, value: &mut f32, default: f32, rect: Rect) -> bool {
    let resp = ui.interact(rect, ui.id().with(("fader", rect.min.x as i32, rect.min.y as i32)), Sense::click_and_drag());
    let mut changed = false;
    if resp.dragged() {
        let fine = if ui.input(|i| i.modifiers.shift) { 0.2 } else { 1.0 };
        *value = (*value - resp.drag_delta().y / rect.height() * fine).clamp(0.0, 1.0);
        changed = true;
    }
    if resp.double_clicked() {
        *value = default;
        changed = true;
    }
    let p = ui.painter();
    let track = Rect::from_center_size(rect.center(), vec2(4.0, rect.height() - 10.0));
    p.rect_filled(track, 2.0, col::DARKER);
    // unity mark
    let y_of = |v: f32| track.bottom() - v * track.height();
    let uy = y_of(default);
    p.line_segment([pos2(rect.left() + 2.0, uy), pos2(rect.right() - 2.0, uy)], Stroke::new(1.0_f32, col::DIM));
    let y = y_of(*value);
    let cap = Rect::from_center_size(pos2(rect.center().x, y), vec2(rect.width() - 4.0, 11.0));
    p.rect_filled(cap, 2.0, Color32::from_rgb(0x9a, 0xa6, 0xac));
    p.line_segment([pos2(cap.left() + 2.0, y), pos2(cap.right() - 2.0, y)], Stroke::new(1.0_f32, col::DARKER));
    resp.on_hover_text(format!("{:.1} dB", crate::model::fader_db(*value)));
    changed
}

/// Stereo peak meter.
pub fn meter(ui: &Ui, rect: Rect, peaks: [f32; 2]) {
    let p = ui.painter();
    p.rect_filled(rect, 1.0, col::DARKER);
    let w = (rect.width() - 1.0) / 2.0;
    for (i, pk) in peaks.iter().enumerate() {
        let db = 20.0 * pk.max(1e-5).log10();
        let t = ((db + 48.0) / 54.0).clamp(0.0, 1.0);
        let x = rect.left() + i as f32 * (w + 1.0);
        let h = rect.height() * t;
        let r = Rect::from_min_max(pos2(x, rect.bottom() - h), pos2(x + w, rect.bottom()));
        let c = if db > -0.5 {
            col::RED
        } else if db > -9.0 {
            Color32::from_rgb(0xe8, 0xc8, 0x4a)
        } else {
            col::GREEN
        };
        p.rect_filled(r, 0.0, c);
    }
}

pub fn panel_header(ui: &mut Ui, text: &str) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 18.0), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, col::DARK);
    ui.painter().text(rect.left_center() + vec2(6.0, 0.0), Align2::LEFT_CENTER, text, FontId::proportional(11.0), col::DIM);
}
