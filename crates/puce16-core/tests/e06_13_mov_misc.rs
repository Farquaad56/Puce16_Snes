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
fn e06_13_reg_transfers() {
    // MOV X,A (5D) - sets N/Z
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.a = 0x80;
    }, &[0x5D]);
    assert_eq!(cpu.x, 0x80);
    assert!(cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 2);

    // MOV A,X (7D) - sets N/Z
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.x = 0;
    }, &[0x7D]);
    assert_eq!(cpu.a, 0);
    assert!(cpu.flag(F_Z));
    assert!(!cpu.flag(F_N));
    assert_eq!(cyc, 2);

    // MOV A,Y (DD) - sets N/Z
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.y = 0x7F;
    }, &[0xDD]);
    assert_eq!(cpu.a, 0x7F);
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 2);

    // MOV Y,A (FD) - sets N/Z
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.a = 0x01;
    }, &[0xFD]);
    assert_eq!(cpu.y, 0x01);
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 2);

    // MOV X,SP (9D) - sets N/Z
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.sp = 0xEF;
    }, &[0x9D]);
    assert_eq!(cpu.x, 0xEF);
    assert!(cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 2);
}

#[test]
fn e06_13_mov_sp_x_no_flags() {
    let (cpu, _, cyc) = run(|cpu, _| {
        cpu.x = 0x30;
        cpu.psw = 0x00; // clear all flags
    }, &[0xBD]);
    assert_eq!(cpu.sp, 0x30);
    assert_eq!(cpu.psw, 0x00); // unchanged
    assert_eq!(cyc, 2);
}

#[test]
fn e06_13_dp_imm() {
    let (cpu, bus, cyc) = run(|_, _| {}, &[0x8F, 0x5A, 0x10]);
    assert_eq!(bus.peek(0x0010), 0x5A);
    assert_eq!(cyc, 5);
    // Verify dummy read of destination before write
    let mut found_dummy = false;
    for cycle in &bus.log {
        if cycle.addr == 0x0010 && cycle.kind == puce16_core::apu::spc700::testbus::CycleKind::Read {
            found_dummy = true;
            break;
        }
    }
    assert!(found_dummy, "dummy read of destination not found");
}

#[test]
fn e06_13_dp_dp() {
    let (cpu, bus, cyc) = run(|_, bus| {
        bus.poke(0x0010, &[0x77]);
    }, &[0xFA, 0x10, 0x20]);
    assert_eq!(bus.peek(0x0020), 0x77);
    assert_eq!(cyc, 5);
    // Verify: read from source (0x0010), write to dest (0x0020), NO dummy read of dest
    let mut found_src_read = false;
    let mut found_dst_write = false;
    for cycle in &bus.log {
        if cycle.addr == 0x0010 && cycle.kind == puce16_core::apu::spc700::testbus::CycleKind::Read {
            found_src_read = true;
        }
        if cycle.addr == 0x0020 && cycle.kind == puce16_core::apu::spc700::testbus::CycleKind::Write {
            found_dst_write = true;
        }
    }
    assert!(found_src_read, "read from source not found");
    assert!(found_dst_write, "write to destination not found");
}

#[test]
fn e06_13_a_x_postinc() {
    let (cpu, bus, cyc) = run(|cpu, bus| {
        cpu.x = 0x10;
        bus.poke(0x0010, &[0x81]);
    }, &[0xBF]);
    assert_eq!(cpu.a, 0x81);
    assert!(cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cpu.x, 0x11);
    assert_eq!(cyc, 4);
}

#[test]
fn e06_13_x_postinc_store() {
    let (cpu, bus, cyc) = run(|cpu, _| {
        cpu.x = 0x10;
        cpu.a = 0x33;
    }, &[0xAF]);
    assert_eq!(bus.peek(0x0010), 0x33);
    assert_eq!(cpu.x, 0x11);
    assert_eq!(cyc, 4);
}

#[test]
fn e06_13_movw_load() {
    // Normal case: load word into Y:A
    let (cpu, bus, cyc) = run(|_, bus| {
        bus.poke(0x0010, &[0x34]);
        bus.poke(0x0011, &[0x12]);
    }, &[0xBA, 0x10]);
    assert_eq!(cpu.y, 0x12);
    assert_eq!(cpu.a, 0x34);
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    assert_eq!(cyc, 5);

    // Z flag: load zero word
    let (cpu, bus, cyc) = run(|_, bus| {
        bus.poke(0x0010, &[0]);
        bus.poke(0x0011, &[0]);
    }, &[0xBA, 0x10]);
    assert!(cpu.flag(F_Z));

    // N flag: load word with bit 15 set
    let (cpu, bus, cyc) = run(|_, bus| {
        bus.poke(0x0010, &[0x00]);
        bus.poke(0x0011, &[0x80]);
    }, &[0xBA, 0x10]);
    assert!(cpu.flag(F_N));
}

#[test]
fn e06_13_movw_load_wrap() {
    // Wrap at 0xFF -> 0x00
    let (cpu, bus, cyc) = run(|_, bus| {
        bus.poke(0x00FF, &[0x11]);
        bus.poke(0x0000, &[0x22]);
    }, &[0xBA, 0xFF]);
    assert_eq!(cpu.a, 0x11);
    assert_eq!(cpu.y, 0x22);
}

#[test]
fn e06_13_movw_store() {
    let (cpu, bus, cyc) = run(|cpu, _| {
        cpu.y = 0xAB;
        cpu.a = 0xCD;
    }, &[0xDA, 0x10]);
    assert_eq!(bus.peek(0x0010), 0xCD); // A stored first (low byte)
    assert_eq!(bus.peek(0x0011), 0xAB); // Y stored second (high byte)
    // Verify dummy read of destination before write
    let mut found_dummy = false;
    for cycle in &bus.log {
        if cycle.addr == 0x0010 && cycle.kind == puce16_core::apu::spc700::testbus::CycleKind::Read {
            found_dummy = true;
            break;
        }
    }
    assert!(found_dummy, "dummy read of destination not found");
}

#[test]
fn e06_13_movw_store_wrap() {
    // Wrap at 0xFF -> 0x00
    let (cpu, bus, cyc) = run(|cpu, _| {
        cpu.y = 0x12;
        cpu.a = 0x34;
    }, &[0xDA, 0xFF]);
    assert_eq!(bus.peek(0x00FF), 0x34); // A stored at 0xFF
    assert_eq!(bus.peek(0x0000), 0x12); // Y wrapped to 0x00
}
