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
pub use self::bg::{Pix, LAYER_OBJ, Pick};
use self::mode7::M7Regs;
use self::output::{FrameBuf, FrameView, bgr555_to_xrgb, output_row, make_view};

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

#[derive(Clone)]
pub struct Ppu {
    region: Region,
    v: u16,
    dot: u16,
    field: bool,
    interlace: bool,
    obj_interlace: bool,
    overscan: bool,
    pseudo_hires: bool,
    extbg: bool,
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
    line_obj: [Option<Pix>; 256],
    vram: Vram,
    cgram: Cgram,
    oam: Oam,
    scroll: Scroll,
    m7: M7Regs,
    regs: Regs,
    fb: FrameBuf,
    view: Vec<u32>,
    view_w: usize,
    view_h: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PpuRegsSnapshot {
    pub v: u16,
    pub dot: u16,
    pub field: bool,
    pub forced_blank: bool,
    pub brightness: u8,
    pub bg_mode: u8,
    pub bg3_priority: bool,
    pub tile16: [bool; 4],
    pub obj_size: u8,
    pub map_base: [u16; 4],
    pub chr_base: [u16; 4],
    pub hofs: [u16; 4],
    pub vofs: [u16; 4],
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
    pub fixed: [u8; 3],
    pub vmain: u8,
    pub vmadd: u16,
    pub cgram_addr: u8,
    pub oam_internal: u16,
    pub interlace: bool,
    pub obj_interlace: bool,
    pub overscan: bool,
    pub pseudo_hires: bool,
    pub extbg: bool,
    pub m7: [i16; 6],
    pub m7_hofs: i16,
    pub m7_vofs: i16,
}

impl Ppu {
    pub fn new(region: Region) -> Ppu {
        Ppu {
            region,
            v: 0,
            dot: 0,
            field: false,
            interlace: false,
            obj_interlace: false,
            overscan: false,
            pseudo_hires: false,
            extbg: false,
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
            line_obj: [None; 256],
            vram: Vram::new(),
            cgram: Cgram::new(),
            oam: Oam::new(),
            scroll: Scroll::new(),
            m7: M7Regs::new(),
            regs: Regs::new(),
            fb: FrameBuf::new(),
            view: vec![0xFF00_0000; 256 * 224],
            view_w: 256,
            view_h: 224,
        }
    }

    pub fn reset(&mut self) {
        // Preserve region (not part of PPU state that resets)
        let r = self.region;

        // Reset registers and counters to power-on state
        self.v = 0;
        self.dot = 0;
        self.field = false;
        self.interlace = false;
        self.obj_interlace = false;
        self.overscan = false;
        self.pseudo_hires = false;
        self.extbg = false;
        self.forced_blank = true;
        self.brightness = 0;
        self.ppu1_mdr = 0;
        self.ppu2_mdr = 0;
        self.latch_h = 0;
        self.latch_v = 0;
        self.counter_latched = false;
        self.ophct_byte = false;
        self.opvct_byte = false;
        self.range_over = false;
        self.time_over = false;

        // Reset register units (preserves VRAM/CGRAM/OAM memory contents)
        self.vram.reset_regs();
        self.cgram.reset_regs();
        self.oam.reset_regs();
        self.scroll = Scroll::new();
        self.m7 = M7Regs::new();
        self.regs = Regs::new();

        // Reset framebuffer and view to defaults (256x224 black)
        self.fb = FrameBuf::new();
        self.view = vec![0xFF00_0000; 256 * 224];
        self.view_w = 256;
        self.view_h = 224;

        // Restore region (it was captured but not reset)
        self.region = r;
    }

