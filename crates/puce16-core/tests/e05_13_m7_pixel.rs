#![allow(dead_code)]
use puce16_core::ppu::Ppu;
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

fn put_m7_map(p: &mut Ppu, idx: u16, tile: u8) {
    p.write(0x15, 0x00);
    p.write(0x16, (idx & 0xFF) as u8);
    p.write(0x17, (idx >> 8) as u8);
    p.write(0x18, tile);
}

fn put_m7_pix(p: &mut Ppu, tile: u16, fy: u16, fx: u16, c: u8) {
    let a = tile * 64 + fy * 8 + fx;
    p.write(0x15, 0x80);
    p.write(0x16, (a & 0xFF) as u8);
    p.write(0x17, (a >> 8) as u8);
    p.write(0x19, c);
}

fn scene_g(p: &mut Ppu) {
    p.write(0x05, 0x07);
    // Identity matrix
    p.write(0x1B, 0x00);
    p.write(0x1B, 0x01);
    p.write(0x1E, 0x00);
    p.write(0x1E, 0x01);
}

#[test]
fn e05_13_scene_g_identity() {
    let mut p = Ppu::new(Region::Ntsc);
    scene_g(&mut p);
    put_m7_map(&mut p, 0, 1);
    put_m7_pix(&mut p, 1, 0, 0, 5);
    put_cgram(&mut p, 5, 0x0555);

    let pix = p.m7_pixel(0, 0, 0).unwrap();
    assert_eq!(pix.rgb, 0x0555);
    assert_eq!(pix.prio, 0);
    assert_eq!(pix.obj_pal, 0);
}

#[test]
fn e05_13_other_positions() {
    let mut p = Ppu::new(Region::Ntsc);
    scene_g(&mut p);
    put_m7_map(&mut p, 0, 1);
    put_m7_pix(&mut p, 1, 0, 0, 5);
    put_cgram(&mut p, 5, 0x0555);

    // px=9 -> map (1,0) = tile 2, fx=1
    put_m7_map(&mut p, 1, 2);
    put_m7_pix(&mut p, 2, 0, 1, 7);
    put_cgram(&mut p, 7, 0x0777);

    // py=8 -> map index 128
    put_m7_map(&mut p, 128, 3);
    put_m7_pix(&mut p, 3, 0, 0, 9);
    put_cgram(&mut p, 9, 0x0999);

    let pix = p.m7_pixel(0, 9, 0).unwrap();
    assert_eq!(pix.rgb, 0x0777);

    let pix = p.m7_pixel(0, 0, 8).unwrap();
    assert_eq!(pix.rgb, 0x0999);

    // tile 1 pixel (0,1) is index 0 -> transparent
    assert!(p.m7_pixel(0, 1, 0).is_none());
}

#[test]
fn e05_13_not_mode7_or_wrong_layer() {
    let mut p = Ppu::new(Region::Ntsc);
    // Not mode 7
    p.write(0x05, 0x01);
    assert!(p.m7_pixel(0, 0, 0).is_none());

    // Mode 7 but no EXTBG for BG2
    scene_g(&mut p);
    put_m7_map(&mut p, 0, 1);
    put_m7_pix(&mut p, 1, 0, 0, 5);
    put_cgram(&mut p, 5, 0x0555);

    assert!(p.m7_pixel(1, 0, 0).is_none());
    assert!(p.m7_pixel(2, 0, 0).is_none());
}

#[test]
fn e05_13_wrap_outside_false() {
    let mut p = Ppu::new(Region::Ntsc);
    scene_g(&mut p);
    put_m7_map(&mut p, 0, 1);
    put_m7_pix(&mut p, 1, 0, 0, 5);
    put_cgram(&mut p, 5, 0x0555);

    // M7HOFS = 1020 -> px will be 1024 at x=4 (wraps to 0)
    p.write(0x0D, 0xFC);
    p.write(0x0D, 0x03);

    let pix = p.m7_pixel(0, 4, 0).unwrap();
    assert_eq!(pix.rgb, 0x0555);
}

#[test]
fn e05_13_outside_transparent() {
    let mut p = Ppu::new(Region::Ntsc);
    scene_g(&mut p);
    put_m7_map(&mut p, 0, 1);
    // px=1020 at x=0 -> fx=4, so put pixel at column 4
    put_m7_pix(&mut p, 1, 0, 4, 5);
    put_cgram(&mut p, 5, 0x0555);

    // M7HOFS = 1020 -> px will be 1024 at x=4 (outside)
    p.write(0x0D, 0xFC);
    p.write(0x0D, 0x03);

    // outside=1, fill=0 -> transparent
    p.write(0x1A, 0x80);

    assert!(p.m7_pixel(0, 4, 0).is_none());
    // Inside still works (px=1020 at x=0 is in map column 127)
    put_m7_map(&mut p, 127, 1);
    let pix = p.m7_pixel(0, 0, 0).unwrap();
    assert_eq!(pix.rgb, 0x0555);
}

