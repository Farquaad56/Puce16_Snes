//! Sprite (object) handling: dimensions, decoding, and per-row sprite list evaluation.
//! Implements spec 05b_PPU_RENDER.md sections 6.1-6.2.

use super::Ppu;
use super::bg::Pix;

/// Return sprite dimensions in pixels for the given size code and large flag.
/// obj_size values > 7 are treated as 7 (largest).
pub fn obj_dims(obj_size: u8, large: bool) -> (u8, u8) {
    let s = if obj_size > 7 { 7 } else { obj_size };
    match s {
        0 => if large { (16, 16) } else { (8, 8) },
        1 => if large { (32, 32) } else { (8, 8) },
        2 => if large { (64, 64) } else { (8, 8) },
        3 => if large { (32, 32) } else { (16, 16) },
        4 => if large { (64, 64) } else { (16, 16) },
        5 => if large { (64, 64) } else { (32, 32) },
        6 => if large { (32, 64) } else { (16, 32) },
        7 => if large { (32, 32) } else { (16, 32) },
        _ => unreachable!(),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Sprite {
    pub x: i16,
    pub y: u8,
    pub tile: u8,
    pub tbit: bool,
    pub pal: u8,
    pub prio: u8,
    pub hflip: bool,
    pub vflip: bool,
    pub large: bool,
}

impl Ppu {
    /// Decode sprite i (0..127) from OAM.
    /// Low table bytes at i*4+0..3: X(7-0), Y, tile(7-0), attr(VHPP CCCt).
    /// High table byte 512+i/4 bits (i%4)*2 = X bit 8, (i%4)*2+1 = large.
    pub fn sprite(&self, i: usize) -> Sprite {
        let oam = self.oam();
        let base = i * 4;
        let x_lo = oam[base] as u16;
        let y = oam[base + 1];
        let tile = oam[base + 2];
        let attr = oam[base + 3];

        // High table: byte at 512 + i/4, bits (i%4)*2 and (i%4)*2+1
        let high_byte_idx = 512 + i / 4;
        let bit_shift = (i % 4) * 2;
        let high_byte = oam[high_byte_idx];
        let x8 = ((high_byte >> bit_shift) & 1) as u16;
        let large_bit = ((high_byte >> (bit_shift + 1)) & 1);

        // Sign-extend X from 9 bits: -256..255
        let x9 = x_lo | (x8 << 8);
        let x = if x9 >= 256 { x9 as i16 - 512 } else { x9 as i16 };

        Sprite {
            x,
            y,
            tile,
            tbit: attr & 1 != 0,
            pal: (attr >> 1) & 7,
            prio: (attr >> 4) & 3,
            hflip: attr & 0x40 != 0,
            vflip: attr & 0x80 != 0,
            large: large_bit == 1,
        }
    }

    /// Evaluate which sprites are on output row r (0 = first visible line).
    /// Returns (list of sprite indices in evaluation order, overflow flag).
    /// Implements spec section 6.2 with the hardware bug where x=-256 counts as on-screen.
    pub fn sprite_list(&self, r: u16) -> (Vec<u8>, bool) {
        let oam = self.oam_unit();
        let first = if oam.rotation() {
            ((oam.internal() >> 2) & 0x7F) as usize
        } else {
            0
        };

        let mut list = Vec::new();
        let mut overflow = false;

        for k in 0..128 {
            let i = (first + k) & 0x7F;
            let s = self.sprite(i);
            let (w, h) = obj_dims(self.regs.obj_size, s.large);

            // Sprite appears one row below Y: Y=255 -> first visible row
            let row = ((r as i32) - (s.y as i32) - 1) & 0xFF;
            if row >= h as i32 {
                continue;
            }

            // Hardware bug: x=-256 counts as on-screen regardless of width
            let x_on_screen = s.x == -256 || (s.x as i32) > -(w as i32);
            if !x_on_screen {
                continue;
            }

            if list.len() == 32 {
                overflow = true;
                break;
            }

            list.push(i as u8);
        }

        (list, overflow)
    }

    /// Render sprite pixels for output row r into line_obj buffer.
    /// Implements spec section 6.3: sliver fetching with 34-slice limit.
    pub fn render_obj_line(&mut self, r: u16) {
        // Clear line buffer
        for i in 0..256 {
            self.line_obj[i] = None;
        }

        let (list, overflow) = self.sprite_list(r);
        if overflow {
            self.range_over = true;
        }

        let mut slivers = 0;

        // Process sprites in reverse order (highest index first) so lowest wins
        for &i in list.iter().rev() {
            let s = self.sprite(i as usize);
            let (w, h) = obj_dims(self.regs.obj_size, s.large);
            let row = ((r as i32) - (s.y as i32) - 1) & 0xFF;

            // Process columns left to right
            for c in 0..(w / 8) {
                let sx = s.x as i32 + c as i32 * 8;

                // Check if this sliver counts toward the limit
                let counts = (sx > -8 && sx < 256) || s.x == -256;
                if !counts {
                    continue;
                }

                // Check 34-slice limit
                if slivers == 34 {
                    self.time_over = true;
                    return;
                }
                slivers += 1;

                // Fetch and render this sliver
                self.fetch_sliver(&s, c as u8, row as u8, sx);
            }
        }
    }

    /// Return sprite pixel at screen x position (None if transparent or out of range).
    pub fn obj_pixel(&self, x: usize) -> Option<Pix> {
        if x >= 256 {
            return None;
        }
        self.line_obj[x]
    }

    /// Fetch one 8-pixel sliver from VRAM and write non-transparent pixels to line_obj.
    fn fetch_sliver(&mut self, s: &Sprite, c: u8, row: u8, sx: i32) {
        let (w, h) = obj_dims(self.regs.obj_size, s.large);

        // Compute tile column with horizontal flip
        let tc = if s.hflip { w / 8 - 1 - c } else { c };

        // Compute row within sprite with vertical flip
        let rv: u8;
        if s.vflip {
            if w == h {
                // Square sprites: full flip
                rv = h - 1 - row;
            } else {
                // Non-square (16x32, 32x64): half-by-half flip (hardware erratum)
                let half = h / 2;
                let half_row = row % half;
                let which_half = row / half;
                rv = which_half * half + (half - 1 - half_row);
            }
        } else {
            rv = row;
        }

        // Compute tile address within name table
        let tile_row = rv >> 3;
        let fine_y = rv & 7;

        // Tile index: low 4 bits of sprite tile + column offset, wrapped to 16 tiles
        let tx = ((s.tile & 0x0F) as u32 + tc as u32) & 0x0F;
        let ty = ((s.tile >> 4) as u32 + tile_row as u32) & 0x0F;
        let tile8 = ty * 16 + tx;

        // VRAM address: obj_base + (tbit ? obj_name_gap : 0) + tile8*16 + fine_y
        let base = self.regs.obj_base as u32;
        let gap = if s.tbit { self.regs.obj_name_gap as u32 } else { 0 };
        let addr = (base + gap + tile8 * 16 + fine_y as u32) & 0x7FFF;

        // Read two bytes from VRAM (low and high planes)
        let vram_mem = self.vram.mem();
        let lo = vram_mem[addr as usize];
        let hi = vram_mem[(addr + 8) as usize];

        // Decode 4-bit color indices for each pixel in the sliver
        for px in 0..8 {
            let bit = if s.hflip { px } else { 7 - px };
            let i = ((lo >> bit) & 1)
                | (((lo >> (8 + bit)) & 1) << 1)
                | (((hi >> bit) & 1) << 2)
                | (((hi >> (8 + bit)) & 1) << 3);

            let dest = sx + px as i32;
            if i != 0 && dest >= 0 && dest < 256 {
                // CGRAM index: 128 + palette*16 + color_index
                let cgram_idx = 128u8 + s.pal * 16 + (i as u8);
                let cgram_mem = self.cgram.mem();
                let rgb = cgram_mem[cgram_idx as usize];
                self.line_obj[dest as usize] = Some(Pix {
                    rgb,
                    prio: s.prio,
                    obj_pal: s.pal,
                });
            }
        }
    }
}