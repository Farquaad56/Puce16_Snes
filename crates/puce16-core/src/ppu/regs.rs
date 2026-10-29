//! PPU scroll registers: BGnHOFS/BGnVOFS with shared latches.

use crate::hash::Fnv64;
use crate::savestate::{State, StateError, StateReader, StateWriter};

#[derive(Clone)]
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

impl State for Scroll {
    fn save(&self, w: &mut StateWriter) {
        // not saved: none (all fields are stateful)
        w.slice_u16(&self.hofs);
        w.slice_u16(&self.vofs);
        w.u8(self.bgofs_latch);
        w.u8(self.bghofs_latch);
    }

    fn load(&mut self, r: &mut StateReader) -> Result<(), StateError> {
        r.slice_u16_into(&mut self.hofs)?;
        r.slice_u16_into(&mut self.vofs)?;
        self.bgofs_latch = r.u8()?;
        self.bghofs_latch = r.u8()?;
        Ok(())
    }
}

#[derive(Clone)]
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

impl State for Regs {
    fn save(&self, w: &mut StateWriter) {
        // not saved: none (all fields are stateful)
        w.u8(self.obj_size);
        w.u16(self.obj_name_gap);
        w.u16(self.obj_base);
        w.u8(self.bg_mode);
        w.bool(self.bg3_priority);
        for n in 0..4 {
            w.bool(self.tile16[n]);
        }
        w.u8(self.mosaic_raw);
        w.u8(self.mosaic_en);
        w.slice_u16(&self.map_base);
        for n in 0..4 {
            w.bool(self.map_x2[n]);
        }
        for n in 0..4 {
            w.bool(self.map_y2[n]);
        }
        w.slice_u16(&self.chr_base);
        w.bytes(&self.win_sel);
        w.bytes(&self.wh);
        w.u8(self.win_log_bg);
        w.u8(self.win_log_obj);
        w.u8(self.win_log_col);
        w.u8(self.tm);
        w.u8(self.ts);
        w.u8(self.tmw);
        w.u8(self.tsw);
        w.bool(self.direct_color);
        w.bool(self.addend_subscreen);
        w.u8(self.math_region);
        w.u8(self.black_region);
        w.u8(self.math_mask);
        w.bool(self.math_half);
        w.bool(self.math_sub);
        w.u8(self.fixed_r);
        w.u8(self.fixed_g);
        w.u8(self.fixed_b);
    }

