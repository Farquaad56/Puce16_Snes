//! Window logic: determine if layers are hidden at a given pixel.
//! Pure functions + extraction of settings from Regs.

use super::regs::Regs;

/// Determine if a layer is hidden (window_out) at pixel x.
/// g = 4-bit group setting, logic = 2-bit window logic, wh = [w1l,w1r,w2l,w2r]
pub fn window_out(g: u8, logic: u8, x: u8, wh: [u8; 4]) -> bool {
    let w1 = (x >= wh[0] && x <= wh[1]) ^ (g & 1 != 0);
    let w2 = (x >= wh[2] && x <= wh[3]) ^ (g & 4 != 0);
    let en1 = g & 2 != 0;
    let en2 = g & 8 != 0;

    match (en1, en2) {
        (false, false) => false,
        (true, false) => w1,
        (false, true) => w2,
        (true, true) => match logic {
            0 => w1 | w2,
            1 => w1 & w2,
            2 => w1 ^ w2,
            _ => !(w1 ^ w2),
        },
    }
}

/// Extract the 4-bit window group setting for a layer.
/// Layers: 0=BG1, 1=BG2, 2=BG3, 3=BG4, 4=OBJ, 5=color math
pub fn layer_group(regs: &Regs, layer: u8) -> u8 {
    match layer {
        0 => regs.win_sel[0] & 0x0F,
        1 => (regs.win_sel[0] >> 4) & 0x0F,
        2 => regs.win_sel[1] & 0x0F,
        3 => (regs.win_sel[1] >> 4) & 0x0F,
        4 => regs.win_sel[2] & 0x0F,
        5 => (regs.win_sel[2] >> 4) & 0x0F,
        _ => 0,
    }
}

/// Extract the 2-bit window logic setting for a layer.
pub fn layer_logic(regs: &Regs, layer: u8) -> u8 {
    match layer {
        0 => regs.win_log_bg & 3,
        1 => (regs.win_log_bg >> 2) & 3,
        2 => (regs.win_log_bg >> 4) & 3,
        3 => (regs.win_log_bg >> 6) & 3,
        4 => regs.win_log_obj,
        5 => regs.win_log_col,
        _ => 0,
    }
}

impl super::Ppu {
    /// Determine if a layer is hidden by windows at pixel x.
    /// Checks TMW/TSW bit for the layer AND whether window_out is true.
    pub fn layer_hidden(&self, main: bool, layer: u8, x: usize) -> bool {
        let wm = if main { self.regs.tmw } else { self.regs.tsw };
        ((wm >> layer) & 1 != 0) && self.window_hides(layer, x)
    }
}
