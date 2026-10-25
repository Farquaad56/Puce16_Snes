// E02-38: misc/flags.rs - CLC SEC CLI SEI CLD SED CLV REP SEP XCE XBA NOP

use puce16_core::cpu::{Cpu, FLAG_C, FLAG_D, FLAG_I, FLAG_M, FLAG_V, FLAG_X};
use puce16_core::cpu::testbus::FlatBus;

fn make_cpu() -> Cpu {
    let mut cpu = Cpu::new();
    cpu.e = false; // native mode by default
    cpu
}

#[test]
fn e02_38_flag_ops() {
    // CLC (18): Clear Carry Flag - 2 cycles
    {
        let mut cpu = make_cpu();
        let mut bus = FlatBus::new();
        cpu.p = 0xFF;
        cpu.pc = 0x8000;
        bus.poke(0x8000, &[0x18]);
        cpu.step(&mut bus);
        assert!(!cpu.flag(FLAG_C), "CLC: C flag cleared");
        assert_eq!(bus.log.len(), 2, "CLC: 2 cycles");
    }

    // SEC (38): Set Carry Flag - 2 cycles
    {
        let mut cpu = make_cpu();
        let mut bus = FlatBus::new();
        cpu.p = 0x00;
        cpu.pc = 0x8000;
        bus.poke(0x8000, &[0x38]);
        cpu.step(&mut bus);
        assert!(cpu.flag(FLAG_C), "SEC: C flag set");
        assert_eq!(bus.log.len(), 2, "SEC: 2 cycles");
    }

    // CLI (58): Clear Interrupt Disable - 2 cycles
    {
        let mut cpu = make_cpu();
        let mut bus = FlatBus::new();
        cpu.p = 0xFF;
        cpu.pc = 0x8000;
        bus.poke(0x8000, &[0x58]);
        cpu.step(&mut bus);
        assert!(!cpu.flag(FLAG_I), "CLI: I flag cleared");
        assert_eq!(bus.log.len(), 2, "CLI: 2 cycles");
    }

    // SEI (78): Set Interrupt Disable - 2 cycles
    {
        let mut cpu = make_cpu();
        let mut bus = FlatBus::new();
        cpu.p = 0x00;
        cpu.pc = 0x8000;
        bus.poke(0x8000, &[0x78]);
        cpu.step(&mut bus);
        assert!(cpu.flag(FLAG_I), "SEI: I flag set");
        assert_eq!(bus.log.len(), 2, "SEI: 2 cycles");
    }

    // CLD (D8): Clear Decimal Mode - 2 cycles
    {
        let mut cpu = make_cpu();
        let mut bus = FlatBus::new();
        cpu.p = 0xFF;
        cpu.pc = 0x8000;
        bus.poke(0x8000, &[0xD8]);
        cpu.step(&mut bus);
        assert!(!cpu.flag(FLAG_D), "CLD: D flag cleared");
        assert_eq!(bus.log.len(), 2, "CLD: 2 cycles");
    }

    // SED (F8): Set Decimal Mode - 2 cycles
    {
        let mut cpu = make_cpu();
        let mut bus = FlatBus::new();
        cpu.p = 0x00;
        cpu.pc = 0x8000;
        bus.poke(0x8000, &[0xF8]);
        cpu.step(&mut bus);
        assert!(cpu.flag(FLAG_D), "SED: D flag set");
        assert_eq!(bus.log.len(), 2, "SED: 2 cycles");
    }

    // CLV (B8): Clear Overflow Flag - 2 cycles
    {
        let mut cpu = make_cpu();
        let mut bus = FlatBus::new();
        cpu.p = 0xFF;
        cpu.pc = 0x8000;
        bus.poke(0x8000, &[0xB8]);
        cpu.step(&mut bus);
        assert!(!cpu.flag(FLAG_V), "CLV: V flag cleared");
        assert_eq!(bus.log.len(), 2, "CLV: 2 cycles");
    }
}

#[test]
fn e02_38_rep_native() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0xFF; // all flags set
    cpu.pc = 0x8000;
    // REP #0x30: clear M and X flags (bits 5 and 4)
    bus.poke(0x8000, &[0xC2, 0x30]);

    cpu.step(&mut bus);

    assert!(!cpu.flag(FLAG_M), "REP: M flag cleared");
    assert!(!cpu.flag(FLAG_X), "REP: X flag cleared");
    assert_eq!(bus.log.len(), 3, "REP: 3 cycles (opcode + operand + idle)");
}

#[test]
fn e02_38_rep_keeps_emulation() {
    let mut cpu = make_cpu();
    cpu.e = true;
    let mut bus = FlatBus::new();
    cpu.p = 0x30; // M=1, X=1 (emulation mode)
    cpu.pc = 0x8000;
    // REP #0x30: try to clear M and X flags
    bus.poke(0x8000, &[0xC2, 0x30]);

    cpu.step(&mut bus);

    assert_eq!(cpu.p & 0x30, 0x30, "REP in emulation: M and X remain set");
}

