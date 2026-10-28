use puce16_core::apu::spc700::{Spc700, SmpBus, testbus::FlatSmpBus, F_C, F_Z, F_I, F_H, F_B, F_P, F_V, F_N};

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
fn e06_19_inc_a_wrap() {
    // INC A: 0xFF -> 0x00, Z=1, N=0, C unchanged
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.psw = F_C; // C=1
        cpu.a = 0xFF;
    }, &[0xBC]);
    assert_eq!(cpu.a, 0);
    assert!(cpu.flag(F_Z));
    assert!(!cpu.flag(F_N));
    assert!(cpu.flag(F_C)); // unchanged
    assert_eq!(cyc, 2);
}

#[test]
fn e06_19_dec_a_wrap() {
    // DEC A: 0x00 -> 0xFF, N=1, Z=0
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.a = 0;
    }, &[0x9C]);
    assert_eq!(cpu.a, 0xFF);
    assert!(cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 2);
}

#[test]
fn e06_19_inc_dec_x_y() {
    // INC X: 0x7F -> 0x80, N=1
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.x = 0x7F;
    }, &[0x3D]);
    assert_eq!(cpu.x, 0x80);
    assert!(cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 2);

    // DEC X: 1 -> 0, Z=1
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.x = 1;
    }, &[0x1D]);
    assert_eq!(cpu.x, 0);
    assert!(cpu.flag(F_Z));
    assert!(!cpu.flag(F_N));
    assert_eq!(cyc, 2);

    // INC Y: 0xFF -> 0, Z=1
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.y = 0xFF;
    }, &[0xFC]);
    assert_eq!(cpu.y, 0);
    assert!(cpu.flag(F_Z));
    assert!(!cpu.flag(F_N));
    assert_eq!(cyc, 2);

    // DEC Y: 0x80 -> 0x7F, N=0
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.y = 0x80;
    }, &[0xDC]);
    assert_eq!(cpu.y, 0x7F);
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 2);
}

#[test]
fn e06_19_inc_dp() {
    // INC dp: 0x7F -> 0x80, N=1
    let (cpu, bus, cyc) = run(|cpu, bus| {
        bus.poke(0x0010, &[0x7F]);
    }, &[0xAB, 0x10]);
    assert_eq!(bus.peek(0x0010), 0x80);
    assert!(cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 4);
}

#[test]
fn e06_19_inc_dpx_zero() {
    // INC dp+X: 0xFF -> 0, Z=1
    let (cpu, bus, cyc) = run(|cpu, bus| {
        cpu.x = 1;
        bus.poke(0x0011, &[0xFF]);
    }, &[0xBB, 0x10]);
    assert_eq!(bus.peek(0x0011), 0);
    assert!(cpu.flag(F_Z));
    assert!(!cpu.flag(F_N));
    assert_eq!(cyc, 5);
}

#[test]
fn e06_19_inc_abs() {
    // INC !abs: 1 -> 2
    let (cpu, bus, cyc) = run(|cpu, bus| {
        bus.poke(0x3000, &[1]);
    }, &[0xAC, 0x00, 0x30]);
    assert_eq!(bus.peek(0x3000), 2);
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 5);
}

#[test]
fn e06_19_dec_memory() {
    // DEC dp: 0 -> 0xFF, N=1
    let (cpu, bus, cyc) = run(|cpu, bus| {
        bus.poke(0x0010, &[0]);
    }, &[0x8B, 0x10]);
    assert_eq!(bus.peek(0x0010), 0xFF);
    assert!(cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 4);

    // DEC dp+X: 1 -> 0, Z=1
    let (cpu, bus, cyc) = run(|cpu, bus| {
        cpu.x = 1;
        bus.poke(0x0011, &[1]);
    }, &[0x9B, 0x10]);
    assert_eq!(bus.peek(0x0011), 0);
    assert!(cpu.flag(F_Z));
    assert!(!cpu.flag(F_N));
    assert_eq!(cyc, 5);

    // DEC !abs: 5 -> 4
    let (cpu, bus, cyc) = run(|cpu, bus| {
        bus.poke(0x3000, &[5]);
    }, &[0x8C, 0x00, 0x30]);
    assert_eq!(bus.peek(0x3000), 4);
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 5);
}

