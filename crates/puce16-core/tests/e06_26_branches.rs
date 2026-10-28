use puce16_core::apu::spc700::{Spc700, SmpBus, testbus::FlatSmpBus, F_C, F_Z, F_I, F_H, F_B, F_P, F_V, F_N};

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
fn e06_26_beq_taken() {
    // BEQ rel: branch if Z flag set
    let (cpu, _, cyc) = run(|c, _| { c.psw = F_Z; }, &[0xF0, 0x05]);
    assert_eq!(cpu.pc, 0x0207, "BEQ taken should jump forward");
    assert_eq!(cyc, 4, "BEQ taken: 2 base + 2 branch extra");
}

#[test]
fn e06_26_beq_not_taken() {
    // BEQ rel: no branch if Z flag clear
    let (cpu, _, cyc) = run(|c, _| { c.psw = 0; }, &[0xF0, 0x05]);
    assert_eq!(cpu.pc, 0x0202, "BEQ not taken should fall through");
    assert_eq!(cyc, 2, "BEQ not taken: 2 base cycles");
}

#[test]
fn e06_26_bne_backward() {
    // BNE rel: branch backward if Z flag clear
    let (cpu, _, cyc) = run(|c, _| { c.psw = 0; }, &[0xD0, 0xFE]);
    assert_eq!(cpu.pc, 0x0200, "BNE taken should jump backward");
    assert_eq!(cyc, 4, "BNE taken: 2 base + 2 branch extra");
}

#[test]
fn e06_26_bpl_bmi() {
    // BPL rel: branch if !N (positive)
    let (cpu1, _, _) = run(|c, _| { c.psw = 0; }, &[0x10, 0x10]);
    assert_eq!(cpu1.pc, 0x0212, "BPL taken when N=0");

    // BPL not taken when N=1
    let (cpu2, _, _) = run(|c, _| { c.psw = F_N; }, &[0x10, 0x10]);
    assert_eq!(cpu2.pc, 0x0202, "BPL not taken when N=1");

    // BMI rel: branch if N (minus)
    let (cpu3, _, _) = run(|c, _| { c.psw = F_N; }, &[0x30, 0x10]);
    assert_eq!(cpu3.pc, 0x0212, "BMI taken when N=1");

    // BMI not taken when N=0
    let (cpu4, _, _) = run(|c, _| { c.psw = 0; }, &[0x30, 0x10]);
    assert_eq!(cpu4.pc, 0x0202, "BMI not taken when N=0");
}

#[test]
fn e06_26_bvc_bvs() {
    // BVC rel: branch if !V (no overflow)
    let (cpu1, _, _) = run(|c, _| { c.psw = 0; }, &[0x50, 0x04]);
    assert_eq!(cpu1.pc, 0x0206, "BVC taken when V=0");

    // BVC not taken when V=1
    let (cpu2, _, _) = run(|c, _| { c.psw = F_V; }, &[0x50, 0x04]);
    assert_eq!(cpu2.pc, 0x0202, "BVC not taken when V=1");

    // BVS rel: branch if V (overflow)
    let (cpu3, _, _) = run(|c, _| { c.psw = F_V; }, &[0x70, 0x04]);
    assert_eq!(cpu3.pc, 0x0206, "BVS taken when V=1");

    // BVS not taken when V=0
    let (cpu4, _, _) = run(|c, _| { c.psw = 0; }, &[0x70, 0x04]);
    assert_eq!(cpu4.pc, 0x0202, "BVS not taken when V=0");
}

#[test]
fn e06_26_bcc_bcs() {
    // BCC rel: branch if !C (no carry)
    let (cpu1, _, _) = run(|c, _| { c.psw = 0; }, &[0x90, 0x04]);
    assert_eq!(cpu1.pc, 0x0206, "BCC taken when C=0");

    // BCC not taken when C=1
    let (cpu2, _, _) = run(|c, _| { c.psw = F_C; }, &[0x90, 0x04]);
    assert_eq!(cpu2.pc, 0x0202, "BCC not taken when C=1");

    // BCS rel: branch if C (carry)
    let (cpu3, _, _) = run(|c, _| { c.psw = F_C; }, &[0xB0, 0x04]);
    assert_eq!(cpu3.pc, 0x0206, "BCS taken when C=1");

    // BCS not taken when C=0
    let (cpu4, _, _) = run(|c, _| { c.psw = 0; }, &[0xB0, 0x04]);
    assert_eq!(cpu4.pc, 0x0202, "BCS not taken when C=0");
}

#[test]
fn e06_26_bra() {
    // BRA rel: unconditional branch forward
    let (cpu1, _, cyc1) = run(|c, _| {}, &[0x2F, 0x7F]);
    assert_eq!(cpu1.pc, 0x0281, "BRA should jump forward");
    assert_eq!(cyc1, 4, "BRA: 4 base cycles (no branch extra)");

    // BRA rel: unconditional branch backward (-128 = 0x80)
    let (cpu2, _, cyc2) = run(|c, _| {}, &[0x2F, 0x80]);
    assert_eq!(cpu2.pc, 0x0182, "BRA should jump backward");
    assert_eq!(cyc2, 4, "BRA: 4 base cycles (no branch extra)");
}

