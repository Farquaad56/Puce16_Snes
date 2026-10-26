//! OAM (Object Attribute Memory) - 544 bytes, word-write protocol with latch.
//!
//! Implements the SNES OAM write/read protocol per spec 05_PPU.md section 2.1.

use crate::hash::Fnv64;

pub struct Oam {
    mem: [u8; 544],
    oamadd_reg: u16, // 9 bits (bits 0-7 from OAMADDL, bit 8 from OAMADDH bit 0)
    internal: u16,   // 10-bit byte address
    latch: u8,
    rotation: bool,
}

impl Default for Oam {
    fn default() -> Self {
        Self::new()
    }
}

impl Oam {
    pub fn new() -> Oam {
        Oam {
            mem: [0; 544],
            oamadd_reg: 0,
            internal: 0,
            latch: 0,
            rotation: false,
        }
    }

    /// Reset registers only; memory is preserved.
    pub fn reset_regs(&mut self) {
        self.oamadd_reg = 0;
        self.internal = 0;
        self.latch = 0;
        self.rotation = false;
    }

    /// OAMADDL ($2103): set lower 8 bits of address register.
    pub fn set_addr_l(&mut self, v: u8) {
        self.oamadd_reg = (self.oamadd_reg & 0x100) | (v as u16);
        self.reload();
    }

    /// OAMADDH ($2105): bit 0 -> bit 8 of address register, bit 7 -> rotation.
    pub fn set_addr_h(&mut self, v: u8) {
        let high_bit = (((v >> 0) & 1) as u16) << 8;
        self.oamadd_reg = (self.oamadd_reg & 0xFF) | (high_bit as u16);
        self.rotation = (v >> 7) & 1 == 1;
        self.reload();
    }

    /// Recalculate internal byte address from oamadd_reg.
    pub fn reload(&mut self) {
        self.internal = ((self.oamadd_reg & 0x1FF) << 1) & 0x3FF;
    }

    /// OAMDATA ($2104): write one byte to OAM.
    /// `ok` indicates whether access is allowed (forced blank or VBlank).
    pub fn write(&mut self, v: u8, ok: bool) {
        let a = self.internal;
        if a & 0x200 != 0 {
            // High table: immediate byte writes, mirrored every 32 bytes
            if ok {
                let idx = (0x200 + (a & 0x1F)) as usize;
                self.mem[idx] = v;
            }
        } else if a & 1 == 0 {
            // Low table, even address: store in latch
            if ok {
                self.latch = v;
            }
        } else {
            // Low table, odd address: write latched byte + this byte as word
            if ok {
                let idx = (a - 1) as usize;
                self.mem[idx] = self.latch;
                self.mem[idx + 1] = v;
            }
        }
        self.internal = (a + 1) & 0x3FF;
    }

    /// OAMDATAREAD ($2138): read one byte from OAM.
    /// Returns 0 if access not allowed (`ok` is false).
    pub fn read(&mut self, ok: bool) -> u8 {
        let a = self.internal;
        let idx = if a & 0x200 != 0 {
            (0x200 + (a & 0x1F)) as usize
        } else {
            a as usize
        };
        let result = if ok { self.mem[idx] } else { 0 };
        self.internal = (a + 1) & 0x3FF;
        result
    }

    pub fn internal(&self) -> u16 {
        self.internal
    }

    pub fn oamadd_reg(&self) -> u16 {
        self.oamadd_reg
    }

    pub fn rotation(&self) -> bool {
        self.rotation
    }

    /// Direct byte access for testing/debugging.
    pub fn byte(&self, i: usize) -> u8 {
        self.mem[i]
    }

    pub fn set_byte(&mut self, i: usize, v: u8) {
        self.mem[i] = v;
    }

    pub fn mem(&self) -> &[u8; 544] {
        &self.mem
    }

    /// Hash OAM state into the given hasher.
    pub fn hash_into(&self, h: &mut Fnv64) {
        h.write(&self.mem);
        h.write_u16(self.oamadd_reg);
        h.write_u16(self.internal);
        h.write_u8(self.latch);
        h.write_u8(if self.rotation { 1 } else { 0 });
    }
}
