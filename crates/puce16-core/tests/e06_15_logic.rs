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
fn e06_15_or_imm() {
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.a = 0x0F;
    }, &[0x08, 0xF0]);
    assert_eq!(cpu.a, 0xFF);
    assert!(cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 2);
}

#[test]
fn e06_15_and_imm_zero() {
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.a = 0xF0;
    }, &[0x28, 0x0F]);
    assert_eq!(cpu.a, 0);
    assert!(cpu.flag(F_Z));
    assert!(!cpu.flag(F_N));
    assert_eq!(cyc, 2);
}

#[test]
fn e06_15_eor_imm() {
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.a = 0x0F;
    }, &[0x48, 0xFF]);
    assert_eq!(cpu.a, 0xF0);
    assert!(cpu.flag(F_N));
    assert_eq!(cyc, 2);
}

#[test]
fn e06_15_or_dp() {
    let (cpu, _, cyc) = run(|cpu, bus| {
        cpu.a = 0x80;
        bus.poke(0x0010, &[0x01]);
    }, &[0x04, 0x10]);
    assert_eq!(cpu.a, 0x81);
    assert!(cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 3);
}

#[test]
fn e06_15_and_abs() {
    let (cpu, _, cyc) = run(|cpu, bus| {
        cpu.a = 0x3C;
        bus.poke(0x3000, &[0x0F]);
    }, &[0x25, 0x00, 0x30]);
    assert_eq!(cpu.a, 0x0C);
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 4);
}

#[test]
fn e06_15_eor_x_indirect() {
    let (cpu, _, cyc) = run(|cpu, bus| {
        cpu.x = 0x10;
        cpu.a = 0xFF;
        bus.poke(0x0010, &[0xFF]);
    }, &[0x46]);
    assert_eq!(cpu.a, 0);
    assert!(cpu.flag(F_Z));
    assert!(!cpu.flag(F_N));
    assert_eq!(cyc, 3);
}

#[test]
fn e06_15_or_dpx_indirect() {
    let (cpu, _, cyc) = run(|cpu, bus| {
        cpu.x = 2;
        cpu.a = 0x01;
        bus.poke(0x0012, &[0x00]);
        bus.poke(0x0013, &[0x30]);
        bus.poke(0x3000, &[0x40]);
    }, &[0x07, 0x10]);
    assert_eq!(cpu.a, 0x41);
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 6);
}

#[test]
fn e06_15_and_absx() {
    let (cpu, _, cyc) = run(|cpu, bus| {
        cpu.x = 1;
        cpu.a = 0xFF;
        bus.poke(0x3000, &[0x55]);
    }, &[0x35, 0xFF, 0x2F]);
    assert_eq!(cpu.a, 0x55);
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 5);
}

#[test]
fn e06_15_eor_absy() {
    let (cpu, _, cyc) = run(|cpu, bus| {
        cpu.y = 2;
        cpu.a = 0xFF;
        bus.poke(0x3002, &[0xAA]);
    }, &[0x56, 0x00, 0x30]);
    assert_eq!(cpu.a, 0x55);
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 5);
}

#[test]
fn e06_15_or_dp_indirect_y() {
    let (cpu, _, cyc) = run(|cpu, bus| {
        cpu.y = 1;
        cpu.a = 0x01;
        bus.poke(0x0020, &[0xFF]);
        bus.poke(0x0021, &[0x2F]);
        bus.poke(0x3000, &[0x08]);
    }, &[0x17, 0x20]);
    assert_eq!(cpu.a, 0x09);
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 6);
}

#[test]
fn e06_15_or_dpx() {
    let (cpu, _, cyc) = run(|cpu, bus| {
        cpu.x = 1;
        cpu.a = 1;
        bus.poke(0x0011, &[0x02]);
    }, &[0x14, 0x10]);
    assert_eq!(cpu.a, 3);
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 4);
}

#[test]
fn e06_15_or_dp_dp() {
    let (cpu, bus, cyc) = run(|_cpu, bus| {
        bus.poke(0x0010, &[0x0F]);
        bus.poke(0x0020, &[0xF0]);
    }, &[0x09, 0x10, 0x20]);
    assert_eq!(bus.peek(0x0020), 0xFF);
    assert!(cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 6);
}

#[test]
fn e06_15_and_dp_dp() {
    let (cpu, bus, cyc) = run(|_cpu, bus| {
        bus.poke(0x0010, &[0x3C]);
        bus.poke(0x0020, &[0xF0]);
    }, &[0x29, 0x10, 0x20]);
    assert_eq!(bus.peek(0x0020), 0x30);
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 6);
}

#[test]
fn e06_15_eor_dp_dp_zero() {
    let (cpu, bus, cyc) = run(|_cpu, bus| {
        bus.poke(0x0010, &[0xFF]);
        bus.poke(0x0020, &[0xFF]);
    }, &[0x49, 0x10, 0x20]);
    assert_eq!(bus.peek(0x0020), 0);
    assert!(cpu.flag(F_Z));
    assert!(!cpu.flag(F_N));
    assert_eq!(cyc, 6);
}

#[test]
fn e06_15_or_dp_imm() {
    let (cpu, bus, cyc) = run(|_cpu, bus| {
        bus.poke(0x0010, &[0xF0]);
    }, &[0x18, 0x0F, 0x10]);
    assert_eq!(bus.peek(0x0010), 0xFF);
    assert!(cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 5);
}

#[test]
fn e06_15_and_dp_imm() {
    let (cpu, bus, cyc) = run(|_cpu, bus| {
        bus.poke(0x0010, &[0x3C]);
    }, &[0x38, 0xF0, 0x10]);
    assert_eq!(bus.peek(0x0010), 0x30);
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 5);
}

#[test]
fn e06_15_eor_xy() {
    // EOR (X),(Y)
    let (cpu, bus, cyc) = run(|cpu, bus| {
        cpu.x = 0x10;
        cpu.y = 0x11;
        bus.poke(0x0010, &[0xF0]);
        bus.poke(0x0011, &[0xFF]);
    }, &[0x59]);
    assert_eq!(bus.peek(0x0010), 0x0F);
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 5);

    // OR (X),(Y) with different values
    let (cpu2, bus2, cyc2) = run(|cpu, bus| {
        cpu.x = 0x10;
        cpu.y = 0x11;
        bus.poke(0x0010, &[0x01]);
        bus.poke(0x0011, &[0x02]);
    }, &[0x19]);
    assert_eq!(bus2.peek(0x0010), 0x03);
    assert!(!cpu2.flag(F_N));
    assert!(!cpu2.flag(F_Z));
    assert_eq!(cyc2, 5);
}

#[test]
fn e06_15_other_flags_untouched() {
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.psw = F_C | F_H | F_V; // C=1, H=1, V=1
        cpu.a = 0x01;
    }, &[0x08, 0x01]);
    assert!(cpu.flag(F_C));
    assert!(cpu.flag(F_H));
    assert!(cpu.flag(F_V));
    // N and Z should be set based on result (0x02)
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 2);
}

#[test]
fn e06_15_p_flag() {
    let (cpu, _, cyc) = run(|cpu, bus| {
        cpu.psw = F_P; // P flag set -> dp base is 0x0100
        cpu.a = 0;
        bus.poke(0x0110, &[0x01]);
    }, &[0x04, 0x10]);
    assert_eq!(cpu.a, 1);
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 3);
}
