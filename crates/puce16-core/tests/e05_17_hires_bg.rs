use puce16_core::ppu::{Pix, Ppu};
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

fn at(p: &mut Ppu, v: u32, dot: u32) {
    for _ in 0..(v * 340 + dot) {
        p.step_dot();
    }
}

fn pix(rgb: u16, prio: u8) -> Option<Pix> {
    Some(Pix { rgb, prio, obj_pal: 0 })
}

fn scene_m5() -> Ppu {
    let mut p = Ppu::new(Region::Ntsc);
    p.write(0x05, 0x05);
    p.write(0x07, 0x04);
    p.write(0x0B, 0x01);
    put_vram(&mut p, 0x400, 0x0402);
    put_vram(&mut p, 0x1020, 0x0080);
    put_vram(&mut p, 0x1030, 0x8000);
    put_cgram(&mut p, 17, 0x1111);
    put_cgram(&mut p, 18, 0x2222);
    p
}

#[test]
fn e05_17_left_right_halves() {
    let p = scene_m5();
    assert_eq!(p.bg_pixel_hires(0, 0, 0), pix(0x1111, 0));
    assert_eq!(p.bg_pixel_hires(0, 1, 0), None);
    assert_eq!(p.bg_pixel_hires(0, 8, 0), pix(0x2222, 0));
    assert_eq!(p.bg_pixel_hires(0, 9, 0), None);
    assert_eq!(p.bg_pixel_hires(0, 16, 0), None);
}

#[test]
fn e05_17_bg_pixel_none_in_hires_modes() {
    let mut p = scene_m5();
    assert_eq!(p.bg_pixel(0, 0, 0), None);
    p.write(0x05, 0x06);
    assert_eq!(p.bg_pixel(0, 0, 0), None);
}

#[test]
fn e05_17_hofs_doubled() {
    let mut p = scene_m5();
    set_scroll(&mut p, 0x0D, 4);
    assert_eq!(p.bg_pixel_hires(0, 0, 0), pix(0x2222, 0));
    set_scroll(&mut p, 0x0D, 8);
    assert_eq!(p.bg_pixel_hires(0, 0, 0), None);
}

#[test]
fn e05_17_tile16_vertical() {
    let mut p = scene_m5();
    p.write(0x05, 0x15);
    put_vram(&mut p, 0x1120, 0x8080);
    put_cgram(&mut p, 19, 0x3333);
    assert_eq!(p.bg_pixel_hires(0, 0, 8), pix(0x3333, 0));
    assert_eq!(p.bg_pixel_hires(0, 8, 8), None);
    assert_eq!(p.bg_pixel_hires(0, 0, 0), pix(0x1111, 0));
    assert_eq!(p.bg_pixel_hires(0, 0, 7), None);
}

#[test]
fn e05_17_tile8_vertical() {
    let p = scene_m5();
    assert_eq!(p.bg_pixel_hires(0, 0, 8), None);
    assert_eq!(p.bg_pixel_hires(0, 0, 0), pix(0x1111, 0));
}

#[test]
fn e05_17_hflip() {
    let mut p = scene_m5();
    put_vram(&mut p, 0x400, 0x4402);
    assert_eq!(p.bg_pixel_hires(0, 15, 0), pix(0x1111, 0));
    assert_eq!(p.bg_pixel_hires(0, 7, 0), pix(0x2222, 0));
    assert_eq!(p.bg_pixel_hires(0, 0, 0), None);
}

#[test]
fn e05_17_vflip_8_high() {
    let mut p = scene_m5();
    put_vram(&mut p, 0x400, 0x8402);
    assert_eq!(p.bg_pixel_hires(0, 0, 7), pix(0x1111, 0));
    assert_eq!(p.bg_pixel_hires(0, 0, 0), None);
}

#[test]
fn e05_17_bg2_two_bpp() {
    let mut p = scene_m5();
    p.write(0x08, 0x08);
    p.write(0x0B, 0x21);
    put_vram(&mut p, 0x800, 0x0C01);
    put_vram(&mut p, 0x2008, 0x0080);
    put_cgram(&mut p, 13, 0x4444);
    assert_eq!(p.bg_pixel_hires(1, 0, 0), pix(0x4444, 0));
    assert_eq!(p.bg_pixel_hires(1, 8, 0), None);
    assert_eq!(p.bg_pixel_hires(2, 0, 0), None);
}

