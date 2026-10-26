//! E03-35: snes.rs run_frame / run_master / step_instruction / hash / SRAM / debug

use puce16_core::snes::{Snes, LoadError};
use puce16_core::cart::MapMode;
use puce16_core::region::Region;

mod common;
use common::{make_rom, make_rom_with, Map};

/// Test ROM: SEI ; CLC ; XCE ; LDA #$42 ; STA $0010 ; BRA -2 (infinite loop)
fn test_rom() -> Vec<u8> {
    let code = [0x78, 0x18, 0xFB, 0xA9, 0x42, 0x8D, 0x10, 0x00, 0x80, 0xFE];
    make_rom(&code, Map::LoRom)
}

/// Test ROM with STP: SEI ; CLC ; XCE ; STP (CPU halts but time still advances)
fn stp_rom() -> Vec<u8> {
    let code = [0x78, 0x18, 0xFB, 0xDB];
    make_rom(&code, Map::LoRom)
}

#[test]
fn e03_35_run_frame_basic() {
    let mut snes = Snes::new(&test_rom(), None).unwrap();
    snes.run_frame();
    assert_eq!(snes.peek_wram(0x10), 0x42);
    assert_eq!(snes.frame_count(), 1);
    let dbg = snes.debug();
    assert_eq!(dbg.pb, 0);
    assert_eq!(dbg.pc, 0x8008); // after BRA -2 (loop back)
    assert!(!dbg.e); // XCE switched to native mode
    assert_eq!(dbg.v, 225); // vblank_start for NTSC non-overscan
    assert!(dbg.dot <= 20); // early in vblank
}

#[test]
fn e03_35_frame_master_range() {
    let mut snes = Snes::new(&test_rom(), None).unwrap();
    snes.run_frame();
    let m1 = snes.master();
    assert!(m1 >= 306_905 && m1 <= 307_000, "frame 1 master: {}", m1);

    snes.run_frame();
    let m2 = snes.master();
    let delta2 = m2 - m1;
    assert!(delta2 >= 357_268 && delta2 <= 357_468, "frame 2 delta: {}", delta2);

    snes.run_frame();
    let m3 = snes.master();
    let delta3 = m3 - m2;
    assert!(delta3 >= 357_264 && delta3 <= 357_464, "frame 3 delta: {}", delta3);
}

#[test]
fn e03_35_run_master_exact() {
    let mut snes = Snes::new(&test_rom(), None).unwrap();
    // CPU reset already consumed some cycles
    let m0 = snes.master();
    snes.run_master(1000);
    let m1 = snes.master();
    assert!(m1 >= m0 + 1000, "master should advance at least 1000");
    assert!(m1 < m0 + 1000 + 80, "last instruction may overshoot slightly");

    // run_master(0) should not change master
    snes.run_master(0);
    assert_eq!(snes.master(), m1);
}

#[test]
fn e03_35_step_instruction() {
    let mut snes = Snes::new(&test_rom(), None).unwrap();
    // Execute SEI, CLC, XCE one at a time
    snes.step_instruction(); // SEI
    snes.step_instruction(); // CLC
    snes.step_instruction(); // XCE
    let dbg = snes.debug();
    assert!(!dbg.e); // XCE switched to native mode
    assert_eq!(dbg.pc, 0x8003); // after XCE (3 bytes executed)
}

#[test]
fn e03_35_state_hash_determinism() {
    let rom = test_rom();
    let mut snes1 = Snes::new(&rom, None).unwrap();
    let mut snes2 = Snes::new(&rom, None).unwrap();

    // Same operations -> same hash
    for _ in 0..3 {
        snes1.run_frame();
        snes2.run_frame();
    }
    assert_eq!(snes1.state_hash(), snes2.state_hash());

    // Different number of frames -> different hash
    let h_after_3 = snes1.state_hash();
    snes1.run_frame();
    assert_ne!(h_after_3, snes1.state_hash());

    // Same run_master -> same hash again
    snes2.run_frame();
    snes2.run_master(1000);
    snes1.run_master(1000);
    assert_eq!(snes1.state_hash(), snes2.state_hash());
}

