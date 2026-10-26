//! PPU scroll registers: BGnHOFS/BGnVOFS with shared latches.

use crate::hash::Fnv64;

pub struct Scroll {
    hofs: [u16; 4],
    vofs: [u16; 4],
    bgofs_latch: u8,
    bghofs_latch: u8,
}

impl Scroll {
    pub fn new() -> Scroll {
        Scroll {
            hofs: [0; 4],
            vofs: [0; 4],
            bgofs_latch: 0,
            bghofs_latch: 0,
        }
    }

    pub fn write_hofs(&mut self, n: usize, v: u8) {
        if n > 3 { return; }
        let low = ((self.bgofs_latch & 0xF8) as u16) | ((self.bghofs_latch & 7) as u16);
        self.hofs[n] = (((v as u16) << 8) | low) & 0x3FF;
        self.bgofs_latch = v;
        self.bghofs_latch = v;
    }

    pub fn write_vofs(&mut self, n: usize, v: u8) {
        if n > 3 { return; }
        let low = self.bgofs_latch as u16;
        self.vofs[n] = (((v as u16) << 8) | low) & 0x3FF;
        self.bgofs_latch = v;
    }

    pub fn hofs(&self, n: usize) -> u16 {
        if n > 3 { return 0; }
        self.hofs[n]
    }

    pub fn vofs(&self, n: usize) -> u16 {
        if n > 3 { return 0; }
        self.vofs[n]
    }

    pub fn hash_into(&self, h: &mut Fnv64) {
        for i in 0..4 {
            h.write_u16(self.hofs[i]);
        }
        for i in 0..4 {
            h.write_u16(self.vofs[i]);
        }
        h.write_u8(self.bgofs_latch);
        h.write_u8(self.bghofs_latch);
    }
}

pub struct Regs {
    pub obj_size: u8,
    pub obj_name_gap: u16,
    pub obj_base: u16,
    pub bg_mode: u8,
    pub bg3_priority: bool,
    pub tile16: [bool; 4],
    pub mosaic_raw: u8,
    pub mosaic_en: u8,
    pub map_base: [u16; 4],
    pub map_x2: [bool; 4],
    pub map_y2: [bool; 4],
    pub chr_base: [u16; 4],
    pub win_sel: [u8; 3],
    pub wh: [u8; 4],
    pub win_log_bg: u8,
    pub win_log_obj: u8,
    pub win_log_col: u8,
    pub tm: u8,
    pub ts: u8,
    pub tmw: u8,
    pub tsw: u8,
    pub direct_color: bool,
    pub addend_subscreen: bool,
    pub math_region: u8,
    pub black_region: u8,
    pub math_mask: u8,
    pub math_half: bool,
    pub math_sub: bool,
    pub fixed_r: u8,
    pub fixed_g: u8,
    pub fixed_b: u8,
}

impl Regs {
    pub fn new() -> Regs {
        Regs {
            obj_size: 0,
            obj_name_gap: 0,
            obj_base: 0,
            bg_mode: 0,
            bg3_priority: false,
            tile16: [false; 4],
            mosaic_raw: 0,
            mosaic_en: 0,
            map_base: [0; 4],
            map_x2: [false; 4],
            map_y2: [false; 4],
            chr_base: [0; 4],
            win_sel: [0; 3],
            wh: [0; 4],
            win_log_bg: 0,
            win_log_obj: 0,
            win_log_col: 0,
            tm: 0,
            ts: 0,
            tmw: 0,
            tsw: 0,
            direct_color: false,
            addend_subscreen: false,
            math_region: 0,
            black_region: 0,
            math_mask: 0,
            math_half: false,
            math_sub: false,
            fixed_r: 0,
            fixed_g: 0,
            fixed_b: 0,
        }
    }

