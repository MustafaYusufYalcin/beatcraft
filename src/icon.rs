//! BeatCraft app icon, drawn in code so every platform package can use it.

/// RGBA pixels of a square icon.
#[cfg(not(target_arch = "wasm32"))]
pub fn pixels(size: u32) -> Vec<u8> {
    let s = size as f32;
    let mut out = vec![0u8; (size * size * 4) as usize];
    let radius = s * 0.22;
    let margin = s * 0.06;
    // step-grid motif: 4 rows x 4 columns, lit cells form a rising pattern
    let lit = [[1, 0, 1, 0], [0, 1, 0, 1], [1, 1, 0, 1], [1, 0, 1, 1]];
    let row_cols: [[u8; 3]; 4] = [[242, 155, 62], [134, 200, 85], [122, 184, 232], [224, 88, 92]];
    for y in 0..size {
        for x in 0..size {
            let fx = x as f32 + 0.5;
            let fy = y as f32 + 0.5;
            // rounded square mask with soft edge
            let lo = margin;
            let hi = s - margin;
            let cx = fx.clamp(lo + radius, hi - radius);
            let cy = fy.clamp(lo + radius, hi - radius);
            let d = ((fx - cx).powi(2) + (fy - cy).powi(2)).sqrt() - radius;
            let alpha = (0.5 - d).clamp(0.0, 1.0);
            if alpha <= 0.0 {
                continue;
            }
            // background gradient
            let t = (fy - lo) / (hi - lo);
            let mut r = 52.0 - 22.0 * t;
            let mut g = 62.0 - 24.0 * t;
            let mut b = 70.0 - 24.0 * t;
            // grid cells
            let inner = hi - lo;
            let gx = (fx - lo - inner * 0.14) / (inner * 0.72);
            let gy = (fy - lo - inner * 0.14) / (inner * 0.72);
            if (0.0..1.0).contains(&gx) && (0.0..1.0).contains(&gy) {
                let col = (gx * 4.0) as usize;
                let row = (gy * 4.0) as usize;
                let lx = (gx * 4.0).fract();
                let ly = (gy * 4.0).fract();
                if lx > 0.1 && lx < 0.9 && ly > 0.1 && ly < 0.9 {
                    if lit[row][col] == 1 {
                        let c = row_cols[row];
                        r = c[0] as f32;
                        g = c[1] as f32;
                        b = c[2] as f32;
                    } else {
                        r = 74.0;
                        g = 84.0;
                        b = 91.0;
                    }
                }
            }
            let i = ((y * size + x) * 4) as usize;
            out[i] = r as u8;
            out[i + 1] = g as u8;
            out[i + 2] = b as u8;
            out[i + 3] = (alpha * 255.0) as u8;
        }
    }
    out
}

#[cfg(not(target_arch = "wasm32"))]
pub fn icon_data() -> egui::IconData {
    egui::IconData { rgba: pixels(256), width: 256, height: 256 }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn write_png(path: &std::path::Path, size: u32) -> Result<(), String> {
    image::save_buffer(path, &pixels(size), size, size, image::ColorType::Rgba8).map_err(|e| e.to_string())
}
