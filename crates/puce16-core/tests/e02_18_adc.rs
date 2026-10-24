// E02-18: alu/adc.rs - ADC binary (8 and 16 bits, all modes)

use puce16_core::cpu::{Cpu, FLAG_C, FLAG_M, FLAG_N, FLAG_V, FLAG_X, FLAG_Z};
use puce16_core::cpu::testbus::FlatBus;

fn make_cpu() -> Cpu {
    let mut cpu = Cpu::new();
    cpu.e = false; // native mode by default
    cpu
}

#[test]
fn e02_18_imm_8_simple() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M; // M=1 (8-bit accumulator)
    cpu.a = 0x1210;
    cpu.pc = 0x8000;
    // ADC #imm8: opcode 69 + operand byte
    bus.poke(0x8000, &[0x69, 0x05]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0x1215, "A should be 0x1215 (high byte preserved)");
    assert!(!cpu.flag(FLAG_N), "N flag should not be set");
    assert!(!cpu.flag(FLAG_Z), "Z flag should not be set");
    assert!(!cpu.flag(FLAG_C), "C flag should not be set");
    assert!(!cpu.flag(FLAG_V), "V flag should not be set");
    // Cycles: 1 (opcode) + 1 (immediate byte) = 2
    assert_eq!(bus.log.len(), 2, "ADC #imm8 should take 2 cycles");
}

#[test]
fn e02_18_imm_8_carry_out() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M; // M=1 (8-bit accumulator)
    cpu.a = 0x12FF;
    cpu.pc = 0x8000;
    // ADC #imm8: opcode 69 + operand byte
    bus.poke(0x8000, &[0x69, 0x01]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0x1200, "A should be 0x1200 (carry out)");
    assert!(cpu.flag(FLAG_C), "C flag should be set");
    assert!(cpu.flag(FLAG_Z), "Z flag should be set");
}

#[test]
fn e02_18_imm_8_overflow() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M; // M=1 (8-bit accumulator)
    cpu.a = 0x0050;
    cpu.pc = 0x8000;
    // ADC #imm8: opcode 69 + operand byte
    bus.poke(0x8000, &[0x69, 0x50]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a & 0xFF, 0xA0, "A.l should be 0xA0 (overflow)");
    assert!(cpu.flag(FLAG_V), "V flag should be set");
    assert!(cpu.flag(FLAG_N), "N flag should be set");
    assert!(!cpu.flag(FLAG_C), "C flag should not be set");
}

#[test]
fn e02_18_imm_8_carry_in() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M | FLAG_C; // M=1, C=1 (carry in)
    cpu.a = 0x0010;
    cpu.pc = 0x8000;
    // ADC #imm8: opcode 69 + operand byte
    bus.poke(0x8000, &[0x69, 0x20]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a & 0xFF, 0x31, "A.l should be 0x31 (with carry in)");
}

#[test]
fn e02_18_imm_16_overflow() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // M=0 (16-bit accumulator)
    cpu.a = 0x7FFF;
    cpu.pc = 0x8000;
    // ADC #imm16: opcode 69 + two operand bytes
    bus.poke(0x8000, &[0x69, 0x01, 0x00]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0x8000, "A should be 0x8000 (overflow)");
    assert!(cpu.flag(FLAG_V), "V flag should be set");
    assert!(cpu.flag(FLAG_N), "N flag should be set");
    assert!(!cpu.flag(FLAG_C), "C flag should not be set");
    // Cycles: 1 (opcode) + 2 (immediate bytes) = 3
    assert_eq!(bus.log.len(), 3, "ADC #imm16 should take 3 cycles");
}

#[test]
fn e02_18_imm_16_carry_zero() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // M=0 (16-bit accumulator)
    cpu.a = 0xFFFF;
    cpu.pc = 0x8000;
    // ADC #imm16: opcode 69 + two operand bytes
    bus.poke(0x8000, &[0x69, 0x01, 0x00]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0x0000, "A should be 0x0000 (carry out)");
    assert!(cpu.flag(FLAG_C), "C flag should be set");
    assert!(cpu.flag(FLAG_Z), "Z flag should be set");
    assert!(!cpu.flag(FLAG_V), "V flag should not be set");
}

#[test]
fn e02_18_imm_16_neg_neg() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // M=0 (16-bit accumulator)
    cpu.a = 0x8000;
    cpu.pc = 0x8000;
    // ADC #imm16: opcode 69 + two operand bytes
    bus.poke(0x8000, &[0x69, 0x00, 0x80]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0x0000, "A should be 0 (overflow wraps to zero)");
    assert!(cpu.flag(FLAG_C), "C flag should be set");
    assert!(cpu.flag(FLAG_V), "V flag should be set");
    assert!(cpu.flag(FLAG_Z), "Z flag should be set");
}

#[test]
fn e02_18_abs_8() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M; // M=1 (8-bit accumulator)
    cpu.db = 0x00;
    cpu.a = 0x0001;
    cpu.pc = 0x8000;
    // ADC abs: opcode 6D + address bytes (little-endian)
    bus.poke(0x8000, &[0x6D, 0x00, 0x20]);
    // Data at absolute address 0x002000
    bus.poke(0x2000, &[0x03]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a & 0xFF, 4, "A.l should be 4 (1 + 3)");
    // Cycles: 1 (opcode) + 2 (address bytes) + 1 (data read) = 4
    assert_eq!(bus.log.len(), 4, "ADC abs should take 4 cycles");
}

#[test]
fn e02_18_dp_16() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // M=0 (16-bit accumulator)
    cpu.d = 0x0000;
    cpu.a = 0x0001;
    cpu.pc = 0x8000;
    // ADC dp: opcode 65 + operand byte
    bus.poke(0x8000, &[0x65, 0x10]);
    // Data at direct page address 0x0010 (little-endian)
    bus.poke(0x10, &[0x34, 0x12]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0x1235, "A should be 0x1235 (1 + 0x1234)");
    // Cycles: 1 (opcode) + 1 (operand byte) + 2 (data reads) = 4
    assert_eq!(bus.log.len(), 4, "ADC dp (16-bit) should take 4 cycles");
}

#[test]
fn e02_18_emulation() {
    let mut cpu = make_cpu();
    cpu.e = true; // emulation mode
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M | FLAG_X; // M=1, X=1 (forced in emulation)
    cpu.a = 0x00FF;
    cpu.pc = 0x8000;
    // ADC #imm8: opcode 69 + operand byte
    bus.poke(0x8000, &[0x69, 0x01]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a & 0xFF, 0x00, "A.l should be 0 (FF + 1 = 100)");
    assert!(cpu.flag(FLAG_C), "C flag should be set");
    assert!(cpu.flag(FLAG_Z), "Z flag should be set");
}
