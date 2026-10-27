// PPU background rendering: types, BPP table, priority order tables.
// Layer numbers: 0..=3 = BG1..BG4, 4 = OBJ, 5 = BACKDROP.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pix {
    pub rgb: u16,
    pub prio: u8,
    pub obj_pal: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pick {
    pub rgb: u16,
    pub layer: u8,
    pub obj_pal: u8,
}

pub const LAYER_OBJ: u8 = 4;
pub const LAYER_BACKDROP: u8 = 5;

// BPP[mode][bg] - bits per pixel for each background layer in each mode.
// bg index: 0=BG1, 1=BG2, 2=BG3, 3=BG4. Value 0 means layer not rendered.
const BPP_TABLE: [[u8; 4]; 8] = [
    // mode 0
    [2, 2, 2, 2],
    // mode 1
    [4, 4, 2, 0],
    // mode 2
    [4, 4, 0, 0],
    // mode 3
    [8, 4, 0, 0],
    // mode 4
    [8, 2, 0, 0],
    // mode 5
    [4, 2, 0, 0],
    // mode 6
    [4, 0, 0, 0],
    // mode 7
    [8, 0, 0, 0],
];

pub fn bpp(mode: u8, bg: usize) -> u8 {
    if (mode as usize) >= BPP_TABLE.len() || bg >= 4 {
        return 0;
    }
    BPP_TABLE[mode as usize][bg]
}

// Priority order tables: list of (layer, level) from front to back.
// layer: 0..=3 = BG1..BG4, 4 = OBJ
// level for OBJ: sprite priority 0..3
// level for BG: tile priority bit (0 or 1), except mode 7 where it's always 0

const PRIORITY_MODE0: [(u8, u8); 12] = [
    (4, 3), // S3
    (0, 1), // 1H
    (1, 1), // 2H
    (4, 2), // S2
    (0, 0), // 1L
    (1, 0), // 2L
    (4, 1), // S1
    (2, 1), // 3H
    (3, 1), // 4H
    (4, 0), // S0
    (2, 0), // 3L
    (3, 0), // 4L
];

const PRIORITY_MODE1_NO_BG3: [(u8, u8); 10] = [
    (4, 3), // S3
    (0, 1), // 1H
    (1, 1), // 2H
    (4, 2), // S2
    (0, 0), // 1L
    (1, 0), // 2L
    (4, 1), // S1
    (2, 1), // 3H
    (4, 0), // S0
    (2, 0), // 3L
];

const PRIORITY_MODE1_BG3: [(u8, u8); 10] = [
    (2, 1), // 3H
    (4, 3), // S3
    (0, 1), // 1H
    (1, 1), // 2H
    (4, 2), // S2
    (0, 0), // 1L
    (1, 0), // 2L
    (4, 1), // S1
    (4, 0), // S0
    (2, 0), // 3L
];

const PRIORITY_MODES_2_TO_5: [(u8, u8); 8] = [
    (4, 3), // S3
    (0, 1), // 1H
    (4, 2), // S2
    (1, 1), // 2H
    (4, 1), // S1
    (0, 0), // 1L
    (4, 0), // S0
    (1, 0), // 2L
];

const PRIORITY_MODE6: [(u8, u8); 6] = [
    (4, 3), // S3
    (0, 1), // 1H
    (4, 2), // S2
    (4, 1), // S1
    (0, 0), // 1L
    (4, 0), // S0
];

const PRIORITY_MODE7_NO_EXTBG: [(u8, u8); 5] = [
    (4, 3), // S3
    (4, 2), // S2
    (4, 1), // S1
    (0, 0), // 1
    (4, 0), // S0
];

const PRIORITY_MODE7_EXTBG: [(u8, u8); 7] = [
    (4, 3), // S3
    (4, 2), // S2
    (1, 1), // 2H
    (4, 1), // S1
    (0, 0), // 1
    (4, 0), // S0
    (1, 0), // 2L
];

pub fn priority_order(mode: u8, bg3_priority: bool, extbg: bool) -> &'static [(u8, u8)] {
    match mode {
        0 => &PRIORITY_MODE0,
        1 if !bg3_priority => &PRIORITY_MODE1_NO_BG3,
        1 => &PRIORITY_MODE1_BG3,
        2 | 3 | 4 | 5 => &PRIORITY_MODES_2_TO_5,
        6 => &PRIORITY_MODE6,
        7 if !extbg => &PRIORITY_MODE7_NO_EXTBG,
        7 => &PRIORITY_MODE7_EXTBG,
        _ => &[], // invalid mode
    }
}

