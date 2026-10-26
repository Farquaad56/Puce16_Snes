//! E04-04: PPU scroll registers - BGnHOFS/BGnVOFS with shared latches.

use puce16_core::hash::Fnv64;
use puce16_core::ppu::regs::Scroll;

#[test]
fn e04_04_power_on() {
    let s = Scroll::new();
    for n in 0..4 {
        assert_eq!(s.hofs(n), 0);
        assert_eq!(s.vofs(n), 0);
    }
    assert_eq!(s.hofs(4), 0);
}

#[test]
fn e04_04_hofs_two_writes() {
    let mut s = Scroll::new();
    s.write_hofs(0, 0x34);
    s.write_hofs(0, 0x01);
    assert_eq!(s.hofs(0), 0x134);
}

#[test]
fn e04_04_hofs_mask_10bits() {
    let mut s = Scroll::new();
    s.write_hofs(0, 0xFF);
    s.write_hofs(0, 0xFF);
    assert_eq!(s.hofs(0), 0x3FF);
}

#[test]
fn e04_04_hofs_latch_shared_between_bg() {
    let mut s = Scroll::new();
    s.write_hofs(0, 0x34);
    s.write_hofs(1, 0x01);
    assert_eq!(s.hofs(1), 0x134);
    assert_eq!(s.hofs(0), 0);
}

#[test]
fn e04_04_vofs_two_writes() {
    let mut s = Scroll::new();
    s.write_vofs(0, 0x34);
    s.write_vofs(0, 0x02);
    assert_eq!(s.vofs(0), 0x234);
}

#[test]
fn e04_04_vofs_other_bg() {
    let mut s = Scroll::new();
    s.write_vofs(3, 0xFF);
    s.write_vofs(3, 0x03);
    assert_eq!(s.vofs(3), 0x3FF);
    assert_eq!(s.vofs(0), 0);
}

#[test]
fn e04_04_hofs_uses_both_latches() {
    let mut s = Scroll::new();
    s.write_vofs(0, 0x35);
    s.write_hofs(0, 0x01);
    assert_eq!(s.hofs(0), 0x130);
}

#[test]
fn e04_04_latch_leftovers() {
    let mut s = Scroll::new();
    s.write_hofs(0, 0x07);
    s.write_vofs(0, 0x01);
    assert_eq!(s.vofs(0), 0x107);
    s.write_hofs(0, 0x00);
    assert_eq!(s.hofs(0), 0x007);
}

#[test]
fn e04_04_bad_index() {
    let mut s = Scroll::new();
    let mut h1 = Fnv64::new();
    s.hash_into(&mut h1);
    let before = h1.finish();

    s.write_hofs(4, 1);
    s.write_vofs(9, 1);

    let mut h2 = Fnv64::new();
    s.hash_into(&mut h2);
    assert_eq!(before, h2.finish());
}

#[test]
fn e04_04_hash() {
    let mut h1 = Fnv64::new();
    let s1 = Scroll::new();
    s1.hash_into(&mut h1);

    let mut h2 = Fnv64::new();
    let s2 = Scroll::new();
    s2.hash_into(&mut h2);
    assert_eq!(h1.finish(), h2.finish());

    let mut s3 = Scroll::new();
    s3.write_hofs(0, 1);
    let mut h3 = Fnv64::new();
    s3.hash_into(&mut h3);
    assert_ne!(h1.finish(), h3.finish());

    let mut s4 = Scroll::new();
    s4.write_vofs(2, 1);
    let mut h4 = Fnv64::new();
    s4.hash_into(&mut h4);
    assert_ne!(h1.finish(), h4.finish());
}
