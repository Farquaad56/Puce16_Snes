use puce16_core::ppu::regs::Regs;
use puce16_core::hash::Fnv64;

#[test]
fn e04_06_power_on() {
    let r = Regs::new();
    assert_eq!(r.obj_size, 0);
    assert_eq!(r.obj_name_gap, 0);
    assert_eq!(r.bg_mode, 0);
    assert_eq!(r.mosaic_size(), 1);
    for i in 0..4 {
        assert_eq!(r.map_base[i], 0);
        assert_eq!(r.chr_base[i], 0);
        assert!(!r.tile16[i]);
    }
}

#[test]
fn e04_06_objsel() {
    let mut r = Regs::new();
    // write(0x01, 0xE3): obj_size=7, gap_words=1 (bits 4-3=00), base=3<<13=0x6000
    r.write(0x01, 0xE3);
    assert_eq!(r.obj_size, 7);
    assert_eq!(r.obj_name_gap, 0x1000);
    assert_eq!(r.obj_base, 0x6000);

    // write(0x01, 0x5A): obj_size=2, gap_words=4 (bits 4-3=11), base=2<<13=0x4000
    r.write(0x01, 0x5A);
    assert_eq!(r.obj_size, 2);
    assert_eq!(r.obj_name_gap, 0x4000);
    assert_eq!(r.obj_base, 0x4000);

    // write(0x01, 0x00): obj_size=0, gap_words=1, base=0
    r.write(0x01, 0x00);
    assert_eq!(r.obj_size, 0);
    assert_eq!(r.obj_name_gap, 0x1000);
    assert_eq!(r.obj_base, 0);
}

#[test]
fn e04_06_bgmode() {
    let mut r = Regs::new();
    // write(0x05, 0xB5): mode=5, bg3_priority=false (bit3=0), tile16=[T,T,F,T]
    r.write(0x05, 0xB5);
    assert_eq!(r.bg_mode, 5);
    assert!(!r.bg3_priority);
    assert!(r.tile16[0]); // BG1 bit4=1
    assert!(r.tile16[1]); // BG2 bit5=1
    assert!(!r.tile16[2]); // BG3 bit6=0
    assert!(r.tile16[3]); // BG4 bit7=1

    // write(0x05, 0x09): mode=1, bg3_priority=true (bit3=1), tile16=[F,F,F,F]
    r.write(0x05, 0x09);
    assert_eq!(r.bg_mode, 1);
    assert!(r.bg3_priority);
    for i in 0..4 {
        assert!(!r.tile16[i]);
    }

    // write(0x05, 0x07): mode=7
    r.write(0x05, 0x07);
    assert_eq!(r.bg_mode, 7);
}

#[test]
fn e04_06_mosaic() {
    let mut r = Regs::new();
    // write(0x06, 0x35): raw=3, size=4, en=5
    r.write(0x06, 0x35);
    assert_eq!(r.mosaic_raw, 3);
    assert_eq!(r.mosaic_size(), 4);
    assert_eq!(r.mosaic_en, 5);

    // write(0x06, 0xF0): raw=15, size=16, en=0
    r.write(0x06, 0xF0);
    assert_eq!(r.mosaic_size(), 16);
    assert_eq!(r.mosaic_en, 0);

    // write(0x06, 0x0F): raw=0, size=1, en=0x0F
    r.write(0x06, 0x0F);
    assert_eq!(r.mosaic_size(), 1);
    assert_eq!(r.mosaic_en, 0x0F);
}

#[test]
fn e04_06_bgsc() {
    let mut r = Regs::new();
    // write(0x07, 0x7B): map_base[0]=0x7800, x2=true, y2=true
    r.write(0x07, 0x7B);
    assert_eq!(r.map_base[0], 0x7800);
    assert!(r.map_x2[0]);
    assert!(r.map_y2[0]);

    // write(0x08, 0x04): map_base[1]=0x0400, x2=false, y2=false
    r.write(0x08, 0x04);
    assert_eq!(r.map_base[1], 0x0400);
    assert!(!r.map_x2[1]);
    assert!(!r.map_y2[1]);

    // write(0x09, 0xFC): map_base[2]=0xFC00
    r.write(0x09, 0xFC);
    assert_eq!(r.map_base[2], 0xFC00);

    // write(0x0A, 0x02): map_base[3]=0, x2=false, y2=true
    r.write(0x0A, 0x02);
    assert_eq!(r.map_base[3], 0);
    assert!(!r.map_x2[3]);
    assert!(r.map_y2[3]);
}

#[test]
fn e04_06_nba() {
    let mut r = Regs::new();
    // write(0x0B, 0x42): chr_base[0]=0x2000, chr_base[1]=0x4000
    r.write(0x0B, 0x42);
    assert_eq!(r.chr_base[0], 0x2000);
    assert_eq!(r.chr_base[1], 0x4000);

    // write(0x0C, 0xF1): chr_base[2]=0x1000, chr_base[3]=0xF000
    r.write(0x0C, 0xF1);
    assert_eq!(r.chr_base[2], 0x1000);
    assert_eq!(r.chr_base[3], 0xF000);
}

#[test]
fn e04_06_return_value() {
    let mut r = Regs::new();
    // Handled registers return true
    assert!(r.write(0x01, 0));
    assert!(r.write(0x05, 0));
    assert!(r.write(0x0C, 0));

    // Unhandled registers return false and don't change state
    let mut h = Fnv64::new();
    r.hash_into(&mut h);
    let hash_before = h.finish();

    assert!(!r.write(0x00, 0xFF));
    assert!(!r.write(0x0D, 1));
    assert!(!r.write(0x33, 1));

    let mut h2 = Fnv64::new();
    r.hash_into(&mut h2);
    let hash_after = h2.finish();
    assert_eq!(hash_before, hash_after);
}

#[test]
fn e04_06_hash() {
    // Two fresh instances have same hash
    let r1 = Regs::new();
    let r2 = Regs::new();
    let mut h1 = Fnv64::new();
    r1.hash_into(&mut h1);
    let mut h2 = Fnv64::new();
    r2.hash_into(&mut h2);
    assert_eq!(h1.finish(), h2.finish());

    // Different after each write
    let base_hash = h1.finish();

    let mut r3 = Regs::new();
    r3.write(0x01, 0x20);
    let mut h3 = Fnv64::new();
    r3.hash_into(&mut h3);
    assert_ne!(base_hash, h3.finish());

    let mut r4 = Regs::new();
    r4.write(0x05, 0x01);
    let mut h4 = Fnv64::new();
    r4.hash_into(&mut h4);
    assert_ne!(base_hash, h4.finish());

    let mut r5 = Regs::new();
    r5.write(0x06, 0x10);
    let mut h5 = Fnv64::new();
    r5.hash_into(&mut h5);
    assert_ne!(base_hash, h5.finish());

    let mut r6 = Regs::new();
    r6.write(0x07, 0x04);
    let mut h6 = Fnv64::new();
    r6.hash_into(&mut h6);
    assert_ne!(base_hash, h6.finish());

    let mut r7 = Regs::new();
    r7.write(0x0B, 0x01);
    let mut h7 = Fnv64::new();
    r7.hash_into(&mut h7);
    assert_ne!(base_hash, h7.finish());
}
