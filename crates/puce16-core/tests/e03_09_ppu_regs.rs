use puce16_core::region::Region;
use puce16_core::ppu::{Ppu, DotEvents};
use puce16_core::hash::Fnv64;

#[test]
fn e03_09_inidisp() {
    let mut p = Ppu::new(Region::Ntsc);
    assert!(p.forced_blank());
    p.write(0x00, 0x0F);
    assert!(!p.forced_blank());
    p.write(0x00, 0x8F);
    assert!(p.forced_blank());
}

#[test]
fn e03_09_setini_flags() {
    let mut p = Ppu::new(Region::Ntsc);
    p.write(0x33, 0x05);
    assert!(p.interlace());
    assert!(p.overscan());
    assert_eq!(p.vblank_start(), 240);
    p.write(0x33, 0x00);
    assert!(!p.interlace());
    assert!(!p.overscan());
    assert_eq!(p.vblank_start(), 225);
    p.write(0x33, 0x04);
    assert!(!p.interlace());
    assert!(p.overscan());
}

#[test]
fn e03_09_ignored_regs() {
    let mut p = Ppu::new(Region::Ntsc);
    let mut h1 = Fnv64::new();
    p.hash_into(&mut h1);
    let hash_before = h1.finish();

    // Read-only / unused registers: writes are ignored (E04 made $2101 and
    // $212F real registers, so they are no longer part of this test).
    p.write(0x34, 0xFF);
    p.write(0x3E, 0xFF);
    p.write(0x3F, 0xFF);

    let mut h2 = Fnv64::new();
    p.hash_into(&mut h2);
    let hash_after = h2.finish();

    assert_eq!(hash_before, hash_after);
}

#[test]
fn e03_09_read_open_bus() {
    let mut p = Ppu::new(Region::Ntsc);
    assert_eq!(p.read(0x37, 0xAB, true), 0xAB);
    // $2100 is write-only: reads return the CPU open bus (E04 made $213F a real register).
    assert_eq!(p.read(0x00, 0x12, false), 0x12);
}

#[test]
fn e03_09_interlace_ntsc_frames() {
    let mut p = Ppu::new(Region::Ntsc);
    p.write(0x33, 0x01);

    // Frame 0 (field false): last_line = 262
    assert_eq!(p.last_line(), 262);
    let mut total_clocks: u32 = 0;
    for _ in 0..(263 * 340) {
        total_clocks += p.dot_clocks();
        p.step_dot();
    }
    assert_eq!(total_clocks, 358732);
    assert_eq!(p.v(), 0);
    assert!(p.field());

    // Frame 1 (field true): last_line = 261
    assert_eq!(p.last_line(), 261);
    total_clocks = 0;
    for _ in 0..(262 * 340) {
        total_clocks += p.dot_clocks();
        p.step_dot();
    }
    assert_eq!(total_clocks, 357368);
}

#[test]
fn e03_09_interlace_pal_frames() {
    let mut p = Ppu::new(Region::Pal);
    p.write(0x33, 0x01);

    // Frame 0 (field false): last_line = 312
    assert_eq!(p.last_line(), 312);
    let mut total_clocks: u32 = 0;
    for _ in 0..(313 * 340) {
        total_clocks += p.dot_clocks();
        p.step_dot();
    }
    assert_eq!(total_clocks, 426932);

    // Frame 1 (field true): last_line = 311
    assert_eq!(p.last_line(), 311);
    total_clocks = 0;
    for _ in 0..(312 * 340 + 1) {
        total_clocks += p.dot_clocks();
        p.step_dot();
    }
    assert_eq!(total_clocks, 425572);
}

#[test]
fn e03_09_overscan_vblank_event() {
    let mut p = Ppu::new(Region::Ntsc);
    p.write(0x33, 0x04);

    for _ in 0..(240 * 340) {
        p.step_dot();
    }
    assert_eq!(p.v(), 240);
    let events = p.step_dot();
    assert!(events.has(DotEvents::VBLANK_START));

    // Verify in_vblank at v=239 vs v=240
    let mut p2 = Ppu::new(Region::Ntsc);
    p2.write(0x33, 0x04);
    for _ in 0..(239 * 340) {
        p2.step_dot();
    }
    assert_eq!(p2.v(), 239);
    assert!(!p2.in_vblank());
    for _ in 0..340 {
        p2.step_dot();
    }
    assert_eq!(p2.v(), 240);
    assert!(p2.in_vblank());
}

#[test]
fn e03_09_hash() {
    let mut h1 = Fnv64::new();
    Ppu::new(Region::Ntsc).hash_into(&mut h1);
    let hash_ntsc = h1.finish();

    let mut h2 = Fnv64::new();
    Ppu::new(Region::Ntsc).hash_into(&mut h2);
    assert_eq!(h2.finish(), hash_ntsc);

    let mut p3 = Ppu::new(Region::Ntsc);
    p3.step_dot();
    let mut h3 = Fnv64::new();
    p3.hash_into(&mut h3);
    assert_ne!(h3.finish(), hash_ntsc);

    let mut h4 = Fnv64::new();
    Ppu::new(Region::Pal).hash_into(&mut h4);
    assert_ne!(h4.finish(), hash_ntsc);
}
