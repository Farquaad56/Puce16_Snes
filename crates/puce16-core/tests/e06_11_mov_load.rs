use puce16_core::apu::spc700::{Spc700, testbus::FlatSmpBus, F_C, F_Z, F_H, F_P, F_V, F_N};

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
fn e06_11_a_imm_negative() {
    let (cpu, _, cyc) = run(|_, _| {}, &[0xE8, 0x80]);
    assert_eq!(cpu.a, 0x80);
    assert!(cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 2);
}

#[test]
fn e06_11_a_dp_zero() {
    let (cpu, _, cyc) = run(|cpu, bus| {
        cpu.a = 5;
        bus.poke(0x0010, &[0]);
    }, &[0xE4, 0x10]);
    assert_eq!(cpu.a, 0);
    assert!(cpu.flag(F_Z));
    assert!(!cpu.flag(F_N));
    assert_eq!(cyc, 3);
}

#[test]
fn e06_11_a_dpx_wrap() {
    let (cpu, _, cyc) = run(|cpu, bus| {
        cpu.x = 2;
        bus.poke(0x0000, &[0x7F]);
    }, &[0xF4, 0xFE]);
    assert_eq!(cpu.a, 0x7F);
    assert_eq!(cyc, 4);
}

#[test]
fn e06_11_a_abs() {
    let (cpu, _, cyc) = run(|_, bus| {
        bus.poke(0x3000, &[0x42]);
    }, &[0xE5, 0x00, 0x30]);
    assert_eq!(cpu.a, 0x42);
    assert_eq!(cyc, 4);
}

#[test]
fn e06_11_a_absx() {
    let (cpu, _, cyc) = run(|cpu, bus| {
        cpu.x = 1;
        bus.poke(0x3000, &[0x11]);
    }, &[0xF5, 0xFF, 0x2F]);
    assert_eq!(cpu.a, 0x11);
    assert_eq!(cyc, 5);
}

#[test]
fn e06_11_a_absy_negative() {
    let (cpu, _, cyc) = run(|cpu, bus| {
        cpu.y = 2;
        bus.poke(0x3002, &[0x99]);
    }, &[0xF6, 0x00, 0x30]);
    assert_eq!(cpu.a, 0x99);
    assert!(cpu.flag(F_N));
    assert_eq!(cyc, 5);
}

#[test]
fn e06_11_a_x_indirect() {
    let (cpu, _, cyc) = run(|cpu, bus| {
        cpu.x = 0x10;
        bus.poke(0x0010, &[0x55]);
    }, &[0xE6]);
    assert_eq!(cpu.a, 0x55);
    assert_eq!(cyc, 3);
}

#[test]
fn e06_11_a_dpx_indirect() {
    let (cpu, _, cyc) = run(|cpu, bus| {
        cpu.x = 2;
        bus.poke(0x0012, &[0x00]);
        bus.poke(0x0013, &[0x30]);
        bus.poke(0x3000, &[0x66]);
    }, &[0xE7, 0x10]);
    assert_eq!(cpu.a, 0x66);
    assert_eq!(cyc, 6);
}

#[test]
fn e06_11_a_dp_indirect_y() {
    let (cpu, _, cyc) = run(|cpu, bus| {
        cpu.y = 1;
        bus.poke(0x0020, &[0xFF]);
        bus.poke(0x0021, &[0x2F]);
        bus.poke(0x3000, &[0x77]);
    }, &[0xF7, 0x20]);
    assert_eq!(cpu.a, 0x77);
    assert_eq!(cyc, 6);
}

#[test]
fn e06_11_x_loads() {
    // MOV X, #imm
    let (cpu, _, cyc) = run(|_, _| {}, &[0xCD, 0x00]);
    assert_eq!(cpu.x, 0);
    assert!(cpu.flag(F_Z));
    assert_eq!(cyc, 2);

    // MOV X, dp
    let (cpu, _, cyc) = run(|_, bus| {
        bus.poke(0x10, &[0x81]);
    }, &[0xF8, 0x10]);
    assert_eq!(cpu.x, 0x81);
    assert!(cpu.flag(F_N));
    assert_eq!(cyc, 3);

    // MOV X, dp+Y (wrap)
    let (cpu, _, cyc) = run(|cpu, bus| {
        cpu.y = 3;
        bus.poke(0x0001, &[0x22]);
    }, &[0xF9, 0xFE]);
    assert_eq!(cpu.x, 0x22);
    assert_eq!(cyc, 4);

    // MOV X, !abs
    let (cpu, _, cyc) = run(|_, bus| {
        bus.poke(0x3000, &[0x33]);
    }, &[0xE9, 0x00, 0x30]);
    assert_eq!(cpu.x, 0x33);
    assert_eq!(cyc, 4);
}

#[test]
fn e06_11_y_loads() {
    // MOV Y, #imm
    let (cpu, _, cyc) = run(|_, _| {}, &[0x8D, 0x7F]);
    assert_eq!(cpu.y, 0x7F);
    assert_eq!(cyc, 2);

    // MOV Y, dp
    let (cpu, _, cyc) = run(|_, bus| {
        bus.poke(0x10, &[0x80]);
    }, &[0xEB, 0x10]);
    assert_eq!(cpu.y, 0x80);
    assert!(cpu.flag(F_N));
    assert_eq!(cyc, 3);

    // MOV Y, dp+X
    let (cpu, _, cyc) = run(|cpu, bus| {
        cpu.x = 1;
        bus.poke(0x0011, &[0x44]);
    }, &[0xFB, 0x10]);
    assert_eq!(cpu.y, 0x44);
    assert_eq!(cyc, 4);

    // MOV Y, !abs
    let (cpu, _, cyc) = run(|_, bus| {
        bus.poke(0x3000, &[0x55]);
    }, &[0xEC, 0x00, 0x30]);
    assert_eq!(cpu.y, 0x55);
    assert_eq!(cyc, 4);
}

#[test]
fn e06_11_other_flags_untouched() {
    let (cpu, _, _) = run(|cpu, _| {
        cpu.psw = F_C | F_H | F_V; // 0x49
    }, &[0xE8, 0x01]);
    assert!(cpu.flag(F_C));
    assert!(cpu.flag(F_H));
    assert!(cpu.flag(F_V));
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
}

#[test]
fn e06_11_direct_page_p_flag() {
    let (cpu, _, _) = run(|cpu, bus| {
        cpu.psw = F_P;
        bus.poke(0x0110, &[0x5A]);
    }, &[0xE4, 0x10]);
    assert_eq!(cpu.a, 0x5A);
}
