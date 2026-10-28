use puce16_core::apu::spc700::{Spc700, testbus::FlatSmpBus, F_C};

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
fn e06_16_adc_signed_overflow() {
    // 0x7F + 0x01 = 0x80: signed overflow, half-carry
    let (cpu, _, cyc) = run(|c, _| { c.a = 0x7F; }, &[0x88, 0x01]);
    assert_eq!(cpu.a, 0x80);
    assert!(cpu.flag(0x80)); // N
    assert!(cpu.flag(0x40)); // V
    assert!(cpu.flag(0x08)); // H
    assert!(!cpu.flag(0x01)); // C
    assert!(!cpu.flag(0x02)); // Z
    assert_eq!(cyc, 2);
}

#[test]
fn e06_16_adc_carry_out() {
    // 0x01 + 0xFF = 0x100: carry out, zero result
    let (cpu, _, cyc) = run(|c, _| { c.a = 0x01; }, &[0x88, 0xFF]);
    assert_eq!(cpu.a, 0x00);
    assert!(cpu.flag(0x02)); // Z
    assert!(cpu.flag(0x01)); // C
    assert!(cpu.flag(0x08)); // H
    assert!(!cpu.flag(0x40)); // V
    assert!(!cpu.flag(0x80)); // N
    assert_eq!(cyc, 2);
}

#[test]
fn e06_16_adc_carry_in() {
    // C=1, 0x00 + 0x01 + 1 = 2: no flags set
    let (cpu, _, cyc) = run(|c, _| { c.a = 0x00; c.psw = F_C; }, &[0x88, 0x01]);
    assert_eq!(cpu.a, 2);
    assert_eq!(cpu.psw & (0x80 | 0x40 | 0x08 | 0x01 | 0x02), 0);
    assert_eq!(cyc, 2);
}

#[test]
fn e06_16_adc_neg_neg() {
    // -128 + -128 = -256 -> 0 with carry and overflow
    let (cpu, _, cyc) = run(|c, _| { c.a = 0x80; }, &[0x88, 0x80]);
    assert_eq!(cpu.a, 0);
    assert!(cpu.flag(0x01)); // C
    assert!(cpu.flag(0x02)); // Z
    assert!(cpu.flag(0x40)); // V
    assert!(!cpu.flag(0x08)); // H
    assert!(!cpu.flag(0x80)); // N
    assert_eq!(cyc, 2);
}

#[test]
fn e06_16_sbc_borrow() {
    // C=0: 0x00 - 0x01 = 0xFF (borrow) -> SBC is ADC with !b, so result is 0xFE
    let (cpu, _, cyc) = run(|c, _| { c.a = 0x00; }, &[0xA8, 0x01]);
    assert_eq!(cpu.a, 0xFE);
    assert!(cpu.flag(0x80)); // N
    assert!(!cpu.flag(0x01)); // C
    assert!(!cpu.flag(0x02)); // Z
    assert!(!cpu.flag(0x40)); // V
    assert!(!cpu.flag(0x08)); // H
    assert_eq!(cyc, 2);
}

#[test]
fn e06_16_sbc_no_borrow() {
    // C=1: 0x05 - 0x01 = 0x04 (no borrow)
    let (cpu, _, cyc) = run(|c, _| { c.a = 0x05; c.psw = F_C; }, &[0xA8, 0x01]);
    assert_eq!(cpu.a, 0x04);
    assert!(cpu.flag(0x01)); // C
    assert!(cpu.flag(0x08)); // H
    assert!(!cpu.flag(0x80)); // N
    assert!(!cpu.flag(0x02)); // Z
    assert!(!cpu.flag(0x40)); // V
    assert_eq!(cyc, 2);
}

#[test]
fn e06_16_sbc_overflow() {
    // C=1: -128 - 1 = +127 (signed overflow)
    let (cpu, _, cyc) = run(|c, _| { c.a = 0x80; c.psw = F_C; }, &[0xA8, 0x01]);
    assert_eq!(cpu.a, 0x7F);
    assert!(cpu.flag(0x01)); // C
    assert!(cpu.flag(0x40)); // V
    assert!(!cpu.flag(0x80)); // N
    assert!(!cpu.flag(0x02)); // Z
    assert!(!cpu.flag(0x08)); // H
    assert_eq!(cyc, 2);
}

#[test]
fn e06_16_adc_dp() {
    // ADC A,dp: a=0x20 + mem[0x10]=0x10 = 0x30
    let (cpu, _, cyc) = run(|c, b| { c.a = 0x20; b.poke(0x10, &[0x10]); }, &[0x84, 0x10]);
    assert_eq!(cpu.a, 0x30);
    assert_eq!(cyc, 3);
}

#[test]
fn e06_16_adc_dp_dp() {
    // ADC dp,dp: mem[0x20] = mem[0x20](0x02) + mem[0x10](0x01) = 0x03
    let (_cpu, bus, cyc) = run(|_c, b| {
        b.poke(0x10, &[0x01]);
        b.poke(0x20, &[0x02]);
    }, &[0x89, 0x10, 0x20]);
    assert_eq!(bus.peek(0x20), 0x03);
    assert_eq!(cyc, 6);
}

