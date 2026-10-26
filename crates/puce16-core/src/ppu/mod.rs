pub mod vram;
pub mod cgram;
pub mod oam;
pub mod regs;
pub mod bg;
pub mod mode7;
pub mod obj;
pub mod window;
pub mod colormath;
pub mod output;
pub mod dot;

use crate::region::Region;
use crate::hash::Fnv64;
use crate::consts::HV_LATCH_DOT_OFFSET;
use self::dot::{line_type, line_dots as dot_line_dots, dot_clocks};
use self::vram::Vram;
use self::cgram::Cgram;
use self::oam::Oam;
use self::regs::{Scroll, Regs};
use self::mode7::M7Regs;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct DotEvents(pub u8);

impl DotEvents {
    pub const LINE_START: u8 = 1;
    pub const HBLANK_START: u8 = 2;
    pub const HBLANK_END: u8 = 4;
    pub const VBLANK_START: u8 = 8;
    pub const VBLANK_END: u8 = 16;

    pub fn has(self, m: u8) -> bool {
        self.0 & m != 0
    }
}

pub struct Ppu {
    region: Region,
    v: u16,
    dot: u16,
    field: bool,
    interlace: bool,
    overscan: bool,
    forced_blank: bool,
    brightness: u8,
    ppu1_mdr: u8,
    ppu2_mdr: u8,
    latch_h: u16,
    latch_v: u16,
    counter_latched: bool,
    ophct_byte: bool,
    opvct_byte: bool,
    range_over: bool,
    time_over: bool,
    vram: Vram,
    cgram: Cgram,
    oam: Oam,
    scroll: Scroll,
    m7: M7Regs,
    regs: Regs,
}

impl Ppu {
    pub fn new(region: Region) -> Ppu {
        Ppu {
            region,
            v: 0,
            dot: 0,
            field: false,
            interlace: false,
            overscan: false,
            forced_blank: true,
            brightness: 0,
            ppu1_mdr: 0,
            ppu2_mdr: 0,
            latch_h: 0,
            latch_v: 0,
            counter_latched: false,
            ophct_byte: false,
            opvct_byte: false,
            range_over: false,
            time_over: false,
            vram: Vram::new(),
            cgram: Cgram::new(),
            oam: Oam::new(),
            scroll: Scroll::new(),
            m7: M7Regs::new(),
            regs: Regs::new(),
        }
    }

    pub fn reset(&mut self) {
        let r = self.region;
        *self = Ppu::new(r);
    }

    pub fn v(&self) -> u16 { self.v }
    pub fn dot(&self) -> u16 { self.dot }
    pub fn field(&self) -> bool { self.field }

    pub fn line_dots(&self) -> u16 {
        let lt = line_type(self.region, self.interlace, self.field, self.v);
        dot_line_dots(lt)
    }

    pub fn dot_clocks(&self) -> u32 {
        let lt = line_type(self.region, self.interlace, self.field, self.v);
        dot_clocks(lt, self.dot)
    }

    pub fn last_line(&self) -> u16 {
        let base = if self.region == Region::Pal { 311 } else { 261 };
        if self.interlace && !self.field { base + 1 } else { base }
    }

    pub fn vblank_start(&self) -> u16 {
        if self.overscan { 240 } else { 225 }
    }

    pub fn in_vblank(&self) -> bool {
        self.v >= self.vblank_start()
    }

    pub fn in_hblank(&self) -> bool {
        self.dot >= 274 || self.dot < 1
    }

    pub fn step_dot(&mut self) -> DotEvents {
        let mut events: u8 = 0;

        // Events at dot == 0
        if self.dot == 0 {
            if self.v == 0 {
                events |= DotEvents::VBLANK_END;
            }
            if self.v == self.vblank_start() {
                events |= DotEvents::VBLANK_START;
            }
            events |= DotEvents::LINE_START;
        }

        // H-blank events
        if self.dot == 274 {
            events |= DotEvents::HBLANK_START;
        }
        if self.dot == 1 {
            events |= DotEvents::HBLANK_END;
        }

        // Advance dot
        let ld = self.line_dots();
        self.dot += 1;
        if self.dot == ld {
            self.dot = 0;
            self.v += 1;
            if self.v > self.last_line() {
                self.v = 0;
                self.field = !self.field;
            }
        }

        DotEvents(events)
    }

