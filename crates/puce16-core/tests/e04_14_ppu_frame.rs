use puce16_core::region::Region;
use puce16_core::ppu::Ppu;
use puce16_core::hash::Fnv64;

fn at(p: &mut Ppu, v: u32, dot: u32) {
    for _ in 0..(v * 340 + dot) {
        p.step_dot();
    }
}

fn set_backdrop(p: &mut Ppu, w: u16) {
    p.write(0x21, 0);
    p.write(0x22, (w & 0xFF) as u8);
    p.write(0x22, (w >> 8) as u8);
}

#[test]
fn e04_14_initial_frame() {
    let mut p = Ppu::new(Region::Ntsc);
    let f = p.frame();
    assert_eq!(f.width, 256);
    assert_eq!(f.height, 224);
    assert_eq!(f.pixels.len(), 57344);
    for &px in f.pixels {
        assert_eq!(px, 0xFF00_0000);
    }
}

#[test]
fn e04_14_forced_blank_black() {
    let mut p = Ppu::new(Region::Ntsc);
    set_backdrop(&mut p, 0x001F);
    at(&mut p, 225, 1);
    let f = p.frame();
    assert_eq!(f.width, 256);
    assert_eq!(f.height, 224);
    for &px in f.pixels {
        assert_eq!(px, 0xFF00_0000);
    }
}

#[test]
fn e04_14_backdrop_red() {
    let mut p = Ppu::new(Region::Ntsc);
    set_backdrop(&mut p, 0x001F);
    p.write(0x00, 0x0F);
    at(&mut p, 225, 1);
    let f = p.frame();
    assert_eq!(f.width, 256);
    assert_eq!(f.height, 224);
    for &px in f.pixels {
        assert_eq!(px, 0xFFFF_0000);
    }
}

#[test]
fn e04_14_frame_not_captured_early() {
    let mut p = Ppu::new(Region::Ntsc);
    set_backdrop(&mut p, 0x001F);
    p.write(0x00, 0x0F);
    at(&mut p, 225, 0);
    let f = p.frame();
    for &px in f.pixels {
        assert_eq!(px, 0xFF00_0000);
    }
    p.step_dot();
    let f2 = p.frame();
    for &px in f2.pixels {
        assert_eq!(px, 0xFFFF_0000);
    }
}

#[test]
fn e04_14_brightness_second_frame() {
    let mut p = Ppu::new(Region::Ntsc);
    set_backdrop(&mut p, 0x001F);
    p.write(0x00, 0x0F);
    at(&mut p, 225, 1);
    p.write(0x00, 0x08);
    for _ in 0..(262 * 340) {
        p.step_dot();
    }
    let f = p.frame();
    for &px in f.pixels {
        assert_eq!(px, 0xFF84_0000);
    }
}

#[test]
fn e04_14_overscan_height() {
    let mut p = Ppu::new(Region::Ntsc);
    p.write(0x33, 0x04);
    set_backdrop(&mut p, 0x001F);
    p.write(0x00, 0x0F);
    at(&mut p, 240, 1);
    let f = p.frame();
    assert_eq!(f.height, 239);
    assert_eq!(f.pixels.len(), 256 * 239);
    for &px in f.pixels {
        assert_eq!(px, 0xFFFF_0000);
    }
}

#[test]
fn e04_14_interlace_fields() {
    let mut p = Ppu::new(Region::Ntsc);
    p.write(0x33, 0x01);
    set_backdrop(&mut p, 0x001F);
    p.write(0x00, 0x0F);
    at(&mut p, 225, 1);
    let f = p.frame();
    assert_eq!(f.height, 448);
    assert_eq!(f.pixels[0], 0xFFFF_0000);
    assert_eq!(f.pixels[256], 0xFF00_0000);
    assert_eq!(f.pixels[256 * 447 - 1], 0xFFFF_0000);
    assert_eq!(f.pixels[256 * 448 - 1], 0xFF00_0000);
    let red_count = f.pixels.iter().filter(|&&px| px == 0xFFFF_0000).count();
    assert_eq!(red_count, 224 * 256);

    for _ in 0..(263 * 340) {
        p.step_dot();
    }
    let f2 = p.frame();
    for &px in f2.pixels {
        assert_eq!(px, 0xFFFF_0000);
    }
}

#[test]
fn e04_14_line_change_vertical() {
    let mut p = Ppu::new(Region::Ntsc);
    set_backdrop(&mut p, 0x001F);
    p.write(0x00, 0x0F);
    at(&mut p, 100, 0);
    set_backdrop(&mut p, 0x7C00);
    for _ in 0..(125 * 340 + 1) {
        p.step_dot();
    }
    let f = p.frame();
    for row in 0..99 {
        for col in 0..256 {
            assert_eq!(f.pixels[row * 256 + col], 0xFFFF_0000);
        }
    }
    for row in 99..224 {
        for col in 0..256 {
            assert_eq!(f.pixels[row * 256 + col], 0xFF00_00FF);
        }
    }
}

#[test]
fn e04_14_mid_line_change() {
    let mut p = Ppu::new(Region::Ntsc);
    set_backdrop(&mut p, 0x001F);
    p.write(0x00, 0x0F);
    at(&mut p, 10, 150);
    set_backdrop(&mut p, 0x7C00);
    for _ in 0..((225 - 10) * 340 + 1) {
        p.step_dot();
    }
    let f = p.frame();
    for col in 0..128 {
        assert_eq!(f.pixels[9 * 256 + col], 0xFFFF_0000);
    }
    for col in 128..256 {
        assert_eq!(f.pixels[9 * 256 + col], 0xFF00_00FF);
    }
    for col in 0..256 {
        assert_eq!(f.pixels[8 * 256 + col], 0xFFFF_0000);
    }
    for col in 0..256 {
        assert_eq!(f.pixels[10 * 256 + col], 0xFF00_00FF);
    }
}

#[test]
fn e04_14_reset_clears_image() {
    let mut p = Ppu::new(Region::Ntsc);
    set_backdrop(&mut p, 0x001F);
    p.write(0x00, 0x0F);
    at(&mut p, 225, 1);
    p.reset();
    let f = p.frame();
    assert_eq!(f.width, 256);
    assert_eq!(f.height, 224);
    for &px in f.pixels {
        assert_eq!(px, 0xFF00_0000);
    }
}

#[test]
fn e04_14_hash_ignores_framebuffer() {
    let mut a = Ppu::new(Region::Ntsc);
    set_backdrop(&mut a, 0x001F);
    a.write(0x00, 0x0F);
    at(&mut a, 225, 1);
    set_backdrop(&mut a, 0);

    let mut b = Ppu::new(Region::Ntsc);
    set_backdrop(&mut b, 0);
    b.write(0x00, 0x0F);
    at(&mut b, 225, 1);

    let mut ha = Fnv64::new();
    a.hash_into(&mut ha);
    let hash_a = ha.finish();

    let mut hb = Fnv64::new();
    b.hash_into(&mut hb);
    let hash_b = hb.finish();

    assert_eq!(hash_a, hash_b);
}