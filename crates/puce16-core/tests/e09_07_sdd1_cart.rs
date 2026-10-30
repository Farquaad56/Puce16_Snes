//! E09-07: S-DD1 header acceptance and Cart integration (ROM, SRAM).

mod common;

use crate::common::{make_rom_with, Map};
use puce16_core::cart::{Cart, CartError};

#[test]
fn e09_07_chipset_43_loads_without_battery() {
    let rom = make_rom_with(&[], Map::LoRom, 15, 0, 0x43, 0x34);
    let cart = Cart::from_bytes(&rom).unwrap();
    assert!(cart.has_sdd1());
    assert!(!cart.has_battery());
}

#[test]
fn e09_07_chipset_45_has_battery_and_sram() {
    // sram_log2=3 -> 8 KiB SRAM (1024 << 3)
    let rom = make_rom_with(&[], Map::LoRom, 15, 3, 0x45, 0x34);
    let cart = Cart::from_bytes(&rom).unwrap();
    assert!(cart.has_sdd1());
    assert!(cart.has_battery());
    assert_eq!(cart.sram().len(), 8192);
}

#[test]
fn e09_07_other_coprocessors_still_refused() {
    // GSU ($13) should be refused
    let rom_gsu = make_rom_with(&[], Map::LoRom, 15, 0, 0x13, 0x34);
    match Cart::from_bytes(&rom_gsu) {
        Err(CartError::UnsupportedCoprocessor("GSU")) => {} // expected
        Ok(_) => panic!("expected UnsupportedCoprocessor(GSU), got Ok"),
        Err(e) => panic!("expected UnsupportedCoprocessor(GSU), got {:?}", e),
    }

    // SA-1 ($34) should be refused
    let rom_sa1 = make_rom_with(&[], Map::LoRom, 15, 0, 0x34, 0x34);
    match Cart::from_bytes(&rom_sa1) {
        Err(CartError::UnsupportedCoprocessor("SA-1")) => {} // expected
        Ok(_) => panic!("expected UnsupportedCoprocessor(SA-1), got Ok"),
        Err(e) => panic!("expected UnsupportedCoprocessor(SA-1), got {:?}", e),
    }

    // Plain LoROM ($00) should load without S-DD1
    let rom_plain = make_rom_with(&[], Map::LoRom, 15, 0, 0x00, 0x34);
    let cart = Cart::from_bytes(&rom_plain).unwrap();
    assert!(!cart.has_sdd1());
}

#[test]
fn e09_07_lorom_window_reads_rom() {
    // Build ROM with known byte at offset 4
    let mut code = vec![0u8; 32];
    code[4] = 0xAB;
    let rom = make_rom_with(&code, Map::LoRom, 15, 0, 0x43, 0x34);
    let cart = Cart::from_bytes(&rom).unwrap();

    // LoROM window: bank 00 offset $8004 -> file offset 4
    assert_eq!(cart.read(0x00, 0x8004), Some(0xAB));
    // Bank 80 is a mirror of bank 00 in LoROM
    assert_eq!(cart.read(0x80, 0x8004), Some(0xAB));
    // Banks 40-7D: no ROM (open bus)
    assert_eq!(cart.read(0x40, 0x8004), None);
    // Offset below $8000 in LoROM window: not ROM
    assert_eq!(cart.read(0x00, 0x2000), None);
}

#[test]
fn e09_07_mmc_window_reads_rom() {
    let mut code = vec![0u8; 32];
    code[4] = 0xAB;
    let rom = make_rom_with(&code, Map::LoRom, 15, 0, 0x43, 0x34);
    let mut cart = Cart::from_bytes(&rom).unwrap();

    // MMC window: bank C0 offset $0004 -> block 0 (reg[4804]=0) -> file offset 4
    assert_eq!(cart.read(0xC0, 0x0004), Some(0xAB));

    // Remap block 0 to point at block 1 via reg $4804
    cart.io_write(0x4804, 0x01);
    // With ROM size 32 KiB (mask 0x7FFF), offset 0x100004 & mask = 0x0004 -> still 0xAB
    assert_eq!(cart.read(0xC0, 0x0004), Some(0xAB));
}

#[test]
fn e09_07_sram_windows_mirror() {
    // Chipset $45 with 8 KiB SRAM (sram_log2=3)
    let rom = make_rom_with(&[], Map::LoRom, 15, 3, 0x45, 0x34);
    let mut cart = Cart::from_bytes(&rom).unwrap();

    // Write via LoROM SRAM window (bank 00, offset $6000)
    cart.write(0x00, 0x6000, 0x5A);
    // Read back via bank 70 mirror (offset & 0x1FFF = 0 -> index 0)
    assert_eq!(cart.read(0x70, 0x0000), Some(0x5A));

    // Write at offset $7FFF
    cart.write(0x00, 0x7FFF, 0x33);
    // Read back via bank 80 (mirror of bank 00)
    assert_eq!(cart.read(0x80, 0x7FFF), Some(0x33));

    // Offset $8000 is ROM window, not SRAM
    cart.write(0x00, 0x8000, 0xFF);
    assert_ne!(cart.read(0x00, 0x8000), Some(0xFF));
}

#[test]
fn e09_07_read_bus_without_sdd1_equals_read() {
    let rom = make_rom_with(&[], Map::LoRom, 15, 0, 0x00, 0x34);
    let mut cart = Cart::from_bytes(&rom).unwrap();

    // Test various bank/addr combinations: read_bus should equal read
    let banks = [0x00, 0x20, 0x40, 0x60, 0x80, 0xA0, 0xC0, 0xE0];
    let addrs = [0x0000, 0x2000, 0x4000, 0x6000, 0x8000, 0xA000, 0xC000, 0xE000];

    for bank in banks {
        for addr in addrs {
            let r = cart.read(bank, addr);
            let rb = cart.read_bus(bank, addr);
            assert_eq!(r, rb, "bank {:02X} addr {:04X}: read={:?} read_bus={:?}", bank, addr, r, rb);
        }
    }
}