#[test]
fn e03_35_joypad_api() {
    let mut snes = Snes::new(&test_rom(), None).unwrap();
    snes.set_joypad(0, 0x8000); // A button
    assert_eq!(snes.joypad(0), 0x8000);
    assert_eq!(snes.joypad(1), 0);

    // Port 2 is ignored
    snes.set_joypad(2, 1);
    assert_eq!(snes.joypad(2), 0);

    // Hash changes after joypad change
    let h1 = snes.state_hash();
    snes.set_joypad(0, 1);
    let h2 = snes.state_hash();
    assert_ne!(h1, h2);
}

#[test]
fn e03_35_sram_option() {
    // ROM without SRAM: sram() returns None
    let rom_no_sram = test_rom();
    let snes1 = Snes::new(&rom_no_sram, None).unwrap();
    assert!(snes1.sram().is_none());

    // ROM with SRAM (sram_log2=3 -> 8 KiB)
    let rom_with_sram = make_rom_with(&[0xEA], Map::LoRom, 15, 3, 0x02, 1);
    let mut snes2 = Snes::new(&rom_with_sram, None).unwrap();
    assert!(snes2.sram().is_some());
    assert_eq!(snes2.sram().unwrap().len(), 8192);

    // Load SRAM data
    snes2.load_sram(&[1, 2, 3]);
    let sram = snes2.sram().unwrap();
    assert_eq!(&sram[..4], &[1, 2, 3, 0]);
}

#[test]
fn e03_35_cart_info() {
    // Real ROM: cart_info returns Some
    let rom = test_rom();
    let snes = Snes::new(&rom, None).unwrap();
    let info = snes.cart_info().unwrap();
    assert_eq!(info.title, "PUCE16 TEST ROM");
    assert_eq!(info.map, MapMode::LoRom);
    assert_eq!(info.region, Region::Ntsc);
    assert_eq!(info.rom_len, 32768);
    assert_eq!(info.sram_len, 0);
    assert!(!info.battery);

    // Pattern variant: cart_info returns None
    let pattern = Snes::new_test_pattern(Region::Ntsc);
    assert!(pattern.cart_info().is_none());
}

#[test]
fn e03_35_pattern_debug_default() {
    let mut snes = Snes::new_test_pattern(Region::Ntsc);
    snes.run_frame(); // frame 1
    let dbg = snes.debug();
    assert_eq!(dbg.pc, 0);
    assert_eq!(dbg.pb, 0);
    assert_eq!(dbg.a, 0);
    assert_eq!(dbg.x, 0);
    assert_eq!(dbg.y, 0);
    assert_eq!(dbg.s, 0);
    assert_eq!(dbg.d, 0);
    assert_eq!(dbg.db, 0);
    assert_eq!(dbg.p, 0);
    assert!(!dbg.e);
    assert_eq!(dbg.v, 0);
    assert_eq!(dbg.dot, 0);
    assert_eq!(dbg.master, 0);
    assert_eq!(dbg.frame, 1); // only frame is non-zero
}

#[test]
fn e03_35_frame_view_real() {
    let snes = Snes::new(&test_rom(), None).unwrap();
    let view = snes.frame();
    assert_eq!(view.width, 256);
    assert_eq!(view.height, 224);
    assert_eq!(view.pixels.len(), 256 * 224);
    // All pixels should be black (0xFF000000) initially
    for &p in view.pixels {
        assert_eq!(p, 0xFF00_0000);
    }
}

#[test]
fn e03_35_stp_still_reaches_vblank() {
    // STP halts the CPU but time still advances (idle cycles)
    let mut snes = Snes::new(&stp_rom(), None).unwrap();
    snes.run_frame();
    assert_eq!(snes.frame_count(), 1);
    let dbg = snes.debug();
    assert_eq!(dbg.pc, 0x8004); // after STP (instruction at 0x8003)
}
