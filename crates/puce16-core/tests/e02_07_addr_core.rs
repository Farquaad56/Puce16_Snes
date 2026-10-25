// E02-07: addr.rs core functions - Ea, read_data/write_data, read_operand, rmw

use puce16_core::cpu::{addr, Cpu};
use puce16_core::cpu::table::Am;
use puce16_core::cpu::testbus::FlatBus;

fn make_cpu() -> Cpu {
    let mut cpu = Cpu::new();
    cpu.e = false; // native mode for 16-bit tests
    cpu
}

#[test]
fn e02_07_read_data_8() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    bus.poke(0x1000, &[0x34, 0x12]);

    let ea = addr::Ea { lo: 0x1000, hi: 0x1001 };
    let v = addr::read_data(&mut cpu, &mut bus, &ea, false);
    assert_eq!(v, 0x34);
    assert_eq!(bus.log.len(), 1);
}

#[test]
fn e02_07_read_data_16() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    bus.poke(0x1000, &[0x34, 0x12]);

    let ea = addr::Ea { lo: 0x1000, hi: 0x1001 };
    let v = addr::read_data(&mut cpu, &mut bus, &ea, true);
    assert_eq!(v, 0x1234);
    assert_eq!(bus.log.len(), 2);
}

#[test]
fn e02_07_write_data_16() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();

    let ea = addr::Ea { lo: 0x2000, hi: 0x2001 };
    addr::write_data(&mut cpu, &mut bus, &ea, 0xABCD, true);
    assert_eq!(bus.peek(0x2000), 0xCD);
    assert_eq!(bus.peek(0x2001), 0xAB);
    assert_eq!(bus.log.len(), 2);
}

#[test]
fn e02_07_read_operand_imm_8() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.pb = 0;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x7F, 0x11]);

    let v = addr::read_operand(&mut cpu, &mut bus, Am::Imm, false);
    assert_eq!(v, 0x7F);
    assert_eq!(cpu.pc, 0x8001);
    assert_eq!(bus.log.len(), 1);
}

#[test]
fn e02_07_read_operand_imm_16() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.pb = 0;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x7F, 0x11]);

    let v = addr::read_operand(&mut cpu, &mut bus, Am::Imm, true);
    assert_eq!(v, 0x117F);
    assert_eq!(cpu.pc, 0x8002);
    assert_eq!(bus.log.len(), 2);
}

#[test]
fn e02_07_rmw_acc_8bit() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.e = false;
    cpu.p = 0x20; // M=1 (8-bit accumulator)
    cpu.a = 0x12FF;

    addr::rmw(&mut cpu, &mut bus, Am::Acc, |_c, v, _w| (v + 1) & 0xFF);
    assert_eq!(cpu.a, 0x1200); // low byte wrapped, high byte preserved
    assert_eq!(bus.log.len(), 1);
}

#[test]
fn e02_07_rmw_acc_16bit() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.e = false;
    cpu.p = 0x00; // M=0 (16-bit accumulator)
    cpu.a = 0x00FF;

    addr::rmw(&mut cpu, &mut bus, Am::Acc, |_c, v, w| {
        assert!(w);
        v + 1
    });
    assert_eq!(cpu.a, 0x0100);
    assert_eq!(bus.log.len(), 1);
}

#[test]
#[should_panic(expected = "ea: mode Imp has no memory operand")]
fn e02_07_ea_panics_for_imm() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    addr::ea(&mut cpu, &mut bus, Am::Imp, false);
}
