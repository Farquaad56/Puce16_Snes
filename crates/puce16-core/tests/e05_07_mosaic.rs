#![allow(dead_code)]
// E05-07 - ppu/bg.rs: mosaique (mosaic)
use puce16_core::ppu::{bg, bg::Pix, Ppu};
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

// Scene A from E05-05: mode 1, BG1 4bpp at map 0x400 chr 0x1000
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
fn e05_07_mosaic_xy_disabled() {
    assert_eq!(bg::mosaic_xy(4, false, 6, 6), (6, 6));
    assert_eq!(bg::mosaic_xy(1, true, 6, 6), (6, 6));
}

#[test]
fn e05_07_mosaic_xy_size4() {
    // x: 0..=3 -> 0, 4..=7 -> 4
    assert_eq!(bg::mosaic_xy(4, true, 0, 1), (0, 1));
    assert_eq!(bg::mosaic_xy(4, true, 3, 4), (0, 1));
    // v: 1..=4 -> 1, 5..=8 -> 5, 9 -> 9
    assert_eq!(bg::mosaic_xy(4, true, 6, 6), (4, 5));
    assert_eq!(bg::mosaic_xy(4, true, 7, 5), (4, 5));
    assert_eq!(bg::mosaic_xy(4, true, 7, 8), (4, 5));
    assert_eq!(bg::mosaic_xy(4, true, 8, 9), (8, 9));
}

#[test]
fn e05_07_mosaic_xy_size16() {
    // xm = 255 - (255 % 16) = 255 - 15 = 240
    // vm = 224 - ((224-1) % 16) = 224 - (223 % 16) = 224 - 15 = 209
    assert_eq!(bg::mosaic_xy(16, true, 255, 224), (240, 209));

    // size 2: lines 1-2 -> vm=1, lines 3-4 -> vm=3
    assert_eq!(bg::mosaic_xy(2, true, 1, 1), (0, 1));
    assert_eq!(bg::mosaic_xy(2, true, 1, 2), (0, 1));
    assert_eq!(bg::mosaic_xy(2, true, 1, 3), (0, 3));
}

#[test]
fn e05_07_mosaic_xy_v0() {
    // v=0: saturating_sub(1) = 0, so vm = 0 - (0 % size) = 0
    assert_eq!(bg::mosaic_xy(4, true, 0, 0), (0, 0));
}

#[test]
fn e05_07_bg_pixel_mosaic_x() {
    let mut p = scene_a();
    // Map entry (0,0): tile 2, pal 1
    put_vram(&mut p, 0x400, 0x0402);

    // Without mosaic: bg_pixel(0,1,0) -> px=1 -> pixel 1 of row 0 (transparent)
    assert_eq!(p.bg_pixel(0, 1, 0), None);

    // Enable mosaic size 2 for BG1 (bit 0 of MOSAIC_EN)
    p.write(0x06, 0x11); // size=2, enable BG1

    // With mosaic: xm = 1 - (1 % 2) = 0 -> px=0 -> pixel 0 of row 0 -> index 1 -> color 0x1234
    assert_eq!(p.bg_pixel(0, 1, 0), Some(Pix { rgb: 0x1234, prio: 0, obj_pal: 0 }));
}

#[test]
fn e05_07_bg_pixel_mosaic_v() {
    let mut p = scene_a();
    // Map entry (0,0): tile 2, pal 1
    put_vram(&mut p, 0x400, 0x0402);

    // Without mosaic: bg_pixel(0,0,2) -> py=2 -> row 2 pixel 0 (transparent)
    assert_eq!(p.bg_pixel(0, 0, 2), None);

    // Enable mosaic size 2 for BG1
    p.write(0x06, 0x11); // size=2, enable BG1

    // With mosaic: vm = 2 - ((2-1) % 2) = 2 - 1 = 1 -> py=1 -> row 1 pixel 0 -> index 2 -> color 0x2345
    assert_eq!(p.bg_pixel(0, 0, 2), Some(Pix { rgb: 0x2345, prio: 0, obj_pal: 0 }));

    // Also test x mosaic with v=1
    // xm = 0 - (0 % 2) = 0, vm = 1 - ((1-1) % 2) = 1 -> row 1 pixel 0 -> index 2 -> color 0x2345
    assert_eq!(p.bg_pixel(0, 1, 1), Some(Pix { rgb: 0x2345, prio: 0, obj_pal: 0 }));
}

#[test]
fn e05_07_mosaic_per_bg() {
    let mut p = scene_a();
    // Map entry (0,0): tile 2, pal 1
    put_vram(&mut p, 0x400, 0x0402);

    // Enable mosaic size 2 for BG2 only (bit 1 of MOSAIC_EN)
    p.write(0x06, 0x12); // size=2, enable BG2

    // BG1 not affected by mosaic: bg_pixel(0,1,0) -> px=1 -> pixel 1 of row 0 (transparent)
    assert_eq!(p.bg_pixel(0, 1, 0), None);
}

#[test]
fn e05_07_mosaic_size1_noop() {
    let mut p = scene_a();
    // Map entry (0,0): tile 2, pal 1
    put_vram(&mut p, 0x400, 0x0402);

    // Enable mosaic size 1 for BG1 (no-op)
    p.write(0x06, 0x01); // size=1, enable BG1

    // Same as no mosaic: bg_pixel(0,1,0) -> px=1 -> pixel 1 of row 0 (transparent)
    assert_eq!(p.bg_pixel(0, 1, 0), None);
}
