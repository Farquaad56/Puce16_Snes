use puce16_core::apu::spc700::{Spc700, testbus::FlatSmpBus, F_C, F_Z, F_N, F_V, F_H};

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
fn e06_17_cmp_equal() {
    // CMP A,#imm: a=0x10 vs 0x10 -> Z=1, C=1, N=0; a unchanged
    let (cpu, _, cyc) = run(|c, _| { c.a = 0x10; }, &[0x68, 0x10]);
    assert!(cpu.flag(F_Z));
    assert!(cpu.flag(F_C));
    assert!(!cpu.flag(F_N));
    assert_eq!(cpu.a, 0x10);
    assert_eq!(cyc, 2);
}

#[test]
fn e06_17_cmp_less() {
    // CMP A,#imm: a=0x10 vs 0x11 -> C=0, N=1, Z=0 (result 0xFF)
    let (cpu, _, cyc) = run(|c, _| { c.a = 0x10; }, &[0x68, 0x11]);
    assert!(!cpu.flag(F_C));
    assert!(cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cpu.a, 0x10);
    assert_eq!(cyc, 2);
}

#[test]
fn e06_17_cmp_greater() {
    // CMP A,#imm: a=0x10 vs 0x0F -> C=1, N=0, Z=0
    let (cpu, _, cyc) = run(|c, _| { c.a = 0x10; }, &[0x68, 0x0F]);
    assert!(cpu.flag(F_C));
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cpu.a, 0x10);
    assert_eq!(cyc, 2);
}

#[test]
fn e06_17_cmp_keeps_v_h() {
    // CMP preserves V and H flags; only sets N,Z,C
    let (cpu, _, cyc) = run(|c, _| { c.psw = F_V | F_H; c.a = 0x10; }, &[0x68, 0x10]);
    assert!(cpu.flag(F_V));
    assert!(cpu.flag(F_H));
    assert!(cpu.flag(F_Z));
    assert!(cpu.flag(F_C));
    assert!(!cpu.flag(F_N));
    assert_eq!(cyc, 2);
}

#[test]
fn e06_17_cmp_a_dp() {
    // CMP A,dp: a=0x10 vs mem[0x10]=0x10 -> Z=1, C=1; cycles 3
    let (cpu, _, cyc) = run(|c, b| { c.a = 0x10; b.poke(0x10, &[0x10]); }, &[0x64, 0x10]);
    assert!(cpu.flag(F_Z));
    assert!(cpu.flag(F_C));
    assert_eq!(cpu.a, 0x10);
    assert_eq!(cyc, 3);
}

#[test]
fn e06_17_cmp_a_abs() {
    // CMP A,!abs: a=0x05 vs mem[0x3000]=0x05 -> Z=1, C=1; cycles 4
    let (cpu, _, cyc) = run(|c, b| { c.a = 0x05; b.poke(0x3000, &[0x05]); }, &[0x65, 0x00, 0x30]);
    assert!(cpu.flag(F_Z));
    assert!(cpu.flag(F_C));
    assert_eq!(cyc, 4);
}

#[test]
fn e06_17_cmp_a_x() {
    // CMP A,(X): a=0x05 vs mem[X]=0x05 -> Z=1, C=1; cycles 3
    let (cpu, _, cyc) = run(|c, b| { c.a = 0x05; c.x = 0x10; b.poke(0x10, &[0x05]); }, &[0x66]);
    assert!(cpu.flag(F_Z));
    assert!(cpu.flag(F_C));
    assert_eq!(cyc, 3);
}

#[test]
fn e06_17_cmp_a_dpx_ind() {
    // CMP A,[dp+X]: a=0x05 vs mem[mem[0x12]]=0x05 -> Z=1, C=1; cycles 6
    let (cpu, _, cyc) = run(|c, b| {
        c.a = 0x05;
        c.x = 2;
        b.poke(0x12, &[0x00, 0x30]);
        b.poke(0x3000, &[0x05]);
    }, &[0x67, 0x10]);
    assert!(cpu.flag(F_Z));
    assert!(cpu.flag(F_C));
    assert_eq!(cyc, 6);
}

#[test]
fn e06_17_cmp_a_dpx() {
    // CMP A,dp+X: a=0x05 vs mem[0x11]=0x05 -> Z=1, C=1; cycles 4
    let (cpu, _, cyc) = run(|c, b| { c.a = 0x05; c.x = 1; b.poke(0x11, &[0x05]); }, &[0x74, 0x10]);
    assert!(cpu.flag(F_Z));
    assert!(cpu.flag(F_C));
    assert_eq!(cyc, 4);
}

#[test]
fn e06_17_cmp_a_absx() {
    // CMP A,!abs+X: a=0x05 vs mem[0x3000]=0x05 -> Z=1, C=1; cycles 5
    let (cpu, _, cyc) = run(|c, b| { c.a = 0x05; c.x = 1; b.poke(0x3000, &[0x05]); }, &[0x75, 0xFF, 0x2F]);
    assert!(cpu.flag(F_Z));
    assert!(cpu.flag(F_C));
    assert_eq!(cyc, 5);
}

