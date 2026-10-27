use puce16_core::ppu::{Ppu, PpuRegsSnapshot};
use puce16_core::region::Region;
use puce16_core::hash::Fnv64;

fn hash_of(ppu: &Ppu) -> u64 {
    let mut h = Fnv64::new();
    ppu.hash_into(&mut h);
    h.finish()
}

#[test]
fn e04_12_snapshot_power_on() {
    let ppu = Ppu::new(Region::Ntsc);
    let s = ppu.regs_snapshot();
    assert_eq!(s.v, 0);
    assert_eq!(s.dot, 0);
    assert!(!s.field);
    assert!(s.forced_blank);
    assert_eq!(s.brightness, 0);
    assert_eq!(s.bg_mode, 0);
    assert_eq!(s.tm, 0);
    assert_eq!(s.vmadd, 0);
    assert_eq!(s.oam_internal, 0);
    for &m in s.m7.iter() {
        assert_eq!(m, 0);
    }
    for &f in s.fixed.iter() {
        assert_eq!(f, 0);
    }
}

#[test]
fn e04_12_snapshot_values() {
    let mut ppu = Ppu::new(Region::Ntsc);
    // brightness=15, forced_blank=false (bit7 not set)
    ppu.write(0x00, 0x0F);
    // bg_mode=7
    ppu.write(0x05, 0x07);
    // tm=0x13
    ppu.write(0x2C, 0x13);
    // fixed_r=10 (bit5 set), color index 10
    ppu.write(0x32, 0x2A);
    // map_base[0] = ((0x7B >> 2) << 10) = 0x7800
    ppu.write(0x07, 0x7B);
    // vmadd = 0x1234
    ppu.write(0x16, 0x34);
    ppu.write(0x17, 0x12);
    // M7A: latch 0x00 then write with high byte 0x01 -> a = 256
    ppu.write(0x1B, 0x00);
    ppu.write(0x1B, 0x01);

    let s = ppu.regs_snapshot();
    assert_eq!(s.brightness, 15);
    assert!(!s.forced_blank);
    assert_eq!(s.bg_mode, 7);
    assert_eq!(s.tm, 0x13);
    assert_eq!(s.fixed[0], 10);
    assert_eq!(s.fixed[1], 0);
    assert_eq!(s.fixed[2], 0);
    assert_eq!(s.map_base[0], 0x7800);
    assert_eq!(s.vmadd, 0x1234);
    assert_eq!(s.m7[0], 256);
}

#[test]
fn e04_12_reset_keeps_memories() {
    let mut ppu = Ppu::new(Region::Ntsc);
    // Write VRAM word 5 low byte = 0x7B (high stays 0)
    ppu.write(0x16, 0x05);
    ppu.write(0x18, 0x7B);
    // Write CGRAM word 2 = 0x0155
    ppu.write(0x21, 2);
    ppu.write(0x22, 0x55);
    ppu.write(0x22, 0x01);
    // Write OAM bytes 0,1 = 0x11, 0x22 (word write protocol)
    ppu.write(0x04, 0x11);
    ppu.write(0x04, 0x22);

    ppu.reset();

    assert_eq!(ppu.vram()[5], 0x007B);
    assert_eq!(ppu.cgram()[2], 0x0155);
    assert_eq!(ppu.oam()[0], 0x11);
    assert_eq!(ppu.oam()[1], 0x22);
}

#[test]
fn e04_12_reset_clears_registers() {
    let mut ppu = Ppu::new(Region::Ntsc);
    // Write various registers and step some dots
    ppu.write(0x00, 0x0F);
    ppu.write(0x05, 0x07);
    ppu.write(0x15, 0x80);
    ppu.write(0x16, 0x34);
    ppu.write(0x2C, 0x1F);
    ppu.write(0x33, 0x4F);
    ppu.write(0x0D, 0x12);
    for _ in 0..12345 {
        ppu.step_dot();
    }

    let fresh = Ppu::new(Region::Ntsc);
    let fresh_snap = fresh.regs_snapshot();

    ppu.reset();
    let snap = ppu.regs_snapshot();

    assert_eq!(snap, fresh_snap);
    assert_eq!(ppu.ppu1_mdr(), 0);
    assert_eq!(ppu.ppu2_mdr(), 0);
    assert!(!ppu.counter_latched());
    assert_eq!(ppu.v(), 0);
    assert_eq!(ppu.dot(), 0);
}