    pub fn brightness(&self) -> u8 {
        self.brightness
    }

    pub fn ppu1_mdr(&self) -> u8 {
        self.ppu1_mdr
    }

    pub fn ppu2_mdr(&self) -> u8 {
        self.ppu2_mdr
    }

    pub fn vram_ok(&self) -> bool {
        self.forced_blank || self.v >= self.vblank_start()
    }

    pub fn oam_ok(&self) -> bool {
        self.forced_blank || self.v >= self.vblank_start()
    }

    pub fn vram(&self) -> &[u16] {
        self.vram.mem()
    }

    pub fn cgram(&self) -> &[u16; 256] {
        self.cgram.mem()
    }

    pub fn oam(&self) -> &[u8; 544] {
        self.oam.mem()
    }

    pub fn vram_unit(&self) -> &Vram {
        &self.vram
    }

    pub fn cgram_unit(&self) -> &Cgram {
        &self.cgram
    }

    pub fn oam_unit(&self) -> &Oam {
        &self.oam
    }

    pub fn regs(&self) -> &Regs {
        &self.regs
    }

    pub fn scroll(&self) -> &Scroll {
        &self.scroll
    }

    pub fn m7(&self) -> &M7Regs {
        &self.m7
    }

    pub fn write(&mut self, reg: u8, v: u8) {
        match reg {
            0x00 => {
                self.forced_blank = (v & 0x80) != 0;
                self.brightness = v & 0x0F;
            }
            0x01 | 0x05..=0x0C | 0x23..=0x32 => {
                self.regs.write(reg, v);
            }
            0x02 => {
                self.oam.set_addr_l(v);
            }
            0x03 => {
                self.oam.set_addr_h(v);
            }
            0x04 => {
                self.oam.write(v, self.oam_ok());
            }
            0x0D..=0x14 => {
                let n = ((reg - 0x0D) / 2) as usize;
                if reg & 1 != 0 {
                    // Odd: HOFS
                    self.scroll.write_hofs(n, v);
                } else {
                    // Even: VOFS
                    self.scroll.write_vofs(n, v);
                }
                // Mode 7 scroll for BG1 registers only
                if reg == 0x0D {
                    self.m7.write_scroll(0, v);
                } else if reg == 0x0E {
                    self.m7.write_scroll(1, v);
                }
            }
            0x15 => {
                self.vram.set_vmain(v);
            }
            0x16 => {
                self.vram.set_addr_l(v);
            }
            0x17 => {
                self.vram.set_addr_h(v);
            }
            0x18 => {
                self.vram.write_low(v, self.vram_ok());
            }
            0x19 => {
                self.vram.write_high(v, self.vram_ok());
            }
            0x1A => {
                self.m7.write_sel(v);
            }
            0x1B..=0x1E => {
                let i = (reg - 0x1B) as usize;
                self.m7.write_matrix(i, v);
            }
            0x1F => {
                self.m7.write_center(0, v);
            }
            0x20 => {
                self.m7.write_center(1, v);
            }
            0x21 => {
                self.cgram.set_addr(v);
            }
            0x22 => {
                self.cgram.write(v);
            }
            0x33 => {
                // interlace bit0, overscan bit2; rest in E04-11
                self.interlace = (v & 1) != 0;
                self.overscan = (v & 4) != 0;
            }
            _ => {}
        }
    }

