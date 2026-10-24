// E02-16: mem/store.rs - STA, STX, STY, STZ instructions (all addressing modes)

use puce16_core::cpu::{Cpu, FLAG_M, FLAG_X};
use puce16_core::cpu::testbus::FlatBus;

fn make_cpu() -> Cpu {
    let mut cpu = Cpu::new();
    cpu.e = false; // native mode by default
    cpu
}

#[test]
fn e02_16_sta_abs_8() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M; // M=1 (8-bit accumulator)
    cpu.a = 0x125A;
    cpu.pc = 0x8000;
    // STA abs: opcode 8D + address bytes (little-endian)
    bus.poke(0x8000, &[0x8D, 0x00, 0x20]);

    cpu.step(&mut bus);

    assert_eq!(bus.peek(0x2000), 0x5A, "low byte of A should be stored");
    // Cycles: 1 (opcode) + 2 (address bytes) + 1 (data write) = 4
    assert_eq!(bus.log.len(), 4, "STA abs (8-bit) should take 4 cycles");
    // Last cycle should be writing to 0x002000
    let last = bus.log.last().unwrap();
    assert_eq!(last.addr, 0x002000, "last cycle should write to 0x002000");
}

#[test]
fn e02_16_sta_abs_16() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // M=0 (16-bit accumulator)
    cpu.a = 0x1234;
    cpu.pc = 0x8000;
    // STA abs: opcode 8D + address bytes (little-endian)
    bus.poke(0x8000, &[0x8D, 0x00, 0x20]);

    cpu.step(&mut bus);

    assert_eq!(bus.peek(0x2000), 0x34, "low byte of A should be stored");
    assert_eq!(bus.peek(0x2001), 0x12, "high byte of A should be stored");
    // Cycles: 1 (opcode) + 2 (address bytes) + 2 (data writes) = 5
    assert_eq!(bus.log.len(), 5, "STA abs (16-bit) should take 5 cycles");
}

#[test]
fn e02_16_sta_abs_x_8() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M | FLAG_X; // M=1, X=1 (8-bit modes)
    cpu.x = 1;
    cpu.a = 0x07;
    cpu.pc = 0x8000;
    // STA abs,x: opcode 9D + address bytes (little-endian)
    bus.poke(0x8000, &[0x9D, 0x00, 0x20]);

    cpu.step(&mut bus);

    assert_eq!(bus.peek(0x2001), 0x07, "A should be stored at addr+X");
    // Cycles: 1 (opcode) + 2 (address bytes) + 1 (idle for index) + 1 (data write) = 5
    assert_eq!(bus.log.len(), 5, "STA abs,x (8-bit) should take 5 cycles with forced idle");
}

#[test]
fn e02_16_sta_ind_y() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M | FLAG_X; // M=1, X=1 (8-bit modes)
    cpu.d = 0x0000;
    cpu.y = 0x10;
    cpu.a = 0x09;
    cpu.pc = 0x8000;
    // STA (dp),y: opcode 91 + operand byte
    bus.poke(0x8000, &[0x91, 0x30]);
    // Pointer at direct page address 0x0030
    bus.poke(0x30, &[0x00, 0x20]);

    cpu.step(&mut bus);

    assert_eq!(bus.peek(0x2010), 0x09, "A should be stored via (dp),y addressing");
    // Cycles: 1 (opcode) + 1 (operand) + 2 (pointer reads) + 1 (idle for index) + 1 (data write) = 6
    assert_eq!(bus.log.len(), 6, "STA (dp),y should take 6 cycles");
}

#[test]
fn e02_16_sta_dp() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M; // M=1 (8-bit accumulator)
    cpu.d = 0x0000;
    cpu.a = 0x33;
    cpu.pc = 0x8000;
    // STA dp: opcode 85 + operand byte
    bus.poke(0x8000, &[0x85, 0x10]);

    cpu.step(&mut bus);

    assert_eq!(bus.peek(0x10), 0x33, "A should be stored at direct page address");
    // Cycles: 1 (opcode) + 1 (operand byte) + 1 (data write) = 3
    assert_eq!(bus.log.len(), 3, "STA dp should take 3 cycles");
}

#[test]
fn e02_16_sta_long() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M; // M=1 (8-bit accumulator)
    cpu.a = 0x44;
    cpu.pc = 0x8000;
    // STA long: opcode 8F + address bytes (addr lo, addr hi, bank)
    bus.poke(0x8000, &[0x8F, 0x56, 0x34, 0x12]);

    cpu.step(&mut bus);

    assert_eq!(bus.peek(0x123456), 0x44, "A should be stored at long address");
    // Cycles: 1 (opcode) + 3 (address bytes) + 1 (data write) = 5
    assert_eq!(bus.log.len(), 5, "STA long should take 5 cycles");
}

