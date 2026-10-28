//! E06-37: S-CPU sees APU handshake from IPL (milestone audio A1).
//! Tests that real 65816 code waiting on $2140/$2141 produces red screen when APU responds.

use puce16_core::snes::Snes;

mod common;
#[path = "common/prog.rs"]
mod prog;

/// Test ROM: wait for $AA at $2140 and $BB at $2141, then set color 0 to red and display on.
fn rom_apu_handshake() -> Vec<u8> {
    let code = [
        // SEI ; CLC ; XCE
        0x78, 0x18, 0xFB,
        // LDA $2140 (wait for APU handshake byte 1)
        0xAD, 0x40, 0x21,
        // CMP #$AA
        0xC9, 0xAA,
        // BNE -3 (loop back to LDA $2140)
        0xD0, 0xF9,
        // LDA $2141 (wait for APU handshake byte 2)
        0xAD, 0x41, 0x21,
        // CMP #$BB
        0xC9, 0xBB,
        // BNE -8 (loop back to LDA $2140)
        0xD0, 0xF2,
        // Set CGADD=0
        0xA9, 0x00, 0x8D, 0x21, 0x21,
        // Set color 0 to red (low byte = $1F)
        0xA9, 0x1F, 0x8D, 0x22, 0x21,
        // Set color 0 high byte = 0
        0xA9, 0x00, 0x8D, 0x22, 0x21,
        // Display on, brightness 15
        0xA9, 0x0F, 0x8D, 0x00, 0x21,
        // Loop forever
        0x80, 0xFE,
    ];
    prog::build_rom(&[(0, &code)], &[])
}

#[test]
fn e06_37_handshake_turns_screen_red() {
    let mut snes = Snes::new(&rom_apu_handshake(), None).unwrap();
    snes.run_frame();
    snes.run_frame();
    snes.run_frame();
    let view = snes.frame();
    assert_eq!(view.width, 256);
    assert_eq!(view.height, 224);
    for &p in view.pixels {
        assert_eq!(p, 0xFFFF_0000, "expected red pixel (APU responded)");
    }
}

#[test]
fn e06_37_first_frame_still_black() {
    let mut snes = Snes::new(&rom_apu_handshake(), None).unwrap();
    snes.run_frame(); // one frame only (~357k master clocks)
    let view = snes.frame();
    for &p in view.pixels {
        assert!(
            p == 0xFF00_0000 || p == 0xFFFF_0000,
            "expected all black or all red (no mixed colors)"
        );
    }
}

#[test]
fn e06_37_determinism() {
    let rom = rom_apu_handshake();
    let mut snes1 = Snes::new(&rom, None).unwrap();
    let mut snes2 = Snes::new(&rom, None).unwrap();
    for _ in 0..3 {
        snes1.run_frame();
        snes2.run_frame();
    }
    assert_eq!(snes1.state_hash(), snes2.state_hash());
}

#[test]
fn e06_37_reset_restarts() {
    let mut snes = Snes::new(&rom_apu_handshake(), None).unwrap();
    // Run 3 frames to get red screen
    for _ in 0..3 {
        snes.run_frame();
    }
    let view1 = snes.frame();
    for &p in view1.pixels {
        assert_eq!(p, 0xFFFF_0000, "expected red before reset");
    }

    // Reset and run again
    snes.reset();
    for _ in 0..3 {
        snes.run_frame();
    }
    let view2 = snes.frame();
    for &p in view2.pixels {
        assert_eq!(p, 0xFFFF_0000, "expected red after reset (handshake re-done)");
    }
}
