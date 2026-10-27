//! Mode 7 register state: M7SEL, matrix coefficients, center, scroll, MPY.
//! Only registers; rendering comes later in E05.

use crate::hash::Fnv64;

/// Sign-extend a 13-bit value to i16.
pub fn sign_extend_13(x: u16) -> i16 {
    ((x as i16) << 3) >> 3
}

#[derive(Clone)]
pub struct M7Regs {
    pub flip_x: bool,
    pub flip_y: bool,
    pub fill: bool,
    pub outside: bool,
    pub a: i16,
    pub b: i16,
    pub c: i16,
    pub d: i16,
    pub cx: i16,
    pub cy: i16,
    pub hofs: i16,
    pub vofs: i16,
    latch: u8,
    b_last_byte: u8,
}

impl M7Regs {
    pub fn new() -> M7Regs {
        M7Regs {
            flip_x: false,
            flip_y: false,
            fill: false,
            outside: false,
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            cx: 0,
            cy: 0,
            hofs: 0,
            vofs: 0,
            latch: 0,
            b_last_byte: 0,
        }
    }

    /// M7SEL ($211A): bit0 flip_x, bit1 flip_y, bit6 fill, bit7 outside.
    pub fn write_sel(&mut self, v: u8) {
        self.flip_x = (v & 1) != 0;
        self.flip_y = (v & 2) != 0;
        self.fill = (v & 0x40) != 0;
        self.outside = (v & 0x80) != 0;
    }

    /// M7A-D ($211B-$211E): i=0..3 -> a,b,c,d. Two-byte write protocol with shared latch.
    pub fn write_matrix(&mut self, i: usize, v: u8) {
        let val = (((v as u16) << 8) | (self.latch as u16)) as i16;
        match i {
            0 => self.a = val,
            1 => { self.b = val; self.b_last_byte = v; }
            2 => self.c = val,
            3 => self.d = val,
            _ => {} // out of range: ignore
        }
        self.latch = v;
    }

    /// M7X/M7Y ($211F/$2120): i=0 -> cx, i=1 -> cy. 13-bit signed value.
    pub fn write_center(&mut self, i: usize, v: u8) {
        let raw = ((v & 0x1F) as u16) << 8 | (self.latch as u16);
        match i {
            0 => self.cx = sign_extend_13(raw),
            1 => self.cy = sign_extend_13(raw),
            _ => {} // out of range: ignore
        }
        self.latch = v;
    }

    /// M7HOFS/M7VOFS ($210D/$210E): i=0 -> hofs, i=1 -> vofs. 13-bit signed value.
    pub fn write_scroll(&mut self, i: usize, v: u8) {
        let raw = ((v as u16) << 8) | (self.latch as u16);
        match i {
            0 => self.hofs = sign_extend_13(raw),
            1 => self.vofs = sign_extend_13(raw),
            _ => {} // out of range: ignore
        }
        self.latch = v;
    }

    /// MPY result: a * (b_last_byte as signed byte). Computed on read.
    pub fn mpy(&self) -> i32 {
        (self.a as i32) * ((self.b_last_byte as i8) as i32)
    }

    /// MPY result as 24-bit unsigned bits.
    pub fn mpy_bits(&self) -> u32 {
        (self.mpy() as u32) & 0xFF_FFFF
    }

    pub fn hash_into(&self, h: &mut Fnv64) {
        h.write_u8(if self.flip_x { 1 } else { 0 });
        h.write_u8(if self.flip_y { 1 } else { 0 });
        h.write_u8(if self.fill { 1 } else { 0 });
        h.write_u8(if self.outside { 1 } else { 0 });
        h.write_u16(self.a as u16);
        h.write_u16(self.b as u16);
        h.write_u16(self.c as u16);
        h.write_u16(self.d as u16);
        h.write_u16(self.cx as u16);
        h.write_u16(self.cy as u16);
        h.write_u16(self.hofs as u16);
        h.write_u16(self.vofs as u16);
        h.write_u8(self.latch);
        h.write_u8(self.b_last_byte);
    }
}