#[test]
fn e05_13_outside_fill_tile0() {
    let mut p = Ppu::new(Region::Ntsc);
    scene_g(&mut p);
    put_m7_map(&mut p, 0, 1);
    put_m7_pix(&mut p, 1, 0, 0, 5);
    put_cgram(&mut p, 5, 0x0555);

    // M7HOFS = 1020 -> px will be 1024 at x=4 (outside)
    p.write(0x0D, 0xFC);
    p.write(0x0D, 0x03);

    // outside=1, fill=1 -> use tile 0
    p.write(0x1A, 0xC0);
    put_m7_pix(&mut p, 0, 0, 0, 9);
    put_cgram(&mut p, 9, 0x0999);

    let pix = p.m7_pixel(0, 4, 0).unwrap();
    assert_eq!(pix.rgb, 0x0999);
}

#[test]
fn e05_13_flip_x() {
    let mut p = Ppu::new(Region::Ntsc);
    scene_g(&mut p);
    put_m7_map(&mut p, 0, 1);
    put_m7_pix(&mut p, 1, 0, 0, 5);
    put_cgram(&mut p, 5, 0x0555);

    // flip_x: sx = 255 - x -> at x=0, sx=255 -> px=255 -> map (31,0)
    p.write(0x1A, 0x01);
    put_m7_map(&mut p, 31, 4);
    put_m7_pix(&mut p, 4, 0, 7, 3);
    put_cgram(&mut p, 3, 0x0333);

    let pix = p.m7_pixel(0, 0, 0).unwrap();
    assert_eq!(pix.rgb, 0x0333);
}

#[test]
fn e05_13_extbg() {
    let mut p = Ppu::new(Region::Ntsc);
    scene_g(&mut p);
    put_m7_map(&mut p, 0, 1);
    put_cgram(&mut p, 5, 0x0111);

    // EXTBG enabled
    p.write(0x33, 0x40);

    // idx = 0x85 -> BG1: cgram[0x85], BG2: prio=1, i7=0x05 -> cgram[5]
    put_m7_pix(&mut p, 1, 0, 0, 0x85);
    put_cgram(&mut p, 0x85, 0x0222);

    let pix = p.m7_pixel(0, 0, 0).unwrap();
    assert_eq!(pix.rgb, 0x0222);
    assert_eq!(pix.prio, 0);

    let pix = p.m7_pixel(1, 0, 0).unwrap();
    assert_eq!(pix.rgb, 0x0111);
    assert_eq!(pix.prio, 1);

    // idx = 0x80 -> BG2: i7=0 -> transparent
    put_m7_pix(&mut p, 1, 0, 0, 0x80);
    put_cgram(&mut p, 0x80, 0x0444);

    let pix = p.m7_pixel(0, 0, 0).unwrap();
    assert_eq!(pix.rgb, 0x0444);
    assert!(p.m7_pixel(1, 0, 0).is_none());

    // idx = 0x05 -> BG2: prio=0
    put_m7_pix(&mut p, 1, 0, 0, 0x05);
    let pix = p.m7_pixel(1, 0, 0).unwrap();
    assert_eq!(pix.prio, 0);
}

#[test]
fn e05_13_direct_color() {
    let mut p = Ppu::new(Region::Ntsc);
    scene_g(&mut p);
    put_m7_map(&mut p, 0, 1);

    // Direct color mode for BG1
    p.write(0x30, 0x01);
    put_m7_pix(&mut p, 1, 0, 0, 0xFF);

    let pix = p.m7_pixel(0, 0, 0).unwrap();
    assert_eq!(pix.rgb, 0x639C);

    // EXTBG BG2 still uses CGRAM indexed mode
    p.write(0x33, 0x40);
    put_cgram(&mut p, 0x7F, 0x0AAA);
    let pix = p.m7_pixel(1, 0, 0).unwrap();
    assert_eq!(pix.rgb, 0x0AAA);
}

#[test]
fn e05_13_mosaic() {
    let mut p = Ppu::new(Region::Ntsc);
    scene_g(&mut p);
    put_m7_map(&mut p, 0, 1);
    put_m7_pix(&mut p, 1, 0, 0, 5);
    put_cgram(&mut p, 5, 0x0555);

    // Mosaic size 2 for BG1 (bit 0 of mosaic_en)
    p.write(0x06, 0x11);

    // At x=1 with mosaic size 2: xm = 1 - (1 % 2) = 0
    let pix = p.m7_pixel(0, 1, 0).unwrap();
    assert_eq!(pix.rgb, 0x0555);

    // Without mosaic, x=1 would be transparent (tile 1 pixel (0,1) is index 0)
    p.write(0x06, 0x00);
    assert!(p.m7_pixel(0, 1, 0).is_none());
}
