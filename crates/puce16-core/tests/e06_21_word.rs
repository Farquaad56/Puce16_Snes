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
fn e06_21_addw_basic() {
    // ADDW YA,dp: basic addition with carry from low to high byte
    let (cpu, _, cyc) = run(|cpu, bus| {
        cpu.psw = 0;
        cpu.y = 0x01;
        cpu.a = 0xFF;
        bus.poke(0x10, &[0x01, 0x00]); // word value 0x0001
    }, &[0x7A, 0x10]);
    assert_eq!(cpu.y, 0x02);
    assert_eq!(cpu.a, 0x00);
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert!(!cpu.flag(F_C));
    assert!(!cpu.flag(F_V));
    assert!(!cpu.flag(F_H));
    assert_eq!(cyc, 5);
}

#[test]
fn e06_21_addw_signed_overflow() {
    // ADDW YA,dp: signed overflow (result crosses sign boundary)
    let (cpu, _, cyc) = run(|cpu, bus| {
        cpu.psw = 0;
        cpu.y = 0x7F;
        cpu.a = 0xFF;
        bus.poke(0x10, &[0x01, 0x00]); // word value 0x0001
    }, &[0x7A, 0x10]);
    assert_eq!(cpu.ya(), 0x8000);
    assert!(cpu.flag(F_N));
    assert!(cpu.flag(F_V));
    assert!(cpu.flag(F_H));
    assert!(!cpu.flag(F_C));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 5);
}

#[test]
fn e06_21_addw_carry_zero() {
    // ADDW YA,dp: result wraps to zero with carry out
    let (cpu, _, cyc) = run(|cpu, bus| {
        cpu.psw = 0;
        cpu.y = 0xFF;
        cpu.a = 0xFF;
        bus.poke(0x10, &[0x01, 0x00]); // word value 0x0001
    }, &[0x7A, 0x10]);
    assert_eq!(cpu.ya(), 0x0000);
    assert!(cpu.flag(F_Z));
    assert!(cpu.flag(F_C));
    assert!(cpu.flag(F_H));
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_V));
    assert_eq!(cyc, 5);
}

#[test]
fn e06_21_addw_ignores_carry_in() {
    // ADDW YA,dp: incoming carry flag is ignored (CLRC implicit)
    let (cpu, _, cyc) = run(|cpu, bus| {
        cpu.psw = F_C;
        cpu.y = 0;
        cpu.a = 0;
        bus.poke(0x10, &[0, 0]); // word value 0x0000
    }, &[0x7A, 0x10]);
    assert_eq!(cpu.ya(), 0);
    assert!(cpu.flag(F_Z));
    assert!(!cpu.flag(F_C));
    assert_eq!(cyc, 5);
}

#[test]
fn e06_21_subw_basic() {
    // SUBW YA,dp: basic subtraction with borrow from high byte
    let (cpu, _, cyc) = run(|cpu, bus| {
        cpu.psw = 0;
        cpu.y = 0x02;
        cpu.a = 0x00;
        bus.poke(0x10, &[0x01, 0x00]); // word value 0x0001
    }, &[0x9A, 0x10]);
    assert_eq!(cpu.ya(), 0x01FF);
    assert!(cpu.flag(F_C));
    assert!(cpu.flag(F_H));
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert!(!cpu.flag(F_V));
    assert_eq!(cyc, 5);
}

#[test]
fn e06_21_subw_zero() {
    // SUBW YA,dp: subtract equal values -> zero result
    let (cpu, _, cyc) = run(|cpu, bus| {
        cpu.psw = 0;
        cpu.y = 0x12;
        cpu.a = 0x34;
        bus.poke(0x10, &[0x34, 0x12]); // word value 0x1234
    }, &[0x9A, 0x10]);
    assert_eq!(cpu.ya(), 0);
    assert!(cpu.flag(F_Z));
    assert!(cpu.flag(F_C));
    assert!(cpu.flag(F_H));
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_V));
    assert_eq!(cyc, 5);
}

#[test]
fn e06_21_cmpw_equal() {
    // CMPW YA,dp: compare equal values -> Z=1, C=1, N=0
    let (cpu, _, cyc) = run(|cpu, bus| {
        cpu.psw = 0;
        cpu.y = 0x12;
        cpu.a = 0x34;
        bus.poke(0x10, &[0x34, 0x12]); // word value 0x1234
    }, &[0x5A, 0x10]);
    assert!(cpu.flag(F_Z));
    assert!(cpu.flag(F_C));
    assert!(!cpu.flag(F_N));
    assert_eq!(cpu.ya(), 0x1234); // YA unchanged
    assert_eq!(cyc, 4);
}

