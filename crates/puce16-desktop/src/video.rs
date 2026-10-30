//! Pure video helpers for desktop rendering (testable without a window).

/// Pixel aspect ratio of the displayed frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ratio {
    /// Square pixels (default).
    Square,
    /// 8:7 pixel aspect: display width multiplied by 8/7.
    Par87,
}

/// Video filter mode.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Filter {
    Nearest,
    Linear,
    Scanlines,
}

impl Filter {
    /// Parse a filter name string; unknown names default to Nearest.
    pub fn from_name(s: &str) -> Filter {
        match s {
            "linear" => Filter::Linear,
            "scanlines" => Filter::Scanlines,
            _ => Filter::Nearest,
        }
    }

    /// Return the canonical name of this filter.
    pub fn name(self) -> &'static str {
        match self {
            Filter::Nearest => "nearest",
            Filter::Linear => "linear",
            Filter::Scanlines => "scanlines",
        }
    }

    /// Cycle to the next filter: Nearest -> Linear -> Scanlines -> Nearest.
    pub fn next(self) -> Filter {
        match self {
            Filter::Nearest => Filter::Linear,
            Filter::Linear => Filter::Scanlines,
            Filter::Scanlines => Filter::Nearest,
        }
    }
}

/// Parse a ratio name string; unknown names default to Square.
pub fn ratio_from_name(s: &str) -> Ratio {
    if s == "par87" {
        Ratio::Par87
    } else {
        Ratio::Square
    }
}

/// Return the canonical name of a ratio.
pub fn ratio_name(r: Ratio) -> &'static str {
    match r {
        Ratio::Square => "square",
        Ratio::Par87 => "par87",
    }
}

/// True only when scanlines should be applied (filter is Scanlines and scale >= 2).
pub fn scanlines_active(f: Filter, scale: usize) -> bool {
    f == Filter::Scanlines && scale >= 2
}

/// Darken odd rows of an RGBA image to simulate CRT scanlines.
/// Odd rows have R, G, B multiplied by 3/4 (integer division); alpha is unchanged.
pub fn apply_scanlines(rgba: &mut [u8], width: usize) {
    if rgba.is_empty() || width == 0 {
        return;
    }
    let height = rgba.len() / (width * 4);
    for y in 1..height {
        if y % 2 != 0 {
            let row_start = y * width * 4;
            let row_end = row_start + width * 4;
            let row = &mut rgba[row_start..row_end];
            for i in (0..row.len()).step_by(4) {
                row[i] = (row[i] as u16 * 3 / 4) as u8; // R
                row[i + 1] = (row[i + 1] as u16 * 3 / 4) as u8; // G
                row[i + 2] = (row[i + 2] as u16 * 3 / 4) as u8; // B
            }
        }
    }
}

/// Convert an XRGB8888 frame to RGBA bytes.
///
/// Each source pixel `p` produces `[R, G, B, 255]`: R = bits 23-16,
/// G = bits 15-8, B = bits 7-0; the high byte (X) of each pixel is ignored.
pub fn xrgb_to_rgba(src: &[u32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(src.len() * 4);
    for &p in src {
        out.push((p >> 16) as u8); // R
        out.push((p >> 8) as u8); // G
        out.push(p as u8); // B
        out.push(255); // A
    }
    out
}

/// Display size in screen points for a frame of `w` x `h` pixels.
///
/// Width: `w * scale`, multiplied by 8/7 (integer, rounded down) when the
/// ratio is [Ratio::Par87]. Height: `h * scale`.
pub fn display_size(w: usize, h: usize, scale: usize, ratio: Ratio) -> (usize, usize) {
    let width = match ratio {
        Ratio::Square => w * scale,
        Ratio::Par87 => w * scale * 8 / 7,
    };
    (width, h * scale)
}

/// Largest integer scale >= 1 such that `w * s <= avail_w` and
/// `h * s <= avail_h` (minimum 1).
pub fn fit_scale(w: usize, h: usize, avail_w: usize, avail_h: usize) -> usize {
    let by_width = if w == 0 { usize::MAX } else { avail_w / w };
    let by_height = if h == 0 { usize::MAX } else { avail_h / h };
    by_width.min(by_height).max(1)
}

use crate::filters_data::{Coefs, INTERP_MEDIUM, INTERP_SHARP, INTERP_SOFT, SCANLINES};
use crate::filters_data::{SCANLINE_MAX, SCANLINE_MIN, SCANLINE_STEP};

/// SNES interpolation filter (MiSTer "SNES Interpolation") used to scale the image.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Interp {
    Off,
    Soft,
    Medium,
    Sharp,
}

