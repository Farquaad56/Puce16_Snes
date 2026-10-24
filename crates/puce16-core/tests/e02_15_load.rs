// E02-15: mem/load.rs - LDA, LDX, LDY instructions (all addressing modes)

use puce16_core::cpu::{Cpu, FLAG_M, FLAG_N, FLAG_X, FLAG_Z};
use puce16_core::cpu::testbus::FlatBus;

fn make_cpu() -> Cpu {
    let mut cpu = Cpu::new();
    cpu.e = false; // native mode by default
    cpu
}

#[test]
fn e02_15_lda_imm_8_keeps_high_byte() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M; // M=1 (8-bit accumulator)
    cpu.a = 0x1234;
    cpu.pc = 0x8000;
    // LDA #imm8: opcode A9 + operand byte
    bus.poke(0x8000, &[0xA9, 0x80]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0x1280, "high byte should be preserved in 8-bit mode");
    assert!(cpu.flag(FLAG_N), "N flag should be set for value >= 0x80");
    assert!(!cpu.flag(FLAG_Z), "Z flag should not be set");
    // Cycles: 1 (opcode) + 1 (immediate byte) = 2
    assert_eq!(bus.log.len(), 2, "LDA #imm8 should take 2 cycles");
}

#[test]
fn e02_15_lda_imm_16_zero() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // M=0 (16-bit accumulator)
    cpu.a = 0x5555;
    cpu.pc = 0x8000;
    // LDA #imm16: opcode A9 + two operand bytes
    bus.poke(0x8000, &[0xA9, 0x00, 0x00]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0x0000, "A should be loaded with zero");
    assert!(cpu.flag(FLAG_Z), "Z flag should be set for value 0");
    assert!(!cpu.flag(FLAG_N), "N flag should not be set");
    // Cycles: 1 (opcode) + 2 (immediate bytes) = 3
    assert_eq!(bus.log.len(), 3, "LDA #imm16 should take 3 cycles");
}

#[test]
fn e02_15_lda_abs() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M; // M=1 (8-bit accumulator)
    cpu.db = 0x00;
    cpu.pc = 0x8000;
    // LDA abs: opcode AD + address bytes (little-endian)
    bus.poke(0x8000, &[0xAD, 0x00, 0x20]);
    // Data at absolute address 0x002000
    bus.poke(0x2000, &[0x42]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a & 0xFF, 0x42, "A.l should be loaded from memory");
    // Cycles: 1 (opcode) + 2 (address bytes) + 1 (data read) = 4
    assert_eq!(bus.log.len(), 4, "LDA abs should take 4 cycles");
    // Last cycle should be reading data at 0x002000
    let last = bus.log.last().unwrap();
    assert_eq!(last.addr, 0x002000, "last cycle should read from 0x002000");
}

#[test]
fn e02_15_lda_long() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M; // M=1 (8-bit accumulator)
    cpu.pc = 0x8000;
    // LDA long: opcode AF + address bytes (addr lo, addr hi, bank)
    bus.poke(0x8000, &[0xAF, 0x56, 0x34, 0x12]);
    // Data at long address 0x123456
    bus.poke(0x123456, &[0x99]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a & 0xFF, 0x99, "A.l should be loaded from long address");
    assert!(cpu.flag(FLAG_N), "N flag should be set for value >= 0x80");
    // Cycles: 1 (opcode) + 3 (address bytes) + 1 (data read) = 5
    assert_eq!(bus.log.len(), 5, "LDA long should take 5 cycles");
}

#[test]
fn e02_15_lda_dp_16() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // M=0 (16-bit accumulator)
    cpu.d = 0x0000;
    cpu.pc = 0x8000;
    // LDA dp: opcode A5 + operand byte
    bus.poke(0x8000, &[0xA5, 0x10]);
    // Data at direct page address 0x0010 (little-endian)
    bus.poke(0x10, &[0x34, 0x12]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0x1234, "A should be loaded with 16-bit value from DP");
    // Cycles: 1 (opcode) + 1 (operand byte) + 2 (data reads) = 4
    assert_eq!(bus.log.len(), 4, "LDA dp (16-bit) should take 4 cycles");
}

