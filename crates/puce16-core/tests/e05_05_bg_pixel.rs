// E05-05 - bg_pixel (tuiles 8x8, defilement, palette)
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

// Scene A: mode 1, BG1 4bpp, tile at map entry (0,0) with pixel data
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
    // Map entry (0,0): tile 2, pal 1, prio 0
    put_vram(&mut p, 0x400, 0x0402);
    // Map entry (1,0): tile 2, pal 1, prio 1
    put_vram(&mut p, 0x401, 0x2402);
    // CGRAM: palette index for BG1 4bpp pal 1 idx 1 = 1*16+1 = 17
    put_cgram(&mut p, 17, 0x1234);
    // Palette index for idx 2 = 18
    put_cgram(&mut p, 18, 0x2345);
    p
}

#[test]
fn e05_05_scene_mode1_basic() {
    let mut p = scene_a();
    // Pixel at (0,0) should be index 1 -> color 0x1234, prio 0
    assert_eq!(p.bg_pixel(0, 0, 0), Some(Pix { rgb: 0x1234, prio: 0, obj_pal: 0 }));
    // Pixel at (1,0) should be transparent (index 0)
    assert_eq!(p.bg_pixel(0, 1, 0), None);
    // Pixel at (0,1) should be index 2 -> color 0x2345, prio 0
    assert_eq!(p.bg_pixel(0, 0, 1), Some(Pix { rgb: 0x2345, prio: 0, obj_pal: 0 }));
    // Pixel at (8,0) should be from map entry (1,0) -> prio 1
    assert_eq!(p.bg_pixel(0, 8, 0), Some(Pix { rgb: 0x1234, prio: 1, obj_pal: 0 }));
}

#[test]
fn e05_05_vofs_minus_one() {
    let mut p = scene_a();
    // Set VOFS BG1 to -1 (0x3FF) so py=0 maps to first visible line
    set_scroll(&mut p, 0x0E, 0x3FF);
    // v=1 -> py=(1+1023)&1023=0 -> index 1
    assert_eq!(p.bg_pixel(0, 0, 1), Some(Pix { rgb: 0x1234, prio: 0, obj_pal: 0 }));
    // v=2 -> py=(2+1023)&1023=1 -> index 2
    assert_eq!(p.bg_pixel(0, 0, 2), Some(Pix { rgb: 0x2345, prio: 0, obj_pal: 0 }));
}

#[test]
fn e05_05_hofs_scroll() {
    let mut p = scene_a();
    // Set HOFS BG1 to 8 so px=0 maps to map entry (1,0)
    set_scroll(&mut p, 0x0D, 8);
    // x=0 -> px=(0+8)&1023=8 -> tx=1 -> entry (1,0) prio 1
    assert_eq!(p.bg_pixel(0, 0, 0), Some(Pix { rgb: 0x1234, prio: 1, obj_pal: 0 }));
    // x=8 -> px=(8+8)&1023=16 -> tx=2 -> entry (2,0) empty -> None
    assert_eq!(p.bg_pixel(0, 8, 0), None);
}

#[test]
fn e05_05_map_64_wide() {
    let mut p = scene_a();
    // Set map to 64 wide (bit 1 of BGnMAP)
    p.write(0x07, 0x05);
    // Put tile at screen 1 entry (32,0) which is at address 0x800
    put_vram(&mut p, 0x800, 0x0402);
    // Scroll to show screen 1: HOFS = 256 -> tx=32
    set_scroll(&mut p, 0x0D, 256);
    // x=0 -> px=(0+256)&1023=256 -> tx=32 -> entry at screen 1
    assert_eq!(p.bg_pixel(0, 0, 0), Some(Pix { rgb: 0x1234, prio: 0, obj_pal: 0 }));
}

#[test]
fn e05_05_unused_layers() {
    let mut p = scene_a();
    // BG4 in mode 1 has bpp=0 -> None
    assert_eq!(p.bg_pixel(3, 0, 0), None);
    // BG3 in mode 1 has bpp=2 but no tile data -> index 0 -> None
    assert_eq!(p.bg_pixel(2, 0, 0), None);
    // Mode 7: bg_pixel returns None for all layers
    p.write(0x05, 0x07);
    assert_eq!(p.bg_pixel(0, 0, 0), None);
}

