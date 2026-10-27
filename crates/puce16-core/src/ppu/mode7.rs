//! Mode 7 register state, affine sampling, and pixel rendering.
//! clip13/m7_sample are pure functions; m7_pixel reads VRAM/CGRAM.

use crate::hash::Fnv64;
use super::bg::{Pix, direct_color_m7};
use super::bg::mosaic_xy;

/// Sign-extend a 13-bit value to i16.
pub fn sign_extend_13(x: u16) -> i16 {
    ((x as i16) << 3) >> 3
}

/// Clip a value to signed 13-bit range with wraparound (spec 05b section 8).
pub fn clip13(n: i32) -> i32 {
    if n & 0x2000 != 0 {
        n | !0x3FF
    } else {
        n & 0x3FF
    }
}

/// Mode 7 affine sampling (spec 05b section 8). Returns (px, py) in the 1024x1024 map.
pub fn m7_sample(r: &M7Regs, x: i32, v: i32) -> (i32, i32) {
    let sx = if r.flip_x { 255 - x } else { x };
    let sy = if r.flip_y { 255 - v } else { v };

    let a = r.a as i32;
    let b = r.b as i32;
    let c = r.c as i32;
    let d = r.d as i32;

    let dh = clip13(r.hofs as i32 - r.cx as i32);
    let dv = clip13(r.vofs as i32 - r.cy as i32);

    let ox = ((a * dh) & !63) + ((b * dv) & !63) + ((b * sy) & !63) + ((r.cx as i32) << 8);
    let oy = ((c * dh) & !63) + ((d * dv) & !63) + ((d * sy) & !63) + ((r.cy as i32) << 8);

    let px = (ox + a * sx) >> 8;
    let py = (oy + c * sx) >> 8;

    (px, py)
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

impl crate::ppu::Ppu {
    /// Mode 7 pixel for BG1 (n=0) or EXTBG BG2 (n=1). Returns None if transparent.
    pub fn m7_pixel(&self, n: usize, x: usize, v: usize) -> Option<Pix> {
        // Must be in mode 7
        if self.regs.bg_mode != 7 {
            return None;
        }
        // EXTBG only for BG2 (n=1)
        if n == 1 && !self.extbg {
            return None;
        }
        if n > 1 {
            return None;
        }

        // Apply mosaic
        let enabled = (self.regs.mosaic_en >> n) & 1 != 0;
        let (xm, vm) = mosaic_xy(self.regs.mosaic_size(), enabled, x, v);

        // Affine sample
        let (px, py) = m7_sample(&self.m7, xm as i32, vm as i32);

        // Determine tile and pixel coordinates based on outside/fill settings
        let mut tile: u16;
        let fx: u8;
        let fy: u8;

        if (px | py) & !0x3FF != 0 {
            // Outside the 1024x1024 map
            if !self.m7.outside {
                // Wrap around
                let map_addr = (((py & 0x3FF) >> 3) << 7) | ((px & 0x3FF) >> 3);
                tile = self.vram.word(map_addr as u16) & 0xFF;
                fx = (px & 7) as u8;
                fy = (py & 7) as u8;
            } else if !self.m7.fill {
                // Transparent outside
                return None;
            } else {
                // Fill with tile 0
                tile = 0;
                fx = (px & 7) as u8;
                fy = (py & 7) as u8;
            }
        } else {
            // Normal case: within map bounds
            let map_addr = ((py >> 3) << 7) | (px >> 3);
            tile = self.vram.word(map_addr as u16) & 0xFF;
            fx = (px & 7) as u8;
            fy = (py & 7) as u8;
        }

        // Read pixel index from high byte of VRAM word
        let tile_data_addr = (tile * 64 + (fy as u16) * 8 + fx as u16) & 0x7FFF;
        let idx = ((self.vram.word(tile_data_addr) >> 8) & 0xFF) as u8;

        if idx == 0 {
            return None;
        }

        // BG1 (n=0): use direct color or CGRAM, priority 0
        if n == 0 {
            let rgb = if self.regs.direct_color {
                direct_color_m7(idx)
            } else {
                self.cgram.word(idx)
            };
            return Some(Pix { rgb, prio: 0, obj_pal: 0 });
        }

        // EXTBG BG2 (n=1): priority from high bit, index masked to 7 bits
        let prio = (idx >> 7) as u8;
        let i7 = idx & 0x7F;
        if i7 == 0 {
            return None;
        }
        Some(Pix { rgb: self.cgram.word(i7), prio, obj_pal: 0 })
    }
}
