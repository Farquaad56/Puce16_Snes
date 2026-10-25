use puce16_core::bus::mmio::{Mmio, HvState};

fn make_st(vblank: bool, hblank: bool, autoread_busy: bool) -> Option<HvState> {
    Some(HvState { vblank, hblank, autoread_busy })
}

#[test]
fn e03_16_rdnmi() {
    let mut m = Mmio::new();
    // flag false, mdr=0xFF: (0x70) | 0x02 | 0 = 0x72
    assert_eq!(m.read(0x4210, 0xFF, 0, make_st(false, false, false)), 0x72);
    // flag true, mdr=0xFF: (0x70) | 0x02 | 0x80 = 0xF2
    m.rdnmi_flag = true;
    assert_eq!(m.read(0x4210, 0xFF, 0, make_st(false, false, false)), 0xF2);
    // flag true, mdr=0x00: (0x00) | 0x02 | 0x80 = 0x82
    m.rdnmi_flag = true;
    assert_eq!(m.read(0x4210, 0x00, 0, make_st(false, false, false)), 0x82);
    // after read, flag is cleared; next read with mdr=0xFF gives 0x72
    assert!(!m.rdnmi_flag);
    assert_eq!(m.read(0x4210, 0xFF, 0, make_st(false, false, false)), 0x72);
}

#[test]
fn e03_16_timeup() {
    let mut m = Mmio::new();
    // flag true + irq_line true, mdr=0xFF: (0x7F) | 0x80 = 0xFF
    m.timeup_flag = true;
    m.irq_line = true;
    assert_eq!(m.read(0x4211, 0xFF, 0, make_st(false, false, false)), 0xFF);
    // after read: timeup_flag and irq_line are false
    assert!(!m.timeup_flag);
    assert!(!m.irq_line);
    // flag false, mdr=0xAB: (0x2B) | 0 = 0x2B
    assert_eq!(m.read(0x4211, 0xAB, 0, make_st(false, false, false)), 0x2B);
    // flag false, mdr=0: 0x00
    assert_eq!(m.read(0x4211, 0x00, 0, make_st(false, false, false)), 0x00);
}

#[test]
fn e03_16_hvbjoy() {
    let mut m = Mmio::new();
    // st={vblank:true,hblank:false,busy:true}, mdr=0xFF: (0x3E) | 0x80 | 0x01 = 0xBF
    assert_eq!(m.read(0x4212, 0xFF, 0, make_st(true, false, true)), 0xBF);
    // st all false, mdr=0: 0x00
    assert_eq!(m.read(0x4212, 0x00, 0, make_st(false, false, false)), 0x00);
    // st={vblank:true,hblank:true,busy:false}, mdr=0: (0x00) | 0x80 | 0x40 = 0xC0
    assert_eq!(m.read(0x4212, 0x00, 0, make_st(true, true, false)), 0xC0);
    // st all false, mdr=0xFF: (0x3E) | 0 = 0x3E
    assert_eq!(m.read(0x4212, 0xFF, 0, make_st(false, false, false)), 0x3E);
}

#[test]
fn e03_16_rdio() {
    let mut m = Mmio::new();
    // power-on: wrio=0xFF
    assert_eq!(m.read(0x4213, 0xFF, 0, make_st(false, false, false)), 0xFF);
    // write then read back
    m.write(0x4201, 0x7F, 0);
    assert_eq!(m.read(0x4213, 0xFF, 0, make_st(false, false, false)), 0x7F);
}

#[test]
fn e03_16_joy_regs() {
    let mut m = Mmio::new();
    m.joy[0] = 0x1234;
    m.joy[1] = 0x5678;
    m.joy[2] = 0x9ABC;
    m.joy[3] = 0xDEF0;
    let st = make_st(false, false, false);
    // low bytes (even addresses)
    assert_eq!(m.read(0x4218, 0, 0, st), 0x34);
    assert_eq!(m.read(0x421A, 0, 0, st), 0x78);
    assert_eq!(m.read(0x421C, 0, 0, st), 0xBC);
    assert_eq!(m.read(0x421E, 0, 0, st), 0xF0);
    // high bytes (odd addresses)
    assert_eq!(m.read(0x4219, 0, 0, st), 0x12);
    assert_eq!(m.read(0x421B, 0, 0, st), 0x56);
    assert_eq!(m.read(0x421D, 0, 0, st), 0x9A);
    assert_eq!(m.read(0x421F, 0, 0, st), 0xDE);
}

#[test]
fn e03_16_out_of_range() {
    let mut m = Mmio::new();
    let st = make_st(false, false, false);
    assert_eq!(m.read(0x4220, 0xEE, 0, st), 0xEE);
    assert_eq!(m.read(0x420F, 0x11, 0, st), 0x11);
    assert_eq!(m.read(0x4200, 0x22, 0, st), 0x22);
}

#[test]
fn e03_16_math_still_works() {
    let mut m = Mmio::new();
    // multiply 5*6=30 at t=0 (done at t=48)
    m.write(0x4202, 5, 0);
    m.write(0x4203, 6, 0);
    let st = make_st(false, false, false);
    assert_eq!(m.read(0x4216, 0, 48, st), 30);
}
