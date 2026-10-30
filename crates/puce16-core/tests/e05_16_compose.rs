use puce16_core::ppu::output::bgr555_to_xrgb;
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

fn set_scroll(p: &mut Ppu, reg: u8, v: u16) {
    p.write(reg, (v & 0xFF) as u8);
    p.write(reg, (v >> 8) as u8);
}

fn at(p: &mut Ppu, v: u32, dot: u32) {
    for _ in 0..(v * 340 + dot) {
        p.step_dot();
    }
}

// OAM image helpers (same idea as E05-10).
fn load_oam(p: &mut Ppu, img: &[u8; 544]) {
    p.write(0x02, 0);
    p.write(0x03, 0);
    for b in img.iter() {
        p.write(0x04, *b);
    }
}

fn spr(img: &mut [u8; 544], i: usize, x: i16, y: u8, tile: u8, attr: u8) {
    img[i * 4] = x as u8;
    img[i * 4 + 1] = y;
    img[i * 4 + 2] = tile;
    img[i * 4 + 3] = attr;
    let sh = (i % 4) * 2;
    let hb = 512 + i / 4;
    img[hb] &= !(3u8 << sh);
    let x8 = ((x >> 8) & 1) as u8;
    img[hb] |= x8 << sh;
}

fn scene_h(p: &mut Ppu) {
    p.write(0x05, 0x01);
    p.write(0x07, 0x04);
    p.write(0x0B, 0x01);
    put_vram(p, 0x1020, 0x0080);
    put_vram(p, 0x1021, 0x8000);
    put_vram(p, 0x400, 0x0402);
    put_vram(p, 0x401, 0x2402);
    put_cgram(p, 17, 0x1234);
    put_cgram(p, 18, 0x2345);
    put_cgram(p, 0, 0x7C00);
}

#[test]
fn e05_16_bg1_scene() {
    let mut p = Ppu::new(Region::Ntsc);
    scene_h(&mut p);
    set_scroll(&mut p, 0x0E, 0x3FF);
    p.write(0x2C, 0x01);
    p.write(0x00, 0x0F);
    at(&mut p, 225, 1);
    let px = &p.frame().pixels;
    assert_eq!(px[0], 0xFFA58C21);
    assert_eq!(px[1], 0xFF0000FF);
    assert_eq!(px[8], 0xFFA58C21);
    assert_eq!(px[256], bgr555_to_xrgb(0x2345, 15));
}

#[test]
fn e05_16_tm_zero_all_backdrop() {
    let mut p = Ppu::new(Region::Ntsc);
    scene_h(&mut p);
    set_scroll(&mut p, 0x0E, 0x3FF);
    p.write(0x2C, 0x00);
    p.write(0x00, 0x0F);
    at(&mut p, 225, 1);
    let px = &p.frame().pixels;
    assert_eq!(px.len(), 57344);
    for i in 0..px.len() {
        assert_eq!(px[i], 0xFF0000FF, "pixel {}", i);
    }
}

#[test]
fn e05_16_obj_over_bg() {
    let mut p = Ppu::new(Region::Ntsc);
    scene_h(&mut p);
    set_scroll(&mut p, 0x0E, 0x3FF);
    p.write(0x2C, 0x11);
    let mut img = [0u8; 544];
    for i in 0..128 {
        spr(&mut img, i, -200, 0, 0, 0);
    }
    spr(&mut img, 0, 0, 0, 0, 0x30);
    load_oam(&mut p, &img);
    put_vram(&mut p, 0, 0x0080);
    put_cgram(&mut p, 129, 0x03E0);
    p.write(0x00, 0x0F);
    at(&mut p, 225, 1);
    let px = &p.frame().pixels;
    assert_eq!(px[0], 0xFF00FF00);
    assert_eq!(px[8], 0xFFA58C21);
    assert_eq!(px[1], 0xFF0000FF);
    // Row 1: sprite data row 1 is empty, so BG1 row 1 shows (index 2).
    assert_eq!(px[256], bgr555_to_xrgb(0x2345, 15));
}

fn fixed_scene(sel: u8) -> u32 {
    let mut p = Ppu::new(Region::Ntsc);
    put_cgram(&mut p, 0, 0x0001);
    p.write(0x2C, 0x00);
    p.write(0x31, sel);
    p.write(0x32, 0x2A);
    p.write(0x00, 0x0F);
    at(&mut p, 225, 1);
    let px = &p.frame().pixels;
    for i in 0..px.len() {
        assert_eq!(px[i], px[0], "pixel {}", i);
    }
    px[0]
}

#[test]
fn e05_16_fixed_add_on_backdrop() {
    assert_eq!(fixed_scene(0x20), 0xFF5A0000);
    assert_eq!(fixed_scene(0x60), 0xFF290000);
    assert_eq!(fixed_scene(0xA0), 0xFF000000);
}