#[test]
fn e02_16_stx_abs_16() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // M=0, X=0 (16-bit modes)
    cpu.x = 0xABCD;
    cpu.pc = 0x8000;
    // STX abs: opcode 8E + address bytes (little-endian)
    bus.poke(0x8000, &[0x8E, 0x00, 0x20]);

    cpu.step(&mut bus);

    assert_eq!(bus.peek(0x2000), 0xCD, "low byte of X should be stored");
    assert_eq!(bus.peek(0x2001), 0xAB, "high byte of X should be stored");
    // Cycles: 1 (opcode) + 2 (address bytes) + 2 (data writes) = 5
    assert_eq!(bus.log.len(), 5, "STX abs (16-bit) should take 5 cycles");
}

#[test]
fn e02_16_sty_dp_8() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M | FLAG_X; // M=1, X=1 (8-bit modes)
    cpu.d = 0x0000;
    cpu.y = 0x12;
    cpu.pc = 0x8000;
    // STY dp: opcode 84 + operand byte
    bus.poke(0x8000, &[0x84, 0x10]);

    cpu.step(&mut bus);

    assert_eq!(bus.peek(0x10), 0x12, "Y should be stored at direct page address");
    // Cycles: 1 (opcode) + 1 (operand byte) + 1 (data write) = 3
    assert_eq!(bus.log.len(), 3, "STY dp should take 3 cycles");
}

#[test]
fn e02_16_stz_dp_16() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // M=0 (16-bit mode)
    cpu.d = 0x0000;
    cpu.pc = 0x8000;
    // STZ dp: opcode 64 + operand byte
    bus.poke(0x8000, &[0x64, 0x10]);
    // Pre-fill with non-zero values to verify they're overwritten
    bus.poke(0x10, &[0xFF, 0xFF]);

    cpu.step(&mut bus);

    assert_eq!(bus.peek(0x10), 0x00, "low byte should be zeroed");
    assert_eq!(bus.peek(0x11), 0x00, "high byte should be zeroed");
    // Cycles: 1 (opcode) + 1 (operand byte) + 2 (data writes) = 4
    assert_eq!(bus.log.len(), 4, "STZ dp (16-bit) should take 4 cycles");
}

#[test]
fn e02_16_stz_abs_x() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = FLAG_M; // M=1, X=0 (8-bit accumulator, 16-bit index)
    cpu.x = 0x0001;
    cpu.pc = 0x8000;
    // STZ abs,x: opcode 9E + address bytes (little-endian)
    bus.poke(0x8000, &[0x9E, 0x00, 0x20]);
    // Pre-fill with non-zero value to verify it's overwritten
    bus.poke(0x2001, &[0x55]);

    cpu.step(&mut bus);

    assert_eq!(bus.peek(0x2001), 0x00, "memory at addr+X should be zeroed");
    // Cycles: 1 (opcode) + 2 (address bytes) + 1 (idle for index) + 1 (data write) = 5
    assert_eq!(bus.log.len(), 5, "STZ abs,x should take 5 cycles with forced idle");
}

#[test]
fn e02_16_flags_untouched() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0xE3; // All flags set except C and V (N=1, Z=1, I=1, D=1, B=1)
    cpu.a = 0x00;
    cpu.x = 0x00;
    cpu.y = 0x00;
    cpu.d = 0x0000;
    cpu.pc = 0x8000;

    // STA dp (opcode 85) - should not modify flags
    bus.poke(0x8000, &[0x85, 0x10]);
    cpu.step(&mut bus);
    assert_eq!(cpu.p, 0xE3, "STA should not modify flags");

    // STX dp (opcode 86) - should not modify flags
    bus.clear_log();
    cpu.pc = 0x8001;
    bus.poke(0x8001, &[0x86, 0x11]);
    cpu.step(&mut bus);
    assert_eq!(cpu.p, 0xE3, "STX should not modify flags");

    // STZ dp (opcode 64) - should not modify flags
    bus.clear_log();
    cpu.pc = 0x8002;
    bus.poke(0x8002, &[0x64, 0x12]);
    cpu.step(&mut bus);
    assert_eq!(cpu.p, 0xE3, "STZ should not modify flags");
}