use puce16_core::ppu::mode7::{sign_extend_13, M7Regs};
use puce16_core::hash::Fnv64;

#[test]
fn e04_05_sign_extend() {
    assert_eq!(sign_extend_13(0x0FFF), 4095);
    assert_eq!(sign_extend_13(0x1000), -4096);
    assert_eq!(sign_extend_13(0x1FFF), -1);
    assert_eq!(sign_extend_13(0x0000), 0);
    // bits 13-15 ignored: 0x2345 & 0x1FFF = 0x0345 -> sign extend positive
    assert_eq!(sign_extend_13(0x2345), 837);
    // 0x1234 & 0x1FFF = 0x1234, bit 12 set -> negative: 4660 - 8192 = -3532
    assert_eq!(sign_extend_13(0x1234), -3532);
}

#[test]
fn e04_05_sel_bits() {
    let mut m7 = M7Regs::new();
    m7.write_sel(0xC3);
    assert!(m7.flip_x);
    assert!(m7.flip_y);
    assert!(m7.fill);
    assert!(m7.outside);

    m7.write_sel(0x00);
    assert!(!m7.flip_x);
    assert!(!m7.flip_y);
    assert!(!m7.fill);
    assert!(!m7.outside);

    m7.write_sel(0x01);
    assert!(m7.flip_x);
    assert!(!m7.flip_y);
    assert!(!m7.fill);
    assert!(!m7.outside);

    m7.write_sel(0x02);
    assert!(!m7.flip_x);
    assert!(m7.flip_y);
    assert!(!m7.fill);
    assert!(!m7.outside);

    m7.write_sel(0x40);
    assert!(!m7.flip_x);
    assert!(!m7.flip_y);
    assert!(m7.fill);
    assert!(!m7.outside);

    m7.write_sel(0x80);
    assert!(!m7.flip_x);
    assert!(!m7.flip_y);
    assert!(!m7.fill);
    assert!(m7.outside);
}

#[test]
fn e04_05_matrix_two_writes() {
    let mut m7 = M7Regs::new();
    // First write: low byte 0x34, latch becomes 0x34
    m7.write_matrix(0, 0x34);
    // Second write: high byte 0x12, value = (0x12 << 8) | 0x34 = 0x1234
    m7.write_matrix(0, 0x12);
    assert_eq!(m7.a, 0x1234);

    // Negative: write 0x00 then 0x80 -> (0x80 << 8) | 0x00 = 0x8000 = -32768
    m7.write_matrix(0, 0x00);
    m7.write_matrix(0, 0x80);
    assert_eq!(m7.a, -32768);

    // Write 0xFF then 0xFF -> (0xFF << 8) | 0xFF = 0xFFFF = -1
    m7.write_matrix(0, 0xFF);
    m7.write_matrix(0, 0xFF);
    assert_eq!(m7.a, -1);
}

#[test]
fn e04_05_matrix_shared_latch() {
    let mut m7 = M7Regs::new();
    // Write to a: low byte 0x11, latch becomes 0x11
    m7.write_matrix(0, 0x11);
    // Write to b: high byte 0x22, value = (0x22 << 8) | 0x11 = 0x2211
    m7.write_matrix(1, 0x22);
    assert_eq!(m7.b, 0x2211);
    // a was only written once: value = (0x11 << 8) | 0 = 0x1100
    assert_eq!(m7.a, 0x1100);
}

#[test]
fn e04_05_matrix_cd() {
    let mut m7 = M7Regs::new();
    // Write c: low byte 0x01, then high byte 0x02 -> 0x0201
    m7.write_matrix(2, 0x01);
    m7.write_matrix(2, 0x02);
    assert_eq!(m7.c, 0x0201);

    // Write d: low byte 0x03, then high byte 0x04 -> 0x0403
    m7.write_matrix(3, 0x03);
    m7.write_matrix(3, 0x04);
    assert_eq!(m7.d, 0x0403);
}

#[test]
fn e04_05_center() {
    let mut m7 = M7Regs::new();
    // cx: low byte 0xFF, high byte 0x01 -> raw = (0x01 & 0x1F) << 8 | 0xFF = 0x01FF
    // sign_extend_13(0x01FF): bit 12 not set -> positive: 511
    m7.write_center(0, 0xFF);
    m7.write_center(0, 0x01);
    assert_eq!(m7.cx, 511);

    // cx: low byte 0x00, high byte 0x10 -> raw = (0x10 & 0x1F) << 8 | 0x00 = 0x1000
    // sign_extend_13(0x1000): bit 12 set -> -4096
    m7.write_center(0, 0x00);
    m7.write_center(0, 0x10);
    assert_eq!(m7.cx, -4096);

    // cx: low byte 0x34, high byte 0x1F -> raw = (0x1F & 0x1F) << 8 | 0x34 = 0x1F34
    // sign_extend_13(0x1F34): bit 12 set -> negative: 7988 - 8192 = -204
    m7.write_center(0, 0x34);
    m7.write_center(0, 0x1F);
    assert_eq!(m7.cx, -204);

    // cy: low byte 0x34, high byte 0x01 -> raw = (0x01 & 0x1F) << 8 | 0x34 = 0x0134
    // sign_extend_13(0x0134): bit 12 not set -> positive: 308
    m7.write_center(1, 0x34);
    m7.write_center(1, 0x01);
    assert_eq!(m7.cy, 308);
}

