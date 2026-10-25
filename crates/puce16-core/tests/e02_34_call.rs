// E02-34: flow/call.rs - JSR, JSL, RTS, RTL instructions

use puce16_core::cpu::{Cpu};
use puce16_core::cpu::testbus::FlatBus;

fn make_cpu() -> Cpu {
    let mut cpu = Cpu::new();
    cpu.e = false; // native mode by default
    cpu
}

#[test]
fn e02_34_jsr_abs() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.s = 0x01FF;
    cpu.pc = 0x8000;
    // JSR $1234: opcode 20, lo=0x34, hi=0x12
    bus.poke(0x8000, &[0x20, 0x34, 0x12]);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x1234, "JSR Abs: PC should be set to target");
    assert_eq!(cpu.s, 0x01FD, "JSR Abs: stack pointer decremented by 2");
    // PC pushed = address of last byte of instruction (0x8002)
    assert_eq!(bus.peek(0x01FF), 0x80, "JSR Abs: high byte of return address pushed");
    assert_eq!(bus.peek(0x01FE), 0x02, "JSR Abs: low byte of return address pushed");
    assert_eq!(bus.log.len(), 6, "JSR Abs: 6 cycles (opcode + lo + hi + idle + push_hi + push_lo)");

    // Verify cycle sequence: R,R,R,Idle,W,W
    let kinds = bus.log.iter().map(|c| c.kind).collect::<Vec<_>>();
    assert_eq!(kinds[0], puce16_core::cpu::testbus::CycleKind::Read);
    assert_eq!(kinds[1], puce16_core::cpu::testbus::CycleKind::Read);
    assert_eq!(kinds[2], puce16_core::cpu::testbus::CycleKind::Read);
    assert_eq!(kinds[3], puce16_core::cpu::testbus::CycleKind::Idle);
    assert_eq!(kinds[4], puce16_core::cpu::testbus::CycleKind::Write);
    assert_eq!(kinds[5], puce16_core::cpu::testbus::CycleKind::Write);
}

#[test]
fn e02_34_jsl() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.s = 0x01FF;
    cpu.pb = 0;
    cpu.pc = 0x8000;
    // JSL $12:3456: opcode 22, lo=0x56, hi=0x34, bank=0x12
    bus.poke(0x8000, &[0x22, 0x56, 0x34, 0x12]);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x3456, "JSL: PC should be set to target");
    assert_eq!(cpu.pb, 0x12, "JSL: PB should be set to bank");
    assert_eq!(cpu.s, 0x01FC, "JSL: stack pointer decremented by 3");
    // Pushed: PB(0), PC-1 high(0x80), PC-1 low(0x03)
    assert_eq!(bus.peek(0x01FF), 0x00, "JSL: PB pushed");
    assert_eq!(bus.peek(0x01FE), 0x80, "JSL: high byte of return address pushed");
    assert_eq!(bus.peek(0x01FD), 0x03, "JSL: low byte of return address pushed");
    assert_eq!(bus.log.len(), 8, "JSL: 8 cycles (opcode + lo + hi + push_PB + idle + bank + push_hi + push_lo)");

    // Verify cycle sequence: R,R,R,W,Idle,R,W,W
    let kinds = bus.log.iter().map(|c| c.kind).collect::<Vec<_>>();
    assert_eq!(kinds[0], puce16_core::cpu::testbus::CycleKind::Read);
    assert_eq!(kinds[1], puce16_core::cpu::testbus::CycleKind::Read);
    assert_eq!(kinds[2], puce16_core::cpu::testbus::CycleKind::Read);
    assert_eq!(kinds[3], puce16_core::cpu::testbus::CycleKind::Write);
    assert_eq!(kinds[4], puce16_core::cpu::testbus::CycleKind::Idle);
    assert_eq!(kinds[5], puce16_core::cpu::testbus::CycleKind::Read);
    assert_eq!(kinds[6], puce16_core::cpu::testbus::CycleKind::Write);
    assert_eq!(kinds[7], puce16_core::cpu::testbus::CycleKind::Write);
}

#[test]
fn e02_34_jsr_abs_x_ind() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // clear all flags
    cpu.s = 0x01FF;
    cpu.x = 2;
    cpu.pb = 0;
    cpu.pc = 0x8000;
    // Pointer at $0:2002 -> $9000 (after X=2 offset)
    bus.poke(0x2002, &[0x00, 0x90]);
    // JSR ($2000,X): opcode FC, lo=0x00, hi=0x20
    bus.poke(0x8000, &[0xFC, 0x00, 0x20]);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x9000, "JSR AbsXInd: PC should be set to indirect target");
    assert_eq!(cpu.s, 0x01FD, "JSR AbsXInd: stack pointer decremented by 2");
    // Pushed: PC high(0x80), PC low(0x02) - PC at time of push is address of aah byte
    assert_eq!(bus.peek(0x01FF), 0x80, "JSR AbsXInd: high byte of return address pushed");
    assert_eq!(bus.peek(0x01FE), 0x02, "JSR AbsXInd: low byte of return address pushed");
    assert_eq!(bus.log.len(), 8, "JSR AbsXInd: 8 cycles (opcode + lo + push_hi + push_lo + hi + idle + read_lo + read_hi)");
}

