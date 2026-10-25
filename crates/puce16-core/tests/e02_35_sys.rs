// E02-35: flow/sys.rs - BRK, COP, RTI, WAI, STP, WDM instructions

use puce16_core::cpu::{Cpu};
use puce16_core::cpu::testbus::FlatBus;

fn make_cpu() -> Cpu {
    let mut cpu = Cpu::new();
    cpu.e = false; // native mode by default
    cpu
}

#[test]
fn e02_35_brk_native() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x01;
    cpu.pb = 0x03;
    cpu.pc = 0x8000;
    cpu.s = 0x01FF;
    // BRK vector at $FFE6 -> $9000
    bus.poke(0xFFE6, &[0x00, 0x90]);
    // BRK instruction: opcode 00 + signature byte 00
    bus.poke(0x8000, &[0x00, 0x00]);

    cpu.step(&mut bus);

    assert_eq!(bus.peek(0x01FF), 0x03, "BRK native: PB pushed");
    assert_eq!(bus.peek(0x01FE), 0x80, "BRK native: PC high pushed (after signature)");
    assert_eq!(bus.peek(0x01FD), 0x02, "BRK native: PC low pushed (after signature)");
    assert_eq!(bus.peek(0x01FC), 0x01, "BRK native: P pushed as-is");
    assert_eq!(cpu.s, 0x01FB, "BRK native: stack pointer decremented by 4");
    assert_eq!(cpu.pc, 0x9000, "BRK native: PC set to vector target");
    assert_eq!(cpu.pb, 0, "BRK native: PB cleared");
    assert!(cpu.flag(puce16_core::cpu::FLAG_I), "BRK native: I flag set");
    assert!(!cpu.flag(puce16_core::cpu::FLAG_D), "BRK native: D flag cleared");
    assert_eq!(bus.log.len(), 8, "BRK native: 8 cycles (opcode + sig + push_PB + push_PCH + push_PCL + push_P + read_vec_lo + read_vec_hi)");
}

#[test]
fn e02_35_brk_emulation() {
    let mut cpu = make_cpu();
    cpu.e = true;
    let mut bus = FlatBus::new();
    cpu.p = 0x30;
    cpu.s = 0x01FF;
    cpu.pc = 0x8000;
    // BRK vector at $FFFE -> $8800 (emulation mode)
    bus.poke(0xFFFE, &[0x00, 0x88]);
    // BRK instruction: opcode 00 + signature byte 00
    bus.poke(0x8000, &[0x00, 0x00]);

    cpu.step(&mut bus);

    assert_eq!(bus.peek(0x01FF), 0x80, "BRK emu: PC high pushed");
    assert_eq!(bus.peek(0x01FE), 0x02, "BRK emu: PC low pushed");
    assert_eq!(bus.peek(0x01FD), 0x30, "BRK emu: P pushed with M and X forced");
    assert_eq!(cpu.s & 0xFF, 0xFC, "BRK emu: stack pointer decremented by 3 (wrapped)");
    assert_eq!(cpu.pc, 0x8800, "BRK emu: PC set to vector target");
    assert_eq!(cpu.pb, 0, "BRK emu: PB cleared");
    assert_eq!(bus.log.len(), 7, "BRK emu: 7 cycles (opcode + sig + push_PCH + push_PCL + push_P + read_vec_lo + read_vec_hi)");
}

#[test]
fn e02_35_cop_native_vector() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.s = 0x01FF;
    cpu.pc = 0x8000;
    // COP vector at $FFE4 -> $A000
    bus.poke(0xFFE4, &[0x00, 0xA0]);
    // COP instruction: opcode 02 + signature byte 00
    bus.poke(0x8000, &[0x02, 0x00]);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0xA000, "COP native: PC set to COP vector target");
    assert_eq!(bus.log.len(), 8, "COP native: 8 cycles (same as BRK)");
}