// Palette index calculation for background pixels.
// Returns CGRAM index (0..=255) for a BG pixel with color index i != 0.
pub fn bg_palette_index(mode: u8, bg: usize, bpp: u8, pal: u8, i: u8) -> u8 {
    let result = match bpp {
        2 => if mode == 0 {
            (bg as u32) * 32 + (pal as u32) * 4 + (i as u32)
        } else {
            (pal as u32) * 4 + (i as u32)
        },
        4 => (pal as u32) * 16 + (i as u32),
        8 => i as u32, // palette ignored for 8bpp indexed
        _ => 0,
    };
    result as u8
}

// Palette index calculation for object pixels.
// Returns CGRAM index: 128 + pal*16 + i
pub fn obj_palette_index(pal: u8, i: u8) -> u8 {
    let result = 128 + (pal as u32) * 16 + (i as u32);
    result as u8
}

// Direct color for BG1 in modes 3/4 (8bpp direct).
// Returns BGR555 value from color index i and palette byte pal.
pub fn direct_color_bg(i: u8, pal: u8) -> u16 {
    let r5 = ((i & 7) << 2) | ((pal & 1) << 1);
    let g5 = (((i >> 3) & 7) << 2) | (pal & 2);
    let b5 = (((i >> 6) & 3) << 3) | (pal & 4);
    r5 as u16 | ((g5 as u16) << 5) | ((b5 as u16) << 10)
}

// Direct color for mode 7 BG1.
// Returns BGR555 value from color index i.
pub fn direct_color_m7(i: u8) -> u16 {
    let r5 = (i & 7) << 2;
    let g5 = ((i >> 3) & 7) << 2;
    let b5 = (i >> 6) << 3;
    r5 as u16 | ((g5 as u16) << 5) | ((b5 as u16) << 10)
}

/// Pick the visible layer at a pixel based on priority order.
/// Returns the first non-transparent, enabled, non-hidden layer with matching priority level.
pub fn pick_layers(order: &[(u8, u8)], layers: &[Option<Pix>; 5], en: u8, hidden: [bool; 5], backdrop: u16) -> Pick {
    for &(layer, level) in order {
        if (en >> layer) & 1 == 0 {
            continue; // layer not enabled
        }
        if hidden[layer as usize] {
            continue; // layer hidden by window
        }
        match layers[layer as usize] {
            Some(p) if p.prio == level => return Pick { rgb: p.rgb, layer, obj_pal: p.obj_pal },
            _ => {}
        }
    }
    Pick { rgb: backdrop, layer: LAYER_BACKDROP, obj_pal: 0 }
}

// Tilemap entry structure decoded from a 16-bit word.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TileEntry {
    pub tile: u16,
    pub pal: u8,
    pub prio: u8,
    pub hflip: bool,
    pub vflip: bool,
}

// Decode a 16-bit tilemap entry.
// Format: VHPC CCTT TTTT TTTT (bit15=V, bit14=H, bit13=P, bits12-10=pal, bits9-0=tile)
pub fn decode_entry(w: u16) -> TileEntry {
    let vflip = (w & 0x8000) != 0;
    let hflip = (w & 0x4000) != 0;
    let prio = ((w >> 13) & 1) as u8;
    let pal = ((w >> 10) & 7) as u8;
    let tile = w & 0x3FF;
    TileEntry { tile, pal, prio, hflip, vflip }
}

// Compute the VRAM word address of a tilemap entry at tile coordinates (tx, ty).
// map_base: base address of the name table.
// x2/y2: true if 64-tile wide/tall (two screens), false if 32-tile.
pub fn map_entry_addr(map_base: u16, x2: bool, y2: bool, tx: u16, ty: u16) -> u16 {
    let tx = tx & if x2 { 63 } else { 31 };
    let ty = ty & if y2 { 63 } else { 31 };
    let scr = ((tx >> 5) & (x2 as u16)) + ((ty >> 5) & (y2 as u16)) * (if x2 { 2 } else { 1 });
    let addr = map_base as u32 + (scr as u32) * 0x400 + ((ty & 31) as u32) * 32 + ((tx & 31) as u32);
    (addr & 0x7FFF) as u16
}

