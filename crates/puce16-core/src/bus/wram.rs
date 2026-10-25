//! WRAM (Work RAM) - 128 KiB with WMADD address register.
//!
//! Power-on value: all bytes 0x55, WMADD = 0.

use crate::hash::Fnv64;

/// Work RAM: 128 KiB (0x20000 bytes) with 17-bit address pointer (WMADD).
pub struct Wram {
    mem: Vec<u8>,
    /// WMADD register (bits 0-16, i.e. values 0..0x20000).
    pub addr: u32,
}

impl Wram {
    const SIZE: usize = 0x20000;
    const MASK: u32 = 0x1FFFF;

    /// Create WRAM filled with power-on value 0x55, WMADD = 0.
    pub fn new() -> Self {
        let mut mem = vec![0u8; Self::SIZE];
        for b in mem.iter_mut() {
            *b = 0x55;
        }
        Wram { mem, addr: 0 }
    }

    /// Reset WMADD to 0 (memory contents preserved).
    pub fn reset_addr(&mut self) {
        self.addr = 0;
    }

    /// Read byte at index (masked to 17 bits).
    pub fn read(&self, idx: u32) -> u8 {
        let i = (idx & Self::MASK) as usize;
        self.mem[i]
    }

    /// Write byte at index (masked to 17 bits).
    pub fn write(&mut self, idx: u32, v: u8) {
        let i = (idx & Self::MASK) as usize;
        self.mem[i] = v;
    }

    /// $2180 read: return byte at WMADD, then increment WMADD.
    pub fn data_read(&mut self) -> u8 {
        let b = self.read(self.addr);
        self.addr = (self.addr + 1) & Self::MASK;
        b
    }

    /// $2180 write: store byte at WMADD, then increment WMADD.
    pub fn data_write(&mut self, v: u8) {
        self.write(self.addr, v);
        self.addr = (self.addr + 1) & Self::MASK;
    }

    /// $2181: set bits 7-0 of WMADD.
    pub fn set_addr_l(&mut self, v: u8) {
        self.addr = (self.addr & !0xFF) | (v as u32);
    }

    /// $2182: set bits 15-8 of WMADD.
    pub fn set_addr_m(&mut self, v: u8) {
        self.addr = (self.addr & !0xFF00) | ((v as u32) << 8);
    }

    /// $2183: bit 0 of input -> bit 16 of WMADD.
    pub fn set_addr_h(&mut self, v: u8) {
        let bit = (v & 1) as u32;
        self.addr = (self.addr & 0xFFFF) | (bit << 16);
    }

    /// Hash entire memory contents and WMADD into the given hasher.
    pub fn hash_into(&self, h: &mut Fnv64) {
        h.write(&self.mem);
        h.write_u32(self.addr);
    }
}