// Scene B: mode 0, BG3 2bpp
fn scene_b() -> Ppu {
    let mut p = Ppu::new(Region::Ntsc);
    // Mode 0
    p.write(0x05, 0x00);
    // BG3 map at 0x800
    p.write(0x09, 0x08);
    // BG3 chr at 0x2000
    p.write(0x0C, 0x02);
    // Map entry (0,0): tile 1, pal 2
    put_vram(&mut p, 0x800, 0x0801);
    // Tile 1 row 0 pixel 0 = index 1 (plane0 bit7)
    put_vram(&mut p, 0x2008, 0x0080);
    // CGRAM: mode 0 BG3 pal 2 idx 1 -> bg_palette_index(0, 2, 2, 2, 1) = 2*32 + 2*4 + 1 = 73
    put_cgram(&mut p, 73, 0x7FFF);
    p
}

#[test]
fn e05_05_mode0_bg3_palette() {
    let mut p = scene_b();
    // BG3 pixel at (0,0) should be index 1 -> color from CGRAM[73]
    assert_eq!(p.bg_pixel(2, 0, 0), Some(Pix { rgb: 0x7FFF, prio: 0, obj_pal: 0 }));
}

// Scene C: mode 3, BG1 8bpp
fn scene_c() -> Ppu {
    let mut p = Ppu::new(Region::Ntsc);
    // Mode 3
    p.write(0x05, 0x03);
    // BG1 map at 0x400
    p.write(0x07, 0x04);
    // BG1 chr at 0x0000
    p.write(0x0B, 0x00);
    // Map entry (0,0): tile 1, pal 3
    put_vram(&mut p, 0x400, 0x0C01);
    // Tile 1 at chr_base + 1*32 = 32
    // Row 0: planes 0 and 1 set bit7 -> index bits 0 and 1
    put_vram(&mut p, 32, 0x8080);
    // Plane 2 (row at offset 8)
    put_vram(&mut p, 40, 0x0080);
    // Plane 6 (row at offset 24)
    put_vram(&mut p, 56, 0x0080);
    // Index = 1 + 2 + 4 + 64 = 71
    // CGRAM[71] for indexed color
    put_cgram(&mut p, 71, 0x4321);
    p
}

#[test]
fn e05_05_mode3_bg1_8bpp() {
    let mut p = scene_c();
    // BG1 pixel at (0,0) should be index 71 -> color from CGRAM[71]
    assert_eq!(p.bg_pixel(0, 0, 0), Some(Pix { rgb: 0x4321, prio: 0, obj_pal: 0 }));
}

#[test]
fn e05_05_mode3_direct_color() {
    let mut p = scene_c();
    // Enable direct color (CGWSEL bit 0)
    p.write(0x30, 0x01);
    // Direct color for index 71, pal 3:
    // r5 = ((71 & 7) << 2) | ((3 & 1) << 1) = (7 << 2) | 1 = 29
    // g5 = (((71 >> 3) & 7) << 2) | (3 & 2) = (8 & 7) << 2 | 2 = 0 | 2 = 2
    // b5 = (((71 >> 6) & 3) << 3) | (3 & 4) = (1 & 3) << 3 | 0 = 8
    // rgb = 29 | (2 << 5) | (8 << 10) = 29 + 64 + 8192 = 8285 = 0x205D... wait let me recalculate
    // Actually: r5=29, g5=2, b5=8 -> 29 | (2<<5) | (8<<10) = 29 + 64 + 8192 = 8285 = 0x205D
    // But the task says 0x205E. Let me check: pal=3, bits are 011 in binary
    // r5 = ((71 & 7) << 2) | ((pal & 1) << 1) = (7 << 2) | (1 << 1) = 28 + 2 = 30
    // g5 = (((71 >> 3) & 7) << 2) | (pal & 2) = (8 & 7) << 2 | 2 = 0 + 2 = 2
    // b5 = (((71 >> 6) & 3) << 3) | (pal & 4) = (1 & 3) << 3 | 0 = 8
    // rgb = 30 | (2 << 5) | (8 << 10) = 30 + 64 + 8192 = 8286 = 0x205E
    assert_eq!(p.bg_pixel(0, 0, 0), Some(Pix { rgb: 0x205E, prio: 0, obj_pal: 0 }));
}