// Decode a row of 8 pixels from tile data at the given VRAM word address.
// Reads plane bits according to bpp (2, 4, or 8). Returns color indices.
pub fn decode_row(vram: &[u16], row_word_addr: u16, bpp: u8) -> [u8; 8] {
    let a = row_word_addr & 0x7FFF;
    let m0 = vram[a as usize];
    let plane0 = (m0 & 0xFF) as u8;
    let plane1 = ((m0 >> 8) & 0xFF) as u8;

    let (plane2, plane3, plane4, plane5, plane6, plane7) = match bpp {
        2 => (0, 0, 0, 0, 0, 0),
        4 => {
            let m1 = vram[((a + 8) & 0x7FFF) as usize];
            ((m1 & 0xFF) as u8, ((m1 >> 8) & 0xFF) as u8, 0, 0, 0, 0)
        }
        8 => {
            let m1 = vram[((a + 8) & 0x7FFF) as usize];
            let m2 = vram[((a + 16) & 0x7FFF) as usize];
            let m3 = vram[((a + 24) & 0x7FFF) as usize];
            (
                (m1 & 0xFF) as u8,
                ((m1 >> 8) & 0xFF) as u8,
                (m2 & 0xFF) as u8,
                ((m2 >> 8) & 0xFF) as u8,
                (m3 & 0xFF) as u8,
                ((m3 >> 8) & 0xFF) as u8,
            )
        }
        _ => panic!("decode_row: invalid bpp {}", bpp),
    };

    let mut row = [0u8; 8];
    for px in 0..8 {
        let bit = 7 - px;
        let p0 = (plane0 >> bit) & 1;
        let p1 = (plane1 >> bit) & 1;
        let p2 = (plane2 >> bit) & 1;
        let p3 = (plane3 >> bit) & 1;
        let p4 = (plane4 >> bit) & 1;
        let p5 = (plane5 >> bit) & 1;
        let p6 = (plane6 >> bit) & 1;
        let p7 = (plane7 >> bit) & 1;
        row[px] = p0 | (p1 << 1) | (p2 << 2) | (p3 << 3) | (p4 << 4) | (p5 << 5) | (p6 << 6) | (p7 << 7);
    }
    row
}

// Mosaic: compute mosaic-adjusted screen coordinates.
// size = regs.mosaic_size() (1..=16). If enabled and size > 1, snap to block top-left.
pub fn mosaic_xy(size: u8, enabled: bool, x: usize, v: usize) -> (usize, usize) {
    if !enabled || size <= 1 {
        return (x, v);
    }
    let s = size as usize;
    let xm = x - (x % s);
    let vm = v - (v.saturating_sub(1) % s);
    (xm, vm)
}

// Offset-per-tile: compute effective scroll offsets for modes 2 and 4.
// BG3 provides per-column scroll overrides for BG1/BG2.
impl super::Ppu {
    fn opt_offsets(&self, n: usize, xm: usize) -> (u16, u16) {
        let mode = self.regs.bg_mode;
        if !(mode == 2 || mode == 4) || n > 1 {
            return (self.scroll.hofs(n), self.scroll.vofs(n));
        }

        let hofs_n = self.scroll.hofs(n);
        let vofs_n = self.scroll.vofs(n);
        let mut hofs_use = hofs_n;
        let mut vofs_use = vofs_n;

        // Column of 8px blocks; column 0 is the partial left tile, never modified
        let n_col = (xm + (hofs_n & 7) as usize) >> 3;
        if n_col < 1 {
            return (hofs_use, vofs_use);
        }

        // Coordinates in BG3 map (8x8 tiles)
        let ex = ((n_col - 1) + (self.scroll.hofs(2) >> 3) as usize) as u16;
        let ey = (self.scroll.vofs(2) >> 3) as u16;

        // Read BG3 map entry at (ex, ey)
        let addr_eh = map_entry_addr(self.regs.map_base[2], self.regs.map_x2[2], self.regs.map_y2[2], ex, ey);
        let eh = self.vram.word(addr_eh);

        // Mode 4: bit15 selects horizontal vs vertical; mode 2 uses two entries
        let ev = if mode == 4 {
            if (eh & 0x8000) != 0 { eh } else { 0 }
        } else {
            let addr_ev = map_entry_addr(self.regs.map_base[2], self.regs.map_x2[2], self.regs.map_y2[2], ex, ey + 1);
            self.vram.word(addr_ev)
        };

        // Enable bits: bit13 for BG1 (n=0), bit14 for BG2 (n=1)
        let bit_enable = if n == 0 { 0x2000 } else { 0x4000 };

        if (eh & bit_enable) != 0 {
            hofs_use = (eh & 0x3F8) | (hofs_n & 7);
        }
        if (ev & bit_enable) != 0 {
            vofs_use = ev & 0x3FF;
        }

        (hofs_use, vofs_use)
    }