#[test]
fn e06_21_cmpw_less() {
    // CMPW YA,dp: compare less than -> C=0, N=1; greater than -> C=1, N=0
    let (cpu, _, _) = run(|cpu, bus| {
        cpu.psw = 0;
        cpu.y = 0x12;
        cpu.a = 0x34;
        bus.poke(0x10, &[0x35, 0x12]); // word value 0x1235 (greater)
    }, &[0x5A, 0x10]);
    assert!(!cpu.flag(F_C));
    assert!(cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));

    let (cpu, _, _) = run(|cpu, bus| {
        cpu.psw = 0;
        cpu.y = 0x12;
        cpu.a = 0x34;
        bus.poke(0x10, &[0x33, 0x12]); // word value 0x1233 (less)
    }, &[0x5A, 0x10]);
    assert!(cpu.flag(F_C));
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
}

#[test]
fn e06_21_cmpw_keeps_v_h() {
    // CMPW YA,dp: V and H flags are unchanged by comparison
    let (cpu, _, _) = run(|cpu, bus| {
        cpu.psw = F_V | F_H;
        cpu.y = 0x12;
        cpu.a = 0x34;
        bus.poke(0x10, &[0x34, 0x12]); // word value 0x1234 (equal)
    }, &[0x5A, 0x10]);
    assert!(cpu.flag(F_V));
    assert!(cpu.flag(F_H));
}

#[test]
fn e06_21_incw() {
    // INCW dp: increment word in memory with carry propagation
    let (cpu, bus, cyc) = run(|cpu, bus| {
        cpu.psw = 0;
        bus.poke(0x10, &[0xFF, 0x00]); // word value 0x00FF
    }, &[0x3A, 0x10]);
    assert_eq!(bus.peek(0x10), 0x00);
    assert_eq!(bus.peek(0x11), 0x01);
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 6);

    // INCW dp: word wraps from 0xFFFF to 0x0000 -> Z=1
    let (cpu, bus, _) = run(|cpu, bus| {
        cpu.psw = 0;
        bus.poke(0x10, &[0xFF, 0xFF]); // word value 0xFFFF
    }, &[0x3A, 0x10]);
    assert_eq!(bus.peek(0x10), 0x00);
    assert_eq!(bus.peek(0x11), 0x00);
    assert!(cpu.flag(F_Z));

    // INCW dp: word crosses sign boundary -> N=1
    let (cpu, bus, _) = run(|cpu, bus| {
        cpu.psw = 0;
        bus.poke(0x10, &[0xFF, 0x7F]); // word value 0x7FFF
    }, &[0x3A, 0x10]);
    assert_eq!(bus.peek(0x10), 0x00);
    assert_eq!(bus.peek(0x11), 0x80);
    assert!(cpu.flag(F_N));
}

#[test]
fn e06_21_decw() {
    // DECW dp: decrement word in memory with borrow propagation
    let (cpu, bus, cyc) = run(|cpu, bus| {
        cpu.psw = 0;
        bus.poke(0x10, &[0x00, 0x01]); // word value 0x0100
    }, &[0x1A, 0x10]);
    assert_eq!(bus.peek(0x10), 0xFF);
    assert_eq!(bus.peek(0x11), 0x00);
    assert_eq!(cyc, 6);

    // DECW dp: word wraps from 0x0000 to 0xFFFF -> N=1
    let (cpu, bus, _) = run(|cpu, bus| {
        cpu.psw = 0;
        bus.poke(0x10, &[0x00, 0x00]); // word value 0x0000
    }, &[0x1A, 0x10]);
    assert_eq!(bus.peek(0x10), 0xFF);
    assert_eq!(bus.peek(0x11), 0xFF);
    assert!(cpu.flag(F_N));

    // DECW dp: word decrements to zero -> Z=1
    let (cpu, bus, _) = run(|cpu, bus| {
        cpu.psw = 0;
        bus.poke(0x10, &[0x01, 0x00]); // word value 0x0001
    }, &[0x1A, 0x10]);
    assert_eq!(bus.peek(0x10), 0x00);
    assert_eq!(bus.peek(0x11), 0x00);
    assert!(cpu.flag(F_Z));
}

#[test]
fn e06_21_page_wrap() {
    // INCW dp: high byte wraps within direct page (b=0xFF -> b+1=0x00)
    let (cpu, bus, _) = run(|cpu, bus| {
        cpu.psw = 0;
        bus.poke(0x00FF, &[0xFF]); // low byte at 0xFF
        bus.poke(0x0000, &[0x00]); // high byte wraps to 0x00
    }, &[0x3A, 0xFF]);
    assert_eq!(bus.peek(0x00FF), 0x00);
    assert_eq!(bus.peek(0x0000), 0x01);
}

#[test]
fn e06_21_p_flag() {
    // INCW dp: P flag selects direct page base (P=1 -> 0x0100)
    let (cpu, bus, _) = run(|cpu, bus| {
        cpu.psw = F_P;
        bus.poke(0x0110, &[0x01, 0x00]); // word at page 1
    }, &[0x3A, 0x10]);
    assert_eq!(bus.peek(0x0110), 0x02);
}
