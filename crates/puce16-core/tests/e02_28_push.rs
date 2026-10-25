// E02-28: mem/push.rs - PHA, PHX, PHY, PHP, PHD, PHB, PHK, PEA, PEI, PER instructions

use puce16_core::cpu::{Cpu, FLAG_M, FLAG_X};
use puce16_core::cpu::testbus::FlatBus;

fn make_cpu() -> Cpu {
    let mut cpu = Cpu::new();
    cpu.e = false; // native mode by default
    cpu
}

#[test]
fn e02_28_pha_8() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M; // M=1 (8-bit accumulator)
    cpu.s = 0x01FF;
    cpu.a = 0x12AB;
    cpu.pc = 0x8000;
    // PHA: opcode 48
    bus.poke(0x8000, &[0x48]);

    cpu.step(&mut bus);

    assert_eq!(bus.peek(0x01FF), 0xAB, "low byte of A should be pushed");
    assert_eq!(cpu.s, 0x01FE, "stack pointer should decrement by 1");
    // Cycles: 1 (opcode) + 1 (idle) + 1 (push) = 3
    assert_eq!(bus.log.len(), 3, "PHA in 8-bit mode should take 3 cycles");
}

#[test]
fn e02_28_pha_16() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // M=0 (16-bit accumulator)
    cpu.s = 0x01FF;
    cpu.a = 0x1234;
    cpu.pc = 0x8000;
    // PHA: opcode 48
    bus.poke(0x8000, &[0x48]);

    cpu.step(&mut bus);

    assert_eq!(bus.peek(0x01FF), 0x12, "high byte of A should be pushed first");
    assert_eq!(bus.peek(0x01FE), 0x34, "low byte of A should be pushed second");
    assert_eq!(cpu.s, 0x01FD, "stack pointer should decrement by 2");
    // Cycles: 1 (opcode) + 1 (idle) + 2 (pushes) = 4
    assert_eq!(bus.log.len(), 4, "PHA in 16-bit mode should take 4 cycles");
}

#[test]
fn e02_28_phx_8_phy_16() {
    // PHX in 8-bit mode
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M | FLAG_X; // M=1, X=1 (8-bit modes)
    cpu.s = 0x01FF;
    cpu.x = 0x00AA;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0xDA]);

    cpu.step(&mut bus);

    assert_eq!(bus.peek(0x01FF), 0xAA, "low byte of X should be pushed");
    assert_eq!(cpu.s, 0x01FE, "stack pointer should decrement by 1");

    // PHY in 16-bit mode
    let mut cpu2 = make_cpu();
    let mut bus2 = FlatBus::new();
    cpu2.p = 0x00; // M=0, X=0 (16-bit modes)
    cpu2.s = 0x01FF;
    cpu2.y = 0xBEEF;
    cpu2.pc = 0x8000;
    bus2.poke(0x8000, &[0x5A]);

    cpu2.step(&mut bus2);

    assert_eq!(bus2.peek(0x01FF), 0xBE, "high byte of Y should be pushed first");
    assert_eq!(bus2.peek(0x01FE), 0xEF, "low byte of Y should be pushed second");
    assert_eq!(cpu2.s, 0x01FD, "stack pointer should decrement by 2");
}

#[test]
fn e02_28_php() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x65;
    cpu.s = 0x01FF;
    cpu.pc = 0x8000;
    // PHP: opcode 08
    bus.poke(0x8000, &[0x08]);

    cpu.step(&mut bus);

    assert_eq!(bus.peek(0x01FF), 0x65, "P register should be pushed");
    assert_eq!(cpu.s, 0x01FE, "stack pointer should decrement by 1");
    // Cycles: 1 (opcode) + 1 (idle) + 1 (push) = 3
    assert_eq!(bus.log.len(), 3, "PHP should take 3 cycles");
}

#[test]
fn e02_28_php_emulation() {
    let mut cpu = make_cpu();
    cpu.e = true; // emulation mode
    let mut bus = FlatBus::new();
    cpu.p = 0x34;
    cpu.s = 0x01FF;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x08]);

    cpu.step(&mut bus);

    assert_eq!(bus.peek(0x01FF), 0x34, "P register should be pushed in emulation mode");
}

#[test]
fn e02_28_phd() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.d = 0xBEEF;
    cpu.s = 0x01FF;
    cpu.pc = 0x8000;
    // PHD: opcode 0B
    bus.poke(0x8000, &[0x0B]);

    cpu.step(&mut bus);

    assert_eq!(bus.peek(0x01FF), 0xBE, "high byte of D should be pushed first");
    assert_eq!(bus.peek(0x01FE), 0xEF, "low byte of D should be pushed second");
    assert_eq!(cpu.s, 0x01FD, "stack pointer should decrement by 2");
    // Cycles: 1 (opcode) + 1 (idle) + 2 (pushes) = 4
    assert_eq!(bus.log.len(), 4, "PHD should take 4 cycles");
}

