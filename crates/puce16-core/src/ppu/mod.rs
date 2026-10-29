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
use crate::savestate::{State, StateError, StateReader, StateWriter};
use crate::consts::{HV_LATCH_DOT_OFFSET, FB_WIDTH, FB_HEIGHT};
use self::dot::{line_type, line_dots as dot_line_dots, dot_clocks};
use self::vram::Vram;
use self::cgram::Cgram;
use self::oam::Oam;
use self::regs::{Scroll, Regs};
pub use self::bg::{Pix, LAYER_OBJ, Pick, LAYER_BACKDROP};
use self::bg::{priority_order, pick_layers};
use self::mode7::M7Regs;
use self::output::{FrameBuf, FrameView, bgr555_to_xrgb, output_row, make_view, pixel_double};
use self::colormath::{fixed_color, color_math, blend, math_layer_ok};

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
    // Image-derived state (NOT hashed, cleared by reset): E05-18.
    line_hires: bool,
    row_hires: Vec<bool>,
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
            line_hires: false,
            row_hires: vec![false; FB_HEIGHT],
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
        self.line_hires = false;
        self.row_hires = vec![false; FB_HEIGHT];

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
            let vis_last = self.vblank_start() - 1;
            if self.v >= 1 && self.v <= vis_last {
                // Evaluate sprites for this line (before LINE_START event)
                self.render_obj_line((self.v - 1) as u16);
            }
            if self.v == self.vblank_start() {
                events |= DotEvents::VBLANK_START;
                // Reload OAM address at VBlank start (if not forced blank)
                if !self.forced_blank {
                    self.oam.reload();
                }
                // Capture frame into view after OAM reload
                let vis_last = self.vblank_start() - 1;
                let h = (vis_last as usize) * (if self.interlace { 2 } else { 1 });
                let frame_hires = self.row_hires[0..h].iter().any(|b| *b);
                if frame_hires {
                    self.fb.set_view(512, h);
                    let mut view: Vec<u32> = Vec::with_capacity(512 * h);
                    for r in 0..h {
                        let start = r * FB_WIDTH;
                        if self.row_hires[r] {
                            view.extend_from_slice(&self.fb.pixels[start..start + 512]);
                        } else {
                            let d = pixel_double(&self.fb.pixels[start..start + 256], 256, 1);
                            view.extend_from_slice(&d);
                        }
                    }
                    self.view = view;
                    self.view_w = 512;
                } else {
                    self.fb.set_view(256, h);
                    self.view = self.fb.compact();
                    self.view_w = 256;
                }
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
            if self.dot == 22 {
                self.line_hires = self.hires_line();
                if row < self.row_hires.len() {
                    self.row_hires[row] = self.line_hires;
                }
            }
            if self.line_hires {
                let (e, o) = if self.forced_blank {
                    (0xFF00_0000, 0xFF00_0000)
                } else {
                    let (ce, co) = self.compose_hires(x, self.v as usize);
                    (bgr555_to_xrgb(ce, self.brightness), bgr555_to_xrgb(co, self.brightness))
                };
                self.fb.set(2 * x, row, e);
                self.fb.set(2 * x + 1, row, o);
            } else if self.forced_blank {
                self.fb.set(x, row, 0xFF00_0000);
            } else {
                let argb = bgr555_to_xrgb(self.compose_pixel(x, self.v as usize), self.brightness);
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

    /// E05: full non-hi-res composition at screen position (x, v).
    /// Returns BGR555 final color before brightness conversion.
    pub fn compose_pixel(&self, x: usize, v: usize) -> u16 {
        let layers = self.sample_layers(x, v);
        let order = priority_order(self.regs.bg_mode, self.regs.bg3_priority, self.extbg);

        // Main screen pick
        let mut main_hidden = [false; 5];
        for l in 0..5 {
            main_hidden[l] = self.layer_hidden(true, l as u8, x);
        }
        let backdrop_main = self.cgram.word(0);
        let main = pick_layers(order, &layers, self.regs.tm, main_hidden, backdrop_main);

        // Fixed color for sub screen / addend
        let fixed = fixed_color(self.regs.fixed_r, self.regs.fixed_g, self.regs.fixed_b);

        // Sub screen pick (only if addend_subscreen)
        let sub = if self.regs.addend_subscreen {
            let mut sub_hidden = [false; 5];
            for l in 0..5 {
                sub_hidden[l] = self.layer_hidden(false, l as u8, x);
            }
            pick_layers(order, &layers, self.regs.ts, sub_hidden, fixed)
        } else {
            Pick { rgb: fixed, layer: LAYER_BACKDROP, obj_pal: 0 }
        };

        // Color window flags and color math
        let (black, prevent) = self.color_window_flags(x);
        color_math(&main, &sub, black, prevent, &self.regs)
    }

    /// True when the current line is rendered with 512 columns.
    pub fn hires_line(&self) -> bool {
        self.regs.bg_mode == 5 || self.regs.bg_mode == 6 || self.pseudo_hires
    }

    /// E05-18: hi-res composition at (x, v): (column 2x, column 2x+1), BGR555.
    pub fn compose_hires(&self, x: usize, v: usize) -> (u16, u16) {
        let order = priority_order(self.regs.bg_mode, self.regs.bg3_priority, self.extbg);
        let lm = self.sample_layers_hx(x, 2 * x + 1, v);
        let ls = self.sample_layers_hx(x, 2 * x, v);
        let fixed = fixed_color(self.regs.fixed_r, self.regs.fixed_g, self.regs.fixed_b);

        let mut main_hidden = [false; 5];
        let mut sub_hidden = [false; 5];
        for l in 0..5 {
            main_hidden[l] = self.layer_hidden(true, l as u8, x);
            sub_hidden[l] = self.layer_hidden(false, l as u8, x);
        }
        let main = pick_layers(order, &lm, self.regs.tm, main_hidden, self.cgram.word(0));
        let sub = pick_layers(order, &ls, self.regs.ts, sub_hidden, fixed);
        let (black, prevent) = self.color_window_flags(x);

        let odd = color_math(&main, &sub, black, prevent, &self.regs);

        let base = if black { 0 } else { sub.rgb };
        let even = if !self.regs.addend_subscreen
            && !prevent
            && math_layer_ok(sub.layer, sub.obj_pal, self.regs.math_mask)
        {
            blend(base, fixed, self.regs.math_sub, self.regs.math_half)
        } else {
            base
        };
        (even, odd)
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

impl State for Ppu {
    fn save(&self, w: &mut StateWriter) {
        // not saved: none (all fields are stateful)
        w.u8(self.region as u8);
        self.vram.save(w);
        self.cgram.save(w);
        self.oam.save(w);
        self.scroll.save(w);
        self.m7.save(w);
        self.regs.save(w);
        w.u16(self.v);
        w.u16(self.dot);
        w.bool(self.field);
        w.bool(self.interlace);
        w.bool(self.obj_interlace);
        w.bool(self.overscan);
        w.bool(self.pseudo_hires);
        w.bool(self.extbg);
        w.bool(self.forced_blank);
        w.u8(self.brightness);
        w.u8(self.ppu1_mdr);
        w.u8(self.ppu2_mdr);
        w.u16(self.latch_h);
        w.u16(self.latch_v);
        w.bool(self.counter_latched);
        w.bool(self.ophct_byte);
        w.bool(self.opvct_byte);
        w.bool(self.range_over);
        w.bool(self.time_over);
        // line_obj: 256 entries of Option<Pix> (rgb u16, prio u8, obj_pal u8)
        for i in 0..256 {
            match self.line_obj[i] {
                None => w.u8(0),
                Some(p) => {
                    w.u8(1);
                    w.u16(p.rgb);
                    w.u8(p.prio);
                    w.u8(p.obj_pal);
                }
            }
        }
        // framebuffer: full 512x478 buffer
        w.slice_u32(&self.fb.pixels);
        // view (compact frame)
        w.u32(self.view.len() as u32);
        w.slice_u32(&self.view);
        w.u32(self.view_w as u32);
        w.u32(self.view_h as u32);
        w.bool(self.line_hires);
        // row_hires: 478 bools
        for i in 0..FB_HEIGHT {
            w.bool(self.row_hires[i]);
        }
    }

    fn load(&mut self, r: &mut StateReader) -> Result<(), StateError> {
        let region_byte = r.u8()?;
        if region_byte != (self.region as u8) {
            return Err(StateError::Mismatch("region"));
        }
        self.vram.load(r)?;
        self.cgram.load(r)?;
        self.oam.load(r)?;
        self.scroll.load(r)?;
        self.m7.load(r)?;
        self.regs.load(r)?;
        self.v = r.u16()?;
        self.dot = r.u16()?;
        self.field = r.bool()?;
        self.interlace = r.bool()?;
        self.obj_interlace = r.bool()?;
        self.overscan = r.bool()?;
        self.pseudo_hires = r.bool()?;
        self.extbg = r.bool()?;
        self.forced_blank = r.bool()?;
        self.brightness = r.u8()?;
        self.ppu1_mdr = r.u8()?;
        self.ppu2_mdr = r.u8()?;
        self.latch_h = r.u16()?;
        self.latch_v = r.u16()?;
        self.counter_latched = r.bool()?;
        self.ophct_byte = r.bool()?;
        self.opvct_byte = r.bool()?;
        self.range_over = r.bool()?;
        self.time_over = r.bool()?;
        // line_obj: 256 entries of Option<Pix>
        for i in 0..256 {
            let tag = r.u8()?;
            match tag {
                0 => self.line_obj[i] = None,
                1 => {
                    let rgb = r.u16()?;
                    let prio = r.u8()?;
                    let obj_pal = r.u8()?;
                    self.line_obj[i] = Some(Pix { rgb, prio, obj_pal });
                }
                _ => return Err(StateError::Mismatch("line_obj tag")),
            }
        }
        // framebuffer: full 512x478 buffer
        r.slice_u32_into(&mut self.fb.pixels)?;
        // view (compact frame)
        let view_len = r.u32()? as usize;
        let mut loaded_view = vec![0u32; view_len];
        r.slice_u32_into(&mut loaded_view)?;
        self.view_w = r.u32()? as usize;
        self.view_h = r.u32()? as usize;
        if loaded_view.len() != self.view_w * self.view_h {
            return Err(StateError::Mismatch("view size"));
        }
        self.view = loaded_view;
        self.line_hires = r.bool()?;
        // row_hires: 478 bools
        for i in 0..FB_HEIGHT {
            self.row_hires[i] = r.bool()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::savestate::{StateWriter, StateReader};

    fn make_ppu_with_state() -> Ppu {
        let mut p = Ppu::new(Region::Ntsc);
        // Set non-default values for all saved fields
        p.v = 100;
        p.dot = 77;
        p.field = true;
        p.interlace = true;
        p.obj_interlace = true;
        p.overscan = true;
        p.pseudo_hires = true;
        p.extbg = true;
        p.forced_blank = false;
        p.brightness = 12;
        p.ppu1_mdr = 0xAB;
        p.ppu2_mdr = 0xCD;
        p.latch_h = 500;
        p.latch_v = 99;
        p.counter_latched = true;
        p.ophct_byte = true;
        p.opvct_byte = false;
        p.range_over = true;
        p.time_over = false;
        // Set some line_obj entries
        p.line_obj[0] = Some(Pix { rgb: 0x1234, prio: 1, obj_pal: 5 });
        p.line_obj[1] = None;
        p.line_obj[2] = Some(Pix { rgb: 0x5678, prio: 2, obj_pal: 10 });
        // Set framebuffer pixel
        p.fb.set(10, 20, 0xFF123456);
        // Set view
        p.view = vec![0xFF_AABBCC; 256 * 224];
        p.view_w = 256;
        p.view_h = 224;
        p.line_hires = true;
        p.row_hires[10] = true;
        p.row_hires[20] = true;
        // Set some VRAM/CGRAM/OAM data via registers
        p.write(0x05, 0x0F); // BG mode + tile sizes
        p.write(0x33, 0x1F); // SETINI: interlace, obj_interlace, overscan, pseudo_hires, extbg
        p
    }

    #[test]
    fn e08_08_roundtrip_all_fields() {
        let original = make_ppu_with_state();

        let mut w = StateWriter::new();
        original.save(&mut w);
        let data = w.finish();

        let mut restored = Ppu::new(Region::Ntsc);
        let mut r = StateReader::new(&data);
        restored.load(&mut r).unwrap();

        // Verify all fields match
        assert_eq!(original.v, restored.v);
        assert_eq!(original.dot, restored.dot);
        assert_eq!(original.field, restored.field);
        assert_eq!(original.interlace, restored.interlace);
        assert_eq!(original.obj_interlace, restored.obj_interlace);
        assert_eq!(original.overscan, restored.overscan);
        assert_eq!(original.pseudo_hires, restored.pseudo_hires);
        assert_eq!(original.extbg, restored.extbg);
        assert_eq!(original.forced_blank, restored.forced_blank);
        assert_eq!(original.brightness, restored.brightness);
        assert_eq!(original.ppu1_mdr, restored.ppu1_mdr);
        assert_eq!(original.ppu2_mdr, restored.ppu2_mdr);
        assert_eq!(original.latch_h, restored.latch_h);
        assert_eq!(original.latch_v, restored.latch_v);
        assert_eq!(original.counter_latched, restored.counter_latched);
        assert_eq!(original.ophct_byte, restored.ophct_byte);
        assert_eq!(original.opvct_byte, restored.opvct_byte);
        assert_eq!(original.range_over, restored.range_over);
        assert_eq!(original.time_over, restored.time_over);

        // Verify line_obj entries
        match (original.line_obj[0], restored.line_obj[0]) {
            (Some(o), Some(r)) => {
                assert_eq!(o.rgb, r.rgb);
                assert_eq!(o.prio, r.prio);
                assert_eq!(o.obj_pal, r.obj_pal);
            }
            _ => panic!("line_obj[0] mismatch"),
        }
        assert!(restored.line_obj[1].is_none());
        match (original.line_obj[2], restored.line_obj[2]) {
            (Some(o), Some(r)) => {
                assert_eq!(o.rgb, r.rgb);
                assert_eq!(o.prio, r.prio);
                assert_eq!(o.obj_pal, r.obj_pal);
            }
            _ => panic!("line_obj[2] mismatch"),
        }

        // Verify framebuffer pixel
        assert_eq!(original.fb.get(10, 20), restored.fb.get(10, 20));

        // Verify view dimensions and content
        assert_eq!(original.view_w, restored.view_w);
        assert_eq!(original.view_h, restored.view_h);
        assert_eq!(original.view.len(), restored.view.len());
        for i in 0..restored.view.len() {
            assert_eq!(original.view[i], restored.view[i]);
        }

        // Verify row_hires
        assert_eq!(original.line_hires, restored.line_hires);
        for i in 0..FB_HEIGHT {
            assert_eq!(original.row_hires[i], restored.row_hires[i]);
        }

        // Re-save and compare bytes
        let mut w2 = StateWriter::new();
        restored.save(&mut w2);
        assert_eq!(data, w2.finish());

        // Hash comparison
        let mut h1 = Fnv64::new();
        original.hash_into(&mut h1);
        let mut h2 = Fnv64::new();
        restored.hash_into(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn e08_08_region_mismatch() {
        let original = Ppu::new(Region::Ntsc);
        let mut w = StateWriter::new();
        original.save(&mut w);
        let data = w.finish();

        let mut restored = Ppu::new(Region::Pal);
        let mut r = StateReader::new(&data);
        let result = restored.load(&mut r);
        assert!(matches!(result, Err(StateError::Mismatch("region"))));
    }

    #[test]
    fn e08_08_behaves_identically_midframe() {
        // Create two PPUs with identical initial state
        let mut p1 = Ppu::new(Region::Ntsc);
        let mut p2 = Ppu::new(Region::Ntsc);

        // Write some VRAM/CGRAM data via registers to create a scene
        for i in 0..10 {
            p1.write(0x16, (i * 3) as u8);
            p1.write(0x17, ((i * 3) >> 8) as u8);
            p1.write(0x18, (i * 5 + 1) as u8);
            p1.write(0x19, (i * 7 + 2) as u8);
        }

        // Run both for the same number of dots to reach mid-frame state
        let warmup_dots = 120_000;
        for _ in 0..warmup_dots {
            p1.step_dot();
            p2.step_dot();
        }

        // Save state from p1 at this point
        let mut w = StateWriter::new();
        p1.save(&mut w);
        let data = w.finish();

        // Load into p2 (replacing its state)
        let mut r = StateReader::new(&data);
        p2.load(&mut r).unwrap();

        // Run both for 50,000 more dots
        let run_dots = 50_000;
        for _ in 0..run_dots {
            p1.step_dot();
            p2.step_dot();
        }

        // Compare hashes
        let mut h1 = Fnv64::new();
        p1.hash_into(&mut h1);
        let mut h2 = Fnv64::new();
        p2.hash_into(&mut h2);
        assert_eq!(h1.finish(), h2.finish());

        // Compare frames pixel by pixel
        let f1 = p1.frame();
        let f2 = p2.frame();
        assert_eq!(f1.width, f2.width);
        assert_eq!(f1.height, f2.height);
        for i in 0..f1.pixels.len() {
            assert_eq!(f1.pixels[i], f2.pixels[i]);
        }
    }

    #[test]
    fn e08_08_framebuffer_roundtrip() {
        let mut p = Ppu::new(Region::Ntsc);
        // Set a specific pixel in the framebuffer
        p.fb.set(10, 20, 0xFF123456);

        let mut w = StateWriter::new();
        p.save(&mut w);
        let data = w.finish();

        let mut restored = Ppu::new(Region::Ntsc);
        let mut r = StateReader::new(&data);
        restored.load(&mut r).unwrap();

        // Verify the pixel survived roundtrip
        assert_eq!(restored.fb.get(10, 20), 0xFF123456);
    }

    #[test]
    fn e08_08_truncated() {
        let original = make_ppu_with_state();

        let mut w = StateWriter::new();
        original.save(&mut w);
        let data = w.finish();

        for n in [0, data.len() / 2, data.len() - 1] {
            let mut r = StateReader::new(&data[..n]);
            let mut p = Ppu::new(Region::Ntsc);
            assert!(p.load(&mut r).is_err(), "truncated at {} should fail", n);
        }
    }
}