use puce16_core::bus::wram::Wram;
use puce16_core::hash::Fnv64;

#[test]
fn e03_10_power_on() {
    let w = Wram::new();
    assert_eq!(w.read(0), 0x55);
    assert_eq!(w.read(0x1FFFF), 0x55);
    assert_eq!(w.addr, 0);
}

#[test]
fn e03_10_mask() {
    let mut w = Wram::new();
    w.write(0x20005, 9);
    assert_eq!(w.read(5), 9);
    assert_eq!(w.read(0x40005), 9);
}

#[test]
fn e03_10_addr_regs() {
    let mut w = Wram::new();
    w.set_addr_l(0x34);
    w.set_addr_m(0x12);
    w.set_addr_h(0x01);
    assert_eq!(w.addr, 0x11234);

    w.set_addr_l(0x99);
    assert_eq!(w.addr, 0x11299);

    w.set_addr_h(0xFE);
    assert_eq!(w.addr, 0x1299);

    w.set_addr_h(0x03);
    assert_eq!(w.addr, 0x11299);
}

#[test]
fn e03_10_data_write_wrap() {
    let mut w = Wram::new();
    w.set_addr_l(0xFE);
    w.set_addr_m(0xFF);
    w.set_addr_h(0x01);
    assert_eq!(w.addr, 0x1FFFE);

    w.data_write(1);
    w.data_write(2);
    w.data_write(3);

    assert_eq!(w.read(0x1FFFE), 1);
    assert_eq!(w.read(0x1FFFF), 2);
    assert_eq!(w.read(0), 3);
    assert_eq!(w.addr, 1);
}

#[test]
fn e03_10_data_read() {
    let mut w = Wram::new();
    w.write(0x10, 0xAA);
    w.write(0x11, 0xBB);
    w.set_addr_l(0x10);

    assert_eq!(w.data_read(), 0xAA);
    assert_eq!(w.data_read(), 0xBB);
    assert_eq!(w.addr, 0x12);
}

#[test]
fn e03_10_reset_addr() {
    let mut w = Wram::new();
    w.write(0x10, 0xCC);
    w.set_addr_l(0x50);
    assert_eq!(w.addr, 0x50);

    w.reset_addr();
    assert_eq!(w.addr, 0);
    assert_eq!(w.read(0x10), 0xCC);
}

#[test]
fn e03_10_hash() {
    let mut h1 = Fnv64::new();
    Wram::new().hash_into(&mut h1);
    let hash1 = h1.finish();

    let mut h2 = Fnv64::new();
    Wram::new().hash_into(&mut h2);
    assert_eq!(h2.finish(), hash1);

    let mut w3 = Wram::new();
    w3.write(5, 1);
    let mut h3 = Fnv64::new();
    w3.hash_into(&mut h3);
    assert_ne!(h3.finish(), hash1);

    let mut w4 = Wram::new();
    w4.set_addr_l(0x10);
    let mut h4 = Fnv64::new();
    w4.hash_into(&mut h4);
    assert_ne!(h4.finish(), hash1);
}
