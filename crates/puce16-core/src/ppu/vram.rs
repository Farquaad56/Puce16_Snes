//! VRAM (Video RAM) - 64KB of video memory with remapping and prefetch.
//!
//! The SNES PPU has 64KB of VRAM organized as 32768 16-bit words. Access is
//! through the VMADDL/VMADDH address registers and VMDATAL/VMDATAH data
//! registers, with a prefetch latch for reads.

use crate::hash::Fnv64;
use crate::savestate::{State, StateError, StateReader, StateWriter};

/// Remap VRAM address based on VMAIN bits 3-2 (remapping mode).
pub fn remap(mode: u8, a: u16) -> u16 {
    match mode & 3 {
        0 => a,
        1 => (a & 0xFF00) | ((a & 0x00E0) >> 5) | ((a & 0x001F) << 3),
        2 => (a & 0xFE00) | ((a & 0x01C0) >> 6) | ((a & 0x003F) << 3),
        _ => (a & 0xFC00) | ((a & 0x0380) >> 7) | ((a & 0x007F) << 3),
    }
}

#[derive(Clone)]
pub struct Vram {
    mem: Vec<u16>,
    vmain: u8,
    vmadd: u16,
    latch: u16,
}

impl Vram {
    pub fn new() -> Vram {
        Vram {
            mem: vec![0u16; 0x8000],
            vmain: 0,
            vmadd: 0,
            latch: 0,
        }
    }

    pub fn reset_regs(&mut self) {
        self.vmain = 0;
        self.vmadd = 0;
        self.latch = 0;
    }

    pub fn set_vmain(&mut self, v: u8) {
        self.vmain = v;
    }

    pub fn vmain(&self) -> u8 {
        self.vmain
    }

    pub fn vmadd(&self) -> u16 {
        self.vmadd
    }

    /// Step size based on VMAIN bits 1-0: 0->1, 1->32, 2/3->128.
    pub fn step(&self) -> u16 {
        match self.vmain & 3 {
            0 => 1,
            1 => 32,
            _ => 128,
        }
    }

    /// Whether increment happens on high byte access (VMAIN bit 7).
    pub fn inc_on_high(&self) -> bool {
        (self.vmain & 0x80) != 0
    }

    /// Compute remapped address for current vmadd.
    fn ra(&self) -> u16 {
        let mode = (self.vmain >> 2) & 3;
        remap(mode, self.vmadd) & 0x7FFF
    }

    /// Prefetch: load latch from VRAM at remapped address.
    fn prefetch(&mut self) {
        let addr = self.ra();
        self.latch = self.mem[addr as usize];
    }

    pub fn set_addr_l(&mut self, v: u8) {
        self.vmadd = (self.vmadd & 0xFF00) | (v as u16);
        self.prefetch();
    }

    pub fn set_addr_h(&mut self, v: u8) {
        self.vmadd = (self.vmadd & 0x00FF) | (((v as u16) & 0x7F) << 8);
        self.prefetch();
    }

    pub fn write_low(&mut self, v: u8, ok: bool) {
        if ok {
            let addr = self.ra() as usize;
            self.mem[addr] = (self.mem[addr] & 0xFF00) | (v as u16);
        }
        if !self.inc_on_high() {
            self.vmadd = (self.vmadd + self.step()) & 0x7FFF;
        }
    }

    pub fn write_high(&mut self, v: u8, ok: bool) {
        if ok {
            let addr = self.ra() as usize;
            self.mem[addr] = (self.mem[addr] & 0x00FF) | ((v as u16) << 8);
        }
        if self.inc_on_high() {
            self.vmadd = (self.vmadd + self.step()) & 0x7FFF;
        }
    }

    pub fn read_low(&mut self, ok: bool) -> u8 {
        let byte = if ok { (self.latch & 0xFF) as u8 } else { 0 };
        if !self.inc_on_high() {
            // Reload latch at address BEFORE increment
            let addr = self.ra() as usize;
            self.latch = self.mem[addr];
            self.vmadd = (self.vmadd + self.step()) & 0x7FFF;
        }
        byte
    }

    pub fn read_high(&mut self, ok: bool) -> u8 {
        let byte = if ok { (self.latch >> 8) as u8 } else { 0 };
        if self.inc_on_high() {
            // Reload latch at address BEFORE increment
            let addr = self.ra() as usize;
            self.latch = self.mem[addr];
            self.vmadd = (self.vmadd + self.step()) & 0x7FFF;
        }
        byte
    }

