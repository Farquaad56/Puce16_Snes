// E02-40: interrupts.rs - poll (NMI/IRQ/WAI/STP), interrupt sequence, RESET

use puce16_core::cpu::{Cpu, FLAG_I};
use puce16_core::cpu::testbus::{CycleKind, FlatBus};

fn make_cpu() -> Cpu {
    let mut cpu = Cpu::new();
    cpu.e = false; // native mode by default
    cpu
}

#[test]
fn e02_40_nmi_native() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00;
    cpu.pb = 0x02;
    cpu.pc = 0x8123;
    cpu.s = 0x01FF;
    // NMI vector at $FFEA -> $9000
    bus.poke(0xFFEA, &[0x00, 0x90]);
    bus.nmi = true;

    let took = puce16_core::cpu::interrupts::poll(&mut cpu, &mut bus);
    assert!(took, "NMI should be taken");
    assert_eq!(cpu.pc, 0x9000, "PC set to NMI vector target");
    assert_eq!(cpu.pb, 0, "PB cleared after interrupt");
    assert!(cpu.flag(FLAG_I), "I flag set after interrupt");
    assert_eq!(cpu.s, 0x01FB, "Stack pointer decremented by 4 (native)");
    // Verify pushed values: PB, PCH, PCL, P
    assert_eq!(bus.peek(0x01FF), 0x02, "PB pushed");
    assert_eq!(bus.peek(0x01FE), 0x81, "PC high pushed");
    assert_eq!(bus.peek(0x01FD), 0x23, "PC low pushed");
    assert_eq!(bus.peek(0x01FC), 0x00, "P pushed as-is (native)");
    // Verify cycle count: 8 cycles for native interrupt
    assert_eq!(bus.log.len(), 8, "Native NMI should take 8 cycles");
    // Verify log sequence: R(pc24), Idle, W(PB), W(PCH), W(PCL), W(P), R(vec_lo), R(vec_hi)
    let kinds: Vec<CycleKind> = bus.log.iter().map(|c| c.kind).collect();
    assert_eq!(kinds[0], CycleKind::Read, "First cycle: phantom opcode read");
    assert_eq!(kinds[1], CycleKind::Idle, "Second cycle: IO idle");
    for i in 2..6 {
        assert_eq!(kinds[i], CycleKind::Write, "Cycles 3-6: stack writes");
    }
    assert_eq!(kinds[6], CycleKind::Read, "Cycle 7: vector low read");
    assert_eq!(kinds[7], CycleKind::Read, "Cycle 8: vector high read");
    // Verify NMI was consumed
    assert!(!bus.nmi, "NMI flag should be cleared after being taken");
}

#[test]
fn e02_40_irq_emulation() {
    let mut cpu = make_cpu();
    cpu.e = true;
    let mut bus = FlatBus::new();
    cpu.p = 0x30; // I=0, M=1, X=1
    cpu.pc = 0x8123;
    cpu.s = 0x01FF;
    // IRQ vector at $FFFE -> $8800 (emulation)
    bus.poke(0xFFFE, &[0x00, 0x88]);
    bus.irq = true;

    let took = puce16_core::cpu::interrupts::poll(&mut cpu, &mut bus);
    assert!(took, "IRQ should be taken");
    assert_eq!(cpu.pc, 0x8800, "PC set to IRQ vector target (emulation)");
    // Verify pushed values: PCH, PCL, P (with B=0 forced in emulation)
    assert_eq!(bus.peek(0x01FF), 0x81, "PC high pushed (emulation)");
    assert_eq!(bus.peek(0x01FE), 0x23, "PC low pushed (emulation)");
    let p_pushed = bus.peek(0x01FD);
    assert_eq!(p_pushed & 0x20, 0x20, "B bit forced to 0 in emulation push");
    assert_eq!(cpu.s & 0xFF, 0xFC, "Stack pointer decremented by 3 (emulation)");
    // Verify cycle count: 7 cycles for emulation interrupt
    assert_eq!(bus.log.len(), 7, "Emulation IRQ should take 7 cycles");
    assert!(cpu.flag(FLAG_I), "I flag set after interrupt");
}

#[test]
fn e02_40_irq_masked() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x04; // I=1 (masked)
    cpu.pc = 0x8000;
    bus.irq = true;

    let took = puce16_core::cpu::interrupts::poll(&mut cpu, &mut bus);
    assert!(!took, "IRQ should be masked when I=1");
    assert_eq!(bus.log.len(), 0, "No cycles consumed when IRQ is masked");
    assert_eq!(cpu.pc, 0x8000, "PC unchanged when IRQ is masked");
}

