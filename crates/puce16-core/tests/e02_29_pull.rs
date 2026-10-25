// E02-29: mem/pull.rs - PLA, PLX, PLY, PLP, PLB, PLD instructions

use puce16_core::cpu::{Cpu, FLAG_M, FLAG_X};
use puce16_core::cpu::testbus::FlatBus;

fn make_cpu() -> Cpu {
    let mut cpu = Cpu::new();
    cpu.e = false; // native mode by default
    cpu
}

#[test]
fn e02_29_pla_8() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M; // M=1 (8-bit accumulator)
    cpu.s = 0x01FE;
    cpu.a = 0x1200;
    cpu.pc = 0x8000;
    bus.poke(0x01FF, &[0x80]);
    // PLA: opcode 68
    bus.poke(0x8000, &[0x68]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0x1280, "A should be 0x1280 (high byte preserved)");
    assert!(cpu.flag(puce16_core::cpu::FLAG_N), "N flag should be set");
    assert_eq!(cpu.s, 0x01FF, "stack pointer should increment by 1");
    // Cycles: 1 (opcode) + 2 (idle) + 1 (pull) = 4
    assert_eq!(bus.log.len(), 4, "PLA in 8-bit mode should take 4 cycles");
}

#[test]
fn e02_29_pla_16() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // M=0 (16-bit accumulator)
    cpu.s = 0x01FD;
    cpu.pc = 0x8000;
    bus.poke(0x01FE, &[0x34, 0x12]);
    // PLA: opcode 68
    bus.poke(0x8000, &[0x68]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0x1234, "A should be 0x1234");
    assert_eq!(cpu.s, 0x01FF, "stack pointer should increment by 2");
    // Cycles: 1 (opcode) + 2 (idle) + 2 (pulls) = 5
    assert_eq!(bus.log.len(), 5, "PLA in 16-bit mode should take 5 cycles");
}

#[test]
fn e02_29_plx_8() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M | FLAG_X; // M=1, X=1 (8-bit modes)
    cpu.s = 0x01FE;
    cpu.x = 0x55;
    cpu.pc = 0x8000;
    bus.poke(0x01FF, &[0x00]);
    // PLX: opcode FA
    bus.poke(0x8000, &[0xFA]);

    cpu.step(&mut bus);

    assert_eq!(cpu.x, 0, "X should be 0");
    assert!(cpu.flag(puce16_core::cpu::FLAG_Z), "Z flag should be set");
}

#[test]
fn e02_29_ply_16() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // M=0, X=0 (16-bit modes)
    cpu.s = 0x01FD;
    cpu.pc = 0x8000;
    bus.poke(0x01FE, &[0xEF, 0xBE]);
    // PLY: opcode 7A
    bus.poke(0x8000, &[0x7A]);

    cpu.step(&mut bus);

    assert_eq!(cpu.y, 0xBEEF, "Y should be 0xBEEF");
    assert!(cpu.flag(puce16_core::cpu::FLAG_N), "N flag should be set");
}

#[test]
fn e02_29_plp_native() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00;
    cpu.s = 0x01FE;
    cpu.pc = 0x8000;
    bus.poke(0x01FF, &[0x30]);
    // PLP: opcode 28
    bus.poke(0x8000, &[0x28]);

    cpu.step(&mut bus);

    assert_eq!(cpu.p, 0x30, "P should be 0x30");
}

#[test]
fn e02_29_plp_x_clears_high() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // M=0, X=0 (16-bit modes)
    cpu.x = 0x1234;
    cpu.y = 0x5678;
    cpu.s = 0x01FE;
    cpu.pc = 0x8000;
    bus.poke(0x01FF, &[0x10]); // X flag set (bit 4)
    // PLP: opcode 28
    bus.poke(0x8000, &[0x28]);

    cpu.step(&mut bus);

    assert_eq!(cpu.x, 0x0034, "X high byte should be cleared when X flag set");
    assert_eq!(cpu.y, 0x0078, "Y high byte should be cleared when X flag set");
}

#[test]
fn e02_29_plp_emulation() {
    let mut cpu = make_cpu();
    cpu.e = true; // emulation mode
    let mut bus = FlatBus::new();
    cpu.p = 0x30;
    cpu.s = 0x01FE;
    cpu.pc = 0x8000;
    bus.poke(0x01FF, &[0x00]);
    // PLP: opcode 28
    bus.poke(0x8000, &[0x28]);

    cpu.step(&mut bus);

    assert_eq!(cpu.p, 0x30, "In emulation mode, M and X bits should be forced");
}

#[test]
fn e02_29_plb() {
    // PLB with value that doesn't set N flag
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M | FLAG_X;
    cpu.s = 0x01FE;
    cpu.pc = 0x8000;
    bus.poke(0x01FF, &[0x7E]);
    // PLB: opcode AB
    bus.poke(0x8000, &[0xAB]);

    cpu.step(&mut bus);

    assert_eq!(cpu.db, 0x7E, "DB should be 0x7E");
    assert!(!cpu.flag(puce16_core::cpu::FLAG_N), "N flag should not be set for 0x7E");
    // Cycles: 1 (opcode) + 2 (idle) + 1 (pull) = 4
    assert_eq!(bus.log.len(), 4, "PLB should take 4 cycles");

    // PLB with value that sets N flag
    let mut cpu2 = make_cpu();
    let mut bus2 = FlatBus::new();
    cpu2.p = FLAG_M | FLAG_X;
    cpu2.s = 0x01FE;
    cpu2.pc = 0x8000;
    bus2.poke(0x01FF, &[0x80]);
    bus2.poke(0x8000, &[0xAB]);

    cpu2.step(&mut bus2);

    assert_eq!(cpu2.db, 0x80, "DB should be 0x80");
    assert!(cpu2.flag(puce16_core::cpu::FLAG_N), "N flag should be set for 0x80");
}

#[test]
fn e02_29_pld() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.s = 0x01FD;
    cpu.d = 0x1234;
    cpu.pc = 0x8000;
    bus.poke(0x01FE, &[0x00, 0x00]);
    // PLD: opcode 2B
    bus.poke(0x8000, &[0x2B]);

    cpu.step(&mut bus);

    assert_eq!(cpu.d, 0, "D should be 0");
    assert!(cpu.flag(puce16_core::cpu::FLAG_Z), "Z flag should be set for D=0");
    assert_eq!(cpu.s, 0x01FF, "stack pointer should increment by 2");
    // Cycles: 1 (opcode) + 2 (idle) + 2 (pulls) = 5
    assert_eq!(bus.log.len(), 5, "PLD should take 5 cycles");
}

#[test]
fn e02_29_pla_emulation_wrap() {
    let mut cpu = make_cpu();
    cpu.e = true; // emulation mode
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M | FLAG_X; // M=1, X=1 (forced in emulation)
    cpu.s = 0x01FF;
    cpu.pc = 0x8000;
    bus.poke(0x0100, &[0x42]);
    // PLA: opcode 68
    bus.poke(0x8000, &[0x68]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a & 0xFF, 0x42, "A.l should be 0x42");
    assert_eq!(cpu.s, 0x0100, "stack pointer should wrap from 0x01FF to 0x0100");
}