impl Interp {
    /// Parse a name; unknown names default to Off.
    pub fn from_name(s: &str) -> Interp {
        match s {
            "soft" => Interp::Soft,
            "medium" => Interp::Medium,
            "sharp" => Interp::Sharp,
            _ => Interp::Off,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Interp::Off => "off",
            Interp::Soft => "soft",
            Interp::Medium => "medium",
            Interp::Sharp => "sharp",
        }
    }

    /// Coefficient table of this interpolation (None when off).
    pub fn table(self) -> Option<&'static Coefs> {
        match self {
            Interp::Off => None,
            Interp::Soft => Some(&INTERP_SOFT),
            Interp::Medium => Some(&INTERP_MEDIUM),
            Interp::Sharp => Some(&INTERP_SHARP),
        }
    }
}

/// Maximum scanline strength shown in the GUI, in percent (gap darkness).
pub const SCAN_STRENGTH_MAX: u32 = 90;

/// Scanline table for a darkness of `strength_pct` percent (0 = no scanlines -> None).
/// MiSTer files are named by the brightness of the gap, so strength 55 selects `Scanlines_45`.
/// The strength is rounded to the nearest available step (5 %).
pub fn scanline_table(strength_pct: u32) -> Option<&'static Coefs> {
    if strength_pct == 0 {
        return None;
    }
    let strength = strength_pct.min(SCAN_STRENGTH_MAX);
    let bright = 100 - strength;
    let rounded = ((bright + SCANLINE_STEP / 2) / SCANLINE_STEP * SCANLINE_STEP)
        .clamp(SCANLINE_MIN, SCANLINE_MAX);
    Some(&SCANLINES[((rounded - SCANLINE_MIN) / SCANLINE_STEP) as usize])
}

/// For each output index `0..n_out` (source length `n_in`, integer `scale`), the index `i` of
/// the first source sample of the interval and the filter phase (0..64). Output index `o`
/// samples the source at `(o + 0.5) / scale - 0.5`; the taps apply to `i-1 ..= i+2`.
fn axis_phases(n_in: usize, scale: usize) -> Vec<(i64, usize)> {
    let _ = n_in;
    let den = 2 * scale as i64;
    (0..n_in * scale)
        .map(|o| {
            let num = (2 * o + 1) as i64 - scale as i64; // = 2 * scale * position
            let i = num.div_euclid(den);
            let frac = num.rem_euclid(den);
            (i, ((frac * 64) / den).clamp(0, 63) as usize)
        })
        .collect()
}

/// Scale an RGBA image by `scale`, filtering each axis with a MiSTer 64-phase 4-tap table:
/// `h` horizontally (None = width unchanged), `v` vertically (None = height unchanged).
/// Source samples outside the image are clamped to the edge. Returns the pixels and the new
/// width and height. With `scale < 2` or both tables None the image is returned unchanged.
pub fn filter_scale(
    rgba: &[u8],
    width: usize,
    scale: usize,
    h: Option<&Coefs>,
    v: Option<&Coefs>,
) -> (Vec<u8>, usize, usize) {
    if width == 0 || rgba.is_empty() {
        return (rgba.to_vec(), width, 0);
    }
    let height = rgba.len() / (width * 4);
    if scale < 2 || (h.is_none() && v.is_none()) {
        return (rgba.to_vec(), width, height);
    }

    // Horizontal pass.
    let (mid, mid_w) = match h {
        Some(coef) => {
            let phases = axis_phases(width, scale);
            let out_w = width * scale;
            let mut out = Vec::with_capacity(out_w * height * 4);
            for y in 0..height {
                let row = &rgba[y * width * 4..(y + 1) * width * 4];
                for &(i, ph) in &phases {
                    let c = coef[ph];
                    let px = |t: i64| ((i - 1 + t).clamp(0, width as i64 - 1) as usize) * 4;
                    let (p0, p1, p2, p3) = (px(0), px(1), px(2), px(3));
                    for ch in 0..3 {
                        let sum = c[0] * row[p0 + ch] as i32
                            + c[1] * row[p1 + ch] as i32
                            + c[2] * row[p2 + ch] as i32
                            + c[3] * row[p3 + ch] as i32;
                        out.push(((sum + 64) >> 7).clamp(0, 255) as u8);
                    }
                    out.push(255);
                }
            }
            (out, out_w)
        }
        None => (rgba.to_vec(), width),
    };

    // Vertical pass.
    match v {
        Some(coef) => {
            let phases = axis_phases(height, scale);
            let row_len = mid_w * 4;
            let out_h = height * scale;
            let mut out = Vec::with_capacity(out_h * row_len);
            for &(i, ph) in &phases {
                let c = coef[ph];
                let line = |t: i64| {
                    let idx = (i - 1 + t).clamp(0, height as i64 - 1) as usize;
                    &mid[idx * row_len..(idx + 1) * row_len]
                };
                let (l0, l1, l2, l3) = (line(0), line(1), line(2), line(3));
                for x in 0..mid_w {
                    let p = x * 4;
                    for ch in 0..3 {
                        let sum = c[0] * l0[p + ch] as i32
                            + c[1] * l1[p + ch] as i32
                            + c[2] * l2[p + ch] as i32
                            + c[3] * l3[p + ch] as i32;
                        out.push(((sum + 64) >> 7).clamp(0, 255) as u8);
                    }
                    out.push(255);
                }
            }
            (out, mid_w, out_h)
        }
        None => (mid, mid_w, height),
    }
}

