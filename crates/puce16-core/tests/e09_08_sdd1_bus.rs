//! E09-08: S-DD1 bus routing ($4800-$480F, DMA snoop $4300-$437F, read_bus on C0-FF).

mod common;

use crate::common::{make_rom_with, Map};
use puce16_core::bus::SysBus;
use puce16_core::cart::Cart;
use puce16_core::region::Region;

/// Build a SysBus with an S-DD1 cart (chipset $43) and compressed stream at offset 0x100.
fn make_sdd1_bus() -> SysBus {
    // Compressed stream for header $00, seed 0x1000 (first 256 bytes).
    let stream_hex = "02cd62493a3edf3334e441bad6873ba4079262e637e3acf8dfecd38d9ca42b671929a234221703b38d00a785c98329725bdce10180091caac2d1650647425abb0a75d51b62bc68997a77749b973c62566110baa17ed622ff3141884ce3d6f7c3a6e3e153";
    let stream: Vec<u8> = (0..stream_hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&stream_hex[i..i + 2], 16).unwrap())
        .collect();

    // Pad to offset 0x100, then place stream.
    let mut code = vec![0u8; 0x100];
    code.extend_from_slice(&stream);

    let rom = make_rom_with(&code, Map::LoRom, 15, 0, 0x43, 0x34);
    let cart = Cart::from_bytes(&rom).unwrap();
    SysBus::new(cart, Region::Ntsc)
}

/// Build a SysBus with a plain LoROM cart (chipset $00, no S-DD1).
fn make_plain_bus() -> SysBus {
    let rom = make_rom_with(&[], Map::LoRom, 15, 0, 0x00, 0x34);
    let cart = Cart::from_bytes(&rom).unwrap();
    SysBus::new(cart, Region::Ntsc)
}

#[test]
fn e09_08_registers_visible_on_the_bus() {
    let mut bus = make_sdd1_bus();

    // Write to $4801 and read it back.
    bus.write_inner(0x004801, 0xA5);
    assert_eq!(bus.read_inner(0x004801), 0xA5);

    // Bank 0x80 is a mirror of bank 0x00 for system registers.
    assert_eq!(bus.read_inner(0x804801), 0xA5);

    // $4802 is not an S-DD1 register -> open bus (returns mdr).
    bus.mdr = 0x77;
    assert_eq!(bus.read_inner(0x004802), 0x77);
}

#[test]
fn e09_08_dma_registers_still_work() {
    let mut bus = make_sdd1_bus();

    // Write to DMA register $4302 (channel 0 address low byte).
    bus.write_inner(0x004302, 0x34);
    assert_eq!(bus.read_inner(0x004302), 0x34);

    // The normal DMA still receives the write.
}

#[test]
fn e09_08_decompressed_stream_on_c0_reads() {
    let mut bus = make_sdd1_bus();

    // Enable S-DD1 channel 0: $4800 and $4801 bit 0 set.
    bus.write_inner(0x004800, 0x01);
    bus.write_inner(0x004801, 0x01);

    // Program DMA channel 0: source address 0xC00100, size 4.
    // $4302 = addr low (0x00), $4303 = addr mid (0x01), $4304 = addr high (0xC0)
    // $4305 = size low (0x04), $4306 = size high (0x00)
    bus.write_inner(0x004302, 0x00);
    bus.write_inner(0x004303, 0x01);
    bus.write_inner(0x004304, 0xC0);
    bus.write_inner(0x004305, 0x04);
    bus.write_inner(0x004306, 0x00);

    // Read from bank C0 offset 0x0100 (address 0xC00100).
    // First 4 reads should return decompressed bytes: 0x75 0x12 0x24 0x91.
    assert_eq!(bus.read_inner(0xC00100), 0x75);
    assert_eq!(bus.read_inner(0xC00100), 0x12);
    assert_eq!(bus.read_inner(0xC00100), 0x24);
    assert_eq!(bus.read_inner(0xC00100), 0x91);

    // After size reaches 0, the 5th read returns normal ROM data.
    let rom_byte = bus.cart.read_bus(0xC0, 0x0100).unwrap_or(0xFF);
    assert_eq!(bus.read_inner(0xC00100), rom_byte);
}

#[test]
fn e09_08_no_sdd1_rom_unchanged() {
    let mut bus = make_plain_bus();

    // Without S-DD1 (chipset $00), $4801 is open bus.
    bus.mdr = 0x55;
    assert_eq!(bus.read_inner(0x004801), 0x55);

    // Reads from bank C0 return ROM data directly, no decompression.
    let rom_byte = bus.cart.read_bus(0xC0, 0x0100).unwrap_or(0xFF);
    assert_eq!(bus.read_inner(0xC00100), rom_byte);
    // Same byte on repeated reads (no state change).
    assert_eq!(bus.read_inner(0xC00100), rom_byte);
}
