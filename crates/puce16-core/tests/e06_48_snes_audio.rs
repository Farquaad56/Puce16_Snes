//! E06-48: Snes::take_audio real implementation and audio sample rate constant.

use puce16_core::snes::{Snes, AUDIO_SAMPLE_RATE};

mod common;

/// Test ROM: SEI ; CLC ; XCE ; BRA -2 (infinite loop).
fn build_rom() -> Vec<u8> {
    let code = [0x78, 0x18, 0xFB, 0x80, 0xFE]; // SEI; CLC; XCE; BRA -2
    common::make_rom(&code, common::Map::LoRom)
}

#[test]
fn e06_48_sample_rate_constant() {
    assert_eq!(AUDIO_SAMPLE_RATE, 32_040);
}

#[test]
fn e06_48_samples_per_run() {
    let rom = build_rom();
    let mut snes = Snes::new(&rom, None).unwrap();
    for _ in 0..3 {
        snes.run_frame();
    }
    let mut v: Vec<[i16; 2]> = vec![];
    snes.take_audio(&mut v);
    assert!(v.len() >= 1450 && v.len() <= 1650, "expected 1450..=1650 pairs, got {}", v.len());
}

#[test]
fn e06_48_silent_by_default() {
    let rom = build_rom();
    let mut snes = Snes::new(&rom, None).unwrap();
    for _ in 0..3 {
        snes.run_frame();
    }
    let mut v: Vec<[i16; 2]> = vec![];
    snes.take_audio(&mut v);
    assert!(v.len() >= 1450, "expected at least 1450 pairs");
    for pair in &v {
        assert_eq!(pair[0], 0, "left channel should be silent (FLG=0xE0)");
        assert_eq!(pair[1], 0, "right channel should be silent (FLG=0xE0)");
    }
}

#[test]
fn e06_48_take_appends_and_drains() {
    let rom = build_rom();
    let mut snes = Snes::new(&rom, None).unwrap();
    for _ in 0..3 {
        snes.run_frame();
    }
    let mut v: Vec<[i16; 2]> = vec![[7, 7]];
    snes.take_audio(&mut v);
    assert_eq!(v[0], [7, 7], "original element preserved");
    assert!(v.len() >= 2, "take_audio should have appended samples");
    let len_after_first = v.len();
    snes.take_audio(&mut v);
    assert_eq!(v.len(), len_after_first, "second immediate take_audio adds nothing");
}

#[test]
fn e06_48_determinism() {
    let rom = build_rom();
    let mut snes1 = Snes::new(&rom, None).unwrap();
    let mut snes2 = Snes::new(&rom, None).unwrap();
    for _ in 0..2 {
        snes1.run_frame();
        snes2.run_frame();
    }
    let mut v1: Vec<[i16; 2]> = vec![];
    let mut v2: Vec<[i16; 2]> = vec![];
    snes1.take_audio(&mut v1);
    snes2.take_audio(&mut v2);
    assert_eq!(v1, v2, "audio vectors must be identical");
    assert_eq!(snes1.state_hash(), snes2.state_hash());
}

#[test]
fn e06_48_per_frame_count_stable() {
    let rom = build_rom();
    let mut snes = Snes::new(&rom, None).unwrap();
    // Run 3 frames to warm up (discard)
    for _ in 0..3 {
        snes.run_frame();
    }
    snes.take_audio(&mut vec![]); // drain any accumulated audio

    // Frames 4 through 8: measure per-frame sample count
    for frame_num in 4..=8 {
        let mut v: Vec<[i16; 2]> = vec![];
        snes.run_frame();
        snes.take_audio(&mut v);
        assert!(v.len() >= 530 && v.len() <= 537,
            "frame {}: expected 530..=537 pairs, got {}", frame_num, v.len());
    }
}
