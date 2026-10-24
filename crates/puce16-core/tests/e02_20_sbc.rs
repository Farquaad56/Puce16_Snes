// E02-20: alu/sbc.rs - SBC binary (8 and 16 bits, all modes)

use puce16_core::cpu::{Cpu, FLAG_C, FLAG_M, FLAG_N, FLAG_V, FLAG_Z};
use puce16_core::cpu::testbus::FlatBus;

fn make_cpu() -> Cpu {
    let mut cpu = Cpu::new();
    cpu.e = false; // native mode by default
    cpu
}

#[test]
fn e02_20_imm_8_basic() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M | FLAG_C; // M=1 (8-bit), C=1
    cpu.a = 0x1210;
    cpu.pc = 0x8000;
    // SBC #imm8: opcode E9 + operand byte
    bus.poke(0x8000, &[0xE9, 0x05]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a & 0xFF, 0x0B, "A.l should be 0x0B (10 - 5 + carry)");
    assert!(cpu.flag(FLAG_C), "C flag should be set (no borrow)");
    assert!(!cpu.flag(FLAG_N), "N flag should not be set");
    assert!(!cpu.flag(FLAG_Z), "Z flag should not be set");
    assert!(!cpu.flag(FLAG_V), "V flag should not be set");
    // Cycles: 1 (opcode) + 1 (immediate byte) = 2
    assert_eq!(bus.log.len(), 2, "SBC #imm8 should take 2 cycles");
}

#[test]
fn e02_20_imm_8_borrow_in() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M; // M=1 (8-bit), C=0 (borrow in)
    cpu.a = 0x0010;
    cpu.pc = 0x8000;
    // SBC #imm8: opcode E9 + operand byte
    bus.poke(0x8000, &[0xE9, 0x05]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a & 0xFF, 0x0A, "A.l should be 0x0A (10 - 5 - borrow)");
    assert!(cpu.flag(FLAG_C), "C flag should be set (no further borrow)");
}

#[test]
fn e02_20_imm_8_underflow() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M | FLAG_C; // M=1, C=1
    cpu.a = 0x0000;
    cpu.pc = 0x8000;
    // SBC #imm8: opcode E9 + operand byte
    bus.poke(0x8000, &[0xE9, 0x01]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a & 0xFF, 0xFF, "A.l should be 0xFF (underflow)");
    assert!(!cpu.flag(FLAG_C), "C flag should not be set (borrow needed)");
    assert!(cpu.flag(FLAG_N), "N flag should be set");
}

#[test]
fn e02_20_imm_8_overflow_pos() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M | FLAG_C; // M=1, C=1
    cpu.a = 0x0080;
    cpu.pc = 0x8000;
    // SBC #imm8: opcode E9 + operand byte
    bus.poke(0x8000, &[0xE9, 0x01]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a & 0xFF, 0x7F, "A.l should be 0x7F (overflow)");
    assert!(cpu.flag(FLAG_V), "V flag should be set");
    assert!(cpu.flag(FLAG_C), "C flag should be set");
}

#[test]
fn e02_20_imm_8_overflow_neg() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M | FLAG_C; // M=1, C=1
    cpu.a = 0x0050;
    cpu.pc = 0x8000;
    // SBC #imm8: opcode E9 + operand byte (subtract -80)
    bus.poke(0x8000, &[0xE9, 0xB0]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a & 0xFF, 0xA0, "A.l should be 0xA0 (overflow negative)");
    assert!(cpu.flag(FLAG_V), "V flag should be set");
    assert!(!cpu.flag(FLAG_C), "C flag should not be set");
    assert!(cpu.flag(FLAG_N), "N flag should be set");
}

#[test]
fn e02_20_imm_16_basic() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_C; // M=0 (16-bit), C=1
    cpu.a = 0x1000;
    cpu.pc = 0x8000;
    // SBC #imm16: opcode E9 + two operand bytes
    bus.poke(0x8000, &[0xE9, 0x01, 0x00]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0x0FFF, "A should be 0x0FFF (4096 - 1)");
    assert!(cpu.flag(FLAG_C), "C flag should be set");
    // Cycles: 1 (opcode) + 2 (immediate bytes) = 3
    assert_eq!(bus.log.len(), 3, "SBC #imm16 should take 3 cycles");
}

#[test]
fn e02_20_imm_16_underflow() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_C; // M=0, C=1
    cpu.a = 0x0000;
    cpu.pc = 0x8000;
    // SBC #imm16: opcode E9 + two operand bytes
    bus.poke(0x8000, &[0xE9, 0x01, 0x00]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0xFFFF, "A should be 0xFFFF (underflow)");
    assert!(!cpu.flag(FLAG_C), "C flag should not be set (borrow needed)");
    assert!(cpu.flag(FLAG_N), "N flag should be set");
}

#[test]
fn e02_20_imm_16_zero() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_C; // M=0, C=1
    cpu.a = 0x1234;
    cpu.pc = 0x8000;
    // SBC #imm16: opcode E9 + two operand bytes (subtract same value)
    bus.poke(0x8000, &[0xE9, 0x34, 0x12]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0x0000, "A should be 0 (result is zero)");
    assert!(cpu.flag(FLAG_Z), "Z flag should be set");
    assert!(cpu.flag(FLAG_C), "C flag should be set");
}

#[test]
fn e02_20_abs_8() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M | FLAG_C; // M=1, C=1
    cpu.db = 0x00;
    cpu.a = 0x000A;
    cpu.pc = 0x8000;
    // SBC abs: opcode ED + address bytes (little-endian)
    bus.poke(0x8000, &[0xED, 0x00, 0x20]);
    // Data at absolute address 0x002000
    bus.poke(0x2000, &[0x03]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a & 0xFF, 7, "A.l should be 7 (10 - 3)");
    // Cycles: 1 (opcode) + 2 (address bytes) + 1 (data read) = 4
    assert_eq!(bus.log.len(), 4, "SBC abs should take 4 cycles");
}