#[test]
fn e05_16_subscreen_add() {
    let mut p = Ppu::new(Region::Ntsc);
    scene_h(&mut p);
    set_scroll(&mut p, 0x0E, 0x3FF);
    put_cgram(&mut p, 0, 0x0001);
    p.write(0x2C, 0x00);
    p.write(0x2D, 0x01);
    p.write(0x30, 0x02);
    p.write(0x31, 0x20);
    p.write(0x00, 0x0F);
    at(&mut p, 225, 1);
    let px = &p.frame().pixels;
    assert_eq!(px[0], 0xFFAD8C21);
    assert_eq!(px[1], 0xFF080000);
}

#[test]
fn e05_16_window_hides_bg() {
    let mut p = Ppu::new(Region::Ntsc);
    scene_h(&mut p);
    set_scroll(&mut p, 0x0E, 0x3FF);
    p.write(0x2C, 0x01);
    p.write(0x2E, 0x01);
    p.write(0x23, 0x02);
    p.write(0x26, 0);
    p.write(0x27, 7);
    p.write(0x00, 0x0F);
    at(&mut p, 225, 1);
    let px = &p.frame().pixels;
    assert_eq!(px[0], 0xFF0000FF);
    assert_eq!(px[8], 0xFFA58C21);
}

#[test]
fn e05_16_mode7_pixel() {
    let mut p = Ppu::new(Region::Ntsc);
    p.write(0x05, 0x07);
    p.write(0x1B, 0x00);
    p.write(0x1B, 0x01);
    p.write(0x1E, 0x00);
    p.write(0x1E, 0x01);
    p.write(0x0E, 0xFF);
    p.write(0x0E, 0x1F);
    p.write(0x15, 0x00);
    p.write(0x16, 0);
    p.write(0x17, 0);
    p.write(0x18, 1);
    p.write(0x15, 0x80);
    p.write(0x16, 64);
    p.write(0x17, 0);
    p.write(0x19, 5);
    put_cgram(&mut p, 5, 0x0555);
    p.write(0x2C, 0x01);
    p.write(0x00, 0x0F);
    at(&mut p, 225, 1);
    assert_eq!(p.frame().pixels[0], 0xFFAD5208);
}

#[test]
fn e05_16_brightness_applied() {
    let mut p = Ppu::new(Region::Ntsc);
    scene_h(&mut p);
    p.write(0x2C, 0x00);
    p.write(0x00, 0x08);
    at(&mut p, 225, 1);
    let px = &p.frame().pixels;
    for i in 0..px.len() {
        assert_eq!(px[i], 0xFF000084, "pixel {}", i);
    }
}

#[test]
fn e05_16_forced_blank_stays_black() {
    let mut p = Ppu::new(Region::Ntsc);
    scene_h(&mut p);
    p.write(0x2C, 0x01);
    at(&mut p, 225, 1);
    let px = &p.frame().pixels;
    for i in 0..px.len() {
        assert_eq!(px[i], 0xFF000000, "pixel {}", i);
    }
}

#[test]
fn e05_16_sprites_evaluated_each_line() {
    let mut p = Ppu::new(Region::Ntsc);
    scene_h(&mut p);
    p.write(0x2C, 0x10);
    let mut img = [0u8; 544];
    for i in 0..128 {
        spr(&mut img, i, -200, 0, 0, 0);
    }
    spr(&mut img, 0, 0, 10, 0, 0x30);
    load_oam(&mut p, &img);
    put_vram(&mut p, 0, 0x0080);
    put_cgram(&mut p, 129, 0x03E0);
    p.write(0x00, 0x0F);
    at(&mut p, 225, 1);
    let px = &p.frame().pixels;
    assert_eq!(px[9 * 256], 0xFF0000FF);
    assert_eq!(px[10 * 256], 0xFF00FF00);
    assert_eq!(px[11 * 256], 0xFF0000FF);
}

#[test]
fn e05_16_range_over_flag_in_frame() {
    let mut p = Ppu::new(Region::Ntsc);
    scene_h(&mut p);
    p.write(0x2C, 0x10);
    let mut img = [0u8; 544];
    for i in 0..128 {
        spr(&mut img, i, -200, 0, 0, 0);
    }
    for i in 0..32 {
        spr(&mut img, i, (8 * i) as i16, 20, 0, 0);
    }
    spr(&mut img, 32, -7, 20, 0, 0); // -7 is on-screen (x > -width); -8 would not count
    load_oam(&mut p, &img);
    p.write(0x00, 0x0F);
    at(&mut p, 225, 1);
    assert_ne!(p.read(0x3E, 0, false) & 0x80, 0);
    // Total of exactly 262*340 + 1 steps from the start.
    for _ in 0..(37 * 340) {
        p.step_dot();
    }
    assert_eq!(p.read(0x3E, 0, false) & 0x80, 0);
}
