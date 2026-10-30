use puce16_core::region::Region;
use puce16_core::ppu::Ppu;
use puce16_core::hash::Fnv64;

fn at(p: &mut Ppu, v: u32, dot: u32) {
    for _ in 0..(v * 340 + dot) {
        p.step_dot();
    }
}

#[test]
fn e04_10_before_latch() {
    let mut p = Ppu::new(Region::Ntsc);
    assert_eq!(p.read(0x3C, 0, false), 0);
    assert_eq!(p.read(0x3D, 0, false), 0);
    assert!(!p.counter_latched());
}

#[test]
fn e04_10_latch_via_slhv() {
    let mut p = Ppu::new(Region::Ntsc);
    at(&mut p, 3, 100);
    assert_eq!(p.read(0x37, 0xEE, true), 0xEE);
    assert!(p.counter_latched());
    assert_eq!(p.read(0x3C, 0, false), 100);
    assert_eq!(p.read(0x3D, 0, false), 3);
}

#[test]
fn e04_10_slhv_needs_wrio7() {
    let mut p = Ppu::new(Region::Ntsc);
    at(&mut p, 3, 100);
    assert_eq!(p.read(0x37, 0, false), 0);
    assert!(!p.counter_latched());
}

#[test]
fn e04_10_ophct_two_bytes() {
    let mut p = Ppu::new(Region::Ntsc);
    at(&mut p, 1, 300);
    p.read(0x37, 0, true);
    assert_eq!(p.read(0x3C, 0, false), 0x2C);
    assert_eq!(p.read(0x3C, 0, false), 0x2D);
    assert_eq!(p.read(0x3C, 0, false), 0x2C);
}

#[test]
fn e04_10_opvct_two_bytes() {
    let mut p = Ppu::new(Region::Ntsc);
    at(&mut p, 261, 5);
    p.read(0x37, 0, true);
    assert_eq!(p.read(0x3D, 0, false), 0x05);
    assert_eq!(p.read(0x3D, 0, false), 0x05);
    assert_eq!(p.read(0x3D, 0, false), 0x05);
}

#[test]
fn e04_10_latch_value_frozen() {
    let mut p = Ppu::new(Region::Ntsc);
    at(&mut p, 2, 10);
    p.read(0x37, 0, true);
    for _ in 0..5 {
        p.step_dot();
    }
    assert_eq!(p.read(0x3C, 0, false), 10);
}

#[test]
fn e04_10_stat78_ntsc() {
    let mut p = Ppu::new(Region::Ntsc);
    assert_eq!(p.read(0x3F, 0, false), 0x03);
    for _ in 0..262 * 340 {
        p.step_dot();
    }
    assert_eq!(p.read(0x3F, 0, false), 0x83);
}

#[test]
fn e04_10_stat78_pal() {
    let mut p = Ppu::new(Region::Pal);
    assert_eq!(p.read(0x3F, 0, false), 0x13);
}

#[test]
fn e04_10_stat78_clears_latch() {
    let mut p = Ppu::new(Region::Ntsc);
    at(&mut p, 3, 100);
    p.read(0x37, 0, true);
    assert_eq!(p.read(0x3F, 0, false), 0x43);
    assert!(!p.counter_latched());
    assert_eq!(p.read(0x3F, 0, false), 0x03);
}

#[test]
fn e04_10_stat78_resets_toggles() {
    let mut p = Ppu::new(Region::Ntsc);
    at(&mut p, 1, 300);
    p.read(0x37, 0, true);
    assert_eq!(p.read(0x3C, 0, false), 0x2C);
    assert_eq!(p.read(0x3F, 0, false), 0x63);
    assert_eq!(p.read(0x3C, 0, false), 0x2C);
}

#[test]
fn e04_10_stat77() {
    let mut p = Ppu::new(Region::Ntsc);
    assert_eq!(p.read(0x3E, 0, false), 0x01);
    p.set_range_over(true);
    assert_eq!(p.read(0x3E, 0, false), 0x81);
    p.set_time_over(true);
    assert_eq!(p.read(0x3E, 0, false), 0xC1);
    p.set_range_over(false);
    p.set_time_over(false);
    p.write(0x1B, 0x10);
    p.write(0x1B, 0x00);
    p.write(0x1C, 0x01);
    assert_eq!(p.read(0x34, 0, false), 0x10);
    assert_eq!(p.read(0x3E, 0, false), 0x11);
}

#[test]
fn e04_10_wrio_falling_edge() {
    let mut p = Ppu::new(Region::Ntsc);
    at(&mut p, 2, 10);
    p.wrio_write(0x80, 0x00);
    assert!(p.counter_latched());

    let mut p2 = Ppu::new(Region::Ntsc);
    at(&mut p2, 2, 10);
    p2.wrio_write(0x00, 0x80);
    p2.wrio_write(0x80, 0x80);
    p2.wrio_write(0x00, 0x00);
    assert!(!p2.counter_latched());

    let mut p3 = Ppu::new(Region::Ntsc);
    at(&mut p3, 2, 10);
    p3.wrio_write(0xFF, 0x7F);
    assert!(p3.counter_latched());
}

#[test]
fn e04_10_hash() {
    let mut p = Ppu::new(Region::Ntsc);
    // Stay on line 0: with the default (all zero) OAM, sprite evaluation from line 2 on
    // would already set range_over (E05-16 evaluates sprites in step_dot).
    at(&mut p, 0, 10);
    let mut h1 = Fnv64::default();
    p.hash_into(&mut h1);
    let hash_before = h1.finish();

    p.read(0x37, 0, true);
    let mut h2 = Fnv64::default();
    p.hash_into(&mut h2);
    assert_ne!(hash_before, h2.finish());

    let mut p2 = Ppu::new(Region::Ntsc);
    at(&mut p2, 0, 10);
    p2.read(0x37, 0, true);
    let mut h3 = Fnv64::default();
    p2.hash_into(&mut h3);
    assert_eq!(h2.finish(), h3.finish());

    p.set_range_over(true);
    let mut h4 = Fnv64::default();
    p.hash_into(&mut h4);
    assert_ne!(h2.finish(), h4.finish());

    p.set_time_over(true);
    let mut h5 = Fnv64::default();
    p.hash_into(&mut h5);
    assert_ne!(h4.finish(), h5.finish());
}
