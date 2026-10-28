use puce16_core::apu::spc700::{Spc700, testbus::FlatSmpBus, F_C, F_Z, F_I, F_H, F_B, F_P, F_V, F_N};

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
fn e06_12_dp_with_dummy_read() {
    let (_cpu, bus, cyc) = run(|cpu, _| {
        cpu.a = 0x5A;
    }, &[0xC4, 0x10]);
    assert_eq!(bus.peek(0x0010), 0x5A);
    assert_eq!(cyc, 4);
    // Log: Read opcode (0x0200), Read operand (0x0201), dummy read dest (0x0010), write dest (0x0010)
    assert_eq!(bus.log.len(), 4);
    assert_eq!(bus.log[0].addr, 0x0200); // opcode fetch
    assert_eq!(bus.log[1].addr, 0x0201); // operand byte
    assert_eq!(bus.log[2].addr, 0x0010); // dummy read of destination
    assert_eq!(bus.log[3].addr, 0x0010); // actual write
}

#[test]
fn e06_12_abs() {
    let (_cpu, bus, cyc) = run(|cpu, _| {
        cpu.a = 0x11;
    }, &[0xC5, 0x00, 0x30]);
    assert_eq!(bus.peek(0x3000), 0x11);
    assert_eq!(cyc, 5);
}

#[test]
fn e06_12_x_indirect() {
    let (_cpu, bus, cyc) = run(|cpu, _| {
        cpu.x = 0x20;
        cpu.a = 0x22;
    }, &[0xC6]);
    assert_eq!(bus.peek(0x0020), 0x22);
    assert_eq!(cyc, 4);
}

#[test]
fn e06_12_dpx_indirect() {
    let (_cpu, bus, cyc) = run(|cpu, bus| {
        cpu.x = 2;
        cpu.a = 0x33;
        bus.poke(0x0012, &[0x00]);
        bus.poke(0x0013, &[0x30]);
    }, &[0xC7, 0x10]);
    assert_eq!(bus.peek(0x3000), 0x33);
    assert_eq!(cyc, 7);
}

#[test]
fn e06_12_dpx() {
    let (_cpu, bus, cyc) = run(|cpu, _| {
        cpu.x = 1;
        cpu.a = 0x44;
    }, &[0xD4, 0x10]);
    assert_eq!(bus.peek(0x0011), 0x44);
    assert_eq!(cyc, 5);
}

#[test]
fn e06_12_absx_absy() {
    // MOV !abs+X, A
    let (_cpu, bus, cyc) = run(|cpu, _| {
        cpu.x = 1;
        cpu.a = 0x55;
    }, &[0xD5, 0xFF, 0x2F]);
    assert_eq!(bus.peek(0x3000), 0x55);
    assert_eq!(cyc, 6);

    // MOV !abs+Y, A
    let (_cpu, bus, cyc) = run(|cpu, _| {
        cpu.y = 2;
        cpu.a = 0x66;
    }, &[0xD6, 0x00, 0x30]);
    assert_eq!(bus.peek(0x3002), 0x66);
    assert_eq!(cyc, 6);
}

#[test]
fn e06_12_dp_indirect_y() {
    let (_cpu, bus, cyc) = run(|cpu, bus| {
        cpu.y = 1;
        cpu.a = 0x77;
        bus.poke(0x0020, &[0xFF]);
        bus.poke(0x0021, &[0x2F]);
    }, &[0xD7, 0x20]);
    assert_eq!(bus.peek(0x3000), 0x77);
    assert_eq!(cyc, 7);
}

#[test]
fn e06_12_from_x() {
    // MOV !abs, X
    let (_cpu, bus, cyc) = run(|cpu, _| {
        cpu.x = 0x88;
    }, &[0xC9, 0x00, 0x30]);
    assert_eq!(bus.peek(0x3000), 0x88);
    assert_eq!(cyc, 5);

    // MOV dp, X
    let (_cpu, bus, cyc) = run(|cpu, _| {
        cpu.x = 0x88;
    }, &[0xD8, 0x10]);
    assert_eq!(bus.peek(0x0010), 0x88);
    assert_eq!(cyc, 4);

    // MOV dp+Y, X (wrap)
    let (_cpu, bus, cyc) = run(|cpu, _| {
        cpu.x = 0x88;
        cpu.y = 3;
    }, &[0xD9, 0xFE]);
    assert_eq!(bus.peek(0x0001), 0x88);
    assert_eq!(cyc, 5);
}

#[test]
fn e06_12_from_y() {
    // MOV dp, Y
    let (_cpu, bus, cyc) = run(|cpu, _| {
        cpu.y = 0x99;
    }, &[0xCB, 0x10]);
    assert_eq!(bus.peek(0x0010), 0x99);
    assert_eq!(cyc, 4);

    // MOV !abs, Y
    let (_cpu, bus, cyc) = run(|cpu, _| {
        cpu.y = 0x99;
    }, &[0xCC, 0x00, 0x30]);
    assert_eq!(bus.peek(0x3000), 0x99);
    assert_eq!(cyc, 5);

    // MOV dp+X, Y
    let (_cpu, bus, cyc) = run(|cpu, _| {
        cpu.y = 0x99;
        cpu.x = 1;
    }, &[0xDB, 0x10]);
    assert_eq!(bus.peek(0x0011), 0x99);
    assert_eq!(cyc, 5);
}

#[test]
fn e06_12_no_flags() {
    let (cpu, _, _) = run(|cpu, _| {
        cpu.psw = F_C | F_Z | F_I | F_H | F_B | F_P | F_V | F_N; // 0xFF - all flags set
        cpu.a = 0x00;
    }, &[0xC4, 0x10]);
    assert_eq!(cpu.psw, 0xFF); // unchanged
}

#[test]
fn e06_12_dummy_read_logged() {
    let (_cpu, bus, _cyc) = run(|cpu, _| {
        cpu.x = 0x20;
        cpu.a = 0x22;
    }, &[0xC6]);
    // The first access to 0x0020 must be the dummy read, before the write.
    let first = bus.log.iter().find(|c| c.addr == 0x0020);
    match first {
        Some(c) => assert_eq!(
            c.kind,
            puce16_core::apu::spc700::testbus::CycleKind::Read,
            "dummy read of destination not found before the write"
        ),
        None => panic!("no access to 0x0020 in log"),
    }
}
