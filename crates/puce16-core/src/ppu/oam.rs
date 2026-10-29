//! OAM (Object Attribute Memory) - 544 bytes, word-write protocol with latch.
//!
//! Implements the SNES OAM write/read protocol per spec 05_PPU.md section 2.1.

use crate::hash::Fnv64;
use crate::savestate::{State, StateError, StateReader, StateWriter};

#[derive(Clone)]
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

impl State for Oam {
    fn save(&self, w: &mut StateWriter) {
        // not saved: none (all fields are stateful)
        w.bytes(&self.mem);
        w.u16(self.oamadd_reg);
        w.u16(self.internal);
        w.u8(self.latch);
        w.bool(self.rotation);
    }

    fn load(&mut self, r: &mut StateReader) -> Result<(), StateError> {
        r.bytes_into(&mut self.mem)?;
        self.oamadd_reg = r.u16()?;
        self.internal = r.u16()?;
        self.latch = r.u8()?;
        self.rotation = r.bool()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::savestate::{StateWriter, StateReader};

    #[test]
    fn e08_06_oam_roundtrip() {
        let mut original = Oam::new();
        for i in 0..544 {
            original.mem[i] = (i as u8) ^ 0x3C;
        }
        original.oamadd_reg = 0x1FF;
        original.internal = 0x200;
        original.latch = 0xEE;
        original.rotation = true;

        let mut w = StateWriter::new();
        original.save(&mut w);
        let data = w.finish();

        let mut restored = Oam::new();
        let mut r = StateReader::new(&data);
        restored.load(&mut r).unwrap();

        assert_eq!(original.mem, restored.mem);
        assert_eq!(original.oamadd_reg, restored.oamadd_reg);
        assert_eq!(original.internal, restored.internal);
        assert_eq!(original.latch, restored.latch);
        assert_eq!(original.rotation, restored.rotation);

        let mut h1 = Fnv64::new();
        original.hash_into(&mut h1);
        let mut h2 = Fnv64::new();
        restored.hash_into(&mut h2);
        assert_eq!(h1.finish(), h2.finish());

        let mut w2 = StateWriter::new();
        restored.save(&mut w2);
        assert_eq!(data, w2.finish());
    }
}
