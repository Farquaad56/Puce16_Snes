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
