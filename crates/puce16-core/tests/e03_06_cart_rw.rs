use puce16_core::cart::{Cart, MapMode};
use puce16_core::hash::Fnv64;

mod common;
use common::{make_rom, make_rom_with, Map};

#[test]
fn e03_06_lorom_read() {
    let rom = make_rom(&[0xA9, 0x01], Map::LoRom);
    let cart = Cart::from_bytes_forced(&rom, Some(MapMode::LoRom), None).unwrap();
    assert_eq!(cart.read(0x00, 0x8000), Some(0xA9));
    assert_eq!(cart.read(0x00, 0x8001), Some(0x01));
    assert_eq!(cart.read(0x80, 0x8000), Some(0xA9)); // mirrored bank
    assert_eq!(cart.read(0x00, 0xFFFC), Some(0x00)); // reset vector low byte
    assert_eq!(cart.read(0x00, 0xFFFD), Some(0x80)); // reset vector high byte
    assert_eq!(cart.read(0x00, 0x7FFF), None);       // below ROM region
    assert_eq!(cart.read(0x7E, 0x8000), None);      // system bank
}

#[test]
fn e03_06_hirom_read() {
    let rom = make_rom(&[0xA9, 0x01], Map::HiRom);
    let cart = Cart::from_bytes_forced(&rom, Some(MapMode::HiRom), None).unwrap();
    assert_eq!(cart.read(0xC0, 0x8000), Some(0xA9));
    assert_eq!(cart.read(0x00, 0x8000), Some(0xA9)); // high half only bank
    assert_eq!(cart.read(0x80, 0x8000), Some(0xA9)); // mirrored
    assert_eq!(cart.read(0xC0, 0xFFFD), Some(0x80));
    assert_eq!(cart.read(0x40, 0xFFFD), Some(0x80)); // mirror of C0
    assert_eq!(cart.read(0x00, 0x7FFF), None);       // below ROM region
}

#[test]
fn e03_06_exhirom_read() {
    let rom = make_rom(&[0xA9, 0x01], Map::ExHiRom);
    let cart = Cart::from_bytes_forced(&rom, Some(MapMode::ExHiRom), None).unwrap();
    assert_eq!(cart.read(0x00, 0x8000), Some(0xA9)); // second half (bank < 0x80)
    assert_eq!(cart.read(0x00, 0xFFFD), Some(0x80));
    assert_eq!(cart.read(0x40, 0xFFFD), Some(0x80));
    assert_eq!(cart.read(0xC0, 0x8000), Some(0x00)); // first half (bank >= 0x80) = zeros
}

#[test]
fn e03_06_rom_write_ignored() {
    let rom = make_rom(&[0xA9], Map::LoRom);
    let mut cart = Cart::from_bytes_forced(&rom, Some(MapMode::LoRom), None).unwrap();
    assert_eq!(cart.write(0x00, 0x8000, 0x55), false);
    assert_eq!(cart.read(0x00, 0x8000), Some(0xA9)); // unchanged
}

#[test]
fn e03_06_lorom_sram() {
    let rom = make_rom_with(&[], Map::LoRom, 15, 3, 0x02, 0x01); // 8KB SRAM
    let mut cart = Cart::from_bytes_forced(&rom, Some(MapMode::LoRom), None).unwrap();
    assert_eq!(cart.write(0x70, 0x0000, 0x5A), true);
    assert_eq!(cart.read(0x70, 0x0000), Some(0x5A));
    assert_eq!(cart.read(0x70, 0x2000), Some(0x5A)); // mirror modulo 0x2000
    assert_eq!(cart.read(0xF0, 0x0000), Some(0x5A)); // mirrored bank
    assert_eq!(cart.write(0x00, 0x8000, 1), false);  // ROM write ignored
    assert_eq!(cart.sram()[0], 0x5A);
}

#[test]
fn e03_06_hirom_sram() {
    let rom = make_rom_with(&[], Map::HiRom, 16, 3, 0x02, 0x01); // 8KB SRAM
    let mut cart = Cart::from_bytes_forced(&rom, Some(MapMode::HiRom), None).unwrap();
    assert_eq!(cart.write(0x20, 0x6000, 0x77), true);
    assert_eq!(cart.read(0x20, 0x6000), Some(0x77));
    assert_eq!(cart.read(0xA0, 0x6000), Some(0x77)); // mirrored bank
    assert_eq!(cart.read(0x30, 0x6000), Some(0x77)); // (bank&0x1F)=0x10 -> offset wraps to 0
    assert_eq!(cart.read(0x20, 0x7FFF), Some(0x00)); // beyond written byte
}

#[test]
fn e03_06_load_sram() {
    let rom = make_rom_with(&[], Map::LoRom, 15, 3, 0x02, 0x01); // 8KB SRAM
    let mut cart = Cart::from_bytes_forced(&rom, Some(MapMode::LoRom), None).unwrap();
    cart.load_sram(&[1, 2, 3]);
    assert_eq!(&cart.sram()[0..4], &[1, 2, 3, 0]);
    assert_eq!(cart.sram().len(), 8192);

    cart.load_sram(&vec![9; 10000]);
    assert_eq!(cart.sram().len(), 8192);
    assert_eq!(cart.sram()[8191], 9);

    cart.load_sram(&[]);
    for &b in cart.sram() {
        assert_eq!(b, 0);
    }
}

#[test]
fn e03_06_hash_sram() {
    let rom = make_rom_with(&[], Map::LoRom, 15, 3, 0x02, 0x01); // 8KB SRAM
    let mut cart1 = Cart::from_bytes_forced(&rom, Some(MapMode::LoRom), None).unwrap();
    let rom2 = make_rom_with(&[], Map::LoRom, 15, 3, 0x02, 0x01);
    let cart2 = Cart::from_bytes_forced(&rom2, Some(MapMode::LoRom), None).unwrap();

    let mut h1 = Fnv64::new();
    cart1.hash_into(&mut h1);
    let hash_before = h1.finish();

    let mut h2 = Fnv64::new();
    cart2.hash_into(&mut h2);
    assert_eq!(h2.finish(), hash_before); // identical carts -> same hash

    cart1.write(0x70, 0x0000, 1);
    let mut h3 = Fnv64::new();
    cart1.hash_into(&mut h3);
    assert_ne!(h3.finish(), hash_before); // after SRAM write -> different hash
}
