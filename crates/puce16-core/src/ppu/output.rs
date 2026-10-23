//! Video output: fixed internal framebuffer and compact frame view.
//!
//! See docs/specs/05_PPU.md section 6 ("Sortie: framebuffer").

use crate::consts::{FB_HEIGHT, FB_WIDTH};
use crate::hash::Fnv64;

/// Black pixel (XRGB8888, alpha forced to `0xFF`).
const BLACK: u32 = 0xFF00_0000;

/// Compact view of a rendered frame (spec 01_ARCHITECTURE section 5).
pub struct FrameView<'a> { pub width: usize, pub height: usize, pub pixels: &'a [u32] }

/// Fixed internal framebuffer: `FB_WIDTH * FB_HEIGHT` XRGB8888 pixels.
///
/// The buffer is always `FB_WIDTH` (512) columns wide; the useful area is
/// `view_w x view_h` starting at the top-left corner.
pub struct FrameBuf {
    /// Full buffer, stride `FB_WIDTH`.
    pub pixels: Vec<u32>,
    /// Useful width in columns (`0..view_w`).
    pub view_w: usize,
    /// Useful height in rows (`0..view_h`).
    pub view_h: usize,
}

impl FrameBuf {
    /// New buffer filled with black (`0xFF000000`), default view 256x224.
    pub fn new() -> Self {
        Self {
            pixels: vec![BLACK; FB_WIDTH * FB_HEIGHT],
            view_w: 256,
            view_h: 224,
        }
    }

    /// Write one pixel; coordinates outside the buffer are ignored.
    pub fn set(&mut self, col: usize, row: usize, argb: u32) {
        if col < FB_WIDTH && row < FB_HEIGHT {
            self.pixels[row * FB_WIDTH + col] = argb;
        }
    }

    /// Read one pixel; coordinates outside the buffer read as black.
    pub fn get(&self, col: usize, row: usize) -> u32 {
        if col < FB_WIDTH && row < FB_HEIGHT {
            self.pixels[row * FB_WIDTH + col]
        } else {
            BLACK
        }
    }

    /// Set the useful area. `w` must be 256 or 512 and `h` at most
    /// `FB_HEIGHT` (478); otherwise panics.
    pub fn set_view(&mut self, w: usize, h: usize) {
        if w != 256 && w != 512 {
            panic!("set_view: width must be 256 or 512, got {}", w);
        }
        if h > FB_HEIGHT {
            panic!("set_view: height must be <= {}, got {}", FB_HEIGHT, h);
        }
        self.view_w = w;
        self.view_h = h;
    }

    /// Copy the useful area (`view_w x view_h`, top-left corner) into a new
    /// buffer with stride `view_w`.
    pub fn compact(&self) -> Vec<u32> {
        let mut out = Vec::with_capacity(self.view_w * self.view_h);
        for row in 0..self.view_h {
            let start = row * FB_WIDTH;
            out.extend_from_slice(&self.pixels[start..start + self.view_w]);
        }
        out
    }

    /// Hash the frame: `view_w`, `view_h` (u32) then the compact pixels
    /// (u32 little-endian, row by row).
    pub fn hash(&self, h: &mut Fnv64) {
        h.write_u32(self.view_w as u32);
        h.write_u32(self.view_h as u32);
        for row in 0..self.view_h {
            let start = row * FB_WIDTH;
            h.write_u32_slice(&self.pixels[start..start + self.view_w]);
        }
    }
}

impl Default for FrameBuf {
    fn default() -> Self {
        Self::new()
    }
}

/// Duplicate each pixel horizontally: `w` columns become `2*w`. Used when the
/// frame is hi-res but a line is not (05_PPU.md section 6). Panics if
/// `src.len() != w * h`.
pub fn pixel_double(src: &[u32], w: usize, h: usize) -> Vec<u32> {
    if src.len() != w * h {
        panic!("pixel_double: expected {} pixels ({}x{}), got {}", w * h, w, h, src.len());
    }
    let mut out = Vec::with_capacity(2 * w * h);
    for row in 0..h {
        for &p in &src[row * w..row * w + w] {
            out.push(p);
            out.push(p);
        }
    }
    out
}

/// Build a compact view over `pixels` (stride `w`). Panics if
/// `pixels.len() != w * h`.
pub fn make_view(w: usize, h: usize, pixels: &[u32]) -> FrameView<'_> {
    if pixels.len() != w * h {
        panic!("make_view: expected {} pixels ({}x{}), got {}", w * h, w, h, pixels.len());
    }
    FrameView { width: w, height: h, pixels }
}
