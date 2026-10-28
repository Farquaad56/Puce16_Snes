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
fn e06_20_daa_low_nibble() {
    // DAA A: low nibble > 9 triggers +6 adjustment
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.psw = 0;
        cpu.a = 0x0A;
    }, &[0xDF]);
    assert_eq!(cpu.a, 0x10);
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 3);
}

#[test]
fn e06_20_daa_carry_out() {
    // DAA A: high nibble > 9 triggers +0x60 and sets C
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.psw = 0;
        cpu.a = 0x9A;
    }, &[0xDF]);
    assert_eq!(cpu.a, 0x00);
    assert!(cpu.flag(F_C));
    assert!(cpu.flag(F_Z));
    assert_eq!(cyc, 3);
}

#[test]
fn e06_20_daa_half_carry() {
    // DAA A: H flag set triggers +6 adjustment even if low nibble <= 9
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.psw = F_H;
        cpu.a = 0x05;
    }, &[0xDF]);
    assert_eq!(cpu.a, 0x0B);
    // H flag is not cleared by DAA (unchanged per spec)
    assert!(cpu.flag(F_H));
    assert_eq!(cyc, 3);
}

#[test]
fn e06_20_daa_carry_in() {
    // DAA A: C flag set triggers +0x60 adjustment
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.psw = F_C;
        cpu.a = 0x12;
    }, &[0xDF]);
    assert_eq!(cpu.a, 0x72);
    assert!(cpu.flag(F_C));
    assert_eq!(cyc, 3);
}

#[test]
fn e06_20_das_no_adjust() {
    // DAS A: C=1 and H=1 with valid BCD -> no adjustment
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.psw = F_C | F_H;
        cpu.a = 0x25;
    }, &[0xBE]);
    assert_eq!(cpu.a, 0x25);
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 3);
}

#[test]
fn e06_20_das_borrow() {
    // DAS A: C=0 triggers -0x60 adjustment (borrow)
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.psw = F_H; // C=0, H=1
        cpu.a = 0x25;
    }, &[0xBE]);
    assert_eq!(cpu.a, 0xC5);
    assert!(cpu.flag(F_N));
    assert!(!cpu.flag(F_C));
    assert_eq!(cyc, 3);
}

#[test]
fn e06_20_das_low() {
    // DAS A: H=0 triggers -6 adjustment on low nibble
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.psw = F_C; // C=1, H=0
        cpu.a = 0x00;
    }, &[0xBE]);
    assert_eq!(cpu.a, 0xFA);
    assert!(cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 3);
}

#[test]
fn e06_20_mul() {
    // MUL YA: 16x16 -> 16-bit result in Y:A
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.y = 0x10;
        cpu.a = 0x10;
    }, &[0xCF]);
    assert_eq!(cpu.a, 0x00);
    assert_eq!(cpu.y, 0x01);
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 9);

    // MUL YA: max values -> overflow into Y
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.y = 0xFF;
        cpu.a = 0xFF;
    }, &[0xCF]);
    assert_eq!(cpu.a, 0x01);
    assert_eq!(cpu.y, 0xFE);
    assert!(cpu.flag(F_N)); // Y has bit7 set
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 9);

    // MUL YA: zero result
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.y = 0;
        cpu.a = 5;
    }, &[0xCF]);
    assert_eq!(cpu.a, 0);
    assert_eq!(cpu.y, 0);
    assert!(cpu.flag(F_Z)); // Z set based on Y
    assert!(!cpu.flag(F_N));
    assert_eq!(cyc, 9);
}

#[test]
fn e06_20_div_normal() {
    // DIV YA,X: normal division (Y < X*2)
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.y = 0;
        cpu.a = 0x64; // 100 decimal
        cpu.x = 0x0A; // 10 decimal
    }, &[0x9E]);
    assert_eq!(cpu.a, 10);   // quotient
    assert_eq!(cpu.y, 0);   // remainder
    assert!(!cpu.flag(F_V));
    assert!(!cpu.flag(F_H));
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 12);
}

#[test]
fn e06_20_div_remainder() {
    // DIV YA,X: division with remainder
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.y = 1;
        cpu.a = 0x05; // ya = 0x0105 = 261 decimal
        cpu.x = 2;
    }, &[0x9E]);
    assert_eq!(cpu.a, 0x82);   // quotient (truncated to 8 bits)
    assert_eq!(cpu.y, 1);     // remainder
    assert!(!cpu.flag(F_V));
    assert!(!cpu.flag(F_H));
    assert!(cpu.flag(F_N));   // A has bit7 set
    assert_eq!(cyc, 12);
}

#[test]
fn e06_20_div_overflow_branch() {
    // DIV YA,X: overflow branch (Y >= X*2)
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.y = 2;
        cpu.a = 0;
        cpu.x = 1;
    }, &[0x9E]);
    assert_eq!(cpu.a, 0xFF);   // complement result
    assert_eq!(cpu.y, 1);     // remainder
    assert!(cpu.flag(F_V));   // Y >= X
    assert!(cpu.flag(F_H));   // (Y & 15) >= (X & 15)
    assert!(cpu.flag(F_N));   // A has bit7 set
    assert_eq!(cyc, 12);
}

#[test]
fn e06_20_div_by_zero_no_panic() {
    // DIV YA,X: division by zero must not panic
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.y = 0x12;
        cpu.a = 0x34;
        cpu.x = 0;
    }, &[0x9E]);
    // With x=0, first branch never taken (y < 0 is false)
    // Uses complement arithmetic: a = 255 - ya/256, y = ya % 256
    assert_eq!(cpu.a, 0xED);   // 255 - 18 = 237 = 0xED
    assert_eq!(cpu.y, 0x34);   // remainder (unchanged)
    assert!(cpu.flag(F_V));   // Y >= X (0x12 >= 0)
    assert!(cpu.flag(F_H));   // (Y & 15) >= (X & 15) (4 >= 0)
    assert!(cpu.flag(F_N));   // A has bit7 set
    assert_eq!(cyc, 12);
}
