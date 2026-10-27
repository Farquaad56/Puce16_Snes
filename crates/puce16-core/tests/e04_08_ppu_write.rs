use puce16_core::region::Region;
use puce16_core::ppu::Ppu;
use puce16_core::hash::Fnv64;

fn at(p: &mut Ppu, v: u32, dot: u32) {
    for _ in 0..(v * 340 + dot) {
        p.step_dot();
    }
}

#[test]
fn e04_08_inidisp() {
    let mut p = Ppu::new(Region::Ntsc);
    assert_eq!(p.brightness(), 0);
    assert!(p.forced_blank());

    p.write(0x00, 0x0F);
    assert_eq!(p.brightness(), 15);
    assert!(!p.forced_blank());

    p.write(0x00, 0x85);
    assert_eq!(p.brightness(), 5);
    assert!(p.forced_blank());
}

#[test]
fn e04_08_vram_write_forced_blank() {
    let mut p = Ppu::new(Region::Ntsc);
    // forced blank is true by default, so VRAM writes are allowed
    p.write(0x15, 0x80);
    p.write(0x16, 0x00);
    p.write(0x17, 0x01);
    p.write(0x18, 0xCD);
    p.write(0x19, 0xAB);
    assert_eq!(p.vram()[0x100], 0xABCD);
    assert_eq!(p.vram_unit().vmadd(), 0x101);
}

#[test]
fn e04_08_vram_ok_boundaries() {
    // Position (0, 0): not in vblank
    let mut p1 = Ppu::new(Region::Ntsc);
    p1.write(0x00, 0x0F);
    at(&mut p1, 0, 0);
    assert!(!p1.vram_ok());
    assert!(!p1.oam_ok());

    // Position (224, 339): still not in vblank
    let mut p2 = Ppu::new(Region::Ntsc);
    p2.write(0x00, 0x0F);
    at(&mut p2, 224, 339);
    assert!(!p2.vram_ok());

    // Position (225, 0): in vblank
    let mut p3 = Ppu::new(Region::Ntsc);
    p3.write(0x00, 0x0F);
    at(&mut p3, 225, 0);
    assert!(p3.vram_ok());
    assert!(p3.oam_ok());

    // Overscan: vblank starts at 240
    let mut p4 = Ppu::new(Region::Ntsc);
    p4.write(0x00, 0x0F);
    p4.write(0x33, 0x04);
    at(&mut p4, 239, 339);
    assert!(!p4.vram_ok());

    let mut p5 = Ppu::new(Region::Ntsc);
    p5.write(0x00, 0x0F);
    p5.write(0x33, 0x04);
    at(&mut p5, 240, 0);
    assert!(p5.vram_ok());

    // Forced blank overrides position check
    let mut p6 = Ppu::new(Region::Ntsc);
    at(&mut p6, 0, 0);
    assert!(p6.vram_ok());
}

#[test]
fn e04_08_vram_blocked_then_ok() {
    let mut p = Ppu::new(Region::Ntsc);
    // display on (forced blank off)
    p.write(0x00, 0x0F);
    p.write(0x15, 0x80);
    p.write(0x16, 0x00);
    p.write(0x17, 0x02); // address 0x200

    // write at v=0 is blocked (not in vblank)
    p.write(0x19, 0x12);
    assert_eq!(p.vram()[0x200], 0);
    assert_eq!(p.vram_unit().vmadd(), 0x201);

    // advance to v=225 (vblank)
    for _ in 0..(225 * 340) {
        p.step_dot();
    }

    // now writes are allowed
    p.write(0x18, 0x34);
    p.write(0x19, 0x56);
    assert_eq!(p.vram()[0x201], 0x5634);
    assert_eq!(p.vram()[0x200], 0);
}

#[test]
fn e04_08_oam_write() {
    let mut p = Ppu::new(Region::Ntsc);
    // forced blank is true by default, so OAM writes are allowed
    p.write(0x02, 0x00);
    p.write(0x03, 0x00);
    p.write(0x04, 0x11);
    p.write(0x04, 0x22);
    assert_eq!(p.oam()[0], 0x11);
    assert_eq!(p.oam()[1], 0x22);

    // display on at v=0: OAM writes are blocked
    p.write(0x00, 0x0F);
    p.write(0x02, 0x00);
    p.write(0x04, 0x33);
    p.write(0x04, 0x44);
    assert_eq!(p.oam()[0], 0x11); // unchanged from previous write
    assert_eq!(p.oam_unit().internal(), 2);
}

#[test]
fn e04_08_cgram_write() {
    let mut p = Ppu::new(Region::Ntsc);
    p.write(0x21, 3);
    p.write(0x22, 0x1F);
    p.write(0x22, 0x03);
    assert_eq!(p.cgram()[3], 0x031F);
    assert_eq!(p.cgram_unit().addr(), 4);

    // CGRAM writes work even when display is on at v=0
    let mut p2 = Ppu::new(Region::Ntsc);
    p2.write(0x00, 0x0F);
    p2.write(0x21, 4);
    p2.write(0x22, 0x01);
    p2.write(0x22, 0x02);
    assert_eq!(p2.cgram()[4], 0x0201);
}

