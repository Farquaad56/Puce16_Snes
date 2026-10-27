use puce16_core::ppu::{Ppu, window};
use puce16_core::region::Region;

fn make_ppu() -> Ppu {
    Ppu::new(Region::Ntsc)
}

#[test]
fn e05_09_window1_only() {
    let wh = [10, 20, 15, 25];
    // g=2: window 1 enabled only
    assert!(window::window_out(2, 0, 10, wh));
    assert!(window::window_out(2, 0, 15, wh));
    assert!(window::window_out(2, 0, 20, wh));
    assert!(!window::window_out(2, 0, 9, wh));
    assert!(!window::window_out(2, 0, 21, wh));
    assert!(!window::window_out(2, 0, 0, wh));
    assert!(!window::window_out(2, 0, 255, wh));
}

#[test]
fn e05_09_window1_inverted() {
    let wh = [10, 20, 15, 25];
    // g=3: window 1 enabled + inverted
    assert!(!window::window_out(3, 0, 10, wh));
    assert!(!window::window_out(3, 0, 15, wh));
    assert!(!window::window_out(3, 0, 20, wh));
    assert!(window::window_out(3, 0, 9, wh));
    assert!(window::window_out(3, 0, 21, wh));
    assert!(window::window_out(3, 0, 0, wh));
    assert!(window::window_out(3, 0, 255, wh));
}

#[test]
fn e05_09_window2_only() {
    let wh = [10, 20, 15, 25];
    // g=8: window 2 enabled only
    assert!(window::window_out(8, 0, 15, wh));
    assert!(window::window_out(8, 0, 25, wh));
    assert!(!window::window_out(8, 0, 14, wh));
    assert!(!window::window_out(8, 0, 26, wh));
    // g=12: window 2 enabled + inverted
    assert!(!window::window_out(12, 0, 15, wh));
    assert!(!window::window_out(12, 0, 20, wh));
    assert!(!window::window_out(12, 0, 25, wh));
    assert!(window::window_out(12, 0, 14, wh));
}

#[test]
fn e05_09_left_greater_than_right() {
    let wh = [20, 10, 0, 0];
    // g=2: window 1 enabled (empty since left > right)
    assert!(!window::window_out(2, 0, 0, wh));
    assert!(!window::window_out(2, 0, 10, wh));
    assert!(!window::window_out(2, 0, 15, wh));
    assert!(!window::window_out(2, 0, 20, wh));
    assert!(!window::window_out(2, 0, 255, wh));
    // g=3: window 1 enabled + inverted (everything is "outside" empty window)
    assert!(window::window_out(3, 0, 0, wh));
    assert!(window::window_out(3, 0, 10, wh));
    assert!(window::window_out(3, 0, 15, wh));
    assert!(window::window_out(3, 0, 20, wh));
    assert!(window::window_out(3, 0, 255, wh));
}

#[test]
fn e05_09_inactive_inversion_no_effect() {
    let wh = [10, 20, 15, 25];
    // g=1: inversion only, no window enabled -> false for all x
    assert!(!window::window_out(1, 0, 0, wh));
    assert!(!window::window_out(1, 0, 15, wh));
    assert!(!window::window_out(1, 0, 255, wh));
    // g=5: window 2 inversion only -> false for all x
    assert!(!window::window_out(5, 0, 0, wh));
    assert!(!window::window_out(5, 0, 15, wh));
    assert!(!window::window_out(5, 0, 255, wh));
    // g=0: nothing enabled -> false for all x
    assert!(!window::window_out(0, 0, 0, wh));
    assert!(!window::window_out(0, 0, 15, wh));
}