#[test]
fn e06_16_adc_dp_imm() {
    // ADC dp,#imm: mem[0x10] = mem[0x10](0x0A) + 0x05 = 0x0F
    let (_cpu, bus, cyc) = run(|_c, b| {
        b.poke(0x10, &[0x0A]);
    }, &[0x98, 0x05, 0x10]);
    assert_eq!(bus.peek(0x10), 0x0F);
    assert_eq!(cyc, 5);
}

#[test]
fn e06_16_adc_xy() {
    // ADC (X),(Y): mem[X]=mem[X](0x01)+mem[Y](0x01)=0x02
    let (_cpu, bus, cyc) = run(|c, b| {
        c.x = 0x10;
        c.y = 0x11;
        b.poke(0x10, &[0x01]);
        b.poke(0x11, &[0x01]);
    }, &[0x99]);
    assert_eq!(bus.peek(0x10), 0x02);
    assert_eq!(cyc, 5);
}

#[test]
fn e06_16_sbc_dp_dp() {
    // SBC dp,dp: mem[0x20] = mem[0x20](0x05) - mem[0x10](0x01) = 0x04 (C=1 means no borrow)
    let (cpu, bus, cyc) = run(|c, b| {
        c.psw = F_C;
        b.poke(0x10, &[0x01]);
        b.poke(0x20, &[0x05]);
    }, &[0xA9, 0x10, 0x20]);
    assert_eq!(bus.peek(0x20), 0x04);
    assert!(cpu.flag(F_C));
    assert_eq!(cyc, 6);
}

#[test]
fn e06_16_sbc_xy() {
    // SBC (X),(Y): mem[X]=mem[X](0x03)-mem[Y](0x01)=0x02
    let (_cpu, bus, cyc) = run(|c, b| {
        c.psw = F_C;
        c.x = 0x10;
        c.y = 0x11;
        b.poke(0x10, &[0x03]);
        b.poke(0x11, &[0x01]);
    }, &[0xB9]);
    assert_eq!(bus.peek(0x10), 0x02);
    assert_eq!(cyc, 5);
}

#[test]
fn e06_16_adc_abs() {
    let (cpu, _, cyc) = run(|c, b| { c.a = 0; b.poke(0x3000, &[0x11]); }, &[0x85, 0x00, 0x30]);
    assert_eq!(cpu.a, 0x11);
    assert_eq!(cyc, 4);
}

#[test]
fn e06_16_adc_x() {
    let (cpu, _, cyc) = run(|c, b| { c.x = 0x10; b.poke(0x10, &[0x11]); }, &[0x86]);
    assert_eq!(cpu.a, 0x11);
    assert_eq!(cyc, 3);
}

#[test]
fn e06_16_adc_dpx_ind() {
    let (cpu, _, cyc) = run(|c, b| {
        c.x = 2;
        b.poke(0x12, &[0x00, 0x30]);
        b.poke(0x3000, &[0x11]);
    }, &[0x87, 0x10]);
    assert_eq!(cpu.a, 0x11);
    assert_eq!(cyc, 6);
}

#[test]
fn e06_16_adc_dpx() {
    let (cpu, _, cyc) = run(|c, b| { c.x = 1; b.poke(0x11, &[0x11]); }, &[0x94, 0x10]);
    assert_eq!(cpu.a, 0x11);
    assert_eq!(cyc, 4);
}

#[test]
fn e06_16_adc_absx() {
    let (cpu, _, cyc) = run(|c, b| { c.x = 1; b.poke(0x3000, &[0x11]); }, &[0x95, 0xFF, 0x2F]);
    assert_eq!(cpu.a, 0x11);
    assert_eq!(cyc, 5);
}

#[test]
fn e06_16_adc_absy() {
    let (cpu, _, cyc) = run(|c, b| { c.y = 2; b.poke(0x3002, &[0x11]); }, &[0x96, 0x00, 0x30]);
    assert_eq!(cpu.a, 0x11);
    assert_eq!(cyc, 5);
}

#[test]
fn e06_16_adc_dp_ind_y() {
    let (cpu, _, cyc) = run(|c, b| {
        c.y = 1;
        b.poke(0x20, &[0xFF, 0x2F]);
        b.poke(0x3000, &[0x11]);
    }, &[0x97, 0x20]);
    assert_eq!(cpu.a, 0x11);
    assert_eq!(cyc, 6);
}

#[test]
fn e06_16_sbc_abs() {
    let (cpu, _, cyc) = run(|c, b| { c.psw = F_C; c.a = 0x20; b.poke(0x3000, &[0x11]); }, &[0xA5, 0x00, 0x30]);
    assert_eq!(cpu.a, 0x0F);
    assert_eq!(cyc, 4);
}

