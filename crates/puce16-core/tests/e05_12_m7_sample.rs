use puce16_core::ppu::mode7::{clip13, m7_sample, M7Regs};

fn identity_regs() -> M7Regs {
    let mut r = M7Regs::new();
    r.a = 0x100;
    r.d = 0x100;
    r
}

#[test]
fn e05_12_clip13_cases() {
    assert_eq!(clip13(5), 5);
    assert_eq!(clip13(0x3FF), 1023);
    assert_eq!(clip13(0x400), 0);
    assert_eq!(clip13(0x1FFF), 1023);
    assert_eq!(clip13(-1), -1);
    assert_eq!(clip13(-1024), -1024);
    assert_eq!(clip13(0x2005), -1019);
}

#[test]
fn e05_12_identity() {
    let r = identity_regs();
    assert_eq!(m7_sample(&r, 7, 9), (7, 9));
    assert_eq!(m7_sample(&r, 0, 0), (0, 0));
    assert_eq!(m7_sample(&r, 255, 223), (255, 223));
}

#[test]
fn e05_12_scroll() {
    let mut r = identity_regs();
    r.hofs = 10;
    r.vofs = 5;
    assert_eq!(m7_sample(&r, 7, 9), (17, 14));
}

#[test]
fn e05_12_flips() {
    let mut r = identity_regs();
    r.flip_x = true;
    assert_eq!(m7_sample(&r, 7, 9), (248, 9));

    let mut r2 = identity_regs();
    r2.flip_y = true;
    assert_eq!(m7_sample(&r2, 7, 9), (7, 246));

    let mut r3 = identity_regs();
    r3.flip_x = true;
    r3.flip_y = true;
    assert_eq!(m7_sample(&r3, 7, 9), (248, 246));
}

#[test]
fn e05_12_zoom_x2() {
    let mut r = M7Regs::new();
    r.a = 0x80;
    r.d = 0x80;
    assert_eq!(m7_sample(&r, 7, 9), (3, 4));
    assert_eq!(m7_sample(&r, 8, 9), (4, 4));
}

#[test]
fn e05_12_rotation_90() {
    let mut r = M7Regs::new();
    r.b = 0x100;
    r.c = -256;
    assert_eq!(m7_sample(&r, 7, 9), (9, -7));
}

#[test]
fn e05_12_center_offset() {
    let mut r = identity_regs();
    r.cx = 100;
    r.cy = 50;
    r.hofs = 100;
    r.vofs = 50;
    assert_eq!(m7_sample(&r, 7, 9), (107, 59));
}

#[test]
fn e05_12_mask_63_matters() {
    let mut r = M7Regs::new();
    r.a = 63;
    r.d = 0x100;
    r.hofs = 1;
    assert_eq!(m7_sample(&r, 4, 0), (0, 0));
}

#[test]
fn e05_12_negative_shift_is_arithmetic() {
    let mut r = M7Regs::new();
    r.c = -256;
    r.d = 0x100;
    assert_eq!(m7_sample(&r, 3, 0), (0, -3));
}

#[test]
fn e05_12_hofs_minus_cx_clipped() {
    let mut r = identity_regs();
    r.hofs = 0x1005;
    assert_eq!(m7_sample(&r, 0, 0), (5, 0));

    let mut r2 = identity_regs();
    r2.hofs = -1;
    assert_eq!(m7_sample(&r2, 0, 0), (-1, 0));
}
