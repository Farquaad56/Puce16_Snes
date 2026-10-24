// E02-08: addr/absolute.rs - Abs, AbsX, AbsY, Long, LongX addressing modes

use puce16_core::cpu::{addr, Bus, Cpu};
use puce16_core::cpu::table::Am;
use puce16_core::cpu::testbus::FlatBus;

fn make_cpu() -> Cpu {
    let mut cpu = Cpu::new();
    cpu.e = false; // native mode for 16-bit tests
    cpu
}

#[test]
fn e02_08_abs() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.db = 0x7E;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x34, 0x12]);

    let e = addr::ea(&mut cpu, &mut bus, Am::Abs, false);
    assert_eq!(e.lo, 0x7E1234);
    assert_eq!(e.hi, 0x7E1235);
    assert_eq!(cpu.pc, 0x8002);
    // Log: 2 reads (operand bytes)
    assert_eq!(bus.log.len(), 2);
}

#[test]
fn e02_08_abs_bank_cross() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.db = 0x01;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0xFF, 0xFF]);

    let e = addr::ea(&mut cpu, &mut bus, Am::Abs, false);
    assert_eq!(e.lo, 0x01FFFF);
    assert_eq!(e.hi, 0x020000); // wraps to next bank
}

#[test]
fn e02_08_absx_8bit_no_cross() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x10; // X=1 (8-bit index)
    cpu.x = 0x05;
    cpu.db = 0;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x10, 0x12]);

    let e = addr::ea(&mut cpu, &mut bus, Am::AbsX, false);
    assert_eq!(e.lo, 0x001215); // 0x1210 + 5
    // Log: 2 reads only (no idle for 8-bit index without page cross)
    assert_eq!(bus.log.len(), 2);
}

#[test]
fn e02_08_absx_8bit_cross() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x10; // X=1 (8-bit index)
    cpu.x = 0x20;
    cpu.db = 0;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0xF0, 0x12]);

    let e = addr::ea(&mut cpu, &mut bus, Am::AbsX, false);
    assert_eq!(e.lo, 0x001310); // 0x12F0 + 0x20 crosses page
    // Log: 2 reads + 1 idle (page cross)
    assert_eq!(bus.log.len(), 3);
}

#[test]
fn e02_08_absx_store_no_cross() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x10; // X=1 (8-bit index)
    cpu.x = 0x05;
    cpu.db = 0;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x10, 0x12]);

    let e = addr::ea(&mut cpu, &mut bus, Am::AbsX, true); // store=true
    assert_eq!(e.lo, 0x001215);
    // Log: 2 reads + 1 idle (store forces idle even without page cross)
    assert_eq!(bus.log.len(), 3);
}

#[test]
fn e02_08_absx_16bit_index() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // X=0 (16-bit index)
    cpu.x = 0x0001;
    cpu.db = 0;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x10, 0x12]);

    let e = addr::ea(&mut cpu, &mut bus, Am::AbsX, false);
    assert_eq!(e.lo, 0x001211); // 0x1210 + 1
    // Log: 2 reads + 1 idle (16-bit index always adds idle for read)
    assert_eq!(bus.log.len(), 3);
}

#[test]
fn e02_08_absx_carry_to_bank() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // X=0 (16-bit index)
    cpu.x = 0x0020;
    cpu.db = 0x01;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0xF0, 0xFF]);

    let e = addr::ea(&mut cpu, &mut bus, Am::AbsX, false);
    assert_eq!(e.lo, 0x020010); // (0x01FFF0 + 0x20) wraps to bank 2
    assert_eq!(e.hi, 0x020011);
}

#[test]
fn e02_08_absy() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x10; // X=1 (8-bit index)
    cpu.y = 0x03;
    cpu.db = 0;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x00, 0x20]);

    let e = addr::ea(&mut cpu, &mut bus, Am::AbsY, false);
    assert_eq!(e.lo, 0x002003); // 0x2000 + 3
    // Log: 2 reads only (no page cross)
    assert_eq!(bus.log.len(), 2);
}

#[test]
fn e02_08_long() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x56, 0x34, 0x12]);

    let e = addr::ea(&mut cpu, &mut bus, Am::Long, false);
    assert_eq!(e.lo, 0x123456);
    assert_eq!(e.hi, 0x123457);
    assert_eq!(cpu.pc, 0x8003);
    // Log: 3 reads (operand bytes)
    assert_eq!(bus.log.len(), 3);

    // Test bank wrap for hi
    cpu.pc = 0x8000;
    bus.clear_log();
    bus.poke(0x8000, &[0xFF, 0xFF, 0x12]);
    let e2 = addr::ea(&mut cpu, &mut bus, Am::Long, false);
    assert_eq!(e2.lo, 0x12FFFF);
    assert_eq!(e2.hi, 0x130000); // wraps to next bank
}

#[test]
fn e02_08_longx() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // X=0 (16-bit index)
    cpu.x = 0x0010;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x56, 0x34, 0x12]);

    let e = addr::ea(&mut cpu, &mut bus, Am::LongX, false);
    assert_eq!(e.lo, 0x123466); // 0x123456 + 0x10
    // Log: 3 reads only (no extra cycle for LongX)
    assert_eq!(bus.log.len(), 3);
}

#[test]
fn e02_08_rmw_abs_8bit() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x20; // M=1 (8-bit accumulator)
    cpu.db = 0;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x34, 0x12]);
    bus.poke(0x1234, &[0x41]);

    addr::rmw(&mut cpu, &mut bus, Am::Abs, |_c, v, _w| (v + 1) & 0xFF);
    assert_eq!(bus.peek(0x1234), 0x42);
    // Log: R 0x8000, R 0x8001, R 0x1234, Idle, W 0x1234=0x42
    assert_eq!(bus.log.len(), 5);
}

#[test]
fn e02_08_rmw_abs_16bit() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // M=0 (16-bit accumulator)
    cpu.db = 0;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x34, 0x12]);
    bus.poke(0x1234, &[0xFF, 0x00]);

    addr::rmw(&mut cpu, &mut bus, Am::Abs, |_c, v, _w| (v + 1) & 0xFFFF);
    assert_eq!(bus.peek(0x1234), 0x00);
    assert_eq!(bus.peek(0x1235), 0x01);
    // Log: R 0x8000, R 0x8001, R 0x1234, R 0x1235, Idle, W 0x1235=0x01, W 0x1234=0x00
    assert_eq!(bus.log.len(), 7);
}