#[test]
fn e05_17_mode6_bg1_only() {
    let mut p = scene_m5();
    p.write(0x05, 0x06);
    assert_eq!(p.bg_pixel_hires(0, 0, 0), pix(0x1111, 0));
    assert_eq!(p.bg_pixel_hires(1, 0, 0), None);
    assert_eq!(p.bg_pixel_hires(0, 8, 0), pix(0x2222, 0));
}

#[test]
fn e05_17_priority_bit() {
    let mut p = scene_m5();
    put_vram(&mut p, 0x400, 0x2402);
    assert_eq!(p.bg_pixel_hires(0, 0, 0), pix(0x1111, 1));
}

#[test]
fn e05_17_interlace_field_false() {
    let mut p = scene_m5();
    p.write(0x05, 0x15);
    put_vram(&mut p, 0x1120, 0x8080);
    put_cgram(&mut p, 19, 0x3333);
    p.write(0x33, 0x01);
    assert_eq!(p.bg_pixel_hires(0, 0, 4), pix(0x3333, 0));
    assert_eq!(p.bg_pixel_hires(0, 0, 0), pix(0x1111, 0));
    p.write(0x33, 0x00);
    assert_eq!(p.bg_pixel_hires(0, 0, 4), None);
}

#[test]
fn e05_17_interlace_field_true() {
    let mut p = scene_m5();
    p.write(0x05, 0x15);
    put_vram(&mut p, 0x1121, 0x0080);
    p.write(0x33, 0x01);
    at(&mut p, 263, 0);
    assert_eq!(p.bg_pixel_hires(0, 0, 4), pix(0x1111, 0));
}

#[test]
fn e05_17_mosaic_vertical() {
    let mut p = scene_m5();
    put_vram(&mut p, 0x1021, 0x8000);
    p.write(0x06, 0x11);
    assert_eq!(p.bg_pixel_hires(0, 0, 2), pix(0x2222, 0));
    p.write(0x06, 0x00);
    assert_eq!(p.bg_pixel_hires(0, 0, 2), None);
}

#[test]
fn e05_17_mosaic_horizontal() {
    let mut p = scene_m5();
    put_vram(&mut p, 0x1020, 0x0040);
    p.write(0x06, 0x11);
    assert_eq!(p.bg_pixel_hires(0, 3, 0), pix(0x1111, 0));
    assert_eq!(p.bg_pixel_hires(0, 2, 0), None);
    p.write(0x06, 0x00);
    assert_eq!(p.bg_pixel_hires(0, 3, 0), None);
}

#[test]
fn e05_17_sample_layers_hx_mode5() {
    let mut p = scene_m5();
    p.write(0x2C, 0x01);
    assert_eq!(p.sample_layers_hx(0, 0, 0)[0], pix(0x1111, 0));
    assert_eq!(p.sample_layers_hx(0, 1, 0)[0], None);
    assert_eq!(p.sample_layers(0, 0)[0], None);
    assert_eq!(p.sample_layers(4, 0)[0], None);
    assert_eq!(p.sample_layers_hx(4, 8, 0)[0], pix(0x2222, 0));
    p.write(0x2C, 0x00);
    p.write(0x2D, 0x00);
    assert_eq!(p.sample_layers_hx(0, 0, 0), [None; 5]);
}

#[test]
fn e05_17_sample_layers_hx_ignored_in_mode1() {
    let mut p = Ppu::new(Region::Ntsc);
    p.write(0x05, 0x01);
    p.write(0x07, 0x04);
    p.write(0x0B, 0x01);
    put_vram(&mut p, 0x1020, 0x0080);
    put_vram(&mut p, 0x1021, 0x8000);
    put_vram(&mut p, 0x400, 0x0402);
    put_vram(&mut p, 0x401, 0x2402);
    put_cgram(&mut p, 17, 0x1234);
    put_cgram(&mut p, 18, 0x2345);
    put_cgram(&mut p, 0, 0x7C00);
    set_scroll(&mut p, 0x0E, 0x3FF);
    p.write(0x2C, 0x01);
    assert_eq!(p.sample_layers_hx(0, 5, 0), p.sample_layers(0, 0));
    // VOFS = -1: scanline v = 1 shows map row 0 (v = 0 would wrap to row 1023).
    assert_eq!(p.sample_layers_hx(0, 0, 1)[0], pix(0x1234, 0));
}
