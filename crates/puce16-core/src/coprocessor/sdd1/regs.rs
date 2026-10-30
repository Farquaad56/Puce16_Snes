//! S-DD1 state: registers $4800-$4807 and ROM/SRAM mapping functions.

use super::decomp::Decompressor;

#[derive(Clone, Debug)]
pub struct Sdd1 {
    pub(super) r4800: u8,
    pub(super) r4801: u8,
    pub(super) mmc: [u8; 4],
    pub(super) dma_addr: [u32; 8],
    pub(super) dma_size: [u16; 8],
    pub(super) dma_ready: bool,
    pub(super) dec: Decompressor,
}

impl Default for Sdd1 {
    fn default() -> Self {
        Self::new()
    }
}

impl Sdd1 {
    /// Create a new S-DD1 state with reset values.
    pub fn new() -> Sdd1 {
        Sdd1 {
            r4800: 0,
            r4801: 0,
            mmc: [0, 1, 2, 3],
            dma_addr: [0; 8],
            dma_size: [0; 8],
            dma_ready: false,
            dec: Decompressor::new(),
        }
    }

    /// Read a register. Returns None for unused registers (bus open).
    pub fn io_read(&self, reg: u16) -> Option<u8> {
        match reg {
            0x4800 => Some(self.r4800),
            0x4801 => Some(self.r4801),
            0x4804 => Some(self.mmc[0]),
            0x4805 => Some(self.mmc[1]),
            0x4806 => Some(self.mmc[2]),
            0x4807 => Some(self.mmc[3]),
            _ if (0x4800..=0x480F).contains(&reg) => None, // UNKNOWN - to confirm: bus open
            _ => None,
        }
    }

    /// Write a register. Unused registers are ignored.
    pub fn io_write(&mut self, reg: u16, v: u8) {
        match reg {
            0x4800 => self.r4800 = v,
            0x4801 => self.r4801 = v,
            0x4804 => self.mmc[0] = v & 0x8F,
            0x4805 => self.mmc[1] = v & 0x8F,
            0x4806 => self.mmc[2] = v & 0x8F,
            0x4807 => self.mmc[3] = v & 0x8F,
            _ => {} // ignored
        }
    }

    /// MMC window (banks C0-FF): compute ROM offset before size masking.
    pub fn mmc_offset(&self, addr: u32) -> usize {
        let block = ((addr >> 20) & 3) as usize;
        let bank_reg = self.mmc[block];
        ((((bank_reg & 0x0F) as u32) << 20) | (addr & 0xFFFFF)) as usize
    }

    /// LoROM window (banks 00-3F/80-BF): compute ROM offset before size masking.
    pub fn lorom_offset(&self, addr: u32) -> usize {
        let bank = (addr >> 16) & 0xFF;
        let mut a = addr;

        // Bit 21 clearing for banks 20-3F and A0-BF based on r4805/r4807 bit 7
        if (bank >= 0x20 && bank <= 0x3F) && (self.mmc[1] & 0x80) != 0 {
            a &= !0x200000; // clear bit 21
        }
        if (bank >= 0xA0 && bank <= 0xBF) && (self.mmc[3] & 0x80) != 0 {
            a &= !0x200000; // clear bit 21
        }

        (((a >> 1) & 0x1F8000) | (a & 0x7FFF)) as usize
    }

    /// Save S-DD1 state: r4800, r4801, mmc[0..4], then for each channel dma_addr (u32), dma_size (u16).
    pub fn save(&self, w: &mut crate::savestate::StateWriter) {
        w.u8(self.r4800);
        w.u8(self.r4801);
        for i in 0..4 {
            w.u8(self.mmc[i]);
        }
        for n in 0..8 {
            w.u32(self.dma_addr[n]);
            w.u16(self.dma_size[n]);
        }
    }

    /// Load S-DD1 state. Resets dma_ready and decompressor after loading registers.
    pub fn load(&mut self, r: &mut crate::savestate::StateReader) -> Result<(), crate::savestate::StateError> {
        self.r4800 = r.u8()?;
        self.r4801 = r.u8()?;
        for i in 0..4 {
            self.mmc[i] = r.u8()?;
        }
        for n in 0..8 {
            self.dma_addr[n] = r.u32()?;
            self.dma_size[n] = r.u16()?;
        }
        // After loading, streaming is abandoned: reset dma_ready and decompressor.
        self.dma_ready = false;
        self.dec = Decompressor::new();
        Ok(())
    }

    /// Hash S-DD1 state into the given hasher (same fields as save).
    pub fn hash_into(&self, h: &mut crate::hash::Fnv64) {
        h.write_u8(self.r4800);
        h.write_u8(self.r4801);
        for i in 0..4 {
            h.write_u8(self.mmc[i]);
        }
        for n in 0..8 {
            h.write_u32(self.dma_addr[n]);
            h.write_u16(self.dma_size[n]);
        }
    }
}