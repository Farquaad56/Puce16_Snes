//! E01-08 tests for the synthetic ROM builder (tests/common/mod.rs).

mod common;

use common::{make_rom, make_rom_with, set_vector, Map};

/// u16 wrapping sum of all bytes of the ROM.
fn byte_sum(rom: &[u8]) -> u16 {
    let mut sum: u16 = 0;
    for &b in rom.iter() {
        sum = sum.wrapping_add(b as u16);
    }
    sum
}

/// Read the checksum (LE) at header+0x1E.
fn read_checksum(rom: &[u8], header: usize) -> u16 {
    u16::from_le_bytes([rom[header + 0x1E], rom[header + 0x1F]])
}

/// Read the complement (LE) at header+0x1C.
fn read_complement(rom: &[u8], header: usize) -> u16 {
    u16::from_le_bytes([rom[header + 0x1C], rom[header + 0x1D]])
}

#[test]
fn e01_08_lorom_layout() {
    let rom = make_rom(&[0xEA], Map::LoRom);
    assert_eq!(rom.len(), 32768);
    assert_eq!(rom[0], 0xEA);
    assert_eq!(&rom[0x7FFC..0x7FFE], &[0x00, 0x80]); // reset vector $8000 LE
    assert_eq!(rom[0x7FD5], 0x20); // map byte: speed=1, LoROM
    assert_eq!(&rom[0x7FC0..0x7FC6], b"PUCE16");
}

#[test]
fn e01_08_hirom_layout() {
    let rom = make_rom(&[0xEA], Map::HiRom);
    assert_eq!(rom.len(), 65536);
    assert_eq!(rom[0x8000], 0xEA);
    assert_eq!(&rom[0xFFFC..0xFFFE], &[0x00, 0x80]); // reset vector $8000 LE
    assert_eq!(rom[0xFFD5], 0x21); // map byte: speed=1, HiROM
}

#[test]
fn e01_08_checksum_valid() {
    for (map, header) in [(Map::LoRom, 0x7FC0usize), (Map::HiRom, 0xFFC0)] {
        let rom = make_rom(&[0xEA], map);
        assert_eq!(byte_sum(&rom), read_checksum(&rom, header));
        assert_eq!(read_checksum(&rom, header) ^ read_complement(&rom, header), 0xFFFF);
    }
}

#[test]
fn e01_08_set_vector() {
    let mut rom = make_rom(&[0xEA], Map::LoRom);
    set_vector(&mut rom, Map::LoRom, "nmi", 0x8123);
    assert_eq!(&rom[0x7FEA..0x7FEC], &[0x23, 0x81]); // native NMI $FFEA
    assert_eq!(&rom[0x7FFA..0x7FFC], &[0x23, 0x81]); // emulation NMI $FFFA
    // The checksum stays valid: set_vector recalculates it.
    let header = 0x7FC0;
    assert_eq!(byte_sum(&rom), read_checksum(&rom, header));
    assert_eq!(read_checksum(&rom, header) ^ read_complement(&rom, header), 0xFFFF);
}

#[test]
fn e01_08_chipset_sram() {
    let rom = make_rom_with(&[], Map::LoRom, 15, 3, 0x02, 1);
    assert_eq!(rom[0x7FD6], 0x02); // chipset
    assert_eq!(rom[0x7FD8], 3); // RAM size exponent (3 = 8 KiB)
    assert_eq!(rom[0x7FD9], 1); // country
}