    pub fn bg_pixel(&self, n: usize, x: usize, v: usize) -> Option<Pix> {
        let mode = self.regs.bg_mode;

        // Step 1: Check bpp and mode
        let b = bpp(mode, n);
        if b == 0 || mode == 7 {
            return None;
        }

        // Step 2: Apply mosaic to screen coordinates (before scroll offset)
        let (xm, vm) = mosaic_xy(
            self.regs.mosaic_size(),
            (self.regs.mosaic_en >> n) & 1 != 0,
            x, v,
        );

        // Step 3: Compute scrolled coordinates with offset-per-tile override
        let (hofs_use, vofs_use) = self.opt_offsets(n, xm);
        let px = (xm + hofs_use as usize) & 0x3FF;
        let py = (vm + vofs_use as usize) & 0x3FF;

        // Step 3: Get tilemap entry and compute tile number with flips
        let is_16x16 = self.regs.tile16[n];
        let tx = if is_16x16 { px >> 4 } else { px >> 3 };
        let ty = if is_16x16 { py >> 4 } else { py >> 3 };
        let addr = map_entry_addr(
            self.regs.map_base[n],
            self.regs.map_x2[n],
            self.regs.map_y2[n],
            tx as u16,
            ty as u16,
        );
        let entry_word = self.vram.word(addr);
        let entry = decode_entry(entry_word);

        // Compute actual tile number (for 16x16, add sub-tile offset)
        let mut tile_num = entry.tile;
        if is_16x16 {
            let xs = (((px >> 3) & 1) as u16) ^ (entry.hflip as u16);
            let ys = (((py >> 3) & 1) as u16) ^ (entry.vflip as u16);
            tile_num = (tile_num + xs + ys * 16) & 0x3FF;
        }

        // Step 4: Decode pixel index from tile data with flips
        let fx = (px & 7) ^ (if entry.hflip { 7 } else { 0 });
        let fy = (py & 7) ^ (if entry.vflip { 7 } else { 0 });
        let words = match b {
            2 => 8,
            4 => 16,
            8 => 32,
            _ => panic!("bg_pixel: invalid bpp"),
        };
        let tile_word = (self.regs.chr_base[n] + tile_num * words) & 0x7FFF;
        let row = (tile_word + fy as u16) & 0x7FFF;
        let row_data = decode_row(self.vram.mem(), row, b);
        let i = row_data[fx];
        if i == 0 {
            return None;
        }

        // Step 5: Get color (direct color or CGRAM lookup)
        let rgb = if self.regs.direct_color && n == 0 && b == 8 && (mode == 3 || mode == 4) {
            direct_color_bg(i, entry.pal)
        } else {
            let pal_idx = bg_palette_index(mode, n, b, entry.pal, i);
            self.cgram.word(pal_idx)
        };

        // Step 6: Return pixel with priority
        Some(Pix { rgb, prio: entry.prio, obj_pal: 0 })
    }

    /// Sample all layers at screen position (x, v). Returns [Option<Pix>; 5].
    pub fn sample_layers(&self, x: usize, v: usize) -> [Option<Pix>; 5] {
        let mut layers = [None; 5];
        let mask = self.regs.tm | self.regs.ts;

        if self.regs.bg_mode == 7 {
            // Mode 7: BG1 via m7_pixel, EXTBG BG2 if enabled
            if (mask >> 0) & 1 != 0 {
                layers[0] = self.m7_pixel(0, x, v);
            }
            if (mask >> 1) & 1 != 0 && self.extbg {
                layers[1] = self.m7_pixel(1, x, v);
            }
        } else {
            // Modes 0-6: standard bg_pixel for each BG layer
            for n in 0..4 {
                if (mask >> n) & 1 != 0 {
                    layers[n] = self.bg_pixel(n, x, v);
                }
            }
        }

        // OBJ is always sampled regardless of mode
        if (mask >> 4) & 1 != 0 {
            layers[4] = self.obj_pixel(x);
        }

        layers
    }

    /// Pick the visible pixel for a screen at position (x, v).
    pub fn pick_screen(&self, main: bool, x: usize, v: usize) -> Pick {
        let layers = self.sample_layers(x, v);
        let order = priority_order(self.regs.bg_mode, self.regs.bg3_priority, self.extbg);
        let en = if main { self.regs.tm } else { self.regs.ts };

        // Compute hidden flags for each layer based on windows
        let mut hidden = [false; 5];
        for l in 0..5 {
            hidden[l] = self.layer_hidden(main, l as u8, x);
        }

        // Backdrop color: main uses CGRAM[0], sub uses fixed color
        let backdrop = if main {
            self.cgram.word(0)
        } else {
            let r5 = (self.regs.fixed_r & 7) << 2;
            let g5 = (self.regs.fixed_g & 7) << 2;
            let b5 = (self.regs.fixed_b & 3) << 3;
            r5 as u16 | ((g5 as u16) << 5) | ((b5 as u16) << 10)
        };

        pick_layers(order, &layers, en, hidden, backdrop)
    }
}