    pub fn write(&mut self, reg: u8, v: u8) -> bool {
        match reg {
            0x01 => {
                self.obj_size = v >> 5;
                let gap_words = (((v >> 3) & 3) as u16) + 1;
                self.obj_name_gap = gap_words << 12;
                self.obj_base = ((v & 7) as u16) << 13;
            }
            0x05 => {
                self.bg_mode = v & 7;
                self.bg3_priority = (v & 8) != 0;
                for n in 0..4 {
                    let bit = ((v >> (4 + n)) & 1) != 0;
                    self.tile16[n] = bit;
                }
            }
            0x06 => {
                self.mosaic_raw = v >> 4;
                self.mosaic_en = v & 0x0F;
            }
            0x07..=0x0A => {
                let n = (reg - 7) as usize;
                self.map_base[n] = ((v >> 2) as u16) << 10;
                self.map_x2[n] = (v & 1) != 0;
                self.map_y2[n] = (v & 2) != 0;
            }
            0x0B => {
                self.chr_base[0] = ((v & 0xF) as u16) << 12;
                self.chr_base[1] = ((v >> 4) as u16) << 12;
            }
            0x0C => {
                self.chr_base[2] = ((v & 0xF) as u16) << 12;
                self.chr_base[3] = ((v >> 4) as u16) << 12;
            }
            0x23 => { self.win_sel[0] = v; }
            0x24 => { self.win_sel[1] = v; }
            0x25 => { self.win_sel[2] = v; }
            0x26..=0x29 => {
                let n = (reg - 0x26) as usize;
                self.wh[n] = v;
            }
            0x2A => { self.win_log_bg = v; }
            0x2B => {
                self.win_log_obj = v & 3;
                self.win_log_col = (v >> 2) & 3;
            }
            0x2C => { self.tm = v & 0x1F; }
            0x2D => { self.ts = v & 0x1F; }
            0x2E => { self.tmw = v & 0x1F; }
            0x2F => { self.tsw = v & 0x1F; }
            0x30 => {
                self.direct_color = (v & 1) != 0;
                self.addend_subscreen = (v & 2) != 0;
                self.math_region = (v >> 4) & 3;
                self.black_region = (v >> 6) & 3;
            }
            0x31 => {
                self.math_mask = v & 0x3F;
                self.math_half = (v & 0x40) != 0;
                self.math_sub = (v & 0x80) != 0;
            }
            0x32 => {
                let c = v & 0x1F;
                if (v & 0x20) != 0 { self.fixed_r = c; }
                if (v & 0x40) != 0 { self.fixed_g = c; }
                if (v & 0x80) != 0 { self.fixed_b = c; }
            }
            _ => return false,
        }
        true
    }

    pub fn mosaic_size(&self) -> u8 {
        self.mosaic_raw + 1
    }

    pub fn hash_into(&self, h: &mut Fnv64) {
        h.write_u8(self.obj_size);
        h.write_u16(self.obj_name_gap);
        h.write_u16(self.obj_base);
        h.write_u8(self.bg_mode);
        h.write_u8(if self.bg3_priority { 1 } else { 0 });
        for n in 0..4 {
            h.write_u8(if self.tile16[n] { 1 } else { 0 });
        }
        h.write_u8(self.mosaic_raw);
        h.write_u8(self.mosaic_en);
        for n in 0..4 {
            h.write_u16(self.map_base[n]);
        }
        for n in 0..4 {
            h.write_u8(if self.map_x2[n] { 1 } else { 0 });
        }
        for n in 0..4 {
            h.write_u8(if self.map_y2[n] { 1 } else { 0 });
        }
        for n in 0..4 {
            h.write_u16(self.chr_base[n]);
        }
        for i in 0..3 {
            h.write_u8(self.win_sel[i]);
        }
        for i in 0..4 {
            h.write_u8(self.wh[i]);
        }
        h.write_u8(self.win_log_bg);
        h.write_u8(self.win_log_obj);
        h.write_u8(self.win_log_col);
        h.write_u8(self.tm);
        h.write_u8(self.ts);
        h.write_u8(self.tmw);
        h.write_u8(self.tsw);
        h.write_u8(if self.direct_color { 1 } else { 0 });
        h.write_u8(if self.addend_subscreen { 1 } else { 0 });
        h.write_u8(self.math_region);
        h.write_u8(self.black_region);
        h.write_u8(self.math_mask);
        h.write_u8(if self.math_half { 1 } else { 0 });
        h.write_u8(if self.math_sub { 1 } else { 0 });
        h.write_u8(self.fixed_r);
        h.write_u8(self.fixed_g);
        h.write_u8(self.fixed_b);
    }
}