#[cfg(test)]
mod filter_tests {
    use super::*;

    fn white(w: usize, h: usize) -> Vec<u8> {
        vec![255u8; w * h * 4]
    }

    #[test]
    fn tables_are_well_formed() {
        for t in SCANLINES.iter().chain([&INTERP_SOFT, &INTERP_MEDIUM, &INTERP_SHARP]) {
            for row in t.iter() {
                assert!(row.iter().sum::<i32>() <= 129);
            }
        }
        assert_eq!(SCANLINES[7][0], [5, 118, 5, 0]); // Scanlines_45
        assert_eq!(SCANLINES[7][32].iter().sum::<i32>(), 58);
    }

    #[test]
    fn scanline_table_maps_strength_to_file() {
        assert!(scanline_table(0).is_none());
        // strength 55 % -> gap brightness 45 % -> Scanlines_45
        assert_eq!(scanline_table(55).unwrap()[32].iter().sum::<i32>(), 58);
        // strength 90 % -> Scanlines_10 ; clamps above
        assert_eq!(scanline_table(90).unwrap()[32].iter().sum::<i32>(), 12);
        assert_eq!(scanline_table(100).unwrap()[32].iter().sum::<i32>(), 12);
        // strength 5 % -> Scanlines_95
        assert_eq!(scanline_table(5).unwrap()[32].iter().sum::<i32>(), 122);
    }

    #[test]
    fn interpolation_keeps_flat_color_and_sizes() {
        let (out, w, h) = filter_scale(&white(4, 3), 4, 3, Interp::Sharp.table(), Interp::Sharp.table());
        assert_eq!((w, h), (12, 9));
        assert_eq!(out.len(), 12 * 9 * 4);
        assert!(out.chunks(4).all(|p| p == [255, 255, 255, 255]));
    }

    #[test]
    fn scanlines_darken_the_gap() {
        let (out, w, h) = filter_scale(&white(4, 4), 4, 4, None, scanline_table(55));
        assert_eq!((w, h), (4, 16));
        let level = |y: usize| out[y * 16] as i32;
        assert!(level(5) >= 220 && level(6) >= 220, "{} {}", level(5), level(6));
        assert!(level(7) < 200 && level(8) < 200, "{} {}", level(7), level(8));
    }

    #[test]
    fn no_table_or_scale_one_is_identity() {
        let img = vec![1u8, 2, 3, 255, 4, 5, 6, 255];
        assert_eq!(filter_scale(&img, 2, 3, None, None), (img.clone(), 2, 1));
        assert_eq!(filter_scale(&img, 2, 1, Interp::Soft.table(), None), (img.clone(), 2, 1));
    }

    #[test]
    fn interp_names_round_trip() {
        for i in [Interp::Off, Interp::Soft, Interp::Medium, Interp::Sharp] {
            assert_eq!(Interp::from_name(i.name()), i);
        }
        assert_eq!(Interp::from_name("zzz"), Interp::Off);
    }
}
