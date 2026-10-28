use puce16_core::apu::spc700::{Spc700, testbus::FlatSmpBus, F_C};

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
fn e06_25_or1_inverted_preserves_carry() {
    // OR1 C,/m.b with m=1: C = C | !1 = C | 0 = C (unchanged)
    let (cpu, _, _) = run(|c, b| {
        c.psw = F_C; // carry already set
        b.poke(0x0005, &[0x02]); // bit 1 set -> /bit=0
    }, &[0x2A, 0x05, 0x20]);
    assert!(cpu.flag(F_C), "OR1 C,/m.b with m=1 should preserve existing carry");
}

#[test]
fn e06_25_or1_inverted_sets_carry_when_clear() {
    // OR1 C,/m.b with m=0: C = C | !0 = C | 1 = 1
    let (cpu, _, _) = run(|c, b| {
        c.psw = 0; // carry clear
        b.poke(0x0005, &[0x00]); // bit 1 clear -> /bit=1
    }, &[0x2A, 0x05, 0x20]);
    assert!(cpu.flag(F_C), "OR1 C,/m.b with m=0 should set carry");
}

#[test]
fn e06_25_oracle_clean_or_regressions() {
    // The two OR1 regressions above are the bugs found by the oracle; this test
    // also checks the plain (non-inverted) form: OR1 C,m.b with the bit set.
    let (cpu, _, _) = run(|c, b| {
        c.psw = 0;
        b.poke(0x0005, &[0x02]); // bit 1 set
    }, &[0x0A, 0x05, 0x20]);
    assert!(cpu.flag(F_C), "OR1 C,m.b with m=1 should set carry");
}