#[test]
fn e04_08_scroll_and_mode7() {
    let mut p = Ppu::new(Region::Ntsc);
    p.write(0x0D, 0x34);
    p.write(0x0D, 0x01);
    assert_eq!(p.scroll().hofs(0), 0x134);
    assert_eq!(p.m7().hofs, 308);

    p.write(0x0E, 0x34);
    p.write(0x0E, 0x02);
    assert_eq!(p.scroll().vofs(0), 0x234);
    assert_eq!(p.m7().vofs, 564);

    // BG3 scroll (no mode 7)
    p.write(0x11, 0x34);
    p.write(0x11, 0x01);
    assert_eq!(p.scroll().hofs(2), 0x134);

    // BG4 scroll (no mode 7)
    p.write(0x14, 0x34);
    p.write(0x14, 0x02);
    assert_eq!(p.scroll().vofs(3), 0x234);
    assert_eq!(p.m7().vofs, 564); // unchanged
}

#[test]
fn e04_08_mode7_regs() {
    let mut p = Ppu::new(Region::Ntsc);
    p.write(0x1B, 0x00);
    p.write(0x1B, 0x01);
    assert_eq!(p.m7().a, 256);

    p.write(0x1F, 0x34);
    p.write(0x1F, 0x01);
    assert_eq!(p.m7().cx, 308);

    p.write(0x1A, 0xC3);
    assert!(p.m7().flip_x);
    assert!(p.m7().flip_y);
    assert!(p.m7().fill);
    assert!(p.m7().outside);
}

#[test]
fn e04_08_config_regs() {
    let mut p = Ppu::new(Region::Ntsc);
    p.write(0x05, 0xB5);
    assert_eq!(p.regs().bg_mode, 5);

    p.write(0x07, 0x7B);
    assert_eq!(p.regs().map_base[0], 0x7800);

    p.write(0x2C, 0xFF);
    assert_eq!(p.regs().tm, 0x1F);

    p.write(0x30, 0xF3);
    assert!(p.regs().direct_color);

    p.write(0x32, 0xFF);
    assert_eq!(p.regs().fixed_r, 31);
}

#[test]
fn e04_08_write_keeps_position() {
    let mut p = Ppu::new(Region::Ntsc);
    at(&mut p, 3, 100);

    // write to various registers
    p.write(0x00, 0x0F);
    p.write(0x05, 0xB5);
    p.write(0x15, 0x80);
    p.write(0x18, 0xCD);
    p.write(0x22, 0xAB);
    p.write(0x33, 0x04);

    assert_eq!(p.v(), 3);
    assert_eq!(p.dot(), 100);
    assert!(!p.field());
}

#[test]
fn e04_08_ignored_regs() {
    let mut p = Ppu::new(Region::Ntsc);
    let base_hash = hash(&p);

    p.write(0x34, 0xFF);
    p.write(0x3F, 0xFF);
    p.write(0xFF, 0xFF);
    assert_eq!(hash(&p), base_hash);

    // reads return open bus (cpu_mdr)
    let r1 = p.read(0x05, 0xAB, false);
    assert_eq!(r1, 0xAB);
    let r2 = p.read(0x37, 0xCD, true);
    assert_eq!(r2, 0xCD);
}

#[test]
fn e04_08_hash() {
    let p = Ppu::new(Region::Ntsc);
    let base = hash(&p);

    // brightness change
    let mut p1 = Ppu::new(Region::Ntsc);
    p1.write(0x00, 0x05);
    assert_ne!(base, hash(&p1));

    // VRAM write (forced blank)
    let mut p2 = Ppu::new(Region::Ntsc);
    p2.write(0x15, 0x80);
    p2.write(0x16, 0x00);
    p2.write(0x17, 0x00);
    p2.write(0x18, 0xFF);
    assert_ne!(base, hash(&p2));

    // CGRAM write
    let mut p3 = Ppu::new(Region::Ntsc);
    p3.write(0x21, 0);
    p3.write(0x22, 1);
    assert_ne!(base, hash(&p3));

    // OAM write
    let mut p4 = Ppu::new(Region::Ntsc);
    p4.write(0x02, 0);
    p4.write(0x03, 0);
    p4.write(0x04, 1);
    assert_ne!(base, hash(&p4));

    // scroll write
    let mut p5 = Ppu::new(Region::Ntsc);
    p5.write(0x0D, 1);
    assert_ne!(base, hash(&p5));

    // mode7 write
    let mut p6 = Ppu::new(Region::Ntsc);
    p6.write(0x1A, 1);
    assert_ne!(base, hash(&p6));

    // regs write
    let mut p7 = Ppu::new(Region::Ntsc);
    p7.write(0x05, 1);
    assert_ne!(base, hash(&p7));

    // two PPUs with same writes have same hash
    let mut a = Ppu::new(Region::Ntsc);
    let mut b = Ppu::new(Region::Ntsc);
    for reg in [0x00u8, 0x05, 0x15, 0x18, 0x22] {
        a.write(reg, 0xAB);
        b.write(reg, 0xAB);
    }
    assert_eq!(hash(&a), hash(&b));
}

fn hash(p: &Ppu) -> u64 {
    let mut h = Fnv64::new();
    p.hash_into(&mut h);
    h.finish()
}
