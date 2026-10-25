// E02-32: flow/branch.rs - Bxx, BRA, BRL branch instructions

use puce16_core::cpu::{Cpu, FLAG_C, FLAG_N, FLAG_V, FLAG_Z};
use puce16_core::cpu::testbus::FlatBus;

fn make_cpu() -> Cpu {
    let mut cpu = Cpu::new();
    cpu.e = false; // native mode by default
    cpu
}

#[test]
fn e02_32_bne_not_taken() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x22; // Z=1 (not taken)
    cpu.pc = 0x8000;
    // BNE +16: opcode D0, offset 0x10
    bus.poke(0x8000, &[0xD0, 0x10]);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x8002, "PC should advance past instruction when not taken");
    assert_eq!(bus.log.len(), 2, "not taken branch: 2 cycles (opcode + operand)");
}

#[test]
fn e02_32_bne_taken() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x20; // Z=0 (taken)
    cpu.pc = 0x8000;
    // BNE +16: opcode D0, offset 0x10 -> target 0x8012
    bus.poke(0x8000, &[0xD0, 0x10]);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x8012, "PC should branch to target");
    assert_eq!(bus.log.len(), 3, "taken branch: 3 cycles (opcode + operand + idle)");
}

#[test]
fn e02_32_bra_negative() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.pc = 0x8000;
    // BRA -2: opcode 80, offset 0xFE (-2 signed) -> target 0x8000 (loop back to itself)
    bus.poke(0x8000, &[0x80, 0xFE]);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x8000, "BRA with negative offset should loop");
    assert_eq!(bus.log.len(), 3, "taken branch: 3 cycles");
}

#[test]
fn e02_32_emulation_page_cross() {
    let mut cpu = make_cpu();
    cpu.e = true; // emulation mode
    let mut bus = FlatBus::new();
    cpu.p = 0x30; // Z=0 (taken)
    cpu.pc = 0x80F0;
    // BNE +32: opcode D0, offset 0x20 -> after operand pc=0x80F2, target=0x8112 (crosses page boundary)
    bus.poke(0x80F0, &[0xD0, 0x20]);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x8112, "PC should branch to target across page");
    assert_eq!(bus.log.len(), 4, "emulation mode page crossing: 4 cycles (extra idle penalty)");
}

#[test]
fn e02_32_native_page_cross_no_penalty() {
    let mut cpu = make_cpu();
    // e=false (native mode) - no page crossing penalty
    let mut bus = FlatBus::new();
    cpu.p = 0x30; // Z=0 (taken)
    cpu.pc = 0x80F0;
    // BNE +32: opcode D0, offset 0x20 -> target 0x8112 (crosses page boundary)
    bus.poke(0x80F0, &[0xD0, 0x20]);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x8112, "PC should branch to target across page");
    assert_eq!(bus.log.len(), 3, "native mode: no page crossing penalty (3 cycles)");
}

#[test]
fn e02_32_emulation_same_page() {
    let mut cpu = make_cpu();
    cpu.e = true; // emulation mode
    let mut bus = FlatBus::new();
    cpu.p = 0x30; // Z=0 (taken)
    cpu.pc = 0x8000;
    // BNE +16: opcode D0, offset 0x10 -> target 0x8012 (same page)
    bus.poke(0x8000, &[0xD0, 0x10]);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x8012, "PC should branch to target on same page");
    assert_eq!(bus.log.len(), 3, "same page: no penalty (3 cycles)");
}

#[test]
fn e02_32_all_conditions() {
    // Test all branch conditions with offset +4
    let tests = [
        // (opcode, mnemonic, flag_to_set, should_take_when_set)
        (0x90, "BCC", FLAG_C, false),   // BCC: taken when C=0
        (0xB0, "BCS", FLAG_C, true),    // BCS: taken when C=1
        (0xF0, "BEQ", FLAG_Z, true),    // BEQ: taken when Z=1
        (0xD0, "BNE", FLAG_Z, false),   // BNE: taken when Z=0
        (0x30, "BMI", FLAG_N, true),    // BMI: taken when N=1
        (0x10, "BPL", FLAG_N, false),   // BPL: taken when N=0
        (0x50, "BVC", FLAG_V, false),   // BVC: taken when V=0
        (0x70, "BVS", FLAG_V, true),    // BVS: taken when V=1
    ];

    for (op, name, flag, take_when_set) in tests {
        // Test taken case
        let mut cpu = make_cpu();
        let mut bus = FlatBus::new();
        if take_when_set {
            cpu.p = flag;
        } else {
            cpu.p = 0x00;
        }
        cpu.pc = 0x8000;
        bus.poke(0x8000, &[op, 0x04]);

        cpu.step(&mut bus);
        assert_eq!(cpu.pc, 0x8006, "{} taken: PC should branch", name);
        assert_eq!(bus.log.len(), 3, "{} taken: 3 cycles", name);

        // Test not-taken case
        let mut cpu2 = make_cpu();
        let mut bus2 = FlatBus::new();
        if take_when_set {
            cpu2.p = 0x00;
        } else {
            cpu2.p = flag;
        }
        cpu2.pc = 0x8000;
        bus2.poke(0x8000, &[op, 0x04]);

        cpu2.step(&mut bus2);
        assert_eq!(cpu2.pc, 0x8002, "{} not taken: PC should advance past instruction", name);
        assert_eq!(bus2.log.len(), 2, "{} not taken: 2 cycles", name);
    }
}

#[test]
fn e02_32_brl() {
    // BRL +256 (0x0100): opcode 82, lo=0x00, hi=0x01 -> target 0x8103
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x82, 0x00, 0x01]);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x8103, "BRL +256: PC should branch forward");
    assert_eq!(bus.log.len(), 4, "BRL: 4 cycles (opcode + lo + hi + idle)");

    // BRL -3 (0xFFFD): opcode 82, lo=0xFD, hi=0xFF -> target 0x8000 (loop back)
    let mut cpu2 = make_cpu();
    let mut bus2 = FlatBus::new();
    cpu2.pc = 0x8000;
    bus2.poke(0x8000, &[0x82, 0xFD, 0xFF]);

    cpu2.step(&mut bus2);

    assert_eq!(cpu2.pc, 0x8000, "BRL -3: PC should loop back");
}

#[test]
fn e02_32_wrap_in_bank() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.pb = 0x05;
    cpu.pc = 0xFFF0;
    // BRA +32: opcode 80, offset 0x20 -> after operand pc=0xFFF2, target wraps to 0x0012
    bus.poke(0x05FFF0, &[0x80, 0x20]);

    cpu.step(&mut bus);

    assert_eq!(cpu.pc, 0x0012, "PC should wrap within bank");
    assert_eq!(cpu.pb, 0x05, "Bank pointer should not change");
}
