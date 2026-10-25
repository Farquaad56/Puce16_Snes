pub mod header;
pub mod mapper;

use crate::region::Region;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MapMode { LoRom, HiRom, ExHiRom }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CartError { TooSmall, NoHeader, UnsupportedCoprocessor(&'static str) }

pub use header::{Header, score_header, pick_from_scores, pick_header};

/// Loaded SNES cartridge.
#[derive(Clone)]
pub struct Cart {
    rom: Vec<u8>,
    rom_mask: usize,
    map: MapMode,
    sram: Vec<u8>,
    has_battery: bool,
    header: Header,
    region: Region,
}

impl Cart {
    /// Load a cartridge from raw bytes (auto-detect mapper and region).
    pub fn from_bytes(data: &[u8]) -> Result<Cart, CartError> {
        Self::from_bytes_forced(data, None, None)
    }

    /// Load with optional forced mapper mode and/or region.
    pub fn from_bytes_forced(
        data: &[u8],
        map: Option<MapMode>,
        region: Option<Region>,
    ) -> Result<Cart, CartError> {
        // Step 1: strip copier header, check minimum size
        let d = strip_copier_header(data);
        if d.len() < 0x8000 {
            return Err(CartError::TooSmall);
        }

        // Step 2: normalize to power of 2
        let rom = normalize(d);
        let rom_mask = rom.len() - 1;

        // Step 3: determine mapper mode
        let map_mode = match map {
            Some(m) => m,
            None => pick_header(&rom)?,
        };

        // Step 4: parse header at correct position for this mapper
        let pos = header::header_pos(map_mode);
        let hdr = header::parse_header(&rom, pos).ok_or(CartError::NoHeader)?;

        // Step 5: check chipset for unsupported coprocessor
        let (battery, coproc) = header::chipset_info(hdr.chipset);
        if let Some(name) = coproc {
            return Err(CartError::UnsupportedCoprocessor(name));
        }

        // Step 6: build SRAM and determine region
        let sram_size = header::sram_bytes(hdr.ram_log2);
        let sram = vec![0u8; sram_size];
        let cart_region = match region {
            Some(r) => r,
            None => header::region_from_country(hdr.country),
        };

        Ok(Cart {
            rom,
            rom_mask,
            map: map_mode,
            sram,
            has_battery: battery,
            header: hdr,
            region: cart_region,
        })
    }

    /// Reference to the parsed cartridge header.
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Mapper mode of this cartridge.
    pub fn map_mode(&self) -> MapMode {
        self.map
    }

    /// Region (NTSC or PAL).
    pub fn region(&self) -> Region {
        self.region
    }

    /// Whether the cartridge has a battery-backed SRAM.
    pub fn has_battery(&self) -> bool {
        self.has_battery
    }

    /// Reference to the SRAM contents (may be empty).
    pub fn sram(&self) -> &[u8] {
        &self.sram
    }

    /// Total ROM size in bytes.
    pub fn rom_len(&self) -> usize {
        self.rom.len()
    }

    /// CPU read from cartridge. SRAM is checked first, then ROM. None = open bus.
    pub fn read(&self, bank: u8, addr: u16) -> Option<u8> {
        if !self.sram.is_empty() {
            if let Some(o) = mapper::sram_offset(self.map, bank, addr, self.sram.len()) {
                return Some(self.sram[o]);
            }
        }
        if let Some(o) = mapper::rom_offset(self.map, bank, addr) {
            return Some(self.rom[o & self.rom_mask]);
        }
        None
    }

    /// CPU write to cartridge. Only SRAM is writable; ROM writes are ignored.
    pub fn write(&mut self, bank: u8, addr: u16, v: u8) -> bool {
        if !self.sram.is_empty() {
            if let Some(o) = mapper::sram_offset(self.map, bank, addr, self.sram.len()) {
                self.sram[o] = v;
                return true;
            }
        }
        false
    }

    /// Load SRAM contents from a byte slice. Copies min(len) bytes, zeros the rest.
    pub fn load_sram(&mut self, data: &[u8]) {
        for i in 0..self.sram.len() {
            self.sram[i] = data.get(i).copied().unwrap_or(0);
        }
    }

    /// Hash SRAM contents into the given hasher (for state serialization).
    pub fn hash_into(&self, h: &mut crate::hash::Fnv64) {
        if !self.sram.is_empty() {
            h.write(&self.sram);
        }
    }
}

/// Strip 512-byte copier header if present (len % 1024 == 512).
pub fn strip_copier_header(data: &[u8]) -> &[u8] {
    if data.len() % 1024 == 512 {
        &data[512..]
    } else {
        data
    }
}

/// Normalize ROM to power of 2 size per SNES wiki algorithm.
pub fn normalize(data: &[u8]) -> Vec<u8> {
    let len = data.len();
    // Find largest power of 2 <= len
    let mut big = 1;
    while big * 2 <= len {
        big *= 2;
    }
    if len == big {
        return data.to_vec();
    }
    // rest = len - big
    let rest_len = len - big;
    // Find smallest power of 2 >= rest_len
    let mut small = 1;
    while small < rest_len {
        small *= 2;
    }
    // Build part: rest padded with zeros to 'small' bytes
    let mut part = vec![0u8; small];
    part[..rest_len].copy_from_slice(&data[big..]);
    // Repeat part big/small times to fill the second half
    let mut result = data.to_vec();
    result.resize(2 * big, 0);
    let repeats = big / small;
    for i in 0..repeats {
        let src_start = i * small;
        let dst_start = big + src_start;
        result[dst_start..dst_start + small].copy_from_slice(&part);
    }
    result
}