#[test]
fn e02_15_lda_ind_y() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M | FLAG_X; // M=1, X=1 (8-bit modes)
    cpu.db = 0x00;
    cpu.y = 0x10;
    cpu.pc = 0x8000;
    // LDA (dp),y: opcode B1 + operand byte
    bus.poke(0x8000, &[0xB1, 0x30]);
    // Pointer at direct page address 0x0030
    bus.poke(0x30, &[0x00, 0x20]);
    // Data at effective address (0x2000 + Y=0x10) = 0x2010
    bus.poke(0x2010, &[0x77]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a & 0xFF, 0x77, "A.l should be loaded via (dp),y addressing");
    // Cycles: 1 (opcode) + 1 (operand) + 2 (pointer reads) + 1 (data read) = 5
    assert_eq!(bus.log.len(), 5, "LDA (dp),y should take 5 cycles");
}

#[test]
fn e02_15_lda_sr_ind_y() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M; // M=1, X=0 (8-bit accumulator, 16-bit index)
    cpu.s = 0x01F0;
    cpu.y = 0x0002;
    cpu.pc = 0x8000;
    // LDA (sr),y: opcode B3 + operand byte
    bus.poke(0x8000, &[0xB3, 0x04]);
    // Pointer at stack-relative address S+sr = 0x01F0+0x04 = 0x01F4
    bus.poke(0x01F4, &[0x00, 0x30]);
    // Data at effective address (0x3000 + Y=0x02) = 0x3002
    bus.poke(0x3002, &[0x5A]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a & 0xFF, 0x5A, "A.l should be loaded via (sr),y addressing");
    // Cycles: 1 (opcode) + 1 (operand) + idle + 2 (pointer reads) + idle + 1 (data read) = 7
    assert_eq!(bus.log.len(), 7, "LDA (sr),y should take 7 cycles");
}

#[test]
fn e02_15_ldx_imm_8() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M | FLAG_X; // M=1, X=1 (8-bit modes)
    cpu.x = 0x0034;
    cpu.pc = 0x8000;
    // LDX #imm8: opcode A2 + operand byte
    bus.poke(0x8000, &[0xA2, 0xFF]);

    cpu.step(&mut bus);

    assert_eq!(cpu.x, 0x00FF, "X should be loaded with zero-extended value");
    assert!(cpu.flag(FLAG_N), "N flag should be set for value >= 0x80");
    // Cycles: 1 (opcode) + 1 (immediate byte) = 2
    assert_eq!(bus.log.len(), 2, "LDX #imm8 should take 2 cycles");
}

#[test]
fn e02_15_ldx_imm_16() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // M=0, X=0 (16-bit modes)
    cpu.pc = 0x8000;
    // LDX #imm16: opcode A2 + two operand bytes
    bus.poke(0x8000, &[0xA2, 0x34, 0x12]);

    cpu.step(&mut bus);

    assert_eq!(cpu.x, 0x1234, "X should be loaded with 16-bit value");
    // Cycles: 1 (opcode) + 2 (immediate bytes) = 3
    assert_eq!(bus.log.len(), 3, "LDX #imm16 should take 3 cycles");
}

#[test]
fn e02_15_ldy_dpx() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M | FLAG_X; // M=1, X=1 (8-bit modes)
    cpu.d = 0x0000;
    cpu.x = 2;
    cpu.pc = 0x8000;
    // LDY dp,x: opcode B4 + operand byte
    bus.poke(0x8000, &[0xB4, 0x10]);
    // Data at direct page address D+operand+X = 0+0x10+2 = 0x12
    bus.poke(0x12, &[0x77]);

    cpu.step(&mut bus);

    assert_eq!(cpu.y, 0x77, "Y should be loaded via dp,x addressing");
    // Cycles: 1 (opcode) + 1 (operand) + idle (index cycle) + 1 (data read) = 4
    assert_eq!(bus.log.len(), 4, "LDY dp,x should take 4 cycles");
}

#[test]
fn e02_15_lda_emulation() {
    let mut cpu = make_cpu();
    cpu.e = true; // emulation mode
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M | FLAG_X; // M=1, X=1 (forced in emulation)
    cpu.pc = 0x8000;
    // LDA #imm8: opcode A9 + operand byte
    bus.poke(0x8000, &[0xA9, 0x80]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a & 0xFF, 0x80, "A.l should be loaded in emulation mode");
    assert!(cpu.flag(FLAG_N), "N flag should be set for value >= 0x80");
    // Cycles: 1 (opcode) + 1 (immediate byte) = 2
    assert_eq!(bus.log.len(), 2, "LDA #imm8 in emulation should take 2 cycles");
}
