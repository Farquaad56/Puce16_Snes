//! E08-12: Snes::save_state and load_state (blob assembly, header).

use puce16_core::{Snes, Region};
use puce16_core::savestate::StateError;
use common::{make_rom, make_rom_with, Map};

mod common;

#[test]
fn e08_12_roundtrip_hash() {
    let rom = make_rom(&[0x78, 0x18, 0xFB, 0x80, 0xFE], Map::LoRom);
    let mut snes = Snes::new(&rom, None).unwrap();
    for _ in 0..5 {
        snes.run_frame();
    }
    let blob = snes.save_state();
    let hash_before = snes.state_hash();

    let mut snes2 = Snes::new(&rom, None).unwrap();
    snes2.load_state(&blob).unwrap();
    assert_eq!(snes2.state_hash(), hash_before);
}

#[test]
fn e08_12_blob_size() {
    let rom = make_rom(&[0x78], Map::LoRom);
    let mut snes = Snes::new(&rom, None).unwrap();
    snes.run_frame();
    let blob = snes.save_state();
    // Expected ~1.3 MiB (VRAM 128K + WRAM 128K + APU RAM 64K + framebuffer)
    assert!(blob.len() >= 1_000_000, "blob too small: {}", blob.len());
    assert!(blob.len() <= 1_600_000, "blob too large: {}", blob.len());
}

#[test]
fn e08_12_resave_identical() {
    let rom = make_rom(&[0x78], Map::LoRom);
    let mut snes = Snes::new(&rom, None).unwrap();
    snes.run_frame();
    let blob = snes.save_state();

    snes.load_state(&blob).unwrap();
    let blob2 = snes.save_state();
    assert_eq!(blob, blob2);
}

#[test]
fn e08_12_continue_equals_uninterrupted() {
    let rom = make_rom(&[0x78], Map::LoRom);

    // Path A: run 5 frames, save, run 5 more
    let mut snes_a = Snes::new(&rom, None).unwrap();
    for _ in 0..5 {
        snes_a.run_frame();
    }
    let blob = snes_a.save_state();
    for _ in 0..5 {
        snes_a.run_frame();
    }
    let hash_a = snes_a.state_hash();

    // Path B: new console, load save from A, run 5 frames
    let mut snes_b = Snes::new(&rom, None).unwrap();
    snes_b.load_state(&blob).unwrap();
    for _ in 0..5 {
        snes_b.run_frame();
    }
    let hash_b = snes_b.state_hash();

    assert_eq!(hash_a, hash_b);
}

#[test]
fn e08_12_bad_version() {
    let rom = make_rom(&[0x78], Map::LoRom);
    let mut snes = Snes::new(&rom, None).unwrap();
    snes.run_frame();
    let mut blob = snes.save_state();

    // Corrupt version field (first 4 bytes) to version 2
    blob[0] = 2;
    blob[1] = 0;
    blob[2] = 0;
    blob[3] = 0;

    let err = snes.load_state(&blob).unwrap_err();
    assert!(matches!(err, StateError::Version(2)));
}

#[test]
fn e08_12_region_mismatch() {
    // Save from NTSC console
    let rom_ntsc = make_rom_with(&[0x78], Map::LoRom, 15, 0, 0x00, 0x01);
    let mut snes_ntsc = Snes::new(&rom_ntsc, Some(Region::Ntsc)).unwrap();
    snes_ntsc.run_frame();
    let blob = snes_ntsc.save_state();

    // Try to load into PAL console
    let rom_pal = make_rom_with(&[0x78], Map::LoRom, 15, 0, 0x00, 0x02);
    let mut snes_pal = Snes::new(&rom_pal, Some(Region::Pal)).unwrap();
    let err = snes_pal.load_state(&blob).unwrap_err();
    assert!(matches!(err, StateError::Mismatch("region")));
}

#[test]
fn e08_12_truncated() {
    let rom = make_rom(&[0x78], Map::LoRom);
    let mut snes = Snes::new(&rom, None).unwrap();
    snes.run_frame();
    let blob = snes.save_state();

    // Truncate to half
    let half = blob.len() / 2;
    let truncated = &blob[..half];
    let err = snes.load_state(truncated).unwrap_err();
    assert!(matches!(err, StateError::Truncated));

    // Add extra byte (trailing data)
    let mut extended = blob.clone();
    extended.push(0xFF);
    let err2 = snes.load_state(&extended).unwrap_err();
    assert!(matches!(err2, StateError::Mismatch("trailing data")));
}

#[test]
fn e08_12_frame_count_restored() {
    let rom = make_rom(&[0x78], Map::LoRom);
    let mut snes = Snes::new(&rom, None).unwrap();
    for _ in 0..5 {
        snes.run_frame();
    }
    assert_eq!(snes.frame_count(), 5);

    let blob = snes.save_state();

    // Load into new console
    let mut snes2 = Snes::new(&rom, None).unwrap();
    snes2.load_state(&blob).unwrap();
    assert_eq!(snes2.frame_count(), 5);

    // Run one more frame - should be 6, not reset to 1
    snes2.run_frame();
    assert_eq!(snes2.frame_count(), 6);
}

#[test]
fn e08_12_sram_roundtrip() {
    // ROM with SRAM (chipset 0x02 = SDD1, sram_log2=3 => 8KB)
    let rom = make_rom_with(&[0x78], Map::LoRom, 15, 3, 0x02, 0x01);
    let mut snes = Snes::new(&rom, None).unwrap();

    // Write a pattern to SRAM
    let sram_size = 8 * 1024;
    let mut pattern = vec![0u8; sram_size];
    for i in 0..sram_size {
        pattern[i] = (i % 256) as u8;
    }
    snes.load_sram(&pattern);

    // Save state
    let blob = snes.save_state();

    // Load into new console and verify SRAM survived
    let mut snes2 = Snes::new(&rom, None).unwrap();
    snes2.load_state(&blob).unwrap();
    let sram_after = snes2.sram().expect("SRAM should exist");
    assert_eq!(sram_after, pattern);
}