#[test]
fn e06_17_cmp_a_absy() {
    // CMP A,!abs+Y: a=0x05 vs mem[0x3002]=0x05 -> Z=1, C=1; cycles 5
    let (cpu, _, cyc) = run(|c, b| { c.a = 0x05; c.y = 2; b.poke(0x3002, &[0x05]); }, &[0x76, 0x00, 0x30]);
    assert!(cpu.flag(F_Z));
    assert!(cpu.flag(F_C));
    assert_eq!(cyc, 5);
}

#[test]
fn e06_17_cmp_a_dp_ind_y() {
    // CMP A,[dp]+Y: a=0x05 vs mem[mem[0x20]+1]=0x05 -> Z=1, C=1; cycles 6
    let (cpu, _, cyc) = run(|c, b| {
        c.a = 0x05;
        c.y = 1;
        b.poke(0x20, &[0xFF, 0x2F]);
        b.poke(0x3000, &[0x05]);
    }, &[0x77, 0x20]);
    assert!(cpu.flag(F_Z));
    assert!(cpu.flag(F_C));
    assert_eq!(cyc, 6);
}

#[test]
fn e06_17_cmp_x_imm() {
    // CMP X,#imm: x=5 vs 5 -> Z=1, C=1; then x=5 vs 6 -> C=0, N=1
    let (cpu, _, cyc) = run(|c, _| { c.x = 5; }, &[0xC8, 0x05]);
    assert!(cpu.flag(F_Z));
    assert!(cpu.flag(F_C));
    assert_eq!(cyc, 2);

    let (cpu2, _, _) = run(|c, _| { c.x = 5; }, &[0xC8, 0x06]);
    assert!(!cpu2.flag(F_C));
    assert!(cpu2.flag(F_N));
}

#[test]
fn e06_17_cmp_y_imm() {
    // CMP Y,#imm: y=5 vs 6 -> C=0, N=1, Z=0
    let (cpu, _, cyc) = run(|c, _| { c.y = 5; }, &[0xAD, 0x06]);
    assert!(!cpu.flag(F_C));
    assert!(cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 2);
}

#[test]
fn e06_17_cmp_x_dp() {
    // CMP X,dp: x=0x80 vs mem[0x10]=0x01 -> C=1, N=0, Z=0 (result 0x7F)
    let (cpu, _, cyc) = run(|c, b| { c.x = 0x80; b.poke(0x10, &[0x01]); }, &[0x3E, 0x10]);
    assert!(cpu.flag(F_C));
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 3);
}

#[test]
fn e06_17_cmp_x_abs() {
    // CMP X,!abs: x=0 vs mem[0x3000]=0x01 -> C=0, N=1 (result 0xFF)
    let (cpu, _, cyc) = run(|c, b| { c.x = 0; b.poke(0x3000, &[0x01]); }, &[0x1E, 0x00, 0x30]);
    assert!(!cpu.flag(F_C));
    assert!(cpu.flag(F_N));
    assert_eq!(cyc, 4);
}

#[test]
fn e06_17_cmp_y_dp() {
    // CMP Y,dp: y=2 vs mem[0x10]=0x02 -> Z=1, C=1; cycles 3
    let (cpu, _, cyc) = run(|c, b| { c.y = 2; b.poke(0x10, &[0x02]); }, &[0x7E, 0x10]);
    assert!(cpu.flag(F_Z));
    assert!(cpu.flag(F_C));
    assert_eq!(cyc, 3);
}

#[test]
fn e06_17_cmp_y_abs() {
    // CMP Y,!abs: y=2 vs mem[0x3000]=0x03 -> C=0, N=1
    let (cpu, _, cyc) = run(|c, b| { c.y = 2; b.poke(0x3000, &[0x03]); }, &[0x5E, 0x00, 0x30]);
    assert!(!cpu.flag(F_C));
    assert!(cpu.flag(F_N));
    assert_eq!(cyc, 4);
}

#[test]
fn e06_17_cmp_dp_dp_no_write() {
    // CMP dp,dp: mem[0x20](5) vs mem[0x10](3) -> C=1; no write-back to mem[0x20]
    let (cpu, bus, cyc) = run(|_c, b| {
        b.poke(0x10, &[0x03]);
        b.poke(0x20, &[0x05]);
    }, &[0x69, 0x10, 0x20]);
    assert!(cpu.flag(F_C));
    assert!(!cpu.flag(F_Z));
    assert!(!cpu.flag(F_N));
    assert_eq!(bus.peek(0x20), 5); // unchanged
    assert_eq!(cyc, 6);
}

#[test]
fn e06_17_cmp_dp_imm() {
    // CMP dp,#imm: mem[0x20](5) vs 3 -> C=1; no write-back
    let (cpu, bus, cyc) = run(|_c, b| {
        b.poke(0x20, &[0x05]);
    }, &[0x78, 0x03, 0x20]);
    assert!(cpu.flag(F_C));
    assert_eq!(bus.peek(0x20), 5); // unchanged
    assert_eq!(cyc, 5);
}

#[test]
fn e06_17_cmp_xy() {
    // CMP (X),(Y): mem[X](1) vs mem[Y](2) -> C=0, N=1 (result 0xFF)
    let (cpu, _, cyc) = run(|c, b| {
        c.x = 0x10;
        c.y = 0x11;
        b.poke(0x10, &[0x01]);
        b.poke(0x11, &[0x02]);
    }, &[0x79]);
    assert!(!cpu.flag(F_C));
    assert!(cpu.flag(F_N));
    assert_eq!(cyc, 5);
}