#[test]
fn e06_16_sbc_x() {
    let (cpu, _, cyc) = run(|c, b| { c.psw = F_C; c.a = 0x20; c.x = 0x10; b.poke(0x10, &[0x11]); }, &[0xA6]);
    assert_eq!(cpu.a, 0x0F);
    assert_eq!(cyc, 3);
}

#[test]
fn e06_16_sbc_dpx_ind() {
    let (cpu, _, cyc) = run(|c, b| {
        c.psw = F_C; c.a = 0x20; c.x = 2;
        b.poke(0x12, &[0x00, 0x30]);
        b.poke(0x3000, &[0x11]);
    }, &[0xA7, 0x10]);
    assert_eq!(cpu.a, 0x0F);
    assert_eq!(cyc, 6);
}

#[test]
fn e06_16_sbc_dpx() {
    let (cpu, _, cyc) = run(|c, b| { c.psw = F_C; c.a = 0x20; c.x = 1; b.poke(0x11, &[0x11]); }, &[0xB4, 0x10]);
    assert_eq!(cpu.a, 0x0F);
    assert_eq!(cyc, 4);
}

#[test]
fn e06_16_sbc_absx() {
    let (cpu, _, cyc) = run(|c, b| { c.psw = F_C; c.a = 0x20; c.x = 1; b.poke(0x3000, &[0x11]); }, &[0xB5, 0xFF, 0x2F]);
    assert_eq!(cpu.a, 0x0F);
    assert_eq!(cyc, 5);
}

#[test]
fn e06_16_sbc_absy() {
    let (cpu, _, cyc) = run(|c, b| { c.psw = F_C; c.a = 0x20; c.y = 2; b.poke(0x3002, &[0x11]); }, &[0xB6, 0x00, 0x30]);
    assert_eq!(cpu.a, 0x0F);
    assert_eq!(cyc, 5);
}

#[test]
fn e06_16_sbc_dp_ind_y() {
    let (cpu, _, cyc) = run(|c, b| {
        c.psw = F_C; c.a = 0x20; c.y = 1;
        b.poke(0x20, &[0xFF, 0x2F]);
        b.poke(0x3000, &[0x11]);
    }, &[0xB7, 0x20]);
    assert_eq!(cpu.a, 0x0F);
    assert_eq!(cyc, 6);
}

struct Form {
    bytes: Vec<u8>,
    x: u8,
    y: u8,
    pokes: Vec<(u16, Vec<u8>)>,
    cycles: u32,
}

// The value 0x11 sits at the effective address of every form.
fn forms(ops: [u8; 7]) -> Vec<Form> {
    vec![
        Form { bytes: vec![ops[0], 0x00, 0x30], x: 0, y: 0, pokes: vec![(0x3000, vec![0x11])], cycles: 4 },
        Form { bytes: vec![ops[1]], x: 0x10, y: 0, pokes: vec![(0x0010, vec![0x11])], cycles: 3 },
        Form { bytes: vec![ops[2], 0x10], x: 2, y: 0, pokes: vec![(0x0012, vec![0x00, 0x30]), (0x3000, vec![0x11])], cycles: 6 },
        Form { bytes: vec![ops[3], 0x10], x: 1, y: 0, pokes: vec![(0x0011, vec![0x11])], cycles: 4 },
        Form { bytes: vec![ops[4], 0xFF, 0x2F], x: 1, y: 0, pokes: vec![(0x3000, vec![0x11])], cycles: 5 },
        Form { bytes: vec![ops[5], 0x00, 0x30], x: 0, y: 2, pokes: vec![(0x3002, vec![0x11])], cycles: 5 },
        Form { bytes: vec![ops[6], 0x20], x: 0, y: 1, pokes: vec![(0x0020, vec![0xFF, 0x2F]), (0x3000, vec![0x11])], cycles: 6 },
    ]
}

fn run_form(f: &Form, psw: u8, a: u8) -> (Spc700, u32) {
    let (cpu, _, cyc) = run(|c, b| {
        c.psw = psw;
        c.a = a;
        c.x = f.x;
        c.y = f.y;
        for (addr, data) in &f.pokes {
            b.poke(*addr, data);
        }
    }, &f.bytes);
    (cpu, cyc)
}

#[test]
fn e06_16_adc_forms_cycles() {
    for f in forms([0x85, 0x86, 0x87, 0x94, 0x95, 0x96, 0x97]) {
        let (cpu, cyc) = run_form(&f, 0, 0);
        assert_eq!(cpu.a, 0x11, "ADC form {:02X}", f.bytes[0]);
        assert_eq!(cyc, f.cycles, "ADC form {:02X} cycles", f.bytes[0]);
    }
}

#[test]
fn e06_16_sbc_forms_cycles() {
    for f in forms([0xA5, 0xA6, 0xA7, 0xB4, 0xB5, 0xB6, 0xB7]) {
        let (cpu, cyc) = run_form(&f, F_C, 0x20);
        assert_eq!(cpu.a, 0x0F, "SBC form {:02X}", f.bytes[0]);
        assert_eq!(cyc, f.cycles, "SBC form {:02X} cycles", f.bytes[0]);
    }
}
