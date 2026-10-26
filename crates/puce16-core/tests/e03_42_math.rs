//! E03-42: Integration - math unit (multiplication/division) and determinism.

mod common;
#[path = "common/prog.rs"]
mod prog;

use puce16_core::snes::Snes;
use prog::build_rom;

/// PROG_MATH: multiply 7*9, divide $1234/7, store results to WRAM.
fn build_math_prog() -> Vec<u8> {
    vec![
        // SEI CLC XCE - disable interrupts, clear carry, switch to native mode
        0x78, 0x18, 0xFB,
        // WRMPYA = 7
        0xA9, 0x07, 0x8D, 0x02, 0x42,
        // WRMPYB = 9 (starts multiplication)
        0xA9, 0x09, 0x8D, 0x03, 0x42,
        // Wait for multiplication to complete (10 NOPs)
        0xEA, 0xEA, 0xEA, 0xEA, 0xEA, 0xEA, 0xEA, 0xEA, 0xEA, 0xEA,
        // $0130 = RDMPYL (low byte of result)
        0xAD, 0x16, 0x42, 0x8D, 0x30, 0x01,
        // $0131 = RDMPYH (high byte of result)
        0xAD, 0x17, 0x42, 0x8D, 0x31, 0x01,
        // WRDIVL = $34 (low byte of dividend $1234)
        0xA9, 0x34, 0x8D, 0x04, 0x42,
        // WRDIVH = $12 (high byte of dividend $1234)
        0xA9, 0x12, 0x8D, 0x05, 0x42,
        // WRDIVB = 7 (starts division)
        0xA9, 0x07, 0x8D, 0x06, 0x42,
        // Wait for division to complete (10 NOPs)
        0xEA, 0xEA, 0xEA, 0xEA, 0xEA, 0xEA, 0xEA, 0xEA, 0xEA, 0xEA,
        // $0132 = RDDIVL (low byte of quotient)
        0xAD, 0x14, 0x42, 0x8D, 0x32, 0x01,
        // $0133 = RDDIVH (high byte of quotient)
        0xAD, 0x15, 0x42, 0x8D, 0x33, 0x01,
        // $0134 = RDMPYL (remainder)
        0xAD, 0x16, 0x42, 0x8D, 0x34, 0x01,
        // BRA L (loop forever)
        0x80, 0xFE,
    ]
}

/// PROG_MATH_EARLY: multiply but read result immediately (too early).
fn build_math_early_prog() -> Vec<u8> {
    vec![
        // SEI CLC XCE - disable interrupts, clear carry, switch to native mode
        0x78, 0x18, 0xFB,
        // WRMPYA = 7
        0xA9, 0x07, 0x8D, 0x02, 0x42,
        // WRMPYB = 9 (starts multiplication)
        0xA9, 0x09, 0x8D, 0x03, 0x42,
        // Read RDMPYL immediately (too early - should get old value)
        0xAD, 0x16, 0x42, 0x8D, 0x30, 0x01,
        // BRA L (loop forever)
        0x80, 0xFE,
    ]
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
fn e03_42_multiplication() {
    let rom = build_rom(&[(0, &build_math_prog())], &[]);
    let mut s = snes(rom);

    s.run_frame();
    settle(&mut s);

    // 7 * 9 = 63 = 0x3F
    assert_eq!(s.peek_wram(0x130), 0x3F, "multiplication low byte wrong");
    assert_eq!(s.peek_wram(0x131), 0x00, "multiplication high byte wrong");
}

#[test]
fn e03_42_division() {
    let rom = build_rom(&[(0, &build_math_prog())], &[]);
    let mut s = snes(rom);

    s.run_frame();
    settle(&mut s);

    // $1234 / 7 = 665 = $0299, remainder 5
    assert_eq!(s.peek_wram(0x132), 0x99, "division quotient low byte wrong");
    assert_eq!(s.peek_wram(0x133), 0x02, "division quotient high byte wrong");
    assert_eq!(s.peek_wram(0x134), 0x05, "division remainder wrong");
}

#[test]
fn e03_42_early_read_returns_old() {
    let rom = build_rom(&[(0, &build_math_early_prog())], &[]);
    let mut s = snes(rom);

    s.run_frame();
    settle(&mut s);

    // Reading too early returns old value (0 at power-on)
    assert_eq!(s.peek_wram(0x130), 0x00, "early read should return old value");
}

#[test]
fn e03_42_determinism_same_rom() {
    let rom = build_rom(&[(0, &build_math_prog())], &[]);

    let mut s1 = snes(rom.clone());
    let mut s2 = snes(rom);

    for _ in 0..2 {
        s1.run_frame();
        settle(&mut s1);
        s2.run_frame();
        settle(&mut s2);
    }

    assert_eq!(s1.state_hash(), s2.state_hash(), "same ROM should produce same state hash");
}

#[test]
fn e03_42_determinism_input_changes_hash() {
    let rom = build_rom(&[(0, &build_math_prog())], &[]);

    let mut s1 = snes(rom.clone());
    let mut s2 = snes(rom);

    // Different input on one instance
    s2.set_joypad(0, 0x8000);

    s1.run_frame();
    settle(&mut s1);
    s2.run_frame();
    settle(&mut s2);

    assert_ne!(s1.state_hash(), s2.state_hash(), "different input should produce different state hash");
}

#[test]
fn e03_42_determinism_chunked_runs() {
    let rom = build_rom(&[(0, &build_math_prog())], &[]);

    let mut s1 = snes(rom.clone());
    let mut s2 = snes(rom);

    // Chunked runs: 3 x 50k master clocks each
    for _ in 0..3 {
        s1.run_master(50_000);
        s2.run_master(50_000);
    }

    // Then a full frame on both
    s1.run_frame();
    settle(&mut s1);
    s2.run_frame();
    settle(&mut s2);

    assert_eq!(s1.state_hash(), s2.state_hash(), "chunked runs should produce same state hash");
}

#[test]
fn e03_42_frames_do_not_change_results() {
    let rom = build_rom(&[(0, &build_math_prog())], &[]);
    let mut s = snes(rom);

    // Run 3 frames - program loops on BRA so math is only done once
    for _ in 0..3 {
        s.run_frame();
        settle(&mut s);
    }

    // Results should be stable across frames
    assert_eq!(s.peek_wram(0x130), 0x3F, "multiplication low byte changed");
    assert_eq!(s.peek_wram(0x131), 0x00, "multiplication high byte changed");
    assert_eq!(s.peek_wram(0x132), 0x99, "division quotient low byte changed");
    assert_eq!(s.peek_wram(0x133), 0x02, "division quotient high byte changed");
    assert_eq!(s.peek_wram(0x134), 0x05, "division remainder changed");
}
