use puce16_core::consts::{FB_HEIGHT, FB_WIDTH};
use puce16_core::hash::Fnv64;
use puce16_core::ppu::output::{make_view, pixel_double, FrameBuf};

const BLACK: u32 = 0xFF00_0000;

#[test]
fn e01_05_new_black() {
    let fb = FrameBuf::new();
    assert_eq!(fb.pixels.len(), FB_WIDTH * FB_HEIGHT);
    assert!(fb.pixels.iter().all(|&p| p == BLACK));
    assert_eq!(fb.view_w, 256);
    assert_eq!(fb.view_h, 224);
}

#[test]
fn e01_05_set_get() {
    let mut fb = FrameBuf::new();
    fb.set(3, 2, 0xFF11_2233);
    assert_eq!(fb.get(3, 2), 0xFF11_2233);
    // Out-of-buffer writes are ignored and must not panic.
    fb.set(600, 0, 0x1234_5678);
    fb.set(0, FB_HEIGHT + 1, 0x1234_5678);
    assert_eq!(fb.get(3, 2), 0xFF11_2233);
}

#[test]
fn e01_05_compact_256() {
    let mut fb = FrameBuf::new(); // default view 256x224; use a 2-row view.
    fb.set_view(256, 2);
    const A: u32 = 0xFF11_1111;
    const B: u32 = 0xFF22_2222;
    const C: u32 = 0xFF33_3333;
    const D: u32 = 0xFF44_4444;
    fb.set(0, 0, A);
    fb.set(255, 0, B);
    fb.set(0, 1, C);
    fb.set(255, 1, D);

    let c = fb.compact();
    assert_eq!(c.len(), 512);
    for i in 0..256 {
        let expect_row0 = if i == 0 { A } else if i == 255 { B } else { BLACK };
        let expect_row1 = if i == 0 { C } else if i == 255 { D } else { BLACK };
        assert_eq!(c[i], expect_row0, "row 0 col {}", i);
        assert_eq!(c[256 + i], expect_row1, "row 1 col {}", i);
    }
    assert_eq!(c[255], B);
    assert_eq!(c[256], C);
}

#[test]
fn e01_05_pixel_double() {
    let src: Vec<u32> = (0..256).map(|i| BLACK | i as u32).collect();
    let out = pixel_double(&src, 256, 1);
    assert_eq!(out.len(), 512);
    for i in 0..256 {
        assert_eq!(out[2 * i], BLACK | i as u32);
        assert_eq!(out[2 * i + 1], BLACK | i as u32);
    }
}

#[test]
fn e01_05_make_view() {
    let pixels = [1u32, 2, 3, 4];
    let v = make_view(2, 2, &pixels);
    assert_eq!(v.width, 2);
    assert_eq!(v.height, 2);
    assert_eq!(v.pixels, &[1, 2, 3, 4]);
}

#[test]
#[should_panic]
fn e01_05_make_view_panics() {
    let pixels = [1u32, 2, 3];
    make_view(2, 2, &pixels);
}

#[test]
fn e01_05_hash_changes() {
    let a = FrameBuf::new();
    let mut b = FrameBuf::new();
    let mut ha = Fnv64::new();
    a.hash(&mut ha);
    let mut hb = Fnv64::new();
    b.hash(&mut hb);
    assert_eq!(ha.finish(), hb.finish());

    b.set(0, 0, 1);
    let mut hc = Fnv64::new();
    b.hash(&mut hc);
    assert_ne!(hc.finish(), ha.finish());
}
