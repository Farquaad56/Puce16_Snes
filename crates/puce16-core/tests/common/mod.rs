//! Shared test helpers (E01-08): synthetic ROM builder.
//!
//! `make_rom` / `make_rom_with` build small valid ROMs with an internal
//! header per spec 04, so bus/DMA/NMI/joypad tests need no external ROM.
//! This module is independent of the (not yet existing) `cart/` code: the
//! header is written directly from the spec.

/// Mapper mode of the synthesized ROM.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)]
pub enum Map { LoRom, HiRom, ExHiRom }

/// 21-byte ASCII title ($20-$7E).
const TITLE: [u8; 21] = *b"PUCE16 TEST ROM      ";

/// File offset that CPU address $00:8000 maps to (start of the code).
fn code_offset(map: Map) -> usize {
    match map {
        Map::LoRom => 0x0000,
        Map::HiRom => 0x8000,
        Map::ExHiRom => 0x40_8000,
    }
}

/// File offset of the internal header (CPU $00:FFC0).
fn header_offset(map: Map) -> usize {
    match map {
        Map::LoRom => 0x7FC0,
        Map::HiRom => 0xFFC0,
        Map::ExHiRom => 0x40_FFC0,
    }
}

/// Smallest ROM size (bytes) that can hold the header for this mapper.
fn min_size(map: Map) -> usize {
    match map {
        Map::LoRom => 1 << 15, // 32 KiB
        Map::HiRom => 1 << 16, // 64 KiB
        Map::ExHiRom => 1 << 23, // 8 MiB
    }
}

/// Mapper nibble of the map byte ($FFD5).
fn mapper_nibble(map: Map) -> u8 {
    match map {
        Map::LoRom => 0x0,
        Map::HiRom => 0x1,
        Map::ExHiRom => 0x5,
    }
}

/// Checksum per spec 04 section 2.4: complement treated as $FFFF and
/// checksum as $0000 while summing all bytes (u16 wrapping).
fn compute_checksum(rom: &[u8], header: usize) -> u16 {
    let mut sum: u16 = 0;
    for (i, &b) in rom.iter().enumerate() {
        let v = match i {
            n if n == header + 0x1C || n == header + 0x1D => 0xFFu16, // complement := $FFFF
            n if n == header + 0x1E || n == header + 0x1F => 0x00u16, // checksum   := $0000
            _ => b as u16,
        };
        sum = sum.wrapping_add(v);
    }
    sum
}

/// Write the checksum (LE) at header+0x1E and its complement (!checksum, LE)
/// at header+0x1C.
fn apply_checksum(rom: &mut [u8], header: usize) {
    let c = compute_checksum(rom, header);
    rom[header + 0x1C] = (!c) as u8;
    rom[header + 0x1D] = ((!c) >> 8) as u8;
    rom[header + 0x1E] = c as u8;
    rom[header + 0x1F] = (c >> 8) as u8;
}

/// Build a valid ROM. `code` is placed at the file offset that CPU address
/// $00:8000 maps to: LoRom file offset 0x0000 ; HiRom file offset 0x8000 ;
/// ExHiRom file offset 0x408000.
pub fn make_rom(code: &[u8], map: Map) -> Vec<u8> {
    let rom_log2 = match map {
        Map::LoRom => 15, // 32 KiB
        Map::HiRom => 16, // 64 KiB
        Map::ExHiRom => 23, // 8 MiB
    };
    make_rom_with(code, map, rom_log2, 0, 0x00, 0x01)
}

/// Build a valid ROM of size `1 << rom_log2` bytes (panics if too small to
/// hold the header: LoRom >= 32 KiB, HiRom >= 64 KiB, ExHiRom >= 8 MiB).
/// Filled with 0x00 except code and header. Header fields per spec 04:
/// title, map byte (speed=1), chipset, ROM size = rom_log2 - 10, RAM size =
/// sram_log2 (exponent N of `1024 << N` bytes; 0 = none), country, dev = 0,
/// version = 0. All vector words are $8000 (LE). Checksum is valid.
pub fn make_rom_with(
    code: &[u8],
    map: Map,
    rom_log2: u8, // 15=32 KiB ..
    sram_log2: u8,
    chipset: u8,
    country: u8,
) -> Vec<u8> {
    let size = 1usize << rom_log2;
    if size < min_size(map) {
        panic!("ROM too small for mapper {:?}: {} bytes", map, size);
    }
    let header = header_offset(map);

    let mut rom = vec![0u8; size];

    // Header fields (spec 04 section 2).
    rom[header..header + TITLE.len()].copy_from_slice(&TITLE);
    rom[header + 0x15] = 0b0010_0000 | mapper_nibble(map); // speed=1, mapper
    rom[header + 0x16] = chipset;
    rom[header + 0x17] = rom_log2 - 10; // ROM size: 1 << N Kio
    rom[header + 0x18] = sram_log2; // RAM size: 1024 << N bytes (0 = none)
    rom[header + 0x19] = country;
    rom[header + 0x1A] = 0x00; // developer
    rom[header + 0x1B] = 0x00; // version

    // Vector zone (header+0x20 .. header+0x3F): every word is $8000 (LE).
    for off in (header + 0x20..header + 0x40).step_by(2) {
        rom[off] = 0x00;
        rom[off + 1] = 0x80;
    }

    // Code at the file offset of CPU $00:8000.
    let start = code_offset(map);
    if start.checked_add(code.len()).map_or(true, |end| end > size) {
        panic!("code does not fit in ROM");
    }
    rom[start..start + code.len()].copy_from_slice(code);

    apply_checksum(&mut rom, header);
    rom
}

/// Vector table: name -> offsets relative to the start of the header.
fn vector_offsets(name: &str) -> &'static [u16] {
    match name {
        "cop" => &[0x24, 0x34], // native $FFE4, emulation $FFF4
        "brk" => &[0x26, 0x3E], // native $FFE6, emulation IRQ/BRK $FFFE
        "nmi" => &[0x2A, 0x3A], // native $FFEA, emulation $FFFA
        "irq" => &[0x2E, 0x3E], // native $FFEE, emulation IRQ/BRK $FFFE
        "reset" => &[0x3C], // emulation $FFFC only
        _ => panic!("unknown vector name: {name}"),
    }
}

/// Overwrite a vector (all of its slots) with `addr` and recalculate the
/// checksum. Panics on an unknown name.
pub fn set_vector(rom: &mut Vec<u8>, map: Map, name: &str, addr: u16) {
    let header = header_offset(map);
    for off in vector_offsets(name) {
        rom[header + *off as usize] = addr as u8;
        rom[header + *off as usize + 1] = (addr >> 8) as u8;
    }
    apply_checksum(rom, header);
}
