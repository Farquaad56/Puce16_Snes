//! CGRAM (Color RAM) - 256 color entries, accessed in byte pairs.
//!
//! Each entry is a 15-bit color value stored as a u16 with bit 15 unused.
//! Writes and reads are done one byte at a time using an internal toggle
//! to track low/high byte position within each word.

use crate::hash::Fnv64;

pub struct Cgram {
    mem: [u16; 256],
    addr: u8,
    toggle: bool,
    latch: u8,
}

impl Cgram {
    pub fn new() -> Cgram {
        Cgram {
            mem: [0u16; 256],
            addr: 0,
            toggle: false,
            latch: 0,
        }
    }

    /// Reset address and control registers; memory contents are preserved.
    pub fn reset_regs(&mut self) {
        self.addr = 0;
        self.toggle = false;
        self.latch = 0;
    }

    /// Set CGADD register: sets address and resets toggle.
    pub fn set_addr(&mut self, v: u8) {
        self.addr = v;
        self.toggle = false;
    }

    /// Write one byte to CGDATA. Pairs of writes form a complete 16-bit word.
    /// First write (toggle=false) latches the low byte; second write (toggle=true)
    /// commits the word and increments the address.
    pub fn write(&mut self, v: u8) {
        if !self.toggle {
            // Low byte: latch it
            self.latch = v;
        } else {
            // High byte: commit the word (bit 15 is masked off)
            let w = ((v & 0x7F) as u16) << 8 | self.latch as u16;
            self.mem[self.addr as usize] = w;
            self.addr = self.addr.wrapping_add(1);
        }
        self.toggle = !self.toggle;
    }

    /// Read one byte from CGDATAREAD. Returns the low or high byte of the word
    /// at the current address, depending on toggle state. The caller stores
    /// this as ppu2_mdr and passes it back for the next read in the pair.
    pub fn read(&mut self, m: u8) -> u8 {
        let w = self.mem[self.addr as usize];
        let byte;
        if !self.toggle {
            // Low byte
            byte = (w & 0xFF) as u8;
        } else {
            // High byte: mask bit 15, OR in the MSB from previous read result
            byte = ((w >> 8) as u8 & 0x7F) | (m & 0x80);
            self.addr = self.addr.wrapping_add(1);
        }
        self.toggle = !self.toggle;
        byte
    }

    pub fn addr(&self) -> u8 {
        self.addr
    }

    pub fn toggle(&self) -> bool {
        self.toggle
    }

    /// Read a complete word at index i.
    pub fn word(&self, i: u8) -> u16 {
        self.mem[i as usize]
    }

    /// Write a complete word at index i (bit 15 is masked off). Used for testing/debugging.
    pub fn set_word(&mut self, i: u8, w: u16) {
        self.mem[i as usize] = w & 0x7FFF;
    }

    /// Return a reference to the entire CGRAM array.
    pub fn mem(&self) -> &[u16; 256] {
        &self.mem
    }

    pub fn hash_into(&self, h: &mut Fnv64) {
        h.write_u16_slice(&self.mem);
        h.write_u8(self.addr);
        h.write_u8(if self.toggle { 1 } else { 0 });
        h.write_u8(self.latch);
    }
}
