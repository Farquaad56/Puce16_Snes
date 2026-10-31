//! E09-12 (gate fix): banks C0-FF are the MMC window, never the LoROM or SRAM window.
//! Regression: with a 32 KiB ROM the wrongly selected window gave the same bytes through
//! the ROM size mask, so only a larger ROM exposes it (found with SFA2 / Star Ocean).

mod common;

use crate::common::{make_rom_with, Map};
use puce16_core::cart::Cart;

fn rom_1mib(chipset: u8, sram_log2: u8) -> Vec<u8> {
    let mut rom = make_rom_with(&[], Map::LoRom, 20, sram_log2, chipset, 0x34);
    rom[0x0032] = 0x11; // LoROM view of 00:8032
    rom[0x8032] = 0x22; // MMC view of C0:8032
    rom[0x6010] = 0x33; // MMC view of C0:6010
    rom
}

#[test]
fn e09_12_c0_window_reads_mmc_not_lorom() {
    let mut c = Cart::from_bytes(&rom_1mib(0x43, 0)).unwrap();
    assert_eq!(c.read(0x00, 0x8032), Some(0x11));
    assert_eq!(c.read(0x80, 0x8032), Some(0x11));
    assert_eq!(c.read(0xC0, 0x8032), Some(0x22));
    assert_eq!(c.read_bus(0xC0, 0x8032), Some(0x22));
    assert_eq!(c.read_bus(0x00, 0x8032), Some(0x11));
}

#[test]
fn e09_12_c0_window_is_not_sram() {
    let mut c = Cart::from_bytes(&rom_1mib(0x45, 3)).unwrap();
    // SRAM window: banks 00-3F/80-BF, $6000-$7FFF (zeros at power-on).
    assert_eq!(c.read(0x00, 0x6010), Some(0x00));
    assert_eq!(c.read(0x80, 0x6010), Some(0x00));
    // Banks C0-FF show the ROM there.
    assert_eq!(c.read(0xC0, 0x6010), Some(0x33));
    assert_eq!(c.read_bus(0xC0, 0x6010), Some(0x33));
}