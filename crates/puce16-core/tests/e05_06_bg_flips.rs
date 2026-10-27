// E05-06 - ppu/bg.rs: tuiles 16x16 et retournements (flips)
use puce16_core::ppu::{bg::Pix, Ppu};
use puce16_core::region::Region;

fn put_vram(p: &mut Ppu, addr: u16, w: u16) {
    p.write(0x15, 0x80);
    p.write(0x16, (addr & 0xFF) as u8);
    p.write(0x17, (addr >> 8) as u8);
    p.write(0x18, (w & 0xFF) as u8);
    p.write(0x19, (w >> 8) as u8);
}

fn put_cgram(p: &mut Ppu, i: u8, w: u16) {
    p.write(0x21, i);
    p.write(0x22, (w & 0xFF) as u8);
    p.write(0x22, (w >> 8) as u8);
}

fn set_scroll(p: &mut Ppu, reg: u8, v: u16) {
    p.write(reg, (v & 0xFF) as u8);
    p.write(reg, (v >> 8) as u8);
}

// Scene D: mode 1, BG1 16x16, 4bpp
fn scene_d() -> Ppu {
    let mut p = Ppu::new(Region::Ntsc);
    // Mode 1
    p.write(0x05, 0x01);
    // BG1 map at 0x400, 32x32
    p.write(0x07, 0x04);
    // BG1 chr at 0x1000
    p.write(0x0B, 0x01);
    // Enable 16x16 tiles for BG1 (bit 0 of BGMODE)
    p.write(0x05, 0x11);
    p
}

#[test]
fn e05_06_scene_16x16() {
    let mut p = scene_d();
    // Map entry (0,0): tile 2, pal 1
    put_vram(&mut p, 0x400, 0x0402);
    // Tile 2 row 0 pixel 0 = index 1 (plane0 bit7)
    put_vram(&mut p, 0x1020, 0x0080);
    // Tile 3 row 0 pixel 0 = index 2 (plane1 bit7)
    put_vram(&mut p, 0x1030, 0x8000);
    // Tile 18 row 0 pixel 0 = index 3 (planes 0 and 1)
    put_vram(&mut p, 0x1120, 0x8080);
    // CGRAM: palette indices for BG1 4bpp pal 1
    put_cgram(&mut p, 17, 0x0011); // idx 1
    put_cgram(&mut p, 18, 0x0022); // idx 2
    put_cgram(&mut p, 19, 0x0033); // idx 3

    // Top-left sub-tile (tile 2) at pixel (0,0) -> index 1 -> color 0x0011
    assert_eq!(p.bg_pixel(0, 0, 0), Some(Pix { rgb: 0x0011, prio: 0, obj_pal: 0 }));
    // Right sub-tile (tile 3) at pixel (8,0) -> index 2 -> color 0x0022
    assert_eq!(p.bg_pixel(0, 8, 0), Some(Pix { rgb: 0x0022, prio: 0, obj_pal: 0 }));
    // Bottom sub-tile (tile 18) at pixel (0,8) -> index 3 -> color 0x0033
    assert_eq!(p.bg_pixel(0, 0, 8), Some(Pix { rgb: 0x0033, prio: 0, obj_pal: 0 }));
    // Pixel (1,0) in tile 2 is transparent (index 0)
    assert_eq!(p.bg_pixel(0, 1, 0), None);
}

#[test]
fn e05_06_hflip_16x16() {
    let mut p = scene_d();
    // Map entry with H flip: tile 2, pal 1, hflip=1 (bit 14)
    put_vram(&mut p, 0x400, 0x4402);
    // Tile 2 row 0 pixel 0 = index 1
    put_vram(&mut p, 0x1020, 0x0080);
    // Tile 3 row 0 pixel 0 = index 2
    put_vram(&mut p, 0x1030, 0x8000);
    put_cgram(&mut p, 17, 0x0011);
    put_cgram(&mut p, 18, 0x0022);

    // With H flip: xs = ((px>>3)&1) ^ hflip
    // At px=15: xs = (1&1)^1 = 0 -> tile 2; fx = (15&7)^7 = 0 -> pixel 0 of row 0
    assert_eq!(p.bg_pixel(0, 15, 0), Some(Pix { rgb: 0x0011, prio: 0, obj_pal: 0 }));
    // At px=7: xs = (0&1)^1 = 1 -> tile 3; fx = (7&7)^7 = 0 -> pixel 0 of row 0
    assert_eq!(p.bg_pixel(0, 7, 0), Some(Pix { rgb: 0x0022, prio: 0, obj_pal: 0 }));
    // At px=0: xs = (0&1)^1 = 1 -> tile 3; fx = (0&7)^7 = 7 -> pixel 7 of row 0 (transparent)
    assert_eq!(p.bg_pixel(0, 0, 0), None);
}

#[test]
fn e05_06_vflip_16x16() {
    let mut p = scene_d();
    // Map entry with V flip: tile 2, pal 1, vflip=1 (bit 15)
    put_vram(&mut p, 0x400, 0x8402);
    // Tile 2 row 0 pixel 0 = index 1
    put_vram(&mut p, 0x1020, 0x0080);
    // Tile 18 row 0 pixel 0 = index 3
    put_vram(&mut p, 0x1120, 0x8080);
    put_cgram(&mut p, 17, 0x0011);
    put_cgram(&mut p, 19, 0x0033);

    // With V flip: ys = ((py>>3)&1) ^ vflip
    // At py=15: ys = (1&1)^1 = 0 -> tile 2; fy = (15&7)^7 = 0 -> row 0 pixel 0
    assert_eq!(p.bg_pixel(0, 0, 15), Some(Pix { rgb: 0x0011, prio: 0, obj_pal: 0 }));
    // At py=7: ys = (0&1)^1 = 1 -> tile 18; fy = (7&7)^7 = 0 -> row 0 pixel 0
    assert_eq!(p.bg_pixel(0, 0, 7), Some(Pix { rgb: 0x0033, prio: 0, obj_pal: 0 }));
}

