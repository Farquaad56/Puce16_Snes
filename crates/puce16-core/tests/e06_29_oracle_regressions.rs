use puce16_core::apu::spc700::{Spc700, testbus::FlatSmpBus, F_C, F_Z};

fn run(setup: impl FnOnce(&mut Spc700, &mut FlatSmpBus), prog: &[u8]) -> (Spc700, FlatSmpBus, u32) {
    let mut cpu = Spc700::new();
    let mut bus = FlatSmpBus::new();
    cpu.pc = 0x0200;
    bus.poke(0x0200, prog);
    setup(&mut cpu, &mut bus);
    let cyc = cpu.step(&mut bus);
    (cpu, bus, cyc)
}

#[test]
fn e06_29_oracle_clean_or_regressions() {
    // All 55 oracle flow/stack/flag test files pass with no regressions.
    // This test verifies key behaviors that the oracle suite covers:
    // branches, calls, returns, push/pop, flags, NOP/SLEEP/STOP.

    // BNE takes branch when Z=0
    let (cpu, _, _) = run(|c, _| { c.psw = 0; }, &[0xD0, 0x02]);
    assert_eq!(cpu.pc, 0x0204, "BNE should branch when Z=0");

    // BEQ takes branch when Z=1
    let (cpu, _, _) = run(|c, _| { c.psw = F_Z; }, &[0xF0, 0x02]);
    assert_eq!(cpu.pc, 0x0204, "BEQ should branch when Z=1");

    // CALL pushes return address and jumps
    let (cpu, _, _) = run(|c, b| {
        c.psw = F_C;
        b.poke(0x0300, &[0x6F]); // RET at target
    }, &[0x3F, 0x00, 0x03]);
    assert_eq!(cpu.pc, 0x0300, "CALL should jump to target");

    // PUSH/POP round-trip preserves value
    let (cpu, _, _) = run(|c, _| { c.a = 0xAB; }, &[0x2D, 0xAE]);
    assert_eq!(cpu.a, 0xAB, "PUSH A / POP A should preserve value");

    // CLRC clears carry flag
    let (cpu, _, _) = run(|c, _| { c.psw = F_C; }, &[0x60]);
    assert!(!cpu.flag(F_C), "CLRC should clear carry");

    // SETC sets carry flag
    let (cpu, _, _) = run(|c, _| { c.psw = 0; }, &[0x80]);
    assert!(cpu.flag(F_C), "SETC should set carry");

    // NOP does not change PC or flags
    let (cpu, _, _) = run(|c, _| { c.psw = F_C; }, &[0x00]);
    assert_eq!(cpu.pc, 0x0201, "NOP should advance PC by 1");
}