#[test]
fn e02_34_rts() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.s = 0x01FD;
    cpu.pc = 0x8000;
    // Return address on stack: $8002 (so RTS returns to $8003)
    bus.poke(0x01FE, &[0x02, 0x80]);
    // RTS: opcode 60
    bus.poke(0x8000, &[0x60]);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x8003, "RTS: PC should be pulled address + 1");
    assert_eq!(cpu.s, 0x01FF, "RTS: stack pointer incremented by 2");
    assert_eq!(bus.log.len(), 6, "RTS: 6 cycles (opcode + idle + idle + pull_lo + pull_hi + idle)");

    // Verify cycle sequence: R,Idle,Idle,R,R,Idle
    let kinds = bus.log.iter().map(|c| c.kind).collect::<Vec<_>>();
    assert_eq!(kinds[0], puce16_core::cpu::testbus::CycleKind::Read);
    assert_eq!(kinds[1], puce16_core::cpu::testbus::CycleKind::Idle);
    assert_eq!(kinds[2], puce16_core::cpu::testbus::CycleKind::Idle);
    assert_eq!(kinds[3], puce16_core::cpu::testbus::CycleKind::Read);
    assert_eq!(kinds[4], puce16_core::cpu::testbus::CycleKind::Read);
    assert_eq!(kinds[5], puce16_core::cpu::testbus::CycleKind::Idle);
}

#[test]
fn e02_34_rtl() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.s = 0x01FC;
    cpu.pc = 0x8000;
    // Return address on stack: $8003 (so RTL returns to $8004), bank=0x05
    bus.poke(0x01FD, &[0x03, 0x80, 0x05]);
    // RTL: opcode 6B
    bus.poke(0x8000, &[0x6B]);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x8004, "RTL: PC should be pulled address + 1");
    assert_eq!(cpu.pb, 0x05, "RTL: PB should be restored from stack");
    assert_eq!(cpu.s, 0x01FF, "RTL: stack pointer incremented by 3");
    assert_eq!(bus.log.len(), 6, "RTL: 6 cycles (opcode + idle + idle + pull_lo + pull_hi + pull_bank)");
}

#[test]
fn e02_34_jsr_rts_roundtrip() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.s = 0x01FF;
    cpu.pc = 0x8000;
    // JSR $9000 at 0x8000, RTS at 0x9000
    bus.poke(0x8000, &[0x20, 0x00, 0x90]);
    bus.poke(0x9000, &[0x60]);

    // Step JSR
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x9000, "JSR should jump to subroutine");

    // Step RTS
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x8003, "RTS should return to instruction after JSR");
    assert_eq!(cpu.s, 0x01FF, "Stack should be restored after roundtrip");
}

#[test]
fn e02_34_jsl_rtl_roundtrip() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.s = 0x01FF;
    cpu.pb = 0;
    cpu.pc = 0x8000;
    // JSL $12:9000 at 0x8000, RTL at $12:9000
    bus.poke(0x8000, &[0x22, 0x00, 0x90, 0x12]);
    bus.poke(0x129000, &[0x6B]);

    // Step JSL
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x9000, "JSL should jump to subroutine");
    assert_eq!(cpu.pb, 0x12, "JSL should set PB to target bank");

    // Step RTL
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x8004, "RTL should return to instruction after JSL");
    assert_eq!(cpu.pb, 0, "RTL should restore original PB");
    assert_eq!(cpu.s, 0x01FF, "Stack should be restored after roundtrip");
}

#[test]
fn e02_34_jsr_emulation_wrap() {
    let mut cpu = make_cpu();
    cpu.e = true; // emulation mode
    let mut bus = FlatBus::new();
    cpu.p = 0x30;
    cpu.s = 0x0100;
    cpu.pc = 0x8000;
    // JSR $9000 in emulation mode - stack wraps within page 1
    bus.poke(0x8000, &[0x20, 0x00, 0x90]);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x9000, "JSR should jump to target in emulation mode");
    // In emulation mode, stack wraps: S=0x100 -> push at 0x100, then S=0xFF (masked)
    assert_eq!(bus.peek(0x0100), 0x80, "JSR emu: high byte pushed at page boundary");
    assert_eq!(bus.peek(0x01FF), 0x02, "JSR emu: low byte wraps to end of page");
    assert_eq!(cpu.s & 0xFF, 0xFE, "JSR emu: stack pointer wrapped correctly");
}