    pub fn regs_snapshot(&self) -> PpuRegsSnapshot {
        PpuRegsSnapshot {
            v: self.v,
            dot: self.dot,
            field: self.field,
            forced_blank: self.forced_blank,
            brightness: self.brightness,
            bg_mode: self.regs.bg_mode,
            bg3_priority: self.regs.bg3_priority,
            tile16: self.regs.tile16,
            obj_size: self.regs.obj_size,
            map_base: self.regs.map_base,
            chr_base: self.regs.chr_base,
            hofs: [self.scroll.hofs(0), self.scroll.hofs(1), self.scroll.hofs(2), self.scroll.hofs(3)],
            vofs: [self.scroll.vofs(0), self.scroll.vofs(1), self.scroll.vofs(2), self.scroll.vofs(3)],
            tm: self.regs.tm,
            ts: self.regs.ts,
            tmw: self.regs.tmw,
            tsw: self.regs.tsw,
            direct_color: self.regs.direct_color,
            addend_subscreen: self.regs.addend_subscreen,
            math_region: self.regs.math_region,
            black_region: self.regs.black_region,
            math_mask: self.regs.math_mask,
            math_half: self.regs.math_half,
            math_sub: self.regs.math_sub,
            fixed: [self.regs.fixed_r, self.regs.fixed_g, self.regs.fixed_b],
            vmain: self.vram.vmain(),
            vmadd: self.vram.vmadd(),
            cgram_addr: self.cgram.addr(),
            oam_internal: self.oam.internal(),
            interlace: self.interlace,
            obj_interlace: self.obj_interlace,
            overscan: self.overscan,
            pseudo_hires: self.pseudo_hires,
            extbg: self.extbg,
            m7: [self.m7.a, self.m7.b, self.m7.c, self.m7.d, self.m7.cx, self.m7.cy],
            m7_hofs: self.m7.hofs,
            m7_vofs: self.m7.vofs,
        }
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
                // Frame start: clear over flags, VBLANK_END event
                self.time_over = false;
                self.range_over = false;
                events |= DotEvents::VBLANK_END;
            }
            if self.v == self.vblank_start() {
                events |= DotEvents::VBLANK_START;
                // Reload OAM address at VBlank start (if not forced blank)
                if !self.forced_blank {
                    self.oam.reload();
                }
                // Capture frame into view after OAM reload
                let vis_last = self.vblank_start() - 1;
                let w = 256;
                let h = (vis_last as usize) * (if self.interlace { 2 } else { 1 });
                self.fb.set_view(w, h);
                self.view = self.fb.compact();
                self.view_w = w;
                self.view_h = h;
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

        // Point 3: render pixel during visible area (after H-blank events)
        let vis_last = self.vblank_start() - 1;
        if self.v >= 1 && self.v <= vis_last && self.dot >= 22 && self.dot <= 277 {
            let x = (self.dot - 22) as usize;
            let row = output_row((self.v - 1) as usize, self.field, self.interlace);
            if self.forced_blank {
                self.fb.set(x, row, 0xFF00_0000);
            } else {
                let argb = bgr555_to_xrgb(self.pixel_bgr555(x), self.brightness);
                self.fb.set(x, row, argb);
            }
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

    /// Return a view of the last complete frame (256x224 or interlaced).
    pub fn frame(&self) -> FrameView<'_> {
        make_view(self.view_w, self.view_h, &self.view)
    }

    /// E04: return backdrop color from CGRAM word 0.
    /// E05 will replace this with full composition.
    fn pixel_bgr555(&self, _x: usize) -> u16 {
        self.cgram.word(0)
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
                // SETINI: interlace bit0, obj_interlace bit1, overscan bit2,
                // pseudo_hires bit3, extbg bit6; bit7 ignored
                self.interlace = (v & 1) != 0;
                self.obj_interlace = (v & 2) != 0;
                self.overscan = (v & 4) != 0;
                self.pseudo_hires = (v & 8) != 0;
                self.extbg = (v & 0x40) != 0;
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

    pub fn obj_interlace(&self) -> bool {
        self.obj_interlace
    }

    pub fn pseudo_hires(&self) -> bool {
        self.pseudo_hires
    }

    pub fn extbg(&self) -> bool {
        self.extbg
    }

    pub fn oam_internal_addr(&self) -> u16 {
        self.oam.internal()
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

    /// Determine if a layer is hidden by windows at pixel x.
    /// Does NOT consult TMW/TSW (that's done in pick, E05-14).
    pub fn window_hides(&self, layer: u8, x: usize) -> bool {
        let g = self::window::layer_group(&self.regs, layer);
        let logic = self::window::layer_logic(&self.regs, layer);
        self::window::window_out(g, logic, x as u8, self.regs.wh)
    }

    pub fn hash_into(&self, h: &mut Fnv64) {
        h.write_u8(self.region as u8);
        h.write_u16(self.v);
        h.write_u16(self.dot);
        h.write_u8(if self.field { 1 } else { 0 });
        h.write_u8(if self.interlace { 1 } else { 0 });
        h.write_u8(if self.obj_interlace { 1 } else { 0 });
        h.write_u8(if self.overscan { 1 } else { 0 });
        h.write_u8(if self.pseudo_hires { 1 } else { 0 });
        h.write_u8(if self.extbg { 1 } else { 0 });
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