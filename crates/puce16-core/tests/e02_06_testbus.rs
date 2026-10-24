use puce16_core::cpu::testbus::{FlatBus, Cycle, CycleKind};
use puce16_core::cpu::Bus;

#[test]
fn e02_06_read_write_log() {
    let mut bus = FlatBus::new();
    bus.write(0x7E1234, 0x5A);
    let v = bus.read(0x7E1234);
    assert_eq!(v, 0x5A);
    assert_eq!(bus.log.len(), 2);
    assert_eq!(bus.log[0], Cycle { addr: 0x7E1234, val: 0x5A, kind: CycleKind::Write });
    assert_eq!(bus.log[1], Cycle { addr: 0x7E1234, val: 0x5A, kind: CycleKind::Read });
}

#[test]
fn e02_06_idle_log() {
    let mut bus = FlatBus::new();
    bus.idle();
    assert_eq!(bus.log.len(), 1);
    assert_eq!(bus.log[0], Cycle { addr: 0, val: 0, kind: CycleKind::Idle });
}

#[test]
fn e02_06_poke_peek() {
    let mut bus = FlatBus::new();
    bus.poke(0x001000, &[1, 2, 3]);
    assert_eq!(bus.peek(0x1001), 2);
    assert_eq!(bus.log.len(), 0);
}

#[test]
fn e02_06_addr_mask() {
    let mut bus = FlatBus::new();
    bus.poke(5, &[9]);
    let v = bus.read(0x1000005);
    assert_eq!(v, 9);
}

#[test]
fn e02_06_poke_wrap() {
    let mut bus = FlatBus::new();
    bus.poke(0xFFFFFF, &[7, 8]);
    assert_eq!(bus.peek(0xFFFFFF), 7);
    assert_eq!(bus.peek(0), 8);
}

#[test]
fn e02_06_reset_mem() {
    let mut bus = FlatBus::new();
    bus.poke(0x10, &[5]);
    bus.reset_mem();
    assert_eq!(bus.peek(0x10), 0);
    assert_eq!(bus.log.len(), 0);
}

#[test]
fn e02_06_nmi_consumed() {
    let mut bus = FlatBus::new();
    bus.nmi = true;
    assert!(bus.nmi_pending());
    assert!(!bus.nmi_pending());

    bus.irq = true;
    assert!(bus.irq_asserted());
    assert!(bus.irq_asserted());
}