pub mod header;
pub mod mapper;

use crate::coprocessor::sdd1::Sdd1;
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
    sdd1: Option<Box<Sdd1>>,
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

        // Step 7: create S-DD1 state if chipset indicates it
        let sdd1 = if header::has_sdd1(hdr.chipset) {
            Some(Box::new(Sdd1::new()))
        } else {
            None
        };

        Ok(Cart {
            rom,
            rom_mask,
            map: map_mode,
            sram,
            has_battery: battery,
            header: hdr,
            region: cart_region,
            sdd1,
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

    /// Whether this cartridge has S-DD1 compression.
    pub fn has_sdd1(&self) -> bool {
        self.sdd1.is_some()
    }

    /// Read an S-DD1 register. Returns None if no S-DD1 or unused register.
    pub fn io_read(&self, reg: u16) -> Option<u8> {
        self.sdd1.as_ref().and_then(|s| s.io_read(reg))
    }

    /// Write an S-DD1 register. No effect if no S-DD1.
    pub fn io_write(&mut self, reg: u16, v: u8) {
        if let Some(ref mut s) = self.sdd1 {
            s.io_write(reg, v);
        }
    }

    /// Intercept a DMA register write for S-DD1. No effect if no S-DD1.
    pub fn dma_write(&mut self, reg: u16, v: u8) {
        if let Some(ref mut s) = self.sdd1 {
            s.dma_write(reg, v);
        }
    }

    /// Get the DMA channel's current address and size for S-DD1. No effect if no S-DD1.
    pub fn dma_channel(&self, n: usize) -> (u32, u16) {
        self.sdd1.as_ref().map(|s| s.dma_channel(n)).unwrap_or((0, 0))
    }

    /// CPU read from cartridge. SRAM is checked first, then ROM. None = open bus.
    pub fn read(&self, bank: u8, addr: u16) -> Option<u8> {
        let addr24 = ((bank as u32) << 16) | (addr as u32);

        // S-DD1 SRAM mapping takes precedence when present
        if self.sdd1.is_some() && !self.sram.is_empty() {
            if let Some(o) = mapper::sdd1_sram_offset(bank, addr, self.sram.len()) {
                return Some(self.sram[o]);
            }
        }

        // S-DD1 ROM mapping when present
        if let Some(ref s) = self.sdd1 {
            // LoROM window: banks 00-3F/80-BF, offsets $8000-$FFFF
            if (bank <= 0x3F || bank >= 0x80) && addr >= 0x8000 {
                let offset = s.lorom_offset(addr24);
                return Some(self.rom[offset & self.rom_mask]);
            }
            // MMC window: banks C0-FF, all offsets (no decompression in read())
            if bank >= 0xC0 {
                let offset = s.mmc_offset(addr24);
                return Some(self.rom[offset & self.rom_mask]);
            }
            // Banks 40-7D: no ROM (open bus)
            return None;
        }

        // Non-S-DD1 path: use existing mapper logic
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

    /// CPU read from cartridge bus (with S-DD1 decompression on C0-FF window).
    pub fn read_bus(&mut self, bank: u8, addr: u16) -> Option<u8> {
        let addr24 = ((bank as u32) << 16) | (addr as u32);

        // S-DD1 SRAM mapping takes precedence when present
        if self.sdd1.is_some() && !self.sram.is_empty() {
            if let Some(o) = mapper::sdd1_sram_offset(bank, addr, self.sram.len()) {
                return Some(self.sram[o]);
            }
        }

        // S-DD1 ROM mapping when present
        if let Some(ref mut s) = self.sdd1 {
            // LoROM window: banks 00-3F/80-BF, offsets $8000-$FFFF
            if (bank <= 0x3F || bank >= 0x80) && addr >= 0x8000 {
                let offset = s.lorom_offset(addr24);
                return Some(self.rom[offset & self.rom_mask]);
            }
            // MMC window: banks C0-FF, all offsets (with decompression via mcu_read)
            if bank >= 0xC0 {
                return Some(s.mcu_read(addr24, &self.rom));
            }
            // Banks 40-7D: no ROM (open bus)
            return None;
        }

        // Non-S-DD1 path: identical to read()
        self.read(bank, addr)
    }

    /// CPU write to cartridge. Only SRAM is writable; ROM writes are ignored.
    pub fn write(&mut self, bank: u8, addr: u16, v: u8) -> bool {
        // S-DD1 SRAM mapping when present
        if self.sdd1.is_some() && !self.sram.is_empty() {
            if let Some(o) = mapper::sdd1_sram_offset(bank, addr, self.sram.len()) {
                self.sram[o] = v;
                return true;
            }
        }

        // Non-S-DD1 path: use existing mapper logic
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
        // Include S-DD1 state in hash when present.
        if let Some(ref s) = self.sdd1 {
            s.hash_into(h);
        }
    }
}

impl crate::savestate::State for Cart {
    fn save(&self, w: &mut crate::savestate::StateWriter) {
        // Save SRAM only (ROM is never saved).
        // not saved: rom (reconstructed from file), header (derived from ROM),
        // map (derived from ROM/header), region (derived from header), has_battery (derived from header)
        w.vec_u8(&self.sram);
        // Save S-DD1 state after SRAM, only if present.
        if let Some(ref s) = self.sdd1 {
            s.save(w);
        }
    }

    fn load(&mut self, r: &mut crate::savestate::StateReader) -> Result<(), crate::savestate::StateError> {
        // Read SRAM length prefix manually to validate size before copying.
        let len = r.u32()? as usize;
        if len != self.sram.len() {
            return Err(crate::savestate::StateError::Mismatch("sram size"));
        }
        // Length matches, now read the bytes into sram in place.
        for i in 0..len {
            let b = r.u8()?;
            self.sram[i] = b;
        }
        // Load S-DD1 state after SRAM, only if present.
        if let Some(ref mut s) = self.sdd1 {
            s.load(r)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::savestate::{State, StateWriter, StateReader};

    fn make_cart_with_sram(sram_size: usize) -> Cart {
        // Create a minimal cart with specified SRAM size.
        let mut ram_log2 = 0u8;
        if sram_size > 0 {
            let mut sz = sram_size;
            while sz > 1 {
                sz >>= 1;
                ram_log2 += 1;
            }
        }
        Cart {
            rom: vec![0u8; 0x40000],
            rom_mask: 0x3FFFF,
            map: MapMode::LoRom,
            sram: vec![0u8; sram_size],
            has_battery: true,
            header: Header {
                title: String::from("TEST"),
                map_byte: 0x21, // LoROM mapper byte
                chipset: 0x01,  // LoROM chipset
                rom_log2: 18,   // 256KB ROM
                ram_log2,
                country: 0x34,  // NTSC-J
                dev_id: 0,
                version: 0,
                checksum: 0,
                complement: 0xFFFF,
            },
            region: Region::Ntsc,
            sdd1: None,
        }
    }

    #[test]
    fn e08_03_cart_roundtrip_and_size_mismatch() {
        // Create cart with 8KB SRAM filled with pattern
        let mut c = make_cart_with_sram(8192);
        for i in 0..c.sram.len() {
            c.sram[i] = (i * 3) as u8;
        }

        // Save
        let mut w = StateWriter::new();
        c.save(&mut w);
        let data = w.finish();

        // Load into fresh cart of same size
        let mut c2 = make_cart_with_sram(8192);
        for i in 0..c2.sram.len() {
            c2.sram[i] = 0xFF; // Different initial values
        }
        let mut r = StateReader::new(&data);
        c2.load(&mut r).unwrap();

        // Verify SRAM restored correctly
        assert_eq!(c.sram, c2.sram);

        // Test size mismatch: load into cart with 2KB SRAM
        let mut c3 = make_cart_with_sram(2048);
        for i in 0..c3.sram.len() {
            c3.sram[i] = 0xAA;
        }
        let original_c3_sram = c3.sram.clone();
        let mut r3 = StateReader::new(&data);
        let result = c3.load(&mut r3);
        assert!(result.is_err());
        if let Err(crate::savestate::StateError::Mismatch("sram size")) = result {
            // Expected error
        } else {
            panic!("Expected Mismatch(\"sram size\"), got {:?}", result);
        }
        // SRAM should be unchanged after failed load
        assert_eq!(original_c3_sram, c3.sram);

        // Test cart with no SRAM: save writes empty vec_u8, load Ok
        let c4 = make_cart_with_sram(0);
        let mut w4 = StateWriter::new();
        c4.save(&mut w4);
        let data4 = w4.finish();

        let mut c5 = make_cart_with_sram(0);
        let mut r5 = StateReader::new(&data4);
        assert!(c5.load(&mut r5).is_ok());
    }

    #[test]
    fn e08_03_cart_truncated() {
        let mut c = make_cart_with_sram(1024);
        for i in 0..c.sram.len() {
            c.sram[i] = (i * 5) as u8;
        }

        let mut w = StateWriter::new();
        c.save(&mut w);
        let data = w.finish();

        // Test truncation at various points
        for n in [0, data.len() / 2, data.len() - 1] {
            let mut r = StateReader::new(&data[..n]);
            let mut c2 = make_cart_with_sram(1024);
            assert!(c2.load(&mut r).is_err(), "truncated at {} should fail", n);
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
