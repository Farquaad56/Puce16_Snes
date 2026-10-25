//! E03-03 tests for score_header, pick_from_scores, and pick_header.

mod common;

use common::{make_rom, Map};
use puce16_core::cart::{score_header, pick_from_scores, pick_header, MapMode, CartError};

#[test]
fn e03_03_lorom_score() {
    let rom = make_rom(&[0x78], Map::LoRom);
    assert_eq!(score_header(&rom, MapMode::LoRom), Some(13));

    // Bad first byte (BRK)
    let rom_bad = make_rom(&[0x00], Map::LoRom);
    assert_eq!(score_header(&rom_bad, MapMode::LoRom), Some(10));

    // Neutral first byte (NOP)
    let rom_neutral = make_rom(&[0xEA], Map::LoRom);
    assert_eq!(score_header(&rom_neutral, MapMode::LoRom), Some(12));

    // Wrong positions should return None for 32 KiB ROM
    assert_eq!(score_header(&rom, MapMode::HiRom), None);
    assert_eq!(score_header(&rom, MapMode::ExHiRom), None);
}

#[test]
fn e03_03_hirom_picks() {
    let rom = make_rom(&[0x78], Map::HiRom);
    assert_eq!(score_header(&rom, MapMode::HiRom), Some(13));
    assert_eq!(score_header(&rom, MapMode::LoRom), Some(-4));
    assert_eq!(pick_header(&rom), Ok(MapMode::HiRom));
}

#[test]
fn e03_03_exhirom_picks() {
    let rom = make_rom(&[0x78], Map::ExHiRom);
    assert_eq!(score_header(&rom, MapMode::ExHiRom), Some(14));
    assert_eq!(score_header(&rom, MapMode::LoRom), Some(-4));
    assert_eq!(score_header(&rom, MapMode::HiRom), Some(-4));
    assert_eq!(pick_header(&rom), Ok(MapMode::ExHiRom));
}

#[test]
fn e03_03_pick_scores() {
    // Tie goes to first in order (LoRom)
    assert_eq!(pick_from_scores([Some(5), Some(5), None]), Some(MapMode::LoRom));
    // Highest wins; tie between HiRom and ExHiRom goes to HiRom
    assert_eq!(pick_from_scores([Some(3), Some(5), Some(5)]), Some(MapMode::HiRom));
    // All None -> None
    assert_eq!(pick_from_scores([None, None, None]), None);
    // All equal negative -> first wins
    assert_eq!(pick_from_scores([Some(-4), Some(-4), Some(-4)]), Some(MapMode::LoRom));
    // Only one valid
    assert_eq!(pick_from_scores([None, Some(-1), Some(-2)]), Some(MapMode::HiRom));
}

#[test]
fn e03_03_no_header() {
    let rom = vec![0u8; 0x4000];
    assert_eq!(pick_header(&rom), Err(CartError::NoHeader));
}