#[test]
fn e02_38_sep_clears_index_high() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // all flags clear (wide mode)
    cpu.x = 0x1234;
    cpu.y = 0x5678;
    cpu.pc = 0x8000;
    // SEP #0x10: set X flag (narrow index mode)
    bus.poke(0x8000, &[0xE2, 0x10]);

    cpu.step(&mut bus);

    assert_eq!(cpu.p & 0x10, 0x10, "SEP: X flag set");
    assert_eq!(cpu.x, 0x0034, "SEP X=1: high byte of X cleared");
    assert_eq!(cpu.y, 0x0078, "SEP X=1: high byte of Y cleared");
    assert_eq!(bus.log.len(), 3, "SEP: 3 cycles (opcode + operand + idle)");
}

#[test]
fn e02_38_xce_to_native() {
    let mut cpu = make_cpu();
    cpu.e = true;
    let mut bus = FlatBus::new();
    cpu.p = 0x30; // C=0, M=1, X=1 (emulation mode)
    cpu.pc = 0x8000;
    // XCE: exchange C and E flags
    bus.poke(0x8000, &[0xFB]);

    cpu.step(&mut bus);

    assert!(!cpu.e, "XCE to native: e flag cleared");
    assert!(cpu.flag(FLAG_C), "XCE to native: C flag set (was e)");
    // In emulation mode, M and X are forced to 1; after switching to native they should be preserved
    assert_eq!(bus.log.len(), 2, "XCE: 2 cycles");
}

#[test]
fn e02_38_xce_to_emulation() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x01; // C=1, M=0, X=0 (native mode)
    cpu.x = 0x1234;
    cpu.y = 0x5678;
    cpu.s = 0x1234;
    cpu.pc = 0x8000;
    // XCE: exchange C and E flags (C=1 -> e=true)
    bus.poke(0x8000, &[0xFB]);

    cpu.step(&mut bus);

    assert!(cpu.e, "XCE to emulation: e flag set");
    assert!(!cpu.flag(FLAG_C), "XCE to emulation: C flag cleared (was e)");
    // In emulation mode, M and X are forced to 1 via set_p()
    assert_eq!(cpu.p & 0x30, 0x30, "XCE to emulation: M and X forced to 1");
    assert_eq!(cpu.x, 0x0034, "XCE to emulation: high byte of X cleared");
    assert_eq!(cpu.y, 0x0078, "XCE to emulation: high byte of Y cleared");
    assert_eq!(cpu.s, 0x0134, "XCE to emulation: S forced to page 1");
}

#[test]
fn e02_38_xba() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.a = 0x1280;
    cpu.pc = 0x8000;
    // XBA: exchange accumulator bytes
    bus.poke(0x8000, &[0xEB]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0x8012, "XBA: bytes exchanged");
    assert!(!cpu.flag(puce16_core::cpu::FLAG_N), "XBA: N flag clear (low byte 0x12)");
    assert!(!cpu.flag(puce16_core::cpu::FLAG_Z), "XBA: Z flag clear (result non-zero)");
    assert_eq!(bus.log.len(), 3, "XBA: 3 cycles (opcode + idle + idle)");
}

#[test]
fn e02_38_xba_zero() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.a = 0x0012;
    cpu.pc = 0x8000;
    // XBA: exchange accumulator bytes -> 0x1200
    bus.poke(0x8000, &[0xEB]);

    cpu.step(&mut bus);

    assert_eq!(cpu.a, 0x1200, "XBA: bytes exchanged");
    assert!(cpu.flag(puce16_core::cpu::FLAG_Z), "XBA zero: Z flag set (low byte is 0)");
}

#[test]
fn e02_38_nop() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.a = 0x1234;
    cpu.x = 0x5678;
    cpu.y = 0x9ABC;
    cpu.p = 0xFF;
    cpu.pc = 0x8000;
    // NOP: no operation
    bus.poke(0x8000, &[0xEA]);

    let a_before = cpu.a;
    let x_before = cpu.x;
    let y_before = cpu.y;
    let p_before = cpu.p;

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x8001, "NOP: PC incremented");
    assert_eq!(cpu.a, a_before, "NOP: A unchanged");
    assert_eq!(cpu.x, x_before, "NOP: X unchanged");
    assert_eq!(cpu.y, y_before, "NOP: Y unchanged");
    assert_eq!(cpu.p, p_before, "NOP: P unchanged");
    assert_eq!(bus.log.len(), 2, "NOP: 2 cycles (opcode + idle)");
}