#[test]
fn e06_19_asl() {
    // ASL A: 0x81 -> 0x02, C=1 (bit7 was set)
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.a = 0x81;
    }, &[0x1C]);
    assert_eq!(cpu.a, 0x02);
    assert!(cpu.flag(F_C));
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 2);

    // ASL dp: 0x40 -> 0x80, N=1, C=0
    let (cpu, bus, cyc) = run(|cpu, bus| {
        bus.poke(0x0010, &[0x40]);
    }, &[0x0B, 0x10]);
    assert_eq!(bus.peek(0x0010), 0x80);
    assert!(cpu.flag(F_N));
    assert!(!cpu.flag(F_C));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 4);

    // ASL dp+X: 0x80 -> 0, Z=1, C=1
    let (cpu, bus, cyc) = run(|cpu, bus| {
        cpu.x = 1;
        bus.poke(0x0011, &[0x80]);
    }, &[0x1B, 0x10]);
    assert_eq!(bus.peek(0x0011), 0);
    assert!(cpu.flag(F_Z));
    assert!(cpu.flag(F_C));
    assert!(!cpu.flag(F_N));
    assert_eq!(cyc, 5);

    // ASL !abs: 1 -> 2
    let (cpu, bus, cyc) = run(|cpu, bus| {
        bus.poke(0x3000, &[1]);
    }, &[0x0C, 0x00, 0x30]);
    assert_eq!(bus.peek(0x3000), 2);
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert!(!cpu.flag(F_C));
    assert_eq!(cyc, 5);
}

#[test]
fn e06_19_lsr() {
    // LSR A: 0x01 -> 0, Z=1, C=1, N=0
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.a = 0x01;
    }, &[0x5C]);
    assert_eq!(cpu.a, 0);
    assert!(cpu.flag(F_Z));
    assert!(cpu.flag(F_C));
    assert!(!cpu.flag(F_N));
    assert_eq!(cyc, 2);

    // LSR A: 0x80 -> 0x40, C=0
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.a = 0x80;
    }, &[0x5C]);
    assert_eq!(cpu.a, 0x40);
    assert!(!cpu.flag(F_C));
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 2);

    // LSR dp: 0x03 -> 1, C=1
    let (cpu, bus, cyc) = run(|cpu, bus| {
        bus.poke(0x0010, &[0x03]);
    }, &[0x4B, 0x10]);
    assert_eq!(bus.peek(0x0010), 1);
    assert!(cpu.flag(F_C));
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 4);

    // LSR dp+X: 2 -> 1, C=0
    let (cpu, bus, cyc) = run(|cpu, bus| {
        cpu.x = 1;
        bus.poke(0x0011, &[2]);
    }, &[0x5B, 0x10]);
    assert_eq!(bus.peek(0x0011), 1);
    assert!(!cpu.flag(F_C));
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 5);

    // LSR !abs: 0x10 -> 0x08
    let (cpu, bus, cyc) = run(|cpu, bus| {
        bus.poke(0x3000, &[0x10]);
    }, &[0x4C, 0x00, 0x30]);
    assert_eq!(bus.peek(0x3000), 0x08);
    assert!(!cpu.flag(F_C));
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 5);
}

#[test]
fn e06_19_rol() {
    // ROL A with C=0: 0x80 -> 0, Z=1, C=1 (bit7 was set)
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.psw = 0; // C=0
        cpu.a = 0x80;
    }, &[0x3C]);
    assert_eq!(cpu.a, 0);
    assert!(cpu.flag(F_Z));
    assert!(cpu.flag(F_C));
    assert!(!cpu.flag(F_N));
    assert_eq!(cyc, 2);

    // ROL A with C=1: 0x00 -> 1, C=0, Z=0
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.psw = F_C; // C=1
        cpu.a = 0;
    }, &[0x3C]);
    assert_eq!(cpu.a, 1);
    assert!(!cpu.flag(F_C));
    assert!(!cpu.flag(F_Z));
    assert!(!cpu.flag(F_N));
    assert_eq!(cyc, 2);

    // ROL dp with C=1: 0x40 -> 0x81, N=1, C=0
    let (cpu, bus, cyc) = run(|cpu, bus| {
        cpu.psw = F_C; // C=1
        bus.poke(0x0010, &[0x40]);
    }, &[0x2B, 0x10]);
    assert_eq!(bus.peek(0x0010), 0x81);
    assert!(cpu.flag(F_N));
    assert!(!cpu.flag(F_C));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 4);

    // ROL dp+X with C=0: 2 -> 4
    let (cpu, bus, cyc) = run(|cpu, bus| {
        cpu.x = 1;
        cpu.psw = 0; // C=0
        bus.poke(0x0011, &[2]);
    }, &[0x3B, 0x10]);
    assert_eq!(bus.peek(0x0011), 4);
    assert!(!cpu.flag(F_C));
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 5);

    // ROL !abs with C=1: 1 -> 3
    let (cpu, bus, cyc) = run(|cpu, bus| {
        cpu.psw = F_C; // C=1
        bus.poke(0x3000, &[1]);
    }, &[0x2C, 0x00, 0x30]);
    assert_eq!(bus.peek(0x3000), 3);
    assert!(!cpu.flag(F_C));
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 5);
}

