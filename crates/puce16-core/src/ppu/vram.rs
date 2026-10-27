//! VRAM (Video RAM) - 64KB of video memory with remapping and prefetch.
//!
//! The SNES PPU has 64KB of VRAM organized as 32768 16-bit words. Access is
//! through the VMADDL/VMADDH address registers and VMDATAL/VMDATAH data
//! registers, with a prefetch latch for reads.

use crate::hash::Fnv64;

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