    pub fn word(&self, a: u16) -> u16 {
        self.mem[a as usize & 0x7FFF]
    }

    pub fn set_word(&mut self, a: u16, w: u16) {
        self.mem[a as usize & 0x7FFF] = w;
    }

    pub fn mem(&self) -> &[u16] {
        &self.mem
    }

    pub fn hash_into(&self, h: &mut Fnv64) {
        h.write_u16_slice(&self.mem);
        h.write_u8(self.vmain);
        h.write_u16(self.vmadd);
        h.write_u16(self.latch);
    }
}

impl State for Vram {
    fn save(&self, w: &mut StateWriter) {
        // not saved: none (all fields are stateful)
        w.vec_u16(&self.mem);
        w.u8(self.vmain);
        w.u16(self.vmadd);
        w.u16(self.latch);
    }

    fn load(&mut self, r: &mut StateReader) -> Result<(), StateError> {
        let mem = r.vec_u16()?;
        if mem.len() != 0x8000 {
            return Err(StateError::Mismatch("vram.mem"));
        }
        self.mem = mem;
        self.vmain = r.u8()?;
        self.vmadd = r.u16()?;
        self.latch = r.u16()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::savestate::{StateWriter, StateReader};

    #[test]
    fn e08_06_vram_roundtrip() {
        let mut original = Vram::new();
        for i in 0..original.mem.len() {
            original.mem[i] = (i as u16) ^ 0x5A5A;
        }
        original.vmain = 0xAB;
        original.vmadd = 0xCDEF;
        original.latch = 0x1234;

        let mut w = StateWriter::new();
        original.save(&mut w);
        let data = w.finish();

        let mut restored = Vram::new();
        let mut r = StateReader::new(&data);
        restored.load(&mut r).unwrap();

        assert_eq!(original.mem, restored.mem);
        assert_eq!(original.vmain, restored.vmain);
        assert_eq!(original.vmadd, restored.vmadd);
        assert_eq!(original.latch, restored.latch);

        let mut h1 = Fnv64::new();
        original.hash_into(&mut h1);
        let mut h2 = Fnv64::new();
        restored.hash_into(&mut h2);
        assert_eq!(h1.finish(), h2.finish());

        let mut w2 = StateWriter::new();
        restored.save(&mut w2);
        assert_eq!(data, w2.finish());
    }

    #[test]
    fn e08_06_sizes() {
        use crate::ppu::{Cgram, Oam};

        let vram = Vram::new();
        let mut w1 = StateWriter::new();
        vram.save(&mut w1);
        assert!(w1.len() >= 65536, "vram state too small: {}", w1.len());

        let cgram = Cgram::new();
        let mut w2 = StateWriter::new();
        cgram.save(&mut w2);
        assert!(w2.len() >= 512, "cgram state too small: {}", w2.len());

        let oam = Oam::new();
        let mut w3 = StateWriter::new();
        oam.save(&mut w3);
        assert!(w3.len() >= 544, "oam state too small: {}", w3.len());
    }

    #[test]
    fn e08_06_truncated_each() {
        use crate::ppu::{Cgram, Oam};

        // VRAM truncated
        let vram = Vram::new();
        let mut w1 = StateWriter::new();
        vram.save(&mut w1);
        let data1 = w1.finish();
        for n in [0, data1.len() / 2, data1.len() - 1] {
            let mut restored = Vram::new();
            let mut r = StateReader::new(&data1[..n]);
            assert!(restored.load(&mut r).is_err(), "vram: expected error at {}", n);
        }

        // CGRAM truncated
        let cgram = Cgram::new();
        let mut w2 = StateWriter::new();
        cgram.save(&mut w2);
        let data2 = w2.finish();
        for n in [0, data2.len() / 2, data2.len() - 1] {
            let mut restored = Cgram::new();
            let mut r = StateReader::new(&data2[..n]);
            assert!(restored.load(&mut r).is_err(), "cgram: expected error at {}", n);
        }

        // OAM truncated
        let oam = Oam::new();
        let mut w3 = StateWriter::new();
        oam.save(&mut w3);
        let data3 = w3.finish();
        for n in [0, data3.len() / 2, data3.len() - 1] {
            let mut restored = Oam::new();
            let mut r = StateReader::new(&data3[..n]);
            assert!(restored.load(&mut r).is_err(), "oam: expected error at {}", n);
        }
    }
}
