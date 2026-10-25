//! E03-05: Cart::from_bytes / from_bytes_forced and accessors.

use puce16_core::cart::{Cart, CartError, MapMode};
use puce16_core::region::Region;

mod common;
use common::{make_rom, make_rom_with, Map};

#[test]
fn e03_05_lorom() {
    let rom = make_rom(&[0xA9, 0x01], Map::LoRom);
    let cart = Cart::from_bytes(&rom).unwrap();
    assert_eq!(cart.map_mode(), MapMode::LoRom);
    assert_eq!(cart.region(), Region::Ntsc); // country 1 -> NTSC
    assert_eq!(cart.rom_len(), 32768);
    assert!(!cart.has_battery());
    assert_eq!(cart.sram().len(), 0);
    assert_eq!(cart.header().title, "PUCE16 TEST ROM");
}

#[test]
fn e03_05_hirom() {
    let rom = make_rom(&[0xA9, 0x01], Map::HiRom);
    let cart = Cart::from_bytes(&rom).unwrap();
    assert_eq!(cart.map_mode(), MapMode::HiRom);
    assert_eq!(cart.rom_len(), 65536);
}

#[test]
fn e03_05_exhirom() {
    let rom = make_rom(&[0xA9, 0x01], Map::ExHiRom);
    let cart = Cart::from_bytes(&rom).unwrap();
    assert_eq!(cart.map_mode(), MapMode::ExHiRom);
    assert_eq!(cart.rom_len(), 8 * 1024 * 1024);
}

#[test]
fn e03_05_copier_header() {
    let rom = make_rom(&[0xA9, 0x01], Map::LoRom);
    // Prepend 512 bytes of copier header
    let mut with_copier = vec![0u8; 512];
    with_copier.extend_from_slice(&rom);
    let cart = Cart::from_bytes(&with_copier).unwrap();
    assert_eq!(cart.map_mode(), MapMode::LoRom);
    assert_eq!(cart.rom_len(), 32768);
}

#[test]
fn e03_05_too_small() {
    let small = vec![0u8; 0x7FFF];
    assert_eq!(Cart::from_bytes(&small).err(), Some(CartError::TooSmall));
    assert_eq!(Cart::from_bytes(&[]).err(), Some(CartError::TooSmall));
}

#[test]
fn e03_05_coprocessor() {
    // Chipset 0x13 -> GSU coprocessor (high nibble 1)
    let rom = make_rom_with(&[], Map::LoRom, 15, 0, 0x13, 1);
    assert_eq!(
        Cart::from_bytes(&rom).err(),
        Some(CartError::UnsupportedCoprocessor("GSU"))
    );

    // Chipset 0x34 -> SA-1 coprocessor (high nibble 3)
    let rom2 = make_rom_with(&[], Map::LoRom, 15, 0, 0x34, 1);
    assert_eq!(
        Cart::from_bytes(&rom2).err(),
        Some(CartError::UnsupportedCoprocessor("SA-1"))
    );
}

#[test]
fn e03_05_sram_battery() {
    // Chipset 0x02 -> battery-backed, no coproc; sram_log2=3 -> 8 KiB
    let rom = make_rom_with(&[], Map::LoRom, 15, 3, 0x02, 1);
    let cart = Cart::from_bytes(&rom).unwrap();
    assert!(cart.has_battery());
    assert_eq!(cart.sram().len(), 8192);
    // All SRAM bytes should be zero-initialized
    for &b in cart.sram() {
        assert_eq!(b, 0);
    }
}

#[test]
fn e03_05_forced() {
    let rom = make_rom(&[0x78], Map::LoRom);

    // Force PAL region
    let cart = Cart::from_bytes_forced(&rom, Some(MapMode::LoRom), Some(Region::Pal)).unwrap();
    assert_eq!(cart.region(), Region::Pal);

    // Force HiRom on a 32 KiB ROM -> no header at 0xFFC0
    let lorom32k = make_rom(&[0x78], Map::LoRom);
    assert_eq!(
        Cart::from_bytes_forced(&lorom32k, Some(MapMode::HiRom), None).err(),
        Some(CartError::NoHeader)
    );
}

#[test]
fn e03_05_pal_country() {
    // Country 2 -> PAL
    let rom = make_rom_with(&[], Map::LoRom, 15, 0, 0x00, 2);
    let cart = Cart::from_bytes(&rom).unwrap();
    assert_eq!(cart.region(), Region::Pal);
}
