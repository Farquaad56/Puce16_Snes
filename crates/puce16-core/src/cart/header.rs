use super::mapper::rom_offset;
use crate::cart::{CartError, MapMode};
use crate::region::Region;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Header {
    pub title: String,
    pub map_byte: u8,
    pub chipset: u8,
    pub rom_log2: u8,
    pub ram_log2: u8,
    pub country: u8,
    pub dev_id: u8,
    pub version: u8,
    pub checksum: u16,
    pub complement: u16,
}

pub fn header_pos(map: MapMode) -> usize {
    match map {
        MapMode::LoRom => 0x7FC0,
        MapMode::HiRom => 0xFFC0,
        MapMode::ExHiRom => 0x40FFC0,
    }
}

pub fn parse_header(rom: &[u8], pos: usize) -> Option<Header> {
    if pos + 0x40 > rom.len() {
        return None;
    }

    // Parse title (21 bytes at offset 0x00 from header start)
    let mut title = String::new();
    for i in 0..21 {
        let b = rom[pos + i];
        if (0x20..=0x7E).contains(&b) {
            title.push(b as char);
        } else {
            title.push('?');
        }
    }
    // Trim trailing spaces
    while title.ends_with(' ') {
        title.pop();
    }

    let map_byte = rom[pos + 0x15];
    let chipset = rom[pos + 0x16];
    let rom_log2 = rom[pos + 0x17];
    let ram_log2 = rom[pos + 0x18];
    let country = rom[pos + 0x19];
    let dev_id = rom[pos + 0x1A];
    let version = rom[pos + 0x1B];

    // complement at +0x1C (u16 LE)
    let complement_lo = rom[pos + 0x1C] as u16;
    let complement_hi = rom[pos + 0x1D] as u16;
    let complement = complement_lo | (complement_hi << 8);

    // checksum at +0x1E (u16 LE)
    let checksum_lo = rom[pos + 0x1E] as u16;
    let checksum_hi = rom[pos + 0x1F] as u16;
    let checksum = checksum_lo | (checksum_hi << 8);

    Some(Header {
        title,
        map_byte,
        chipset,
        rom_log2,
        ram_log2,
        country,
        dev_id,
        version,
        checksum,
        complement,
    })
}

pub fn compute_checksum(rom: &[u8], pos: usize) -> u16 {
    let mut sum: u32 = 0;
    for i in 0..rom.len() {
        // Skip the real complement and checksum bytes (4 bytes at pos+0x1C..pos+0x20)
        if i >= pos + 0x1C && i < pos + 0x20 {
            continue;
        }
        sum = sum.wrapping_add(rom[i] as u32);
    }
    // Add the forced complement value (0xFFFF = 65535)
    sum = sum.wrapping_add(0xFF).wrapping_add(0xFF);
    (sum & 0xFFFF) as u16
}

pub fn region_from_country(c: u8) -> Region {
    match c {
        0x00 | 0x01 | 0x0D | 0x0F | 0x10 => Region::Ntsc,
        _ => Region::Pal,
    }
}

pub fn sram_bytes(ram_log2: u8) -> usize {
    if ram_log2 == 0 || ram_log2 > 8 {
        return 0;
    }
    (1024 << ram_log2) as usize
}

pub fn chipset_info(chipset: u8) -> (bool, Option<&'static str>) {
    let low = chipset & 0xF;
    let has_battery = matches!(low, 2 | 5 | 6);

    if low < 3 {
        return (has_battery, None);
    }

    let high = chipset >> 4;
    let name = match high {
        0 => "DSP",
        1 => "GSU",
        2 => "OBC1",
        3 => "SA-1",
        4 => "S-DD1",
        5 => "S-RTC",
        0xE => "other",
        0xF => "custom",
        _ => "unknown",
    };

    (has_battery, Some(name))
}

pub fn score_header(rom: &[u8], map: MapMode) -> Option<i32> {
    let pos = header_pos(map);
    if pos + 0x40 > rom.len() {
        return None;
    }

    let h = parse_header(rom, pos)?;
    let mut score: i32 = 0;

    // Checksum + complement == $FFFF
    if (h.checksum as u32) + (h.complement as u32) == 0xFFFF {
        score += 4;
    }

    // Computed checksum matches header checksum
    if compute_checksum(rom, pos) == h.checksum {
        score += 2;
    }

    // Map byte: bits 7-5 must be 001 and mapper nibble consistent with position
    let map_bits_ok = (h.map_byte & 0xE0) == 0x20;
    let mapper_nibble = h.map_byte & 0x0F;
    let nibble_ok = match map {
        MapMode::LoRom => matches!(mapper_nibble, 0 | 2 | 3 | 0xA),
        MapMode::HiRom => mapper_nibble == 1,
        MapMode::ExHiRom => mapper_nibble == 5,
    };
    if map_bits_ok && nibble_ok {
        score += 2;
    }

    // Declared ROM size plausible: >= normalized size / 2
    if h.rom_log2 < 32 {
        let declared_size = (1u64 << h.rom_log2) * 1024;
        if declared_size >= rom.len() as u64 / 2 {
            score += 1;
        }
    }

    // RESET vector at pos + 0x3C (LE u16)
    let reset_lo = rom[pos + 0x3C] as u16;
    let reset_hi = rom[pos + 0x3D] as u16;
    let reset = reset_lo | (reset_hi << 8);
    if reset >= 0x8000 {
        score += 2;
    } else {
        score -= 4;
    }

    // First byte at RESET address via mapper
    if let Some(off) = rom_offset(map, 0x00, reset) {
        let b = rom[off & (rom.len() - 1)];
        match b {
            0x78 | 0x18 | 0x38 | 0x9C | 0x64 | 0x74 | 0x9E | 0x4C | 0x5C | 0xDC | 0xA9 | 0xC2 | 0xE2 => {
                score += 1;
            }
            0x00 | 0x02 | 0xDB | 0x42 | 0xFF => {
                score -= 2;
            }
            _ => {}
        }
    }

    // Title entirely ASCII $20-$7E (check raw bytes, not parsed string)
    let title_ok = (0..21).all(|i| (0x20..=0x7E).contains(&rom[pos + i]));
    if title_ok {
        score += 1;
    }

    // ROM/RAM size ranges plausible
    if (7..=14).contains(&h.rom_log2) && (0..=8).contains(&h.ram_log2) {
        score += 1;
    }

    Some(score)
}

pub fn pick_from_scores(s: [Option<i32>; 3]) -> Option<MapMode> {
    let mut best_score: Option<i32> = None;
    let mut best_mode = MapMode::LoRom;

    for (i, score) in s.iter().enumerate() {
        if let Some(sc) = score {
            if best_score.is_none() || *sc > best_score.unwrap() {
                best_score = Some(*sc);
                best_mode = match i {
                    0 => MapMode::LoRom,
                    1 => MapMode::HiRom,
                    _ => MapMode::ExHiRom,
                };
            }
        }
    }

    if best_score.is_some() {
        Some(best_mode)
    } else {
        None
    }
}

pub fn pick_header(rom: &[u8]) -> Result<MapMode, CartError> {
    let scores = [
        score_header(rom, MapMode::LoRom),
        score_header(rom, MapMode::HiRom),
        score_header(rom, MapMode::ExHiRom),
    ];

    match pick_from_scores(scores) {
        Some(mode) => Ok(mode),
        None => Err(CartError::NoHeader),
    }
}