#[test]
fn e06_26_branch_keeps_flags() {
    // Branches should not modify any flags
    let (cpu1, _, _) = run(|c, _| { c.psw = 0xC3; }, &[0xF0, 0x05]);
    assert_eq!(cpu1.psw, 0xC3, "BEQ taken should preserve all flags");

    let (cpu2, _, _) = run(|c, _| { c.psw = 0xC3; }, &[0xD0, 0x05]);
    assert_eq!(cpu2.psw, 0xC3, "BNE taken should preserve all flags");
}

#[test]
fn e06_26_cbne_dp() {
    // CBNE dp,rel: compare A with memory at dp, branch if not equal
    // Equal case: no branch
    let (cpu1, _, cyc1) = run(|c, b| {
        c.a = 0x07;
        b.poke(0x0010, &[0x07]);
    }, &[0x2E, 0x10, 0x05]);
    assert_eq!(cpu1.pc, 0x0203, "CBNE not taken when A == M");
    assert_eq!(cyc1, 5, "CBNE not taken: 5 base cycles");

    // Not equal case: branch
    let (cpu2, _, cyc2) = run(|c, b| {
        c.a = 0x08;
        b.poke(0x0010, &[0x07]);
    }, &[0x2E, 0x10, 0x05]);
    assert_eq!(cpu2.pc, 0x0208, "CBNE taken when A != M");
    assert_eq!(cyc2, 7, "CBNE taken: 5 base + 2 branch extra");
}

#[test]
fn e06_26_cbne_dpx() {
    // CBNE dp+X,rel: compare A with memory at dp+X, branch if not equal
    // Not equal case: branch (A=0, M[dp+X]=1)
    let (cpu1, _, cyc1) = run(|c, b| {
        c.a = 0;
        c.x = 1;
        b.poke(0x0011, &[0x01]);
    }, &[0xDE, 0x10, 0x05]);
    assert_eq!(cpu1.pc, 0x0208, "CBNE dp+X taken when A != M");
    assert_eq!(cyc1, 8, "CBNE dp+X taken: 6 base + 2 branch extra");

    // Equal case: no branch (A=1, M[dp+X]=1)
    let (cpu2, _, cyc2) = run(|c, b| {
        c.a = 1;
        c.x = 1;
        b.poke(0x0011, &[0x01]);
    }, &[0xDE, 0x10, 0x05]);
    assert_eq!(cpu2.pc, 0x0203, "CBNE dp+X not taken when A == M");
    assert_eq!(cyc2, 6, "CBNE dp+X not taken: 6 base cycles");
}

#[test]
fn e06_26_dbnz_y() {
    // DBNZ Y,rel: decrement Y and branch if not zero
    // Y=2 -> Y=1 (not zero), branch taken
    let (cpu1, _, cyc1) = run(|c, _| { c.y = 2; }, &[0xFE, 0x05]);
    assert_eq!(cpu1.y, 1, "DBNZ Y should decrement Y");
    assert_eq!(cpu1.pc, 0x0207, "DBNZ Y taken when result != 0");
    assert_eq!(cyc1, 6, "DBNZ Y taken: 4 base + 2 branch extra");

    // Y=1 -> Y=0 (zero), no branch
    let (cpu2, _, cyc2) = run(|c, _| { c.y = 1; }, &[0xFE, 0x05]);
    assert_eq!(cpu2.y, 0, "DBNZ Y should decrement to zero");
    assert_eq!(cpu2.pc, 0x0202, "DBNZ Y not taken when result == 0");
    assert_eq!(cyc2, 4, "DBNZ Y not taken: 4 base cycles");

    // Y=0 -> Y=0xFF (wrap), branch taken
    let (cpu3, _, _) = run(|c, _| { c.y = 0; }, &[0xFE, 0x05]);
    assert_eq!(cpu3.y, 0xFF, "DBNZ Y should wrap from 0 to 0xFF");
    assert_eq!(cpu3.pc, 0x0207, "DBNZ Y taken when wrapped result != 0");
}

#[test]
fn e06_26_dbnz_dp() {
    // DBNZ dp,rel: decrement memory at dp and branch if not zero
    // M[dp]=2 -> M[dp]=1 (not zero), branch taken
    let (cpu1, bus1, cyc1) = run(|c, b| {
        b.poke(0x0010, &[2]);
    }, &[0x6E, 0x10, 0x05]);
    assert_eq!(bus1.peek(0x0010), 1, "DBNZ dp should decrement memory");
    assert_eq!(cpu1.pc, 0x0208, "DBNZ dp taken when result != 0");
    assert_eq!(cyc1, 7, "DBNZ dp taken: 5 base + 2 branch extra");

    // M[dp]=1 -> M[dp]=0 (zero), no branch
    let (cpu2, bus2, cyc2) = run(|c, b| {
        b.poke(0x0010, &[1]);
    }, &[0x6E, 0x10, 0x05]);
    assert_eq!(bus2.peek(0x0010), 0, "DBNZ dp should decrement to zero");
    assert_eq!(cpu2.pc, 0x0203, "DBNZ dp not taken when result == 0");
    assert_eq!(cyc2, 5, "DBNZ dp not taken: 5 base cycles");
}

#[test]
fn e06_26_dbnz_keeps_flags() {
    // DBNZ should not modify any flags
    let (cpu, _, _) = run(|c, _| {
        c.psw = 0xC3;
        c.y = 2;
    }, &[0xFE, 0x05]);
    assert_eq!(cpu.psw, 0xC3, "DBNZ Y should preserve all flags");
}