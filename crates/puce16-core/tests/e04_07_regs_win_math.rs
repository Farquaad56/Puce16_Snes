use puce16_core::ppu::regs::Regs;
use puce16_core::hash::Fnv64;

#[test]
fn e04_07_win_sel() {
    let mut r = Regs::new();
    r.write(0x23, 0xA5);
    r.write(0x24, 0x5A);
    r.write(0x25, 0xFF);
    assert_eq!(r.win_sel[0], 0xA5);
    assert_eq!(r.win_sel[1], 0x5A);
    assert_eq!(r.win_sel[2], 0xFF);
}

#[test]
fn e04_07_wh() {
    let mut r = Regs::new();
    r.write(0x26, 1);
    r.write(0x27, 2);
    r.write(0x28, 3);
    r.write(0x29, 4);
    assert_eq!(r.wh[0], 1);
    assert_eq!(r.wh[1], 2);
    assert_eq!(r.wh[2], 3);
    assert_eq!(r.wh[3], 4);
}

#[test]
fn e04_07_win_log() {
    let mut r = Regs::new();
    r.write(0x2A, 0xE4);
    assert_eq!(r.win_log_bg, 0xE4);
    r.write(0x2B, 0xFE);
    assert_eq!(r.win_log_obj, 2);
    assert_eq!(r.win_log_col, 3);
    r.write(0x2B, 0x04);
    assert_eq!(r.win_log_obj, 0);
    assert_eq!(r.win_log_col, 1);
}

#[test]
fn e04_07_layer_masks() {
    let mut r = Regs::new();
    r.write(0x2C, 0xFF);
    assert_eq!(r.tm, 0x1F);
    r.write(0x2D, 0xE5);
    assert_eq!(r.ts, 0x05);
    r.write(0x2E, 0xBF);
    assert_eq!(r.tmw, 0x1F);
    r.write(0x2F, 0x81);
    assert_eq!(r.tsw, 0x01);
}

#[test]
fn e04_07_cgwsel() {
    let mut r = Regs::new();
    r.write(0x30, 0xF3);
    assert!(r.direct_color);
    assert!(r.addend_subscreen);
    assert_eq!(r.math_region, 3);
    assert_eq!(r.black_region, 3);

    r.write(0x30, 0x44);
    assert!(!r.direct_color);
    assert!(!r.addend_subscreen);
    assert_eq!(r.math_region, 0);
    assert_eq!(r.black_region, 1);

    r.write(0x30, 0x24);
    assert_eq!(r.math_region, 2);
    assert_eq!(r.black_region, 0);
}

#[test]
fn e04_07_cgadsub() {
    let mut r = Regs::new();
    r.write(0x31, 0xE5);
    assert_eq!(r.math_mask, 0x25);
    assert!(r.math_half);
    assert!(r.math_sub);

    r.write(0x31, 0x3F);
    assert_eq!(r.math_mask, 0x3F);
    assert!(!r.math_half);
    assert!(!r.math_sub);

    r.write(0x31, 0x40);
    assert_eq!(r.math_mask, 0);
    assert!(r.math_half);
    assert!(!r.math_sub);
}

#[test]
fn e04_07_coldata() {
    let mut r = Regs::new();
    r.write(0x32, 0x2A);
    assert_eq!(r.fixed_r, 10);
    assert_eq!(r.fixed_g, 0);
    assert_eq!(r.fixed_b, 0);

    r.write(0x32, 0x5F);
    assert_eq!(r.fixed_r, 10);
    assert_eq!(r.fixed_g, 31);
    assert_eq!(r.fixed_b, 0);

    r.write(0x32, 0x85);
    assert_eq!(r.fixed_r, 10);
    assert_eq!(r.fixed_g, 31);
    assert_eq!(r.fixed_b, 5);

    r.write(0x32, 0xFF);
    assert_eq!(r.fixed_r, 31);
    assert_eq!(r.fixed_g, 31);
    assert_eq!(r.fixed_b, 31);

    r.write(0x32, 0xC1);
    assert_eq!(r.fixed_r, 31);
    assert_eq!(r.fixed_g, 1);
    assert_eq!(r.fixed_b, 1);
}

#[test]
fn e04_07_returns() {
    let mut r = Regs::new();
    for reg in 0x23..=0x32 {
        assert!(r.write(reg, 0), "write(0x{:02X}, 0) should return true", reg);
    }
    assert!(!r.write(0x33, 1));
    assert!(!r.write(0x22, 1));
}

#[test]
fn e04_07_hash() {
    let r = Regs::new();
    let base = hash(&r);

    let mut r1 = Regs::new();
    r1.write(0x23, 1);
    assert_ne!(base, hash(&r1));

    let mut r2 = Regs::new();
    r2.write(0x26, 1);
    assert_ne!(base, hash(&r2));

    let mut r3 = Regs::new();
    r3.write(0x2A, 1);
    assert_ne!(base, hash(&r3));

    let mut r4 = Regs::new();
    r4.write(0x2C, 1);
    assert_ne!(base, hash(&r4));

    let mut r5 = Regs::new();
    r5.write(0x30, 1);
    assert_ne!(base, hash(&r5));

    let mut r6 = Regs::new();
    r6.write(0x31, 1);
    assert_ne!(base, hash(&r6));

    let mut r7 = Regs::new();
    r7.write(0x32, 0x21);
    assert_ne!(base, hash(&r7));
}

fn hash(r: &Regs) -> u64 {
    let mut h = Fnv64::new();
    r.hash_into(&mut h);
    h.finish()
}
