//! Pure video helpers for desktop rendering (testable without a window).

/// Pixel aspect ratio of the displayed frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ratio {
    /// Square pixels (default).
    Square,
    /// 8:7 pixel aspect: display width multiplied by 8/7.
    Par87,
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
