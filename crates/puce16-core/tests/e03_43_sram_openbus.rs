//! E03-43: Integration - SRAM, open bus, and power-on register values.

mod common;
#[path = "common/prog.rs"]
mod prog;

use puce16_core::snes::Snes;
use prog::{build_rom, build_rom_with};

/// PROG_BUS: test SRAM read/write, open bus, DMA/WRAM power-on values.
fn build_bus_prog() -> Vec<u8> {
    vec![
        // SEI CLC XCE - disable interrupts, clear carry, switch to native mode
        0x78, 0x18, 0xFB,
        // LDA $700000 (SRAM read)
        0xAF, 0x00, 0x00, 0x70,
        // STA $0145 (store SRAM value to WRAM for inspection)
        0x8D, 0x45, 0x01,
        // LDA #$5A
        0xA9, 0x5A,
        // STA $700000 (SRAM write)
        0x8F, 0x00, 0x00, 0x70,
        // LDA $5000 (open bus - unmapped address)
        0xAD, 0x00, 0x50,
        // STA $0140 (store open bus value)
        0x8D, 0x40, 0x01,
        // LDA $4300 (DMAP0 power-on value)
        0xAD, 0x00, 0x43,
        // STA $0141
        0x8D, 0x41, 0x01,
        // LDA $1FFF (WRAM power-on value)
        0xAD, 0xFF, 0x1F,
        // STA $0142
        0x8D, 0x42, 0x01,
        // LDA $4210 (RDNMI - flag false, partial open bus)
        0xAD, 0x10, 0x42,
        // STA $0143
        0x8D, 0x43, 0x01,
        // LDA $4213 (RDIO power-on value)
        0xAD, 0x13, 0x42,
        // STA $0144
        0x8D, 0x44, 0x01,
        // BRA L (loop forever)
        0x80, 0xFE,
    ]
}

fn snes(rom: Vec<u8>) -> Snes {
    Snes::new(&rom, None).ok().unwrap()
}

/// Let the CPU take any pending interrupt after run_frame.
fn settle(s: &mut Snes) {
    for _ in 0..12 {
        s.step_instruction();
    }
}

#[test]
fn e03_43_sram_written() {
    let rom = build_rom_with(&[(0, &build_bus_prog())], &[], 3, 0x02);
    let mut s = snes(rom);

    s.run_frame();
    settle(&mut s);

    assert_eq!(s.sram().unwrap()[0], 0x5A, "SRAM write failed");
}

#[test]
fn e03_43_sram_preloaded() {
    let rom = build_rom_with(&[(0, &build_bus_prog())], &[], 3, 0x02);
    let mut s = snes(rom);

    // Preload SRAM with a known value before running the program
    s.load_sram(&[0x77]);

    s.run_frame();
    settle(&mut s);

    // The program reads SRAM first (LDA $700000) and stores to $0145
    assert_eq!(s.peek_wram(0x145), 0x77, "SRAM read of preloaded value failed");
    // Then the program writes 0x5A back to SRAM
    assert_eq!(s.sram().unwrap()[0], 0x5A, "SRAM write after preload failed");
}

#[test]
fn e03_43_open_bus_value() {
    let rom = build_rom_with(&[(0, &build_bus_prog())], &[], 3, 0x02);
    let mut s = snes(rom);

    s.run_frame();
    settle(&mut s);

    // LDA $5000 reads open bus - should return last value seen on data bus
    // The last write before this was STA $700000 with value 0x5A, but the
    // address bytes of that instruction (0x00, 0x00, 0x70) were also on the bus.
    // Actually, for LDA $5000, the operand is read as part of the instruction fetch,
    // so the last value on the bus would be the high byte of the address (0x50).
    assert_eq!(s.peek_wram(0x140), 0x50, "open bus value wrong");
}

#[test]
fn e03_43_dma_power_on() {
    let rom = build_rom_with(&[(0, &build_bus_prog())], &[], 3, 0x02);
    let mut s = snes(rom);

    s.run_frame();
    settle(&mut s);

    // DMAP0 ($4300) should be 0xFF at power-on
    assert_eq!(s.peek_wram(0x141), 0xFF, "DMAP0 power-on value wrong");
}

#[test]
fn e03_43_wram_power_on() {
    let rom = build_rom_with(&[(0, &build_bus_prog())], &[], 3, 0x02);
    let mut s = snes(rom);

    s.run_frame();
    settle(&mut s);

    // WRAM ($1FFF) should be 0x55 at power-on
    assert_eq!(s.peek_wram(0x142), 0x55, "WRAM power-on value wrong");
}

#[test]
fn e03_43_rdnmi_partial_open_bus() {
    let rom = build_rom_with(&[(0, &build_bus_prog())], &[], 3, 0x02);
    let mut s = snes(rom);

    s.run_frame();
    settle(&mut s);

    // RDNMI ($4210) with flag false: bits 6-4 are open bus (from mdr), bit 1 is the flag
    // Expected: (0x42 & 0x70) | 0x02 = 0x72... wait, let me recalculate.
    // The spec says: peek_wram(0x143)==0x42 ((0x42 & 0x70) | 0x02, flag faux).
    // So the expected value is 0x42. This means mdr at that point was 0x42 (from reading $4210's address bytes),
    // and the result is (mdr & 0x70) | 0x02 = (0x42 & 0x70) | 0x02 = 0x40 | 0x02 = 0x42.
    assert_eq!(s.peek_wram(0x143), 0x42, "RDNMI partial open bus value wrong");
}

#[test]
fn e03_43_rdio_power_on() {
    let rom = build_rom_with(&[(0, &build_bus_prog())], &[], 3, 0x02);
    let mut s = snes(rom);

    s.run_frame();
    settle(&mut s);

    // RDIO ($4213) should be 0xFF at power-on
    assert_eq!(s.peek_wram(0x144), 0xFF, "RDIO power-on value wrong");
}

#[test]
fn e03_43_no_sram_rom() {
    // Same program but with no SRAM (chipset 0)
    let rom = build_rom(&[(0, &build_bus_prog())], &[]);
    let mut s = snes(rom);

    s.run_frame();
    settle(&mut s);

    assert!(s.sram().is_none(), "ROM should have no SRAM");

    // Without SRAM, LDA $700000 reads the ROM mirror (LoROM bank 0x70, addr < 0x8000)
    // which maps to offset 0 of the ROM = first byte of program = SEI = 0x78
    assert_eq!(s.peek_wram(0x145), 0x78, "LoROM mirror read without SRAM wrong");
}
