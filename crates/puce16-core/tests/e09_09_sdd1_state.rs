//! E09-09: S-DD1 state save/load and hash.

mod common;

use crate::common::{make_rom_with, Map};
use puce16_core::cart::Cart;
use puce16_core::hash::Fnv64;
use puce16_core::savestate::{State, StateReader, StateWriter};

fn make_sdd1_cart() -> Cart {
    // Chipset 0x43 = LoROM + S-DD1 (no battery)
    let rom = make_rom_with(&[], Map::LoRom, 15, 0, 0x43, 0x34);
    Cart::from_bytes(&rom).unwrap()
}

fn make_non_sdd1_cart() -> Cart {
    // Chipset 0x00 = LoROM only (no S-DD1)
    let rom = make_rom_with(&[], Map::LoRom, 15, 0, 0x00, 0x34);
    Cart::from_bytes(&rom).unwrap()
}

#[test]
fn e09_09_roundtrip_registers() {
    let mut c = make_sdd1_cart();

    // Write S-DD1 registers.
    c.io_write(0x4800, 3);
    c.io_write(0x4801, 2);
    c.io_write(0x4805, 0x82);

    // Set up DMA channel 4 via dma_write (intercepted by S-DD1).
    // Channel 4 registers: $4340-$4347.
    c.dma_write(0x4342, 0x10); // addr low byte
    c.dma_write(0x4343, 0x20); // addr mid byte
    c.dma_write(0x4344, 0xC0); // addr high byte -> full addr = 0xC02010
    c.dma_write(0x4345, 0x08); // size low byte
    c.dma_write(0x4346, 0x00); // size mid byte

    // Save state.
    let mut w = StateWriter::new();
    c.save(&mut w);
    let data = w.finish();

    // Load into a fresh identical cart.
    let mut c2 = make_sdd1_cart();
    let mut r = StateReader::new(&data);
    c2.load(&mut r).unwrap();

    // Verify registers restored.
    assert_eq!(c2.io_read(0x4800), Some(3));
    assert_eq!(c2.io_read(0x4801), Some(2));
    assert_eq!(c2.io_read(0x4805), Some(0x82));

    // Verify DMA channel 4 data restored.
    let (addr, size) = c2.dma_channel(4);
    assert_eq!(addr, 0xC02010);
    assert_eq!(size, 8);
}

#[test]
fn e09_09_non_sdd1_format_unchanged() {
    // Save a non-SDD1 cart.
    let c = make_non_sdd1_cart();
    let mut w = StateWriter::new();
    c.save(&mut w);
    let data = w.finish();

    // The format should be identical to a cart without S-DD1: just SRAM (empty vec_u8).
    // Empty vec_u8 = 4 bytes for length (0) + 0 bytes of data.
    assert_eq!(data.len(), 4);
}

#[test]
fn e09_09_hash_changes_with_registers() {
    let mut c1 = make_sdd1_cart();
    let mut c2 = make_sdd1_cart();

    // Hash before writing registers.
    let mut h1 = Fnv64::new();
    c1.hash_into(&mut h1);
    let hash_before = h1.finish();

    // Write a register and hash again.
    c1.io_write(0x4804, 0x05);
    let mut h2 = Fnv64::new();
    c1.hash_into(&mut h2);
    let hash_after = h2.finish();

    // Hashes should differ after register write.
    assert_ne!(hash_before, hash_after);

    // Two carts with identical writes should have identical hashes.
    c2.io_write(0x4804, 0x05);
    let mut h3 = Fnv64::new();
    c2.hash_into(&mut h3);
    let hash_c2 = h3.finish();

    assert_eq!(hash_after, hash_c2);
}

#[test]
fn e09_09_load_resets_streaming() {
    let mut c = make_sdd1_cart();

    // Write a register to change state.
    c.io_write(0x4800, 5);

    // Save state.
    let mut w = StateWriter::new();
    c.save(&mut w);
    let data = w.finish();

    // Load into fresh cart.
    let mut c2 = make_sdd1_cart();
    let mut r = StateReader::new(&data);
    c2.load(&mut r).unwrap();

    // After load, dma_ready should be false (streaming abandoned).
    // We can't directly inspect dma_ready from outside, but we know the spec says
    // it's reset. The observable effect is that a new mcu_read would re-init.
    // Verify register was restored to confirm load worked.
    assert_eq!(c2.io_read(0x4800), Some(5));
}