// Scene A from E05-05 for 8x8 flip tests
fn scene_a() -> Ppu {
    let mut p = Ppu::new(Region::Ntsc);
    // Mode 1
    p.write(0x05, 0x01);
    // BG1 map at 0x400, 32x32
    p.write(0x07, 0x04);
    // BG1 chr at 0x1000
    p.write(0x0B, 0x01);
    // Tile 2 data: row 0 pixel 0 = index 1 (plane0 bit7)
    put_vram(&mut p, 0x1020, 0x0080);
    // Row 1 pixel 0 = index 2 (plane1 bit7)
    put_vram(&mut p, 0x1021, 0x8000);
    // CGRAM: palette index for BG1 4bpp pal 1 idx 1 = 1*16+1 = 17
    put_cgram(&mut p, 17, 0x1234);
    // Palette index for idx 2 = 18
    put_cgram(&mut p, 18, 0x2345);
    p
}

#[test]
fn e05_06_flips_8x8() {
    let mut p = scene_a();
    // Map entry (0,0): tile 2, pal 1, hflip=1
    put_vram(&mut p, 0x400, 0x4402);

    // H flip: fx = (px&7)^7. At px=7: fx=0 -> pixel 0 of row 0 -> index 1 -> color 0x1234
    assert_eq!(p.bg_pixel(0, 7, 0), Some(Pix { rgb: 0x1234, prio: 0, obj_pal: 0 }));
    // At px=0: fx=7 -> pixel 7 of row 0 (transparent)
    assert_eq!(p.bg_pixel(0, 0, 0), None);

    // Now test V flip
    let mut p2 = scene_a();
    put_vram(&mut p2, 0x400, 0x8402);
    // V flip: fy = (py&7)^7. At py=7: fy=0 -> row 0 pixel 0 -> index 1 -> color 0x1234
    assert_eq!(p2.bg_pixel(0, 0, 7), Some(Pix { rgb: 0x1234, prio: 0, obj_pal: 0 }));
    // At py=0: fy=7 -> row 7 pixel 0 (transparent)
    assert_eq!(p2.bg_pixel(0, 0, 0), None);
}

#[test]
fn e05_06_both_flips_8x8() {
    let mut p = scene_a();
    // Map entry with both H and V flips: tile 2, pal 1, hflip=1, vflip=1 (bits 14+15)
    put_vram(&mut p, 0x400, 0xC402);

    // Both flips: fx = (px&7)^7, fy = (py&7)^7
    // At px=7, py=7: fx=0, fy=0 -> pixel 0 of row 0 -> index 1 -> color 0x1234
    assert_eq!(p.bg_pixel(0, 7, 7), Some(Pix { rgb: 0x1234, prio: 0, obj_pal: 0 }));
    // At px=0, py=0: fx=7, fy=7 -> pixel 7 of row 7 (transparent)
    assert_eq!(p.bg_pixel(0, 0, 0), None);
}

#[test]
fn e05_06_tile_number_wrap_16x16() {
    let mut p = scene_d();
    // Map entry with tile number 0x3F0 (near wrap boundary), palette 1
    put_vram(&mut p, 0x400, (1 << 10) | 0x3F0);
    // Tile 0 data at chr_base + 0*16 = 0x1000
    put_vram(&mut p, 0x1000, 0x0080);
    put_cgram(&mut p, 17, 0x0011);

    // At py=8: ys = (1&1)^0 = 1 -> tile_num = (0x3F0 + 16) & 0x3FF = 0
    // So we read from tile 0 at row 0 pixel 0 -> index 1 -> color 0x0011
    assert_eq!(p.bg_pixel(0, 0, 8), Some(Pix { rgb: 0x0011, prio: 0, obj_pal: 0 }));
}

#[test]
fn e05_06_tile16_per_bg() {
    let mut p = scene_d();
    // BG2 map at 0x800, 32x32 (default 8x8 tiles)
    p.write(0x08, 0x08);
    // Both BG1 and BG2 use chr base 0x1000
    p.write(0x0B, 0x11);

    // BG1 map entry (0,0): tile 2, pal 1 (16x16)
    put_vram(&mut p, 0x400, 0x0402);
    // Tile 3 row 0 pixel 0 = index 2 (for BG1 right sub-tile)
    put_vram(&mut p, 0x1030, 0x8000);

    // BG2 map entry (1,0): tile 2, pal 1 (8x8)
    put_vram(&mut p, 0x801, 0x0402);
    // Tile 2 row 0 pixel 0 = index 1 (for BG2)
    put_vram(&mut p, 0x1020, 0x0080);

    put_cgram(&mut p, 17, 0x0011);
    put_cgram(&mut p, 18, 0x0022);

    // BG2 at pixel (8,0): tx=1 -> entry (1,0) -> tile 2 pixel 0 -> index 1 -> color 0x0011
    assert_eq!(p.bg_pixel(1, 8, 0), Some(Pix { rgb: 0x0011, prio: 0, obj_pal: 0 }));
    // BG1 at pixel (8,0): right sub-tile -> tile 3 pixel 0 -> index 2 -> color 0x0022
    assert_eq!(p.bg_pixel(0, 8, 0), Some(Pix { rgb: 0x0022, prio: 0, obj_pal: 0 }));
}