    fn load(&mut self, r: &mut StateReader) -> Result<(), StateError> {
        self.obj_size = r.u8()?;
        self.obj_name_gap = r.u16()?;
        self.obj_base = r.u16()?;
        self.bg_mode = r.u8()?;
        self.bg3_priority = r.bool()?;
        for n in 0..4 {
            self.tile16[n] = r.bool()?;
        }
        self.mosaic_raw = r.u8()?;
        self.mosaic_en = r.u8()?;
        r.slice_u16_into(&mut self.map_base)?;
        for n in 0..4 {
            self.map_x2[n] = r.bool()?;
        }
        for n in 0..4 {
            self.map_y2[n] = r.bool()?;
        }
        r.slice_u16_into(&mut self.chr_base)?;
        r.bytes_into(&mut self.win_sel)?;
        r.bytes_into(&mut self.wh)?;
        self.win_log_bg = r.u8()?;
        self.win_log_obj = r.u8()?;
        self.win_log_col = r.u8()?;
        self.tm = r.u8()?;
        self.ts = r.u8()?;
        self.tmw = r.u8()?;
        self.tsw = r.u8()?;
        self.direct_color = r.bool()?;
        self.addend_subscreen = r.bool()?;
        self.math_region = r.u8()?;
        self.black_region = r.u8()?;
        self.math_mask = r.u8()?;
        self.math_half = r.bool()?;
        self.math_sub = r.bool()?;
        self.fixed_r = r.u8()?;
        self.fixed_g = r.u8()?;
        self.fixed_b = r.u8()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::savestate::{StateWriter, StateReader};

    #[test]
    fn e08_07_scroll_roundtrip() {
        let mut original = Scroll::new();
        for i in 0..4 {
            original.hofs[i] = (i * 100 + 5) as u16;
            original.vofs[i] = (i * 200 + 7) as u16;
        }
        original.bgofs_latch = 0xAB;
        original.bghofs_latch = 0xCD;

        let mut w = StateWriter::new();
        original.save(&mut w);
        let data = w.finish();

        let mut restored = Scroll::new();
        let mut r = StateReader::new(&data);
        restored.load(&mut r).unwrap();

        for i in 0..4 {
            assert_eq!(original.hofs[i], restored.hofs[i]);
            assert_eq!(original.vofs[i], restored.vofs[i]);
        }
        assert_eq!(original.bgofs_latch, restored.bgofs_latch);
        assert_eq!(original.bghofs_latch, restored.bghofs_latch);

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
    fn e08_07_regs_roundtrip() {
        let mut original = Regs::new();
        // Set each field to a distinct non-zero value (i+1 pattern)
        original.obj_size = 1;
        original.obj_name_gap = 2;
        original.obj_base = 3;
        original.bg_mode = 4;
        original.bg3_priority = true;
        for n in 0..4 {
            original.tile16[n] = (n % 2) == 0;
        }
        original.mosaic_raw = 8;
        original.mosaic_en = 9;
        for n in 0..4 {
            original.map_base[n] = (n + 10) as u16;
        }
        for n in 0..4 {
            original.map_x2[n] = (n % 3) == 0;
        }
        for n in 0..4 {
            original.map_y2[n] = (n % 3) == 1;
        }
        for n in 0..4 {
            original.chr_base[n] = (n + 14) as u16;
        }
        for i in 0..3 {
            original.win_sel[i] = (i + 18) as u8;
        }
        for i in 0..4 {
            original.wh[i] = (i + 21) as u8;
        }
        original.win_log_bg = 25;
        original.win_log_obj = 26;
        original.win_log_col = 27;
        original.tm = 28;
        original.ts = 29;
        original.tmw = 30;
        original.tsw = 31;
        original.direct_color = true;
        original.addend_subscreen = true;
        original.math_region = 34;
        original.black_region = 35;
        original.math_mask = 36;
        original.math_half = true;
        original.math_sub = true;
        original.fixed_r = 39;
        original.fixed_g = 40;
        original.fixed_b = 41;

        let mut w = StateWriter::new();
        original.save(&mut w);
        let data = w.finish();

        let mut restored = Regs::new();
        let mut r = StateReader::new(&data);
        restored.load(&mut r).unwrap();

        assert_eq!(original.obj_size, restored.obj_size);
        assert_eq!(original.obj_name_gap, restored.obj_name_gap);
        assert_eq!(original.obj_base, restored.obj_base);
        assert_eq!(original.bg_mode, restored.bg_mode);
        assert_eq!(original.bg3_priority, restored.bg3_priority);
        for n in 0..4 {
            assert_eq!(original.tile16[n], restored.tile16[n]);
        }
        assert_eq!(original.mosaic_raw, restored.mosaic_raw);
        assert_eq!(original.mosaic_en, restored.mosaic_en);
        for n in 0..4 {
            assert_eq!(original.map_base[n], restored.map_base[n]);
        }
        for n in 0..4 {
            assert_eq!(original.map_x2[n], restored.map_x2[n]);
        }
        for n in 0..4 {
            assert_eq!(original.map_y2[n], restored.map_y2[n]);
        }
        for n in 0..4 {
            assert_eq!(original.chr_base[n], restored.chr_base[n]);
        }
        for i in 0..3 {
            assert_eq!(original.win_sel[i], restored.win_sel[i]);
        }
        for i in 0..4 {
            assert_eq!(original.wh[i], restored.wh[i]);
        }
        assert_eq!(original.win_log_bg, restored.win_log_bg);
        assert_eq!(original.win_log_obj, restored.win_log_obj);
        assert_eq!(original.win_log_col, restored.win_log_col);
        assert_eq!(original.tm, restored.tm);
        assert_eq!(original.ts, restored.ts);
        assert_eq!(original.tmw, restored.tmw);
        assert_eq!(original.tsw, restored.tsw);
        assert_eq!(original.direct_color, restored.direct_color);
        assert_eq!(original.addend_subscreen, restored.addend_subscreen);
        assert_eq!(original.math_region, restored.math_region);
        assert_eq!(original.black_region, restored.black_region);
        assert_eq!(original.math_mask, restored.math_mask);
        assert_eq!(original.math_half, restored.math_half);
        assert_eq!(original.math_sub, restored.math_sub);
        assert_eq!(original.fixed_r, restored.fixed_r);
        assert_eq!(original.fixed_g, restored.fixed_g);
        assert_eq!(original.fixed_b, restored.fixed_b);

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
    fn e08_07_truncated_each() {
        // Scroll truncated
        let scroll = Scroll::new();
        let mut w = StateWriter::new();
        scroll.save(&mut w);
        let sdata = w.finish();
        for n in [0, sdata.len() / 2, sdata.len() - 1] {
            let mut r = StateReader::new(&sdata[..n]);
            let mut s = Scroll::new();
            assert!(s.load(&mut r).is_err(), "scroll truncated at {}", n);
        }

        // Regs truncated
        let regs = Regs::new();
        let mut w = StateWriter::new();
        regs.save(&mut w);
        let rdata = w.finish();
        for n in [0, rdata.len() / 2, rdata.len() - 1] {
            let mut r = StateReader::new(&rdata[..n]);
            let mut rg = Regs::new();
            assert!(rg.load(&mut r).is_err(), "regs truncated at {}", n);
        }

        // M7Regs truncated
        let m7 = crate::ppu::mode7::M7Regs::new();
        let mut w = StateWriter::new();
        m7.save(&mut w);
        let mdata = w.finish();
        for n in [0, mdata.len() / 2, mdata.len() - 1] {
            let mut r = StateReader::new(&mdata[..n]);
            let mut m = crate::ppu::mode7::M7Regs::new();
            assert!(m.load(&mut r).is_err(), "m7 truncated at {}", n);
        }
    }
}