#[test]
fn e05_09_logic_combinations() {
    let wh = [10, 20, 15, 25];
    // g=0xA: both windows enabled, no inversion
    // x=12: w1=true (in window 1), w2=false -> OR=T AND=F XOR=T XNOR=F
    assert!(window::window_out(0xA, 0, 12, wh));   // OR
    assert!(!window::window_out(0xA, 1, 12, wh));  // AND
    assert!(window::window_out(0xA, 2, 12, wh));   // XOR
    assert!(!window::window_out(0xA, 3, 12, wh));  // XNOR
    // x=17: w1=true, w2=true -> OR=T AND=T XOR=F XNOR=T
    assert!(window::window_out(0xA, 0, 17, wh));
    assert!(window::window_out(0xA, 1, 17, wh));
    assert!(!window::window_out(0xA, 2, 17, wh));
    assert!(window::window_out(0xA, 3, 17, wh));
    // x=22: w1=false, w2=true -> OR=T AND=F XOR=T XNOR=F
    assert!(window::window_out(0xA, 0, 22, wh));
    assert!(!window::window_out(0xA, 1, 22, wh));
    assert!(window::window_out(0xA, 2, 22, wh));
    assert!(!window::window_out(0xA, 3, 22, wh));
    // x=30: w1=false, w2=false -> OR=F AND=F XOR=F XNOR=T
    assert!(!window::window_out(0xA, 0, 30, wh));
    assert!(!window::window_out(0xA, 1, 30, wh));
    assert!(!window::window_out(0xA, 2, 30, wh));
    assert!(window::window_out(0xA, 3, 30, wh));
}

#[test]
fn e05_09_logic_with_inversion() {
    let wh = [10, 20, 15, 25];
    // g=0xB: window 1 inverted + both enabled, logic=OR(0)
    // x=12: w1=false (inverted), w2=false -> OR=F
    assert!(!window::window_out(0xB, 0, 12, wh));
    // x=30: w1=true (inverted), w2=false -> OR=T
    assert!(window::window_out(0xB, 0, 30, wh));
}

#[test]
fn e05_09_layer_group_extraction() {
    let mut ppu = make_ppu();
    // win_sel[0]=0xA5: BG1=5, BG2=A; win_sel[1]=0x3C: BG3=C, BG4=3; win_sel[2]=0x71: OBJ=1, color=7
    ppu.write(0x23, 0xA5);
    ppu.write(0x24, 0x3C);
    ppu.write(0x25, 0x71);

    assert_eq!(window::layer_group(ppu.regs(), 0), 5);   // BG1
    assert_eq!(window::layer_group(ppu.regs(), 1), 0xA); // BG2
    assert_eq!(window::layer_group(ppu.regs(), 2), 0xC); // BG3
    assert_eq!(window::layer_group(ppu.regs(), 3), 3);   // BG4
    assert_eq!(window::layer_group(ppu.regs(), 4), 1);   // OBJ
    assert_eq!(window::layer_group(ppu.regs(), 5), 7);   // color
    assert_eq!(window::layer_group(ppu.regs(), 6), 0);   // out of range
}

#[test]
fn e05_09_layer_logic_extraction() {
    let mut ppu = make_ppu();
    // win_log_bg=0xE4: BG1=0, BG2=1, BG3=2, BG4=3
    ppu.write(0x2A, 0xE4);
    assert_eq!(window::layer_logic(ppu.regs(), 0), 0);
    assert_eq!(window::layer_logic(ppu.regs(), 1), 1);
    assert_eq!(window::layer_logic(ppu.regs(), 2), 2);
    assert_eq!(window::layer_logic(ppu.regs(), 3), 3);

    // win_log_obj=0xFE: OBJ logic = FE & 3 = 2, color logic = (FE >> 2) & 3 = 3
    ppu.write(0x2B, 0xFE);
    assert_eq!(window::layer_logic(ppu.regs(), 4), 2); // OBJ
    assert_eq!(window::layer_logic(ppu.regs(), 5), 3); // color
}

#[test]
fn e05_09_ppu_window_hides() {
    let mut ppu = make_ppu();
    // BG1 window 1 enabled (g=2)
    ppu.write(0x23, 0x02);
    // Window positions: left=10, right=20
    ppu.write(0x26, 10);
    ppu.write(0x27, 20);

    assert!(ppu.window_hides(0, 15));   // BG1 at x=15 is hidden (in window)
    assert!(!ppu.window_hides(0, 9));   // BG1 at x=9 is not hidden
    assert!(!ppu.window_hides(1, 15));  // BG2 has group 0 -> never hidden

    // TMW doesn't affect window_hides (that's done in pick)
    ppu.write(0x2C, 0);
    assert!(ppu.window_hides(0, 15));   // still hidden regardless of TMW
}