    pub fn read(&mut self, reg: u8, cpu_mdr: u8, wrio7: bool) -> u8 {
        match reg {
            0x34 | 0x35 | 0x36 => {
                let bits = self.m7.mpy_bits();
                let shift = (reg - 0x34) * 8;
                let val = ((bits >> shift) & 0xFF) as u8;
                self.ppu1_mdr = val;
                val
            }
            0x37 => {
                if wrio7 {
                    self.latch_counters();
                }
                cpu_mdr
            }
            0x38 => {
                let val = self.oam.read(self.oam_ok());
                self.ppu1_mdr = val;
                val
            }
            0x39 => {
                let val = self.vram.read_low(self.vram_ok());
                self.ppu1_mdr = val;
                val
            }
            0x3A => {
                let val = self.vram.read_high(self.vram_ok());
                self.ppu1_mdr = val;
                val
            }
            0x3B => {
                let val = self.cgram.read(self.ppu2_mdr);
                self.ppu2_mdr = val;
                val
            }
            0x3C => {
                let val = if !self.ophct_byte {
                    (self.latch_h & 0xFF) as u8
                } else {
                    (((self.latch_h >> 8) & 1) as u8) | (self.ppu2_mdr & 0xFE)
                };
                self.ophct_byte = !self.ophct_byte;
                self.ppu2_mdr = val;
                val
            }
            0x3D => {
                let val = if !self.opvct_byte {
                    (self.latch_v & 0xFF) as u8
                } else {
                    (((self.latch_v >> 8) & 1) as u8) | (self.ppu2_mdr & 0xFE)
                };
                self.opvct_byte = !self.opvct_byte;
                self.ppu2_mdr = val;
                val
            }
            0x3E => {
                let val = ((if self.range_over { 1 } else { 0 }) << 7)
                    | ((if self.time_over { 1 } else { 0 }) << 6)
                    | (self.ppu1_mdr & 0x10)
                    | 0x01;
                self.ppu1_mdr = val;
                val
            }
            0x3F => {
                let val = ((if self.field { 1 } else { 0 }) << 7)
                    | ((if self.counter_latched { 1 } else { 0 }) << 6)
                    | (self.ppu2_mdr & 0x20)
                    | ((if self.region == Region::Pal { 1 } else { 0 }) << 4)
                    | 0x03;
                self.counter_latched = false;
                self.ophct_byte = false;
                self.opvct_byte = false;
                self.ppu2_mdr = val;
                val
            }
            _ => cpu_mdr,
        }
    }

    pub fn wrio_write(&mut self, old: u8, new: u8) {
        if (old & 0x80) != 0 && (new & 0x80) == 0 {
            self.latch_counters();
        }
    }

    pub fn forced_blank(&self) -> bool {
        self.forced_blank
    }

    pub fn interlace(&self) -> bool {
        self.interlace
    }

    pub fn overscan(&self) -> bool {
        self.overscan
    }

    pub fn latch_counters(&mut self) {
        let dot_now = (self.dot as u32 + HV_LATCH_DOT_OFFSET) as u16;
        self.latch_h = dot_now;
        self.latch_v = self.v;
        self.counter_latched = true;
    }

    pub fn counter_latched(&self) -> bool {
        self.counter_latched
    }

    pub fn set_range_over(&mut self, b: bool) {
        self.range_over = b;
    }

    pub fn range_over(&self) -> bool {
        self.range_over
    }

    pub fn set_time_over(&mut self, b: bool) {
        self.time_over = b;
    }

    pub fn time_over(&self) -> bool {
        self.time_over
    }

    pub fn hash_into(&self, h: &mut Fnv64) {
        h.write_u8(self.region as u8);
        h.write_u16(self.v);
        h.write_u16(self.dot);
        h.write_u8(if self.field { 1 } else { 0 });
        h.write_u8(if self.interlace { 1 } else { 0 });
        h.write_u8(if self.overscan { 1 } else { 0 });
        h.write_u8(if self.forced_blank { 1 } else { 0 });
        h.write_u8(self.brightness);
        self.vram.hash_into(h);
        self.cgram.hash_into(h);
        self.oam.hash_into(h);
        self.scroll.hash_into(h);
        self.m7.hash_into(h);
        self.regs.hash_into(h);
        h.write_u8(self.ppu1_mdr);
        h.write_u8(self.ppu2_mdr);
        h.write_u16(self.latch_h);
        h.write_u16(self.latch_v);
        h.write_u8(if self.counter_latched { 1 } else { 0 });
        h.write_u8(if self.ophct_byte { 1 } else { 0 });
        h.write_u8(if self.opvct_byte { 1 } else { 0 });
        h.write_u8(if self.range_over { 1 } else { 0 });
        h.write_u8(if self.time_over { 1 } else { 0 });
    }
}