#[test]
fn e06_19_ror() {
    // ROR A with C=0: 0x01 -> 0, Z=1, C=1 (bit0 was set)
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.psw = 0; // C=0
        cpu.a = 0x01;
    }, &[0x7C]);
    assert_eq!(cpu.a, 0);
    assert!(cpu.flag(F_Z));
    assert!(cpu.flag(F_C));
    assert!(!cpu.flag(F_N));
    assert_eq!(cyc, 2);

    // ROR A with C=1: 0x00 -> 0x80, N=1, C=0
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.psw = F_C; // C=1
        cpu.a = 0;
    }, &[0x7C]);
    assert_eq!(cpu.a, 0x80);
    assert!(cpu.flag(F_N));
    assert!(!cpu.flag(F_C));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 2);

    // ROR dp with C=1: 0x02 -> 0x81, C=0 (bit0 was 0)
    let (cpu, bus, cyc) = run(|cpu, bus| {
        cpu.psw = F_C; // C=1
        bus.poke(0x0010, &[0x02]);
    }, &[0x6B, 0x10]);
    assert_eq!(bus.peek(0x0010), 0x81);
    assert!(!cpu.flag(F_C));
    assert!(cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 4);

    // ROR dp+X with C=0: 2 -> 1
    let (cpu, bus, cyc) = run(|cpu, bus| {
        cpu.x = 1;
        cpu.psw = 0; // C=0
        bus.poke(0x0011, &[2]);
    }, &[0x7B, 0x10]);
    assert_eq!(bus.peek(0x0011), 1);
    assert!(!cpu.flag(F_C));
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 5);

    // ROR !abs with C=1: 3 -> 0x81 (bit0 was 1)
    let (cpu, bus, cyc) = run(|cpu, bus| {
        cpu.psw = F_C; // C=1
        bus.poke(0x3000, &[3]);
    }, &[0x6C, 0x00, 0x30]);
    assert_eq!(bus.peek(0x3000), 0x81);
    assert!(cpu.flag(F_C));
    assert!(cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 5);
}

#[test]
fn e06_19_xcn() {
    // XCN A: swap nibbles
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.a = 0x1F;
    }, &[0x9F]);
    assert_eq!(cpu.a, 0xF1);
    assert!(cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 5);

    // XCN A: zero stays zero
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.a = 0;
    }, &[0x9F]);
    assert_eq!(cpu.a, 0);
    assert!(cpu.flag(F_Z));
    assert!(!cpu.flag(F_N));
    assert_eq!(cyc, 5);

    // XCN A: 0x0A -> 0xA0
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.a = 0x0A;
    }, &[0x9F]);
    assert_eq!(cpu.a, 0xA0);
    assert!(cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 5);
}

#[test]
fn e06_19_shift_cycles() {
    // Verify cycle counts for shift/rotate instructions
    let (_, _, cyc) = run(|cpu, _| { cpu.a = 1; }, &[0x1C]); // ASL A
    assert_eq!(cyc, 2);

    let (_, bus, cyc) = run(|cpu, bus| { bus.poke(0x0010, &[1]); }, &[0x0B, 0x10]); // ASL dp
    assert_eq!(cyc, 4);

    let (_, bus, cyc) = run(|cpu, bus| { cpu.x = 1; bus.poke(0x0011, &[1]); }, &[0x1B, 0x10]); // ASL dp+X
    assert_eq!(cyc, 5);

    let (_, bus, cyc) = run(|cpu, bus| { bus.poke(0x3000, &[1]); }, &[0x0C, 0x00, 0x30]); // ASL !abs
    assert_eq!(cyc, 5);

    let (_, _, cyc) = run(|cpu, _| { cpu.a = 1; }, &[0x7C]); // ROR A
    assert_eq!(cyc, 2);
}
