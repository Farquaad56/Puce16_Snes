// E02-33: flow/jump.rs - JMP and JML instructions (all addressing modes)

use puce16_core::cpu::{Cpu};
use puce16_core::cpu::testbus::FlatBus;

fn make_cpu() -> Cpu {
    let mut cpu = Cpu::new();
    cpu.e = false; // native mode by default
    cpu
}

#[test]
fn e02_33_jmp_abs() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.pb = 0x03;
    cpu.pc = 0x8000;
    // JMP $1234: opcode 4C, lo=0x34, hi=0x12
    bus.poke(0x038000, &[0x4C, 0x34, 0x12]);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x1234, "JMP Abs: PC should be set to target");
    assert_eq!(cpu.pb, 0x03, "JMP Abs: PB should not change");
    assert_eq!(bus.log.len(), 3, "JMP Abs: 3 cycles (opcode + lo + hi)");
}

#[test]
fn e02_33_jml_long() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.pc = 0x8000;
    // JML $12:3456: opcode 5C, lo=0x56, hi=0x34, bank=0x12
    bus.poke(0x8000, &[0x5C, 0x56, 0x34, 0x12]);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x3456, "JML Long: PC should be set to target");
    assert_eq!(cpu.pb, 0x12, "JML Long: PB should be set to bank");
    assert_eq!(bus.log.len(), 4, "JML Long: 4 cycles (opcode + lo + hi + bank)");
}

#[test]
fn e02_33_jmp_ind() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.pb = 0x03;
    cpu.pc = 0x8000;
    // Pointer at $0:2000 -> $5678
    bus.poke(0x2000, &[0x78, 0x56]);
    // JMP ($2000): opcode 6C, lo=0x00, hi=0x20
    bus.poke(0x038000, &[0x6C, 0x00, 0x20]);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x5678, "JMP Ind: PC should be set to indirect target");
    assert_eq!(bus.log.len(), 5, "JMP Ind: 5 cycles (opcode + lo + hi + read_lo + read_hi)");
}

#[test]
fn e02_33_jmp_ind_wrap() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.pc = 0x8000;
    // Pointer at $0:FFFF -> wraps to $0:0000 for high byte
    bus.poke(0xFFFF, &[0x34]);
    bus.poke(0x0000, &[0x12]);
    // JMP ($FFFF): opcode 6C, lo=0xFF, hi=0xFF
    bus.poke(0x8000, &[0x6C, 0xFF, 0xFF]);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x1234, "JMP Ind wrap: PC should be set to wrapped target");
}

#[test]
fn e02_33_jml_ind_long() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.pc = 0x8000;
    // Pointer at $0:2000 -> $12:3456 (lo=0x56, mid=0x34, bank=0x12)
    bus.poke(0x2000, &[0x56, 0x34, 0x12]);
    // JML [$2000]: opcode DC, lo=0x00, hi=0x20
    bus.poke(0x8000, &[0xDC, 0x00, 0x20]);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x3456, "JML Ind Long: PC should be set to indirect target");
    assert_eq!(cpu.pb, 0x12, "JML Ind Long: PB should be set to bank from pointer");
    assert_eq!(bus.log.len(), 6, "JML Ind Long: 6 cycles (opcode + lo + hi + read_lo + read_mid + read_bank)");
}

#[test]
fn e02_33_jmp_abs_x_ind() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.pb = 0x03;
    cpu.p = 0; // clear all flags
    cpu.x = 0x0004;
    cpu.pc = 0x8000;
    // Pointer at $03:2004 -> $9000
    bus.poke(0x032004, &[0x00, 0x90]);
    // JMP ($2000,X): opcode 7C, lo=0x00, hi=0x20
    bus.poke(0x038000, &[0x7C, 0x00, 0x20]);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x9000, "JMP AbsX Ind: PC should be set to indexed indirect target");
    assert_eq!(cpu.pb, 0x03, "JMP AbsX Ind: PB should not change");
    assert_eq!(bus.log.len(), 6, "JMP AbsX Ind: 6 cycles (opcode + lo + hi + idle + read_lo + read_hi)");

    // Verify cycle sequence: R,R,R,Idle,R,R
    let kinds = bus.log.iter().map(|c| c.kind).collect::<Vec<_>>();
    assert_eq!(kinds[0], puce16_core::cpu::testbus::CycleKind::Read);
    assert_eq!(kinds[1], puce16_core::cpu::testbus::CycleKind::Read);
    assert_eq!(kinds[2], puce16_core::cpu::testbus::CycleKind::Read);
    assert_eq!(kinds[3], puce16_core::cpu::testbus::CycleKind::Idle);
    assert_eq!(kinds[4], puce16_core::cpu::testbus::CycleKind::Read);
    assert_eq!(kinds[5], puce16_core::cpu::testbus::CycleKind::Read);

    // Verify addresses for the pointer reads
    assert_eq!(bus.log[4].addr, 0x032004, "low byte read at correct address");
    assert_eq!(bus.log[5].addr, 0x032005, "high byte read at correct address");
}

#[test]
fn e02_33_jmp_abs_x_ind_wrap() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.pb = 0x03;
    cpu.x = 1;
    cpu.pc = 0x8000;
    // Pointer at $03:0000 -> $5678 (after wrap from FF+1=00)
    bus.poke(0x030000, &[0x78, 0x56]);
    // JMP ($FFFF,X): opcode 7C, lo=0xFF, hi=0xFF -> effective address $03:0000
    bus.poke(0x038000, &[0x7C, 0xFF, 0xFF]);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x5678, "JMP AbsX Ind wrap: PC should be set to wrapped target");
}