#[test]
fn e04_05_scroll() {
    let mut m7 = M7Regs::new();
    // hofs: low byte 0x34, high byte 0x01 -> raw = (0x01 << 8) | 0x34 = 0x0134
    // sign_extend_13(0x0134): bit 12 not set -> positive: 308
    m7.write_scroll(0, 0x34);
    m7.write_scroll(0, 0x01);
    assert_eq!(m7.hofs, 308);

    // hofs: low byte 0xFF, high byte 0x1F -> raw = (0x1F << 8) | 0xFF = 0x1FFF
    // sign_extend_13(0x1FFF): bit 12 set -> -1
    m7.write_scroll(0, 0xFF);
    m7.write_scroll(0, 0x1F);
    assert_eq!(m7.hofs, -1);

    // vofs: low byte 0x00, high byte 0x10 -> raw = (0x10 << 8) | 0x00 = 0x1000
    // sign_extend_13(0x1000): bit 12 set -> -4096
    m7.write_scroll(1, 0x00);
    m7.write_scroll(1, 0x10);
    assert_eq!(m7.vofs, -4096);
}

#[test]
fn e04_05_mpy_examples() {
    let mut m7 = M7Regs::new();
    // a = (0x01 << 8) | 0x00 = 256
    m7.write_matrix(0, 0x00);
    m7.write_matrix(0, 0x01);
    // b_last_byte = 0xFF -> -1 as i8
    m7.write_matrix(1, 0xFF);
    assert_eq!(m7.mpy(), -256);
    assert_eq!(m7.mpy_bits(), 0xFFFF00);

    // a = (0xFF << 8) | 0xFF = -1
    m7.write_matrix(0, 0xFF);
    m7.write_matrix(0, 0xFF);
    // b_last_byte = 0x7F -> 127 as i8
    m7.write_matrix(1, 0x7F);
    assert_eq!(m7.mpy(), -127);
    assert_eq!(m7.mpy_bits(), 0xFFFF81);
}

#[test]
fn e04_05_mpy_extremes() {
    let mut m7 = M7Regs::new();
    // a = (0x7F << 8) | 0xFF = 0x7FFF = 32767
    m7.write_matrix(0, 0xFF);
    m7.write_matrix(0, 0x7F);
    // b_last_byte = 0x80 -> -128 as i8
    m7.write_matrix(1, 0x80);
    // 32767 * (-128) = -4,194,176
    assert_eq!(m7.mpy(), -4194176);
    assert_eq!(m7.mpy_bits(), 0xC00080);

    // a = (0x01 << 8) | 0x00 = 256
    m7.write_matrix(0, 0x00);
    m7.write_matrix(0, 0x01);
    // b_last_byte = 0x7F -> 127 as i8
    m7.write_matrix(1, 0x7F);
    assert_eq!(m7.mpy(), 32512);
    assert_eq!(m7.mpy_bits(), 0x007F00);
}

#[test]
fn e04_05_bad_index() {
    let mut m7 = M7Regs::new();
    // Out-of-range indices should not panic or change state (except latch updates)
    m7.write_matrix(4, 1);
    m7.write_center(2, 1);
    m7.write_scroll(2, 1);

    // Values should be unchanged from defaults
    assert_eq!(m7.a, 0);
    assert_eq!(m7.b, 0);
    assert_eq!(m7.c, 0);
    assert_eq!(m7.d, 0);
    assert_eq!(m7.cx, 0);
    assert_eq!(m7.cy, 0);
    assert_eq!(m7.hofs, 0);
    assert_eq!(m7.vofs, 0);

    // Hash should be consistent with an instance that had the same latch writes
    let mut m7_fresh = M7Regs::new();
    // Replicate the exact write sequence to get identical state
    m7_fresh.write_matrix(4, 1);
    m7_fresh.write_center(2, 1);
    m7_fresh.write_scroll(2, 1);

    let mut h = Fnv64::new();
    m7.hash_into(&mut h);
    let hash_after_bad = h.finish();

    let mut h2 = Fnv64::new();
    m7_fresh.hash_into(&mut h2);
    let hash_fresh = h2.finish();

    assert_eq!(hash_after_bad, hash_fresh);
}

#[test]
fn e04_05_hash() {
    // Two fresh instances have same hash
    let m1 = M7Regs::new();
    let m2 = M7Regs::new();
    let mut h1 = Fnv64::new();
    m1.hash_into(&mut h1);
    let mut h2 = Fnv64::new();
    m2.hash_into(&mut h2);
    assert_eq!(h1.finish(), h2.finish());

    // Different after write_sel(1)
    let mut m3 = M7Regs::new();
    m3.write_sel(1);
    let mut h3 = Fnv64::new();
    m3.hash_into(&mut h3);
    assert_ne!(h1.finish(), h3.finish());

    // Different after write_matrix(0, 1)
    let mut m4 = M7Regs::new();
    m4.write_matrix(0, 1);
    let mut h4 = Fnv64::new();
    m4.hash_into(&mut h4);
    assert_ne!(h1.finish(), h4.finish());
}
