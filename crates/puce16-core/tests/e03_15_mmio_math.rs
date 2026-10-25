use puce16_core::bus::mmio::{Mmio, MUL_CLOCKS, DIV_CLOCKS};

#[test]
fn e03_15_mul_timing() {
    let mut m = Mmio::new();
    // master=100: write WRMPYA=7, WRMPYB=9 -> result 63 at t=148
    m.write(0x4202, 7, 100);
    m.write(0x4203, 9, 100);
    // Before completion (t=147): old value (0)
    assert_eq!(m.read(0x4216, 0, 147, None), 0);
    // At completion (t=148): result visible
    assert_eq!(m.read(0x4216, 0, 148, None), 0x3F);
    assert_eq!(m.read(0x4217, 0, 148, None), 0x00);
}

#[test]
fn e03_15_mul_max() {
    let mut m = Mmio::new();
    // 0xFF * 0xFF = 65025 = 0xFE01
    m.write(0x4202, 0xFF, 0);
    m.write(0x4203, 0xFF, 0);
    assert_eq!(m.read(0x4216, 0, MUL_CLOCKS, None), 0x01);
    assert_eq!(m.read(0x4217, 0, MUL_CLOCKS, None), 0xFE);
}

#[test]
fn e03_15_mul_power_on_a() {
    let mut m = Mmio::new();
    // WRMPYA power-on value is 0xFF; don't write it
    m.write(0x4203, 2, 0);
    // 0xFF * 2 = 510 = 0x01FE
    assert_eq!(m.read(0x4216, 0, MUL_CLOCKS, None), 0xFE);
    assert_eq!(m.read(0x4217, 0, MUL_CLOCKS, None), 0x01);
}

#[test]
fn e03_15_div_basic() {
    let mut m = Mmio::new();
    // Dividend: 0x1234 = 4660; divisor: 7
    // 4660 / 7 = 665 remainder 5
    m.write(0x4204, 0x34, 0);
    m.write(0x4205, 0x12, 0);
    m.write(0x4206, 7, 0);
    // Before completion (t=95): old value
    assert_eq!(m.read(0x4214, 0, 95, None), 0);
    // At completion (t=96)
    assert_eq!(m.read(0x4214, 0, DIV_CLOCKS, None), 0x99); // quotient low: 665 & 0xFF = 153 = 0x99
    assert_eq!(m.read(0x4215, 0, DIV_CLOCKS, None), 0x02); // quotient high: 665 >> 8 = 2
    assert_eq!(m.read(0x4216, 0, DIV_CLOCKS, None), 0x05); // remainder low: 5
    assert_eq!(m.read(0x4217, 0, DIV_CLOCKS, None), 0x00); // remainder high: 0
}

#[test]
fn e03_15_div_by_zero() {
    let mut m = Mmio::new();
    // Dividend: 0x1234; divisor: 0 -> quotient=0xFFFF, remainder=dividend
    m.write(0x4204, 0x34, 0);
    m.write(0x4205, 0x12, 0);
    m.write(0x4206, 0, 0);
    assert_eq!(m.read(0x4214, 0, DIV_CLOCKS, None), 0xFF);
    assert_eq!(m.read(0x4215, 0, DIV_CLOCKS, None), 0xFF);
    assert_eq!(m.read(0x4216, 0, DIV_CLOCKS, None), 0x34);
    assert_eq!(m.read(0x4217, 0, DIV_CLOCKS, None), 0x12);
}

#[test]
fn e03_15_div_ffff_by_ff() {
    let mut m = Mmio::new();
    // Dividend: 0xFFFF (power-on default); divisor: 0xFF
    // 65535 / 255 = 257 remainder 0 -> quotient=0x0101, remainder=0
    m.write(0x4206, 0xFF, 0);
    assert_eq!(m.read(0x4214, 0, DIV_CLOCKS, None), 0x01);
    assert_eq!(m.read(0x4215, 0, DIV_CLOCKS, None), 0x01);
    assert_eq!(m.read(0x4216, 0, DIV_CLOCKS, None), 0x00);
    assert_eq!(m.read(0x4217, 0, DIV_CLOCKS, None), 0x00);
}

#[test]
fn e03_15_settle_before_new_op() {
    let mut m = Mmio::new();
    // First: multiply 3*4=12 at t=0 (done at t=48)
    m.write(0x4202, 3, 0);
    m.write(0x4203, 4, 0);
    // Set up division: dividend=100, divisor=7 -> quotient=14, remainder=2
    m.write(0x4204, 100, 0);
    m.write(0x4205, 0, 0);
    // Start division at t=60 (after multiplication completed)
    m.write(0x4206, 7, 60);
    // At t=100: multiplication result should be visible (settled before new op)
    assert_eq!(m.read(0x4216, 0, 100, None), 12);
    // Division completes at t=156
    assert_eq!(m.read(0x4214, 0, 156, None), 14);
    assert_eq!(m.read(0x4216, 0, 156, None), 2);
}

#[test]
fn e03_15_overwrite_in_flight() {
    let mut m = Mmio::new();
    // Start multiply 3*4 at t=0 (would complete at t=48)
    m.write(0x4202, 3, 0);
    m.write(0x4203, 4, 0);
    // Set up division: dividend=100, divisor=7
    m.write(0x4204, 100, 0);
    m.write(0x4205, 0, 0);
    // Start division at t=10 (before multiply completes) -> overwrites
    m.write(0x4206, 7, 10);
    // Multiply result is lost; read at t=60 shows old value (0)
    assert_eq!(m.read(0x4216, 0, 60, None), 0);
    // Division completes at t=106
    assert_eq!(m.read(0x4214, 0, 106, None), 14);
    assert_eq!(m.read(0x4216, 0, 106, None), 2);
}

#[test]
fn e03_15_other_reg_open_bus() {
    let mut m = Mmio::new();
    // Unimplemented read registers return open bus value (mdr)
    assert_eq!(m.read(0x4220, 0xCD, 0, None), 0xCD);
}
