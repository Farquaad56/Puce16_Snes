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

impl crate::savestate::State for Wram {
    fn save(&self, w: &mut crate::savestate::StateWriter) {
        // mem (128 KiB, bytes without prefix) then addr (u32)
        w.bytes(&self.mem);
        w.u32(self.addr);
    }

    fn load(&mut self, r: &mut crate::savestate::StateReader) -> Result<(), crate::savestate::StateError> {
        // Read mem into place
        for i in 0..Self::SIZE {
            let b = r.u8()?;
            self.mem[i] = b;
        }
        self.addr = r.u32()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::savestate::{State, StateWriter, StateReader};

    #[test]
    fn e08_03_wram_roundtrip() {
        let mut w = Wram::new();
        // Fill with pattern: mem[i] = (i * 7) as u8
        for i in 0..w.mem.len() {
            w.write(i as u32, (i * 7) as u8);
        }
        w.addr = 0x1ABCD;

        // Save
        let mut sw = StateWriter::new();
        w.save(&mut sw);
        let data = sw.finish();

        // Verify saved size: 0x20000 + 4 bytes
        assert_eq!(data.len(), 0x20000 + 4, "saved size should be 128 KiB + 4");

        // Load into fresh instance
        let mut w2 = Wram::new();
        let mut sr = StateReader::new(&data);
        w2.load(&mut sr).unwrap();

        // Verify memory restored identically
        for i in 0..w.mem.len() {
            assert_eq!(w.read(i as u32), w2.read(i as u32), "byte {} mismatch", i);
        }
        assert_eq!(w.addr, w2.addr, "addr mismatch");

        // Verify re-save produces identical bytes
        let mut sw2 = StateWriter::new();
        w2.save(&mut sw2);
        assert_eq!(data, sw2.finish());

        // Verify hash_into matches
        let mut h1 = Fnv64::new();
        w.hash_into(&mut h1);
        let mut h2 = Fnv64::new();
        w2.hash_into(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn e08_03_wram_truncated() {
        let mut w = Wram::new();
        for i in 0..w.mem.len() {
            w.write(i as u32, (i * 7) as u8);
        }
        w.addr = 0x1ABCD;

        let mut sw = StateWriter::new();
        w.save(&mut sw);
        let data = sw.finish();

        // Test truncation at various points
        for n in [0, data.len() / 2, data.len() - 1] {
            let mut sr = StateReader::new(&data[..n]);
            let mut w2 = Wram::new();
            assert!(w2.load(&mut sr).is_err(), "truncated at {} should fail", n);
        }
    }

    /// Truncation for each of the three E08-03 types. Wram and Joypads are covered here;
    /// Cart has its own `e08_03_cart_truncated` (its test helper is private to cart/mod.rs).
    #[test]
    fn e08_03_truncated_each() {
        use crate::joypad::Joypads;
        use crate::savestate::StateError;

        let mut w = Wram::new();
        w.addr = 0x1ABCD;
        let mut sw = StateWriter::new();
        w.save(&mut sw);
        let data = sw.finish();
        for n in [0, data.len() / 2, data.len() - 1] {
            let mut w2 = Wram::new();
            assert_eq!(w2.load(&mut StateReader::new(&data[..n])), Err(StateError::Truncated), "wram n={n}");
        }

        let mut j = Joypads::new();
        j.set_buttons(0, 0xAAA0);
        j.set_buttons(1, 0x5550);
        let mut sj = StateWriter::new();
        j.save(&mut sj);
        let data = sj.finish();
        for n in [0, data.len() / 2, data.len() - 1] {
            let mut j2 = Joypads::new();
            assert_eq!(j2.load(&mut StateReader::new(&data[..n])), Err(StateError::Truncated), "joypads n={n}");
        }
    }
}