#[test]
fn e02_35_brk_clears_decimal() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x08; // D=1, all other flags clear
    cpu.s = 0x01FF;
    cpu.pc = 0x8000;
    bus.poke(0xFFE6, &[0x00, 0x90]);
    bus.poke(0x8000, &[0x00, 0x00]);

    cpu.step(&mut bus);

    assert!(!cpu.flag(puce16_core::cpu::FLAG_D), "BRK: D flag cleared in P register");
    assert!(cpu.flag(puce16_core::cpu::FLAG_I), "BRK: I flag set");
    // The pushed P byte should still have D=1 (0x08)
    let p_pushed = bus.peek(0x01FC);
    assert_eq!(p_pushed & 0x08, 0x08, "BRK: pushed P byte retains D=1");
}

#[test]
fn e02_35_rti_native() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.s = 0x01FB;
    cpu.pc = 0x8000;
    // Stack: P=0x24, PC=$9000, PB=0x05 (pushed in that order)
    bus.poke(0x01FC, &[0x24, 0x00, 0x90, 0x05]);
    // RTI: opcode 40
    bus.poke(0x8000, &[0x40]);

    cpu.step(&mut bus);

    assert_eq!(cpu.p, 0x24, "RTI native: P restored from stack");
    assert_eq!(cpu.pc, 0x9000, "RTI native: PC restored (not incremented)");
    assert_eq!(cpu.pb, 0x05, "RTI native: PB restored from stack");
    assert_eq!(cpu.s, 0x01FF, "RTI native: stack pointer incremented by 4");
    assert_eq!(bus.log.len(), 7, "RTI native: 7 cycles (opcode + idle + idle + pull_P + pull_PCL + pull_PCH + pull_PB)");
}

#[test]
fn e02_35_rti_emulation() {
    let mut cpu = make_cpu();
    cpu.e = true;
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // will be overwritten by set_p which forces M and X in emu mode
    cpu.s = 0x01FC;
    cpu.pc = 0x8000;
    // Stack: P=0x00, PC=$9000 (no PB in emulation)
    bus.poke(0x01FD, &[0x00, 0x00, 0x90]);
    // RTI: opcode 40
    bus.poke(0x8000, &[0x40]);

    cpu.step(&mut bus);

    assert_eq!(cpu.p, 0x30, "RTI emu: P restored with M and X forced by set_p");
    assert_eq!(cpu.pc, 0x9000, "RTI emu: PC restored (not incremented)");
    assert_eq!(cpu.s & 0xFF, 0xFF, "RTI emu: stack pointer incremented by 3 (wrapped)");
    assert_eq!(bus.log.len(), 6, "RTI emu: 6 cycles (opcode + idle + idle + pull_P + pull_PCL + pull_PCH)");
}

#[test]
fn e02_35_wai() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.pc = 0x8000;
    // WAI: opcode CB
    bus.poke(0x8000, &[0xCB]);

    cpu.step(&mut bus);

    assert!(cpu.waiting, "WAI: waiting flag set");
    assert_eq!(cpu.pc, 0x8001, "WAI: PC incremented past opcode");
    assert_eq!(bus.log.len(), 3, "WAI: 3 cycles (opcode + idle + idle)");
}

#[test]
fn e02_35_stp() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.pc = 0x8000;
    // STP: opcode DB
    bus.poke(0x8000, &[0xDB]);

    cpu.step(&mut bus);

    assert!(cpu.halted, "STP: halted flag set");
    assert_eq!(cpu.pc, 0x8001, "STP: PC incremented past opcode");
    assert_eq!(bus.log.len(), 3, "STP: 3 cycles (opcode + idle + idle)");
}

#[test]
fn e02_35_wdm() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.pc = 0x8000;
    // WDM: opcode 42 + signature byte 99 (ignored)
    bus.poke(0x8000, &[0x42, 0x99]);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x8002, "WDM: PC incremented past opcode and signature");
    assert_eq!(bus.log.len(), 2, "WDM: 2 cycles (opcode + fetch signature)");
}