#[test]
fn e04_12_reset_keeps_region() {
    let mut ppu = Ppu::new(Region::Pal);
    ppu.reset();
    // STAT78 bit 4 indicates PAL (value 0x13 for PAL, 0x03 for NTSC)
    assert_eq!(ppu.read(0x3F, 0, false), 0x13);
    assert_eq!(ppu.last_line(), 311);
}

#[test]
fn e04_12_hash_covers_everything() {
    let base = Ppu::new(Region::Ntsc);
    let base_hash = hash_of(&base);

    // CGRAM address change
    let mut p1 = base.clone();
    p1.write(0x21, 1);
    assert_ne!(hash_of(&p1), base_hash, "CGRAM addr should affect hash");

    // VRAM address change
    let mut p2 = base.clone();
    p2.write(0x16, 1);
    assert_ne!(hash_of(&p2), base_hash, "VRAM addr should affect hash");

    // CGRAM data latch (write alone)
    let mut p3 = base.clone();
    p3.write(0x22, 5);
    assert_ne!(hash_of(&p3), base_hash, "CGRAM data write should affect hash");

    // Counter latch via read STAT74 with WIO bit
    let mut p4 = base.clone();
    p4.read(0x37, 0, true);
    assert_ne!(hash_of(&p4), base_hash, "Counter latch should affect hash");

    // Range over flag
    let mut p5 = base.clone();
    p5.set_range_over(true);
    assert_ne!(hash_of(&p5), base_hash, "Range over should affect hash");

    // SETINI extbg bit
    let mut p6 = base.clone();
    p6.write(0x33, 0x40);
    assert_ne!(hash_of(&p6), base_hash, "SETINI extbg should affect hash");

    // BG1HOFS (also affects M7 scroll)
    let mut p7 = base.clone();
    p7.write(0x0D, 3);
    assert_ne!(hash_of(&p7), base_hash, "BG1HOFS should affect hash");

    // M7X center X low byte
    let mut p8 = base.clone();
    p8.write(0x1F, 3);
    assert_ne!(hash_of(&p8), base_hash, "M7X should affect hash");

    // TM register
    let mut p9 = base.clone();
    p9.write(0x2C, 1);
    assert_ne!(hash_of(&p9), base_hash, "TM should affect hash");

    // OBJ mode/size
    let mut p10 = base.clone();
    p10.write(0x01, 0x20);
    assert_ne!(hash_of(&p10), base_hash, "OBJ mode should affect hash");

    // Brightness bit
    let mut p11 = base.clone();
    p11.write(0x00, 0x01);
    assert_ne!(hash_of(&p11), base_hash, "Brightness should affect hash");

    // One dot step
    let mut p12 = base.clone();
    p12.step_dot();
    assert_ne!(hash_of(&p12), base_hash, "Dot step should affect hash");

    // OAM address low byte
    let mut p13 = base.clone();
    p13.write(0x02, 5);
    assert_ne!(hash_of(&p13), base_hash, "OAM addr should affect hash");
}

#[test]
fn e04_12_hash_after_reset_equals_new_except_memories() {
    // Write only registers (no memory data) then reset; should match fresh PPU
    let mut ppu = Ppu::new(Region::Ntsc);
    ppu.write(0x05, 0x07);   // bg_mode
    ppu.write(0x2C, 0x1F);   // TM
    ppu.write(0x33, 0x4F);   // SETINI

    let fresh = Ppu::new(Region::Ntsc);
    let fresh_hash = hash_of(&fresh);

    ppu.reset();
    assert_eq!(hash_of(&ppu), fresh_hash, "Reset should restore power-on state when only registers were modified");
}

#[test]
fn e04_12_reset_idempotent() {
    let mut ppu = Ppu::new(Region::Ntsc);
    // Modify some state first
    ppu.write(0x05, 0x07);
    for _ in 0..100 {
        ppu.step_dot();
    }

    ppu.reset();
    let hash1 = hash_of(&ppu);

    ppu.reset();
    let hash2 = hash_of(&ppu);

    assert_eq!(hash1, hash2, "Double reset should equal single reset");
}
