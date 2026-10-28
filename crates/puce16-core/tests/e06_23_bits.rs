use puce16_core::apu::spc700::{Spc700, testbus::FlatSmpBus, F_C, F_Z, F_V, F_N};

fn run(setup: impl FnOnce(&mut Spc700, &mut FlatSmpBus), prog: &[u8]) -> (Spc700, FlatSmpBus, u32) {
    let mut cpu = Spc700::new();
    let mut bus = FlatSmpBus::new();
    cpu.pc = 0x0200;
    for (i, &b) in prog.iter().enumerate() {
        bus.poke(0x0200 + i as u16, &[b]);
    }
    setup(&mut cpu, &mut bus);
    let cyc = cpu.step(&mut bus);
    (cpu, bus, cyc)
}

#[test]
fn e06_23_set1() {
    // SET1 dp.0: set bit 0 in direct page byte
    let (_cpu, bus, cyc) = run(|cpu, bus| {
        cpu.psw = 0x02;
        bus.poke(0x10, &[0x00]);
    }, &[0x02, 0x10]);
    assert_eq!(bus.peek(0x10), 0x01);
    assert_eq!(cyc, 4);

    // SET1 dp.7: set bit 7 in direct page byte
    let (_cpu, bus, cyc) = run(|cpu, bus| {
        cpu.psw = 0x02;
        bus.poke(0x10, &[0x00]);
    }, &[0xE2, 0x10]);
    assert_eq!(bus.peek(0x10), 0x80);
    assert_eq!(cyc, 4);

    // SET1 dp.2: set bit 2 in direct page byte
    let (_cpu, bus, cyc) = run(|cpu, bus| {
        cpu.psw = 0x02;
        bus.poke(0x10, &[0x00]);
    }, &[0x42, 0x10]);
    assert_eq!(bus.peek(0x10), 0x04);
    assert_eq!(cyc, 4);
}

#[test]
fn e06_23_clr1() {
    // CLR1 dp.3: clear bit 3 in direct page byte
    let (_cpu, bus, cyc) = run(|cpu, bus| {
        cpu.psw = 0x02;
        bus.poke(0x10, &[0xFF]);
    }, &[0x72, 0x10]);
    assert_eq!(bus.peek(0x10), 0xF7);
    assert_eq!(cyc, 4);

    // CLR1 dp.0: clear bit 0 in direct page byte
    let (_cpu, bus, cyc) = run(|cpu, bus| {
        cpu.psw = 0x02;
        bus.poke(0x10, &[0xFF]);
    }, &[0x12, 0x10]);
    assert_eq!(bus.peek(0x10), 0xFE);
    assert_eq!(cyc, 4);
}

#[test]
fn e06_23_set_clr_no_flags() {
    // SET1 does not affect any flags
    let (cpu, _bus, _cyc) = run(|cpu, bus| {
        cpu.psw = 0xC3;
        bus.poke(0x10, &[0x00]);
    }, &[0x02, 0x10]);
    assert_eq!(cpu.psw, 0xC3);
}

#[test]
fn e06_23_bbs_taken() {
    // BBS dp.2,+5: branch taken when bit is set
    let (cpu, _bus, cyc) = run(|cpu, bus| {
        cpu.psw = 0x02;
        bus.poke(0x10, &[0x04]);
    }, &[0x43, 0x10, 0x05]);
    assert_eq!(cpu.pc, 0x0208); // 0x0203 + 5
    assert_eq!(cyc, 7); // 5 base + 2 branch taken
}

#[test]
fn e06_23_bbs_not_taken() {
    // BBS dp.2,+5: branch not taken when bit is clear
    let (cpu, _bus, cyc) = run(|cpu, bus| {
        cpu.psw = 0x02;
        bus.poke(0x10, &[0x00]);
    }, &[0x43, 0x10, 0x05]);
    assert_eq!(cpu.pc, 0x0203); // no branch
    assert_eq!(cyc, 5);
}

#[test]
fn e06_23_bbc_taken_backward() {
    // BBC dp.1,-4: backward branch taken when bit is clear
    let (cpu, _bus, cyc) = run(|cpu, bus| {
        cpu.psw = 0x02;
        bus.poke(0x10, &[0x00]);
    }, &[0x33, 0x10, 0xFC]); // rel = -4 (signed)
    assert_eq!(cpu.pc, 0x01FF); // 0x0203 + (-4)
    assert_eq!(cyc, 7); // 5 base + 2 branch taken
}

