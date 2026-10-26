//! E03-41: Integration - auto-read of joypads ($4218-$421F).

mod common;
#[path = "common/prog.rs"]
mod prog;

use puce16_core::joypad::{BTN_A, BTN_B, BTN_LEFT, BTN_RIGHT, BTN_START};
use puce16_core::snes::Snes;
use prog::build_rom;

/// PROG_JOY: enable auto-read (NMITIMEN.0=1), loop reading JOY registers to WRAM $0120-$0122.
fn build_joy_prog() -> Vec<u8> {
    vec![
        // SEI CLC XCE - disable interrupts, clear carry, switch to native mode
        0x78, 0x18, 0xFB,
        // NMITIMEN = 1 (auto-read enabled, no NMI)
        0xA9, 0x01, 0x8D, 0x00, 0x42,
        // L: $0120 = JOY1L
        0xAD, 0x18, 0x42, 0x8D, 0x20, 0x01,
        // $0121 = JOY1H
        0xAD, 0x19, 0x42, 0x8D, 0x21, 0x01,
        // $0122 = JOY2H
        0xAD, 0x1B, 0x42, 0x8D, 0x22, 0x01,
        // BRA L
        0x80, 0xEC,
    ]
}

/// PROG_JOY_OFF: same as PROG_JOY but with NMITIMEN write replaced by NOPs.
fn build_joy_off_prog() -> Vec<u8> {
    let mut p = build_joy_prog();
    // Replace the 5-byte NMITIMEN=1 instruction (offset 3) with 5 NOPs
    for i in 0..5 {
        p[3 + i] = 0xEA;
    }
    p
}

fn snes(rom: Vec<u8>) -> Snes {
    Snes::new(&rom, None).ok().unwrap()
}

/// Let the CPU take any pending interrupt after run_frame.
fn settle(s: &mut Snes) {
    for _ in 0..12 {
        s.step_instruction();
    }
}

#[test]
fn e03_41_zero_before_vblank() {
    let rom = build_rom(&[(0, &build_joy_prog())], &[]);
    let mut s = snes(rom);

    // Set buttons before any frame runs
    s.set_joypad(0, BTN_A | BTN_START);

    // Run some master clocks but not a full frame - auto-read hasn't happened yet
    s.run_master(100_000);

    // JOY registers should still be zero (program has copied zeros)
    assert_eq!(s.peek_wram(0x120), 0, "JOY1L not zero before vblank");
    assert_eq!(s.peek_wram(0x121), 0, "JOY1H not zero before vblank");
    assert_eq!(s.peek_wram(0x122), 0, "JOY2H not zero before vblank");
}

#[test]
fn e03_41_values_after_autoread() {
    let rom = build_rom(&[(0, &build_joy_prog())], &[]);
    let mut s = snes(rom);

    // Set buttons: port 0 has A+Start (0x1080), port 1 has B (0x8000)
    s.set_joypad(0, BTN_A | BTN_START);
    s.set_joypad(1, BTN_B);

    // Run a full frame so auto-read occurs during vblank
    s.run_frame();
    settle(&mut s);
    // Wait for the 4224 clocks of auto-read to complete
    s.run_master(5000);

    // After auto-read: JOY1L=0x80 (low byte of 0x1080), JOY1H=0x10, JOY2H=0x80 (high byte of 0x8000)
    assert_eq!(s.peek_wram(0x120), 0x80, "JOY1L wrong after autoread");
    assert_eq!(s.peek_wram(0x121), 0x10, "JOY1H wrong after autoread");
    assert_eq!(s.peek_wram(0x122), 0x80, "JOY2H wrong after autoread");
}

#[test]
fn e03_41_old_value_during_autoread() {
    let rom = build_rom(&[(0, &build_joy_prog())], &[]);
    let mut s = snes(rom);

    // Set buttons
    s.set_joypad(0, BTN_A | BTN_START);
    s.set_joypad(1, BTN_B);

    // Run frame - returns just after vblank start, before 4224 clocks complete
    s.run_frame();
    settle(&mut s);

    // During auto-read operation, JOY registers still hold old (zero) values
    assert_eq!(s.peek_wram(0x120), 0, "JOY1L should be old value during autoread");
}

#[test]
fn e03_41_next_frame_updates() {
    let rom = build_rom(&[(0, &build_joy_prog())], &[]);
    let mut s = snes(rom);

    // First frame: set buttons and run
    s.set_joypad(0, BTN_A | BTN_START);
    s.run_frame();
    settle(&mut s);
    s.run_master(5000);

    // Verify first frame values were captured
    assert_eq!(s.peek_wram(0x120), 0x80, "first frame JOY1L wrong");

    // Change buttons for next frame
    s.set_joypad(0, 0x0001);

    // Run second frame - should capture new button state
    s.run_frame();
    settle(&mut s);
    s.run_master(5000);

    assert_eq!(s.peek_wram(0x120), 0x01, "second frame JOY1L not updated");
    assert_eq!(s.peek_wram(0x121), 0x00, "second frame JOY1H not updated");
}

#[test]
fn e03_41_autoread_off() {
    let rom = build_rom(&[(0, &build_joy_off_prog())], &[]);
    let mut s = snes(rom);

    // Set buttons but auto-read is disabled (NMITIMEN.0=0)
    s.set_joypad(0, 0xFFF0);

    for _ in 0..2 {
        s.run_frame();
        settle(&mut s);
    }

    // Without auto-read, JOY registers remain zero
    assert_eq!(s.peek_wram(0x120), 0, "JOY1L changed without autoread");
}

#[test]
fn e03_41_opposite_dirs_not_filtered_by_core() {
    let rom = build_rom(&[(0, &build_joy_prog())], &[]);
    let mut s = snes(rom);

    // Set both left and right (opposites) - core should NOT filter these
    s.set_joypad(0, BTN_LEFT | BTN_RIGHT);

    s.run_frame();
    settle(&mut s);
    s.run_master(5000);

    // LEFT=0x0200, RIGHT=0x0100 -> combined 0x0300 -> high byte 0x03
    assert_eq!(s.peek_wram(0x121), 0x03, "opposite directions filtered by core");
}
