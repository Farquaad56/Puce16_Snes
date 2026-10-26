//! E04-03: OAM module - address registers, word-write protocol, high table, reads.

use puce16_core::hash::Fnv64;
use puce16_core::ppu::oam::Oam;

#[test]
fn e04_03_addr_regs() {
    let mut o = Oam::new();
    // OAMADDL sets lower 8 bits, recalculates internal
    o.set_addr_l(0x05);
    assert_eq!(o.internal(), 0x0A);
    assert_eq!(o.oamadd_reg(), 5);

    // OAMADDH bit 0 -> bit 8 of address register
    o.set_addr_h(0x01);
    assert_eq!(o.oamadd_reg(), 0x105);
    assert_eq!(o.internal(), 0x20A);

    // OAMADDH bit 7 -> rotation flag
    o.set_addr_h(0x80);
    assert_eq!(o.oamadd_reg(), 5);
    assert_eq!(o.internal(), 0x0A);
    assert!(o.rotation());

    o.set_addr_h(0x00);
    assert!(!o.rotation());
}

#[test]
fn e04_03_word_write() {
    let mut o = Oam::new();
    o.set_addr_l(0); // internal = 0 (even)
    o.write(0x11, true);
    assert_eq!(o.byte(0), 0); // only latched, not written yet
    assert_eq!(o.internal(), 1);

    o.write(0x22, true); // odd address: write word
    assert_eq!(o.byte(0), 0x11);
    assert_eq!(o.byte(1), 0x22);
    assert_eq!(o.internal(), 2);
}

#[test]
fn e04_03_sequence() {
    let mut o = Oam::new();
    o.set_addr_l(0);
    o.write(0x11, true); // latch at internal=0
    o.write(0x22, true); // write word at 0-1, internal now 2
    o.write(0x33, true); // latch at internal=2
    o.write(0x44, true); // write word at 2-3, internal now 4

    assert_eq!(o.byte(0), 0x11);
    assert_eq!(o.byte(1), 0x22);
    assert_eq!(o.byte(2), 0x33);
    assert_eq!(o.byte(3), 0x44);
    assert_eq!(o.internal(), 4);
}

#[test]
fn e04_03_odd_start() {
    // Case 1: write at odd address, then reposition to even
    let mut o = Oam::new();
    o.set_addr_l(0);
    o.write(0x55, true); // latch at internal=0, now internal=1 (odd)

    // Reposition to even address - latch is preserved but overwritten by next write
    o.set_addr_l(0); // internal back to 0

    o.write(0x66, true); // new latch at internal=0 (even), overwrites old latch
    o.write(0x77, true); // write word: byte(0)=0x66, byte(1)=0x77
    assert_eq!(o.byte(0), 0x66);
    assert_eq!(o.byte(1), 0x77);

    // Case 2: consecutive writes without repositioning
    let mut o2 = Oam::new();
    o2.set_addr_l(0);
    o2.write(0x55, true); // latch at internal=0, now internal=1 (odd)
    o2.write(0x66, true); // write word: byte(0)=0x55, byte(1)=0x66
    assert_eq!(o2.byte(0), 0x55);
    assert_eq!(o2.byte(1), 0x66);
}

#[test]
fn e04_03_high_table() {
    let mut o = Oam::new();
    // Set address to high table (bit 8 of oamadd_reg -> internal bit 9)
    o.set_addr_h(0x01); // oamadd_reg=0x100, internal=0x200

    o.write(0xAA, true);
    assert_eq!(o.byte(0x200), 0xAA);
    assert_eq!(o.internal(), 0x201);

    o.write(0xBB, true);
    assert_eq!(o.byte(0x201), 0xBB);
}

#[test]
fn e04_03_high_mirror() {
    let mut o = Oam::new();
    // High table mirrors every 32 bytes (5 bits)
    o.set_addr_h(0x01);
    o.set_addr_l(0x10); // internal = 0x220

    o.write(0xCC, true);
    assert_eq!(o.byte(0x200), 0xCC); // mirrored to 0x200 + (0x20 & 0x1F) = 0x200
    assert_eq!(o.internal(), 0x221);

    o.set_addr_l(0x11); // internal = 0x222
    o.write(0xDD, true);
    assert_eq!(o.byte(0x202), 0xDD); // mirrored to 0x200 + (0x22 & 0x1F) = 0x202
}