#[test]
fn e06_23_bbc_not_taken() {
    // BBC dp.1,-4: branch not taken when bit is set
    let (cpu, _bus, cyc) = run(|cpu, bus| {
        cpu.psw = 0x02;
        bus.poke(0x10, &[0x02]);
    }, &[0x33, 0x10, 0xFC]); // rel = -4 (signed)
    assert_eq!(cpu.pc, 0x0203); // no branch
    assert_eq!(cyc, 5);
}

#[test]
fn e06_23_bbs_dp_n_opcodes() {
    // Verify opcode encoding: BBS dp.7 = E3, BBC dp.7 = F3
    let (cpu, _bus, _cyc) = run(|cpu, bus| {
        cpu.psw = 0x02;
        bus.poke(0x10, &[0x80]); // bit 7 set
    }, &[0xE3, 0x10, 0x02]);
    assert_eq!(cpu.pc, 0x0205); // branch taken

    let (cpu, _bus, _cyc) = run(|cpu, bus| {
        cpu.psw = 0x02;
        bus.poke(0x10, &[0x80]); // bit 7 set
    }, &[0xF3, 0x10, 0x02]);
    assert_eq!(cpu.pc, 0x0203); // branch not taken (bit is set)
}

#[test]
fn e06_23_tset1() {
    // TSET1 !abs: test and set bits in absolute address
    let (cpu, bus, cyc) = run(|cpu, bus| {
        cpu.psw = 0x02;
        cpu.a = 0x0F;
        bus.poke(0x3000, &[0xF0]);
    }, &[0x0E, 0x00, 0x30]);
    assert_eq!(bus.peek(0x3000), 0xFF); // 0xF0 | 0x0F = 0xFF
    assert!(!cpu.flag(F_N)); // N=0 (result of A-t is positive)
    assert!(!cpu.flag(F_Z)); // Z=0 (A != t)
    assert_eq!(cyc, 6);

    // TSET1 with equal values: Z=1
    let (cpu, bus, _cyc) = run(|cpu, bus| {
        cpu.psw = 0x02;
        cpu.a = 0x80;
        bus.poke(0x3000, &[0x80]);
    }, &[0x0E, 0x00, 0x30]);
    assert!(cpu.flag(F_Z)); // Z=1 (A == t)
    assert_eq!(bus.peek(0x3000), 0x80);

    // TSET1 with A < t: N=1
    let (cpu, bus, _cyc) = run(|cpu, bus| {
        cpu.psw = 0x02;
        cpu.a = 0x01;
        bus.poke(0x3000, &[0x02]);
    }, &[0x0E, 0x00, 0x30]);
    assert!(cpu.flag(F_N)); // N=1 (A - t is negative)
    assert_eq!(bus.peek(0x3000), 0x03); // 0x02 | 0x01 = 0x03
}

#[test]
fn e06_23_tclr1() {
    // TCLR1 !abs: test and clear bits in absolute address
    let (cpu, bus, cyc) = run(|cpu, bus| {
        cpu.psw = 0x02;
        cpu.a = 0x0F;
        bus.poke(0x3000, &[0xFF]);
    }, &[0x4E, 0x00, 0x30]);
    assert_eq!(bus.peek(0x3000), 0xF0); // 0xFF & ~0x0F = 0xF0
    assert!(!cpu.flag(F_N)); // N=0 (A - t is positive)
    assert!(!cpu.flag(F_Z)); // Z=0 (A != t)
    assert_eq!(cyc, 6);
}

#[test]
fn e06_23_tset1_keeps_c_v() {
    // TSET1 preserves C and V flags
    let (cpu, _bus, _cyc) = run(|cpu, bus| {
        cpu.psw = F_C | F_V;
        cpu.a = 0x00;
        bus.poke(0x3000, &[0x00]);
    }, &[0x0E, 0x00, 0x30]);
    assert!(cpu.flag(F_C)); // C preserved
    assert!(cpu.flag(F_V)); // V preserved
    assert!(cpu.flag(F_Z)); // Z=1 (A == t)
}