#[test]
fn e02_28_phb_phk() {
    // PHB: opcode 8B
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.db = 0x7E;
    cpu.s = 0x01FF;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x8B]);

    cpu.step(&mut bus);

    assert_eq!(bus.peek(0x01FF), 0x7E, "DB register should be pushed");
    assert_eq!(cpu.s, 0x01FE, "stack pointer should decrement by 1");

    // PHK: opcode 4B
    let mut cpu2 = make_cpu();
    let mut bus2 = FlatBus::new();
    cpu2.pb = 0x00;
    cpu2.s = 0x01FF;
    cpu2.pc = 0x8000;
    bus2.poke(0x8000, &[0x4B]);

    cpu2.step(&mut bus2);

    assert_eq!(bus2.peek(0x01FF), 0x00, "PB register should be pushed");
}

#[test]
fn e02_28_pha_emulation_wrap() {
    let mut cpu = make_cpu();
    cpu.e = true; // emulation mode
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M | FLAG_X; // M=1, X=1 (forced in emulation)
    cpu.s = 0x0100;
    cpu.a = 0x77;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x48]);

    cpu.step(&mut bus);

    assert_eq!(bus.peek(0x0100), 0x77, "A.l should be pushed at S");
    assert_eq!(cpu.s, 0x01FF, "stack pointer should wrap from 0x0100 to 0x01FF");
}

#[test]
fn e02_28_pea() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.s = 0x01FF;
    cpu.pc = 0x8000;
    // PEA: opcode F4 + address bytes (little-endian)
    bus.poke(0x8000, &[0xF4, 0x34, 0x12]);

    cpu.step(&mut bus);

    assert_eq!(bus.peek(0x01FF), 0x12, "low byte of address should be pushed second");
    assert_eq!(bus.peek(0x01FE), 0x34, "high byte of address should be pushed first");
    assert_eq!(cpu.s, 0x01FD, "stack pointer should decrement by 2");
    // Cycles: 1 (opcode) + 2 (address bytes) + 2 (pushes) = 5
    assert_eq!(bus.log.len(), 5, "PEA should take 5 cycles");
}

#[test]
fn e02_28_pei() {
    // PEI with D=0: opcode D4 + operand byte
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.d = 0x0000;
    cpu.s = 0x01FF;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0xD4, 0x10]);
    // Data at direct page address 0x0010 (little-endian)
    bus.poke(0x10, &[0x78, 0x56]);

    cpu.step(&mut bus);

    assert_eq!(bus.peek(0x01FF), 0x56, "low byte of value should be pushed second");
    assert_eq!(bus.peek(0x01FE), 0x78, "high byte of value should be pushed first");
    // Cycles: 1 (opcode) + 1 (operand) + 2 (data reads) + 2 (pushes) = 6
    assert_eq!(bus.log.len(), 6, "PEI with D=0 should take 6 cycles");

    // PEI with D!=0 adds an idle cycle
    let mut cpu2 = make_cpu();
    let mut bus2 = FlatBus::new();
    cpu2.d = 0x0001;
    cpu2.s = 0x01FF;
    cpu2.pc = 0x8000;
    bus2.poke(0x8000, &[0xD4, 0x10]);
    // Data at direct page address D+operand = 0x0001+0x10 = 0x0011 (little-endian)
    bus2.poke(0x11, &[0x78, 0x56]);

    cpu2.step(&mut bus2);

    // Cycles: 1 (opcode) + 1 (operand) + 1 (idle for D!=0) + 2 (data reads) + 2 (pushes) = 7
    assert_eq!(bus2.log.len(), 7, "PEI with D!=0 should take 7 cycles");
}

#[test]
fn e02_28_per_positive() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.s = 0x01FF;
    cpu.pc = 0x8000;
    // PER: opcode 62 + offset bytes (little-endian)
    // Target = PC after operand (0x8003) + offset (0x0010) = 0x8013
    bus.poke(0x8000, &[0x62, 0x10, 0x00]);

    cpu.step(&mut bus);

    assert_eq!(bus.peek(0x01FF), 0x80, "high byte of target should be pushed first");
    assert_eq!(bus.peek(0x01FE), 0x13, "low byte of target should be pushed second");
    // Cycles: 1 (opcode) + 2 (offset bytes) + 1 (idle) + 2 (pushes) = 6
    assert_eq!(bus.log.len(), 6, "PER should take 6 cycles");
}

#[test]
fn e02_28_per_negative() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.s = 0x01FF;
    cpu.pc = 0x8000;
    // PER with negative offset: opcode 62 + offset bytes (little-endian)
    // Target = PC after operand (0x8003) + offset (-3 = 0xFFFD) = 0x8000
    bus.poke(0x8000, &[0x62, 0xFD, 0xFF]);

    cpu.step(&mut bus);

    assert_eq!(bus.peek(0x01FF), 0x80, "high byte of target should be pushed first");
    assert_eq!(bus.peek(0x01FE), 0x00, "low byte of target should be pushed second");
}
