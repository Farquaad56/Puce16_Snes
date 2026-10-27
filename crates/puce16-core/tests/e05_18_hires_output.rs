use puce16_core::hash::Fnv64;
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

fn hash_of(p: &Ppu) -> u64 {
    let mut h = Fnv64::new();
    p.hash_into(&mut h);
    h.finish()
}

fn scene_h() -> Ppu {
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
    p
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
fn e05_18_pseudo_hires_sub_black() {
    let mut p = scene_h();
    p.write(0x2C, 0x01);
    p.write(0x33, 0x08);
    p.write(0x00, 0x0F);
    at(&mut p, 225, 1);
    let f = p.frame();
    assert_eq!(f.width, 512);
    assert_eq!(f.height, 224);
    assert_eq!(f.pixels.len(), 512 * 224);
    assert_eq!(f.pixels[1], 0xFFA58C21);
    assert_eq!(f.pixels[0], 0xFF000000);
    assert_eq!(f.pixels[3], 0xFF0000FF);
    assert_eq!(f.pixels[2], 0xFF000000);
}

#[test]
fn e05_18_pseudo_hires_ts_bg1() {
    let mut p = scene_h();
    p.write(0x2C, 0x01);
    p.write(0x2D, 0x01);
    p.write(0x33, 0x08);
    p.write(0x00, 0x0F);
    at(&mut p, 225, 1);
    let px = &p.frame().pixels;
    assert_eq!(px[0], 0xFFA58C21);
    assert_eq!(px[1], 0xFFA58C21);
    assert_eq!(px[2], 0xFF000000);
    assert_eq!(px[3], 0xFF0000FF);
}

#[test]
fn e05_18_pseudo_hires_math() {
    let mut p = scene_h();
    p.write(0x2C, 0x01);
    p.write(0x33, 0x08);
    p.write(0x31, 0x01);
    p.write(0x32, 0x2A);
    p.write(0x00, 0x0F);
    at(&mut p, 225, 1);
    {
        let px = &p.frame().pixels;
        assert_eq!(px[1], 0xFFF78C21);
        assert_eq!(px[0], 0xFF520000);
        assert_eq!(px[3], 0xFF0000FF);
        assert_eq!(px[2], 0xFF520000);
    }

    let mut q = scene_h();
    q.write(0x2C, 0x01);
    q.write(0x33, 0x08);
    q.write(0x31, 0x21);
    q.write(0x32, 0x2A);
    q.write(0x00, 0x0F);
    at(&mut q, 225, 1);
    let px = &q.frame().pixels;
    assert_eq!(px[0], 0xFFA50000);
    assert_eq!(px[3], 0xFF5200FF);
    assert_eq!(px[2], 0xFFA50000);
    assert_eq!(px[1], 0xFFF78C21);
}

#[test]
fn e05_18_pseudo_hires_addend_subscreen() {
    let mut p = scene_h();
    p.write(0x2C, 0x01);
    p.write(0x2D, 0x01);
    p.write(0x33, 0x08);
    p.write(0x30, 0x02);
    p.write(0x31, 0x01);
    p.write(0x00, 0x0F);
    at(&mut p, 225, 1);
    let px = &p.frame().pixels;
    assert_eq!(px[1], 0xFFFFFF42);
    assert_eq!(px[0], 0xFFA58C21);
}

#[test]
fn e05_18_pseudo_hires_black_region() {
    let mut p = scene_h();
    p.write(0x33, 0x08);
    p.write(0x30, 0xC0);
    p.write(0x2C, 0x01);
    p.write(0x00, 0x0F);
    at(&mut p, 225, 1);
    {
        let px = &p.frame().pixels;
        for c in 0..512 {
            assert_eq!(px[c], 0xFF000000, "col {}", c);
        }
    }

    let mut q = scene_h();
    q.write(0x33, 0x08);
    q.write(0x30, 0xC0);
    q.write(0x2C, 0x01);
    q.write(0x31, 0x01);
    q.write(0x32, 0x2A);
    q.write(0x00, 0x0F);
    at(&mut q, 225, 1);
    let px = &q.frame().pixels;
    assert_eq!(px[1], 0xFF520000);
    assert_eq!(px[0], 0xFF000000);
    assert_eq!(px[3], 0xFF000000);
}

#[test]
fn e05_18_mode5_frame() {
    let mut p = scene_m5();
    set_scroll(&mut p, 0x0E, 0x3FF);
    put_cgram(&mut p, 0, 0x7C00);
    p.write(0x2C, 0x01);
    p.write(0x2D, 0x01);
    p.write(0x00, 0x0F);
    at(&mut p, 225, 1);
    let f = p.frame();
    assert_eq!(f.width, 512);
    assert_eq!(f.height, 224);
    assert_eq!(f.pixels[0], 0xFF8C4221);
    assert_eq!(f.pixels[1], 0xFF0000FF);
    assert_eq!(f.pixels[8], 0xFF108C42);
    assert_eq!(f.pixels[9], 0xFF0000FF);
    assert_eq!(f.pixels[2], 0xFF000000);
    assert_eq!(f.pixels[3], 0xFF0000FF);
}

// Shared by the two "mixed rows" tests. Returns the PPU after frame 0 capture.
fn mixed_rows_ppu() -> Ppu {
    let mut p = scene_h();
    p.write(0x2C, 0x00);
    p.write(0x33, 0x08);
    p.write(0x00, 0x0F);
    at(&mut p, 100, 0);
    p.write(0x33, 0x00);
    at(&mut p, 125, 1);
    p
}

#[test]
fn e05_18_mixed_rows_doubled() {
    let p = mixed_rows_ppu();
    let f = p.frame();
    assert_eq!(f.width, 512);
    assert_eq!(f.height, 224);
    for r in 0..=98usize {
        assert_eq!(f.pixels[r * 512], 0xFF000000, "row {} col 0", r);
        assert_eq!(f.pixels[r * 512 + 1], 0xFF0000FF, "row {} col 1", r);
    }
    for r in 99..=223usize {
        assert_eq!(f.pixels[r * 512], 0xFF0000FF, "row {} col 0", r);
        assert_eq!(f.pixels[r * 512 + 1], 0xFF0000FF, "row {} col 1", r);
    }
    assert_eq!(f.pixels[223 * 512 + 510], 0xFF0000FF);
    assert_eq!(f.pixels[223 * 512 + 511], 0xFF0000FF);
}

#[test]
fn e05_18_frame_back_to_256() {
    let mut p = mixed_rows_ppu();
    assert_eq!(p.frame().width, 512);
    for _ in 0..(262 * 340) {
        p.step_dot();
    }
    let f = p.frame();
    assert_eq!(f.width, 256);
    assert_eq!(f.height, 224);
    for i in 0..f.pixels.len() {
        assert_eq!(f.pixels[i], 0xFF0000FF, "pixel {}", i);
    }
}

#[test]
fn e05_18_interlace_pseudo_hires() {
    let mut p = scene_h();
    p.write(0x2C, 0x00);
    p.write(0x33, 0x09);
    p.write(0x00, 0x0F);
    at(&mut p, 225, 1);
    let f = p.frame();
    assert_eq!(f.width, 512);
    assert_eq!(f.height, 448);
    assert_eq!(f.pixels[1], 0xFF0000FF);
    assert_eq!(f.pixels[0], 0xFF000000);
    assert_eq!(f.pixels[512 + 1], 0xFF000000);
    assert_eq!(f.pixels[446 * 512 + 1], 0xFF0000FF);
    assert_eq!(f.pixels[447 * 512 + 1], 0xFF000000);
    let blue = f.pixels.iter().filter(|&&c| c == 0xFF0000FF).count();
    assert_eq!(blue, 224 * 256);
}

#[test]
fn e05_18_forced_blank_hires() {
    let mut p = scene_h();
    p.write(0x33, 0x08);
    at(&mut p, 225, 1);
    let f = p.frame();
    assert_eq!(f.width, 512);
    for i in 0..f.pixels.len() {
        assert_eq!(f.pixels[i], 0xFF000000, "pixel {}", i);
    }
}

#[test]
fn e05_18_non_hires_frame_stays_256() {
    let mut p = scene_h();
    p.write(0x2C, 0x01);
    p.write(0x00, 0x0F);
    at(&mut p, 225, 1);
    let f = p.frame();
    assert_eq!(f.width, 256);
    assert_eq!(f.height, 224);
    assert_eq!(f.pixels[0], 0xFFA58C21);
}

#[test]
fn e05_18_hash_ignores_hires_state() {
    let mut a = scene_h();
    a.write(0x33, 0x08);
    a.write(0x00, 0x0F);
    at(&mut a, 225, 1);
    a.write(0x33, 0x00);

    let mut b = scene_h();
    b.write(0x33, 0x00);
    b.write(0x00, 0x0F);
    at(&mut b, 225, 1);

    assert_eq!(hash_of(&a), hash_of(&b));
    assert_eq!(a.frame().width, 512);
    assert_eq!(b.frame().width, 256);
}
