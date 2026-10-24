// 16-bit button format shared by core and desktop (spec 08, section 1).
pub const BTN_B: u16 = 1 << 15;
pub const BTN_Y: u16 = 1 << 14;
pub const BTN_SELECT: u16 = 1 << 13;
pub const BTN_START: u16 = 1 << 12;
pub const BTN_UP: u16 = 1 << 11;
pub const BTN_DOWN: u16 = 1 << 10;
pub const BTN_LEFT: u16 = 1 << 9;
pub const BTN_RIGHT: u16 = 1 << 8;
pub const BTN_A: u16 = 1 << 7;
pub const BTN_X: u16 = 1 << 6;
pub const BTN_L: u16 = 1 << 5;
pub const BTN_R: u16 = 1 << 4;

/// If both directions of a pair are pressed, clear that whole pair.
/// All other bits pass through unchanged.
pub fn filter_opposites(b: u16) -> u16 {
    let mut out = b;
    if b & BTN_LEFT != 0 && b & BTN_RIGHT != 0 {
        out &= !(BTN_LEFT | BTN_RIGHT);
    }
    if b & BTN_UP != 0 && b & BTN_DOWN != 0 {
        out &= !(BTN_UP | BTN_DOWN);
    }
    out
}
