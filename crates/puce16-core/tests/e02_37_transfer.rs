// E02-37: misc/transfer.rs - TAX TAY TXA TYA TSX TXS TXY TYX TCD TDC TCS TSC

use puce16_core::cpu::{Cpu, FLAG_N, FLAG_Z};
use puce16_core::cpu::testbus::FlatBus;

fn make_cpu() -> Cpu {
    let mut cpu = Cpu::new();
    cpu.e = false; // native mode by default
    cpu
}

#[test]
fn e02_37_tax_16() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x20; // M=1, X=0 (wide index)
    cpu.a = 0x1234;
    cpu.pc = 0x8000;
    // TAX: opcode AA
    bus.poke(0x8000, &[0xAA]);

    cpu.step(&mut bus);

    assert_eq!(cpu.x, 0x1234, "TAX 16-bit: X = A (full width)");
    assert_eq!(bus.log.len(), 2, "TAX: 2 cycles (opcode + idle)");
}

#[test]
fn e02_37_tax_8() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x10; // X=1, M=0 (narrow index)
    cpu.a = 0x1280;
    cpu.x = 0x0099;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0xAA]);

    cpu.step(&mut bus);

    assert_eq!(cpu.x, 0x0080, "TAX 8-bit: X = A & 0xFF");
    assert!(cpu.flag(FLAG_N), "TAX 8-bit: N flag set (bit 7 of result)");
}

#[test]
fn e02_37_txa_m8() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x20; // M=1, X=0 (narrow accumulator)
    cpu.x = 0xABCD;
    cpu.a = 0x1200;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x8A]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0x12CD, "TXA M=1: A high byte preserved, low from X");
}

#[test]
fn e02_37_txa_m16() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // M=0 (wide accumulator)
    cpu.x = 0x1234;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x8A]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0x1234, "TXA M=0: A = X (full width)");
}

#[test]
fn e02_37_tay_tya() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // wide mode
    cpu.a = 0x8000;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0xA8]);

    cpu.step(&mut bus);

    assert_eq!(cpu.y, 0x8000, "TAY: Y = A");
    assert!(cpu.flag(FLAG_N), "TAY: N flag set");

    // Now TYA back
    cpu.a = 0xFFFF;
    cpu.pc = 0x8001;
    bus.poke(0x8001, &[0x98]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0x8000, "TYA: A = Y");
}

#[test]
fn e02_37_tsx_8() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x10; // X=1 (narrow index)
    cpu.s = 0x01FF;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0xBA]);

    cpu.step(&mut bus);

    assert_eq!(cpu.x, 0x00FF, "TSX 8-bit: X = S & 0xFF");
    assert!(cpu.flag(FLAG_N), "TSX 8-bit: N flag set (bit 7 of FF)");
}

#[test]
fn e02_37_txs_native() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00;
    cpu.x = 0x1234;
    cpu.s = 0xFFFF;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x9A]);

    let p_before = cpu.p;
    cpu.step(&mut bus);

    assert_eq!(cpu.s, 0x1234, "TXS native: S = X (full width)");
    assert_eq!(cpu.p, p_before, "TXS: flags unchanged");
}

#[test]
fn e02_37_txs_emulation() {
    let mut cpu = make_cpu();
    cpu.e = true;
    let mut bus = FlatBus::new();
    cpu.p = 0x30; // M=1, X=1 (forced in emulation)
    cpu.x = 0x56;
    cpu.s = 0xFFFF;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x9A]);

    cpu.step(&mut bus);

    assert_eq!(cpu.s, 0x0156, "TXS emulation: S = 0x0100 | (X & 0xFF)");
}

#[test]
fn e02_37_txy_tyx() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // wide mode
    cpu.x = 0x1234;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x9B]);

    cpu.step(&mut bus);

    assert_eq!(cpu.y, 0x1234, "TXY: Y = X (full width)");

    // Now TYX in narrow mode
    cpu.p = 0x10; // X=1 (narrow index)
    cpu.y = 0x0080;
    cpu.x = 0xFFFF;
    cpu.pc = 0x8001;
    bus.poke(0x8001, &[0xBB]);

    cpu.step(&mut bus);

    assert_eq!(cpu.x, 0x0080, "TYX 8-bit: X = Y & 0xFF");
    assert!(cpu.flag(FLAG_N), "TYX 8-bit: N flag set (bit 7 of 80)");
}

#[test]
fn e02_37_tcd_tdc() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.a = 0x8000;
    cpu.d = 0xFFFF;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x5B]);

    cpu.step(&mut bus);

    assert_eq!(cpu.d, 0x8000, "TCD: D = A (always 16-bit)");
    assert!(cpu.flag(FLAG_N), "TCD: N flag set from D");

    // Now TDC back
    cpu.a = 0xFFFF;
    cpu.d = 0x0000;
    cpu.pc = 0x8001;
    bus.poke(0x8001, &[0x7B]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0x0000, "TDC: A = D (always 16-bit)");
    assert!(cpu.flag(FLAG_Z), "TDC: Z flag set from A=0");
}

#[test]
fn e02_37_tcs_native_emulation() {
    // Native mode
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.a = 0x2345;
    cpu.s = 0xFFFF;
    cpu.p = 0x01;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x1B]);

    let p_before = cpu.p;
    cpu.step(&mut bus);

    assert_eq!(cpu.s, 0x2345, "TCS native: S = A (full width)");
    assert_eq!(cpu.p, p_before, "TCS: flags unchanged in native mode");

    // Emulation mode
    let mut cpu2 = make_cpu();
    cpu2.e = true;
    let mut bus2 = FlatBus::new();
    cpu2.a = 0x4567;
    cpu2.s = 0xFFFF;
    cpu2.p = 0x30;
    cpu2.pc = 0x8000;
    bus2.poke(0x8000, &[0x1B]);

    let p_before2 = cpu2.p;
    cpu2.step(&mut bus2);

    assert_eq!(cpu2.s, 0x0167, "TCS emulation: S = 0x0100 | (A & 0xFF)");
    assert_eq!(cpu2.p, p_before2, "TCS: flags unchanged in emulation mode");
}

#[test]
fn e02_37_tsc() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.s = 0x01FF;
    cpu.a = 0xFFFF;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x3B]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0x01FF, "TSC: A = S (always 16-bit)");
    assert!(!cpu.flag(FLAG_N), "TSC: N flag clear (bit 15 of 0x01FF is 0)");
    assert!(!cpu.flag(FLAG_Z), "TSC: Z flag clear (result non-zero)");
}
