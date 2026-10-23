use puce16_core::{Region, Snes};

#[test]
fn e01_06_pixel_origin() {
    let mut s = Snes::new_test_pattern(Region::Ntsc);
    s.run_frame(); // f = 1
    let v = s.frame();
    assert_eq!(v.pixels[0], 0xFF01_0040);
}

#[test]
fn e01_06_pixel_formula() {
    let mut s = Snes::new_test_pattern(Region::Ntsc);
    for _ in 0..3 {
        s.run_frame(); // f = 3
    }
    let v = s.frame();
    assert_eq!(v.pixels[5 * 256 + 10], 0xFF0D_0540);
    assert_eq!(v.pixels[223 * 256 + 255], 0xFF02_DF40);
}

#[test]
fn e01_06_view_size() {
    let mut s = Snes::new_test_pattern(Region::Ntsc);
    s.run_frame();
    let v = s.frame();
    assert_eq!(v.width, 256);
    assert_eq!(v.height, 224);
    assert_eq!(v.pixels.len(), 256 * 224);
}

#[test]
fn e01_06_deterministic() {
    let mut a = Snes::new_test_pattern(Region::Ntsc);
    let mut b = Snes::new_test_pattern(Region::Ntsc);
    for _ in 0..10 {
        a.run_frame();
        b.run_frame();
    }
    assert_eq!(a.state_hash(), b.state_hash());

    // One extra frame on `a` (11 vs 10) must change the hash.
    a.run_frame();
    assert_ne!(a.state_hash(), b.state_hash());
}

#[test]
fn e01_06_joypad_marker() {
    let mut s = Snes::new_test_pattern(Region::Ntsc);
    s.set_joypad(0, 0x8000);
    s.run_frame(); // f = 1
    assert_eq!(s.frame().pixels[0], 0xFFFF_FFFF);

    // Out-of-range port is ignored: no panic, state unchanged.
    s.set_joypad(2, 1);
    assert_eq!(s.joypad(0), 0x8000);
}
