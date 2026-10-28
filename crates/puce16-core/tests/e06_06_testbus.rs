use puce16_core::apu::spc700::testbus::{FlatSmpBus, Cycle, CycleKind};
use puce16_core::apu::spc700::SmpBus;

#[test]
fn e06_06_read_write_log() {
    let mut bus = FlatSmpBus::new();
    bus.write(0x1234, 0x5A);
    let v = bus.read(0x1234);
    assert_eq!(v, 0x5A);
    assert_eq!(bus.log.len(), 2);
    assert_eq!(bus.log[0], Cycle { addr: 0x1234, val: 0x5A, kind: CycleKind::Write });
    assert_eq!(bus.log[1], Cycle { addr: 0x1234, val: 0x5A, kind: CycleKind::Read });
}

#[test]
fn e06_06_idle_log() {
    let mut bus = FlatSmpBus::new();
    bus.idle();
    assert_eq!(bus.log.len(), 1);
    assert_eq!(bus.log[0], Cycle { addr: 0, val: 0, kind: CycleKind::Idle });
}

#[test]
fn e06_06_poke_peek() {
    let mut bus = FlatSmpBus::new();
    bus.poke(0x0010, &[1, 2, 3]);
    assert_eq!(bus.peek(0x0011), 2);
    assert_eq!(bus.log.len(), 0);
}

#[test]
fn e06_06_poke_wrap() {
    let mut bus = FlatSmpBus::new();
    bus.poke(0xFFFF, &[7, 8]);
    assert_eq!(bus.peek(0xFFFF), 7);
    assert_eq!(bus.peek(0x0000), 8);
}

#[test]
fn e06_06_reset_mem() {
    let mut bus = FlatSmpBus::new();
    bus.poke(0x10, &[5]);
    bus.write(0x20, 3);
    assert_eq!(bus.peek(0x10), 5);
    assert_eq!(bus.log.len(), 1);
    bus.reset_mem();
    assert_eq!(bus.peek(0x10), 0);
    assert_eq!(bus.peek(0x20), 0);
    assert_eq!(bus.log.len(), 0);
}

#[test]
fn e06_06_flat_registers() {
    let mut bus = FlatSmpBus::new();
    // No mapped registers: write to $F4 just writes RAM
    bus.write(0x00F4, 0x99);
    assert_eq!(bus.read(0x00F4), 0x99);
    // Read at $FFC0 reads RAM (zero after new)
    assert_eq!(bus.read(0xFFC0), 0);
}