#[test]
fn e02_40_nmi_priority_over_irq() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // I=0
    cpu.pc = 0x8123;
    cpu.s = 0x01FF;
    // Both NMI and IRQ vectors set up
    bus.poke(0xFFEA, &[0x00, 0x90]); // NMI -> $9000
    bus.poke(0xFFEE, &[0x00, 0x88]); // IRQ -> $8800
    bus.nmi = true;
    bus.irq = true;

    let took = puce16_core::cpu::interrupts::poll(&mut cpu, &mut bus);
    assert!(took, "Interrupt should be taken");
    assert_eq!(cpu.pc, 0x9000, "NMI has priority over IRQ - PC set to NMI vector");
}

#[test]
fn e02_40_wai_waits() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.waiting = true;
    // No interrupts pending

    let took = puce16_core::cpu::interrupts::poll(&mut cpu, &mut bus);
    assert!(took, "WAI should consume a step while waiting");
    assert_eq!(bus.log.len(), 1, "WAI idle cycle consumed");
    assert_eq!(bus.log[0].kind, CycleKind::Idle, "WAI produces an idle cycle");
    assert!(cpu.waiting, "CPU still waiting after no interrupt");
}

#[test]
fn e02_40_wai_wakes_on_masked_irq() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x04; // I=1 (masked)
    cpu.waiting = true;
    bus.irq = true;

    let took = puce16_core::cpu::interrupts::poll(&mut cpu, &mut bus);
    assert!(!took, "WAI wakes but does not take masked IRQ");
    assert!(!cpu.waiting, "CPU no longer waiting after waking");
    assert_eq!(bus.log.len(), 0, "No cycles consumed when WAI wakes on masked IRQ");
}

#[test]
fn e02_40_wai_nmi() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.waiting = true;
    cpu.p = 0x00;
    cpu.pc = 0x8123;
    cpu.s = 0x01FF;
    bus.poke(0xFFEA, &[0x00, 0x90]); // NMI -> $9000
    bus.nmi = true;

    let took = puce16_core::cpu::interrupts::poll(&mut cpu, &mut bus);
    assert!(took, "WAI takes NMI");
    assert!(!cpu.waiting, "CPU no longer waiting after NMI");
    assert_eq!(cpu.pc, 0x9000, "PC set to NMI vector target");
}

#[test]
fn e02_40_stp() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.halted = true;
    // Even with interrupts pending, STP takes priority
    bus.nmi = true;
    bus.irq = true;

    let took = puce16_core::cpu::interrupts::poll(&mut cpu, &mut bus);
    assert!(took, "STP consumes a step");
    assert_eq!(bus.log.len(), 1, "STP produces one idle cycle");
    assert_eq!(bus.log[0].kind, CycleKind::Idle, "STP idle cycle");
    assert!(cpu.halted, "CPU remains halted");
}

#[test]
fn e02_40_reset() {
    let mut cpu = make_cpu();
    // Set up non-default values to verify they're reset
    cpu.e = false;
    cpu.p = 0x00;
    cpu.x = 0x1234;
    cpu.y = 0x5678;
    cpu.s = 0x0000;
    cpu.d = 0x1111;
    cpu.db = 5;
    cpu.pb = 6;

    let mut bus = FlatBus::new();
    // Reset vector at $FFFC -> $8000
    bus.poke(0xFFFC, &[0x00, 0x80]);

    puce16_core::cpu::interrupts::reset(&mut cpu, &mut bus);

    assert!(cpu.e, "Reset sets emulation mode");
    assert_eq!(cpu.p, 0x34, "Reset sets P=0x34 (M, X, I)");
    assert_eq!(cpu.pc, 0x8000, "PC set to reset vector target");
    assert_eq!(cpu.pb, 0, "PB cleared on reset");
    assert_eq!(cpu.db, 0, "DB cleared on reset");
    assert_eq!(cpu.d, 0, "D cleared on reset");
    assert_eq!(cpu.s, 0x01FF, "S set to 0x01FF on reset");
    assert_eq!(cpu.x & 0xFF, 0x34, "X truncated to low byte (0x34)");
    assert_eq!(cpu.y & 0xFF, 0x78, "Y truncated to low byte (0x78)");
    // Verify cycle count: 5 idle + 2 reads = 7 cycles
    assert_eq!(bus.log.len(), 7, "Reset sequence takes 7 cycles");
}

#[test]
fn e02_40_step_takes_interrupt() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00;
    cpu.pc = 0x8000;
    cpu.s = 0x01FF;
    // NMI vector at $FFEA -> $9000
    bus.poke(0xFFEA, &[0x00, 0x90]);
    // NOP instruction at PC (should not execute due to interrupt)
    bus.poke(0x8000, &[0xEA]);
    bus.nmi = true;

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x9000, "step() takes NMI before executing instruction");
}
