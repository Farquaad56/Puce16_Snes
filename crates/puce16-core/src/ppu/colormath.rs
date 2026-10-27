//! Color math and color window region logic (spec 05b sections 11.3, 11.4).

use super::bg::{LAYER_OBJ, LAYER_BACKDROP};
use super::regs::Regs;
use super::Pick;

/// Window region evaluation: t=0 always false, t=1 outside only, t=2 inside only, t=3 always true.
pub fn region(t: u8, inside: bool) -> bool {
    match t {
        0 => false,
        1 => !inside,
        2 => inside,
        _ => true,
    }
}

/// Pack three 5-bit color components into a BGR555 word.
pub fn fixed_color(r: u8, g: u8, b: u8) -> u16 {
    let r = r as u16;
    let g = g as u16;
    let b = b as u16;
    (b << 10) | (g << 5) | r
}

/// Per-channel blend: t = base_c +/- add_c; if half, arithmetic right shift by 1; clamp to 0..=31.
pub fn blend(base: u16, add: u16, sub: bool, half: bool) -> u16 {
    let mut result: u16 = 0;
    for ch in 0..3 {
        let shift = ch * 5;
        let base_c = (base >> shift) & 0x1F;
        let add_c = (add >> shift) & 0x1F;
        let t: i8 = if sub {
            (base_c as i8) - (add_c as i8)
        } else {
            (base_c as i8) + (add_c as i8)
        };
        let shifted = if half { t >> 1 } else { t };
        let clamped = if shifted < 0 { 0 } else if shifted > 31 { 31 } else { shifted };
        result |= (clamped as u16) << shift;
    }
    result
}

/// Check whether a layer is eligible for color math based on CGADSUB mask.
pub fn math_layer_ok(layer: u8, obj_pal: u8, mask: u8) -> bool {
    match layer {
        0..=3 => (mask >> layer) & 1 != 0,
        LAYER_OBJ => ((mask >> 4) & 1 != 0) && obj_pal >= 4,
        LAYER_BACKDROP => (mask >> 5) & 1 != 0,
        _ => false,
    }
}

/// Main color math operation (spec 11.4). Returns BGR555 result.
pub fn color_math(main: &Pick, sub: &Pick, black: bool, prevent: bool, regs: &Regs) -> u16 {
    let base = if black { 0 } else { main.rgb };
    let layer_ok = math_layer_ok(main.layer, main.obj_pal, regs.math_mask);
    if !layer_ok || prevent {
        return base;
    }

    let (addend, sub_is_back) = if regs.addend_subscreen {
        (sub.rgb, sub.layer == LAYER_BACKDROP)
    } else {
        (fixed_color(regs.fixed_r, regs.fixed_g, regs.fixed_b), false)
    };

    let halve = regs.math_half && !(regs.addend_subscreen && sub_is_back);
    blend(base, addend, regs.math_sub, halve)
}

impl super::Ppu {
    /// Evaluate color window flags at pixel x: (black, prevent).
    pub fn color_window_flags(&self, x: usize) -> (bool, bool) {
        let inside = self.window_hides(5, x);
        let black = region(self.regs.black_region, inside);
        let prevent = region(self.regs.math_region, inside);
        (black, prevent)
    }
}
