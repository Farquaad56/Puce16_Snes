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
