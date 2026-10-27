//! E04-15: snes.rs frame() returns PPU output for Real variant.
//! Tests that real 65816 code writing PPU registers produces visible color changes.

use puce16_core::snes::Snes;
use puce16_core::region::Region;

mod common;
#[path = "common/prog.rs"]
mod prog;

/// Test ROM: SEI ; CLC ; XCE ; set CGRAM color 0 to red (0x001F) ; display on, brightness 15 ; loop
fn rom_red() -> Vec<u8> {
    let code = [
        0x78, 0x18, 0xFB, // SEI ; CLC ; XCE
        0xA9, 0x00, 0x8D, 0x21, 0x21, // LDA #$00 ; STA $2121 (CGADD=0)
        0xA9, 0x1F, 0x8D, 0x22, 0x21, // LDA #$1F ; STA $2122 (color low byte = red)
        0xA9, 0x00, 0x8D, 0x22, 0x21, // LDA #$00 ; STA $2122 (color high byte = 0)
        0xA9, 0x0F, 0x8D, 0x00, 0x21, // LDA #$0F ; STA $2100 (display on, brightness 15)
        0x80, 0xFE, // BRA -2 (loop)
    ];
    prog::build_rom(&[(0, &code)], &[])
}

/// Test ROM: same as red but color 0 = blue (0x7C00)
fn rom_blue() -> Vec<u8> {
    let code = [
        0x78, 0x18, 0xFB, // SEI ; CLC ; XCE
        0xA9, 0x00, 0x8D, 0x21, 0x21, // LDA #$00 ; STA $2121 (CGADD=0)
        0xA9, 0x00, 0x8D, 0x22, 0x21, // LDA #$00 ; STA $2122 (color low byte = 0)
        0xA9, 0x7C, 0x8D, 0x22, 0x21, // LDA #$7C ; STA $2122 (color high byte = blue)
        0xA9, 0x0F, 0x8D, 0x00, 0x21, // LDA #$0F ; STA $2100 (display on, brightness 15)
        0x80, 0xFE, // BRA -2 (loop)
    ];
    prog::build_rom(&[(0, &code)], &[])
}

/// Test ROM: red color but brightness 8 instead of 15
fn rom_dark() -> Vec<u8> {
    let code = [
        0x78, 0x18, 0xFB, // SEI ; CLC ; XCE
        0xA9, 0x00, 0x8D, 0x21, 0x21, // LDA #$00 ; STA $2121 (CGADD=0)
        0xA9, 0x1F, 0x8D, 0x22, 0x21, // LDA #$1F ; STA $2122 (color low byte = red)
        0xA9, 0x00, 0x8D, 0x22, 0x21, // LDA #$00 ; STA $2122 (color high byte = 0)
        0xA9, 0x08, 0x8D, 0x00, 0x21, // LDA #$08 ; STA $2100 (display on, brightness 8)
        0x80, 0xFE, // BRA -2 (loop)
    ];
    prog::build_rom(&[(0, &code)], &[])
}

/// Test ROM: no PPU writes at all (forced blank)
fn rom_blank() -> Vec<u8> {
    let code = [0x78, 0x18, 0xFB, 0x80, 0xFE]; // SEI ; CLC ; XCE ; BRA -2
    prog::build_rom(&[(0, &code)], &[])
}

#[test]
fn e04_15_red_frame() {
    let mut snes = Snes::new(&rom_red(), None).unwrap();
    snes.run_frame();
    snes.run_frame();
    let view = snes.frame();
    assert_eq!(view.width, 256);
    assert_eq!(view.height, 224);
    for &p in view.pixels {
        assert_eq!(p, 0xFFFF_0000, "expected red pixel");
    }
}

#[test]
fn e04_15_blue_frame() {
    let mut snes = Snes::new(&rom_blue(), None).unwrap();
    snes.run_frame();
    snes.run_frame();
    let view = snes.frame();
    for &p in view.pixels {
        assert_eq!(p, 0xFF00_00FF, "expected blue pixel");
    }
}

#[test]
fn e04_15_dark_frame() {
    let mut snes = Snes::new(&rom_dark(), None).unwrap();
    snes.run_frame();
    snes.run_frame();
    let view = snes.frame();
    for &p in view.pixels {
        assert_eq!(p, 0xFF84_0000, "expected dark red pixel");
    }
}

#[test]
fn e04_15_black_before_display() {
    let mut snes = Snes::new(&rom_blank(), None).unwrap();
    snes.run_frame();
    let view = snes.frame();
    for &p in view.pixels {
        assert_eq!(p, 0xFF00_0000, "expected black pixel (forced blank)");
    }
}

#[test]
fn e04_15_first_frame_complete() {
    let mut snes = Snes::new(&rom_red(), None).unwrap();
    snes.run_frame(); // first frame only
    let view = snes.frame();
    for &p in view.pixels {
        assert_eq!(p, 0xFFFF_0000, "expected red pixel on first frame");
    }
}

#[test]
fn e04_15_determinism() {
    let rom = rom_red();
    let mut snes1 = Snes::new(&rom, None).unwrap();
    let mut snes2 = Snes::new(&rom, None).unwrap();
    for _ in 0..2 {
        snes1.run_frame();
        snes2.run_frame();
    }
    assert_eq!(snes1.state_hash(), snes2.state_hash());

    let mut snes3 = Snes::new(&rom_blue(), None).unwrap();
    for _ in 0..2 {
        snes3.run_frame();
    }
    assert_ne!(snes1.state_hash(), snes3.state_hash());
}

#[test]
fn e04_15_pattern_unchanged() {
    let mut snes = Snes::new_test_pattern(Region::Ntsc);
    snes.run_frame();
    let view = snes.frame();
    assert_eq!(view.pixels[0], 0xFF01_0040, "pattern pixel (0,0) unchanged");
}

#[test]
fn e04_15_reset_clears_picture() {
    let mut snes = Snes::new(&rom_red(), None).unwrap();
    snes.run_frame();
    snes.run_frame();
    snes.reset();
    snes.run_frame();
    assert_eq!(snes.frame_count(), 1);
    let view = snes.frame();
    for &p in view.pixels {
        assert_eq!(p, 0xFFFF_0000, "expected red pixel after reset");
    }
}