#[test]
fn e04_03_wrap_10bits() {
    let mut o = Oam::new();
    // Set internal near the end of 10-bit range (0x3FF)
    o.set_addr_l(0xFF);
    o.set_addr_h(0x01); // oamadd_reg=0x1FF, internal=(0x1FF<<1)&0x3FF = 0x3FE

    assert_eq!(o.internal(), 0x3FE);

    o.write(1, true);
    o.write(2, true);
    assert_eq!(o.byte(0x21E), 1); // high table: 0x200 + (0x3FE & 0x1F) = 0x21E
    assert_eq!(o.byte(0x21F), 2); // high table: 0x200 + (0x3FF & 0x1F) = 0x21F
    assert_eq!(o.internal(), 0); // wrapped around
}

#[test]
fn e04_03_read_low_table() {
    let mut o = Oam::new();
    o.set_byte(0, 0x10);
    o.set_byte(1, 0x20);

    o.set_addr_l(0); // internal = 0
    assert_eq!(o.read(true), 0x10);
    assert_eq!(o.read(true), 0x20);
    assert_eq!(o.internal(), 2);
}

#[test]
fn e04_03_read_high_table() {
    let mut o = Oam::new();
    o.set_byte(0x204, 0x66);
    o.set_byte(0x205, 0x77);

    // Read from high table
    o.set_addr_h(0x01);
    o.set_addr_l(0x02); // internal = 0x204
    assert_eq!(o.read(true), 0x66);
    assert_eq!(o.read(true), 0x77);
    assert_eq!(o.internal(), 0x206);

    // High table mirror read
    o.set_byte(0x200, 0x99);
    o.set_addr_l(0x10); // internal = 0x220 -> mirrors to 0x200
    assert_eq!(o.read(true), 0x99);
}

#[test]
fn e04_03_blocked() {
    let mut o = Oam::new();
    o.set_addr_l(0);

    // Write blocked (ok=false) at even address - latch not updated
    o.write(9, false);
    assert_eq!(o.byte(0), 0);
    assert_eq!(o.byte(1), 0);
    assert_eq!(o.internal(), 1);

    // Write allowed at odd address - writes word with old latch (0)
    o.write(0x31, true);
    assert_eq!(o.byte(0), 0); // latch was not updated by blocked write
    assert_eq!(o.byte(1), 0x31);

    // Read blocked returns 0 but still increments address
    o.set_byte(5, 0x5A);
    o.set_addr_l(2); // internal = 4
    assert_eq!(o.read(false), 0);
    assert_eq!(o.internal(), 5);
    assert_eq!(o.read(true), 0x5A);
}

#[test]
fn e04_03_reload() {
    let mut o = Oam::new();
    o.set_addr_l(0x07); // internal = 14 (0x0E)

    // Advance internal by writing
    o.write(1, true);
    o.write(2, true);
    o.write(3, true);
    assert_eq!(o.internal(), 17);

    // Reload resets to address register value
    o.reload();
    assert_eq!(o.internal(), 14);

    // With high bit set in address register
    o.set_addr_h(0x01); // oamadd_reg = 0x107
    o.reload();
    assert_eq!(o.internal(), 0x20E); // (0x107 & 0x1FF) << 1 = 0x20E
}

#[test]
fn e04_03_hash() {
    let mut h1 = Fnv64::new();
    let o1 = Oam::new();
    o1.hash_into(&mut h1);

    let mut h2 = Fnv64::new();
    let o2 = Oam::new();
    o2.hash_into(&mut h2);

    assert_eq!(h1.finish(), h2.finish()); // identical fresh OAMs hash the same

    // Different after modifying memory
    let mut o3 = Oam::new();
    o3.set_byte(7, 1);
    let mut h3 = Fnv64::new();
    o3.hash_into(&mut h3);
    assert_ne!(h1.finish(), h3.finish());

    // Different after modifying address register
    let mut o4 = Oam::new();
    o4.set_addr_l(1);
    let mut h4 = Fnv64::new();
    o4.hash_into(&mut h4);
    assert_ne!(h1.finish(), h4.finish());

    // Different after modifying rotation flag
    let mut o5 = Oam::new();
    o5.set_addr_h(0x80);
    let mut h5 = Fnv64::new();
    o5.hash_into(&mut h5);
    assert_ne!(h1.finish(), h5.finish());
}
