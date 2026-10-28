use puce16_core::apu::spc700::{Spc700, testbus::FlatSmpBus, F_C};

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
fn e06_24_or1() {
    // OR1 C,m.b: bit set -> C=1; bit clear -> C unchanged (still 0)
    let (cpu, _, cyc) = run(|c, b| {
        c.psw = 0;
        b.poke(0x0005, &[0x02]); // bit 1 set at addr 0x0005
    }, &[0x0A, 0x05, 0x20]); // v=0x2005: addr=0x0005, bit=1
    assert!(cpu.flag(F_C), "OR1 with bit set should set C");
    assert_eq!(cyc, 5);

    let (cpu2, _, _) = run(|c, b| {
        c.psw = 0;
        b.poke(0x0005, &[0x00]); // bit 1 clear
    }, &[0x0A, 0x05, 0x20]);
    assert!(!cpu2.flag(F_C), "OR1 with bit clear should not set C");
}

#[test]
fn e06_24_or1_inverted() {
    // OR1 C,/m.b: inverted bit
    let (cpu, _, _) = run(|c, b| {
        c.psw = 0;
        b.poke(0x0005, &[0x00]); // bit 1 clear -> /bit=1
    }, &[0x2A, 0x05, 0x20]);
    assert!(cpu.flag(F_C), "OR1 with inverted clear bit should set C");

    let (cpu2, _, _) = run(|c, b| {
        c.psw = 0;
        b.poke(0x0005, &[0x02]); // bit 1 set -> /bit=0
    }, &[0x2A, 0x05, 0x20]);
    assert!(!cpu2.flag(F_C), "OR1 with inverted set bit should not set C");
}

#[test]
fn e06_24_and1() {
    // AND1 C,m.b: C=1 and bit=1 -> C=1; C=1 and bit=0 -> C=0
    let (cpu, _, cyc) = run(|c, b| {
        c.psw = F_C;
        b.poke(0x0005, &[0x02]); // bit 1 set
    }, &[0x4A, 0x05, 0x20]);
    assert!(cpu.flag(F_C), "AND1 with C=1 and bit=1 should keep C");
    assert_eq!(cyc, 4);

    let (cpu2, _, _) = run(|c, b| {
        c.psw = F_C;
        b.poke(0x0005, &[0x00]); // bit 1 clear
    }, &[0x4A, 0x05, 0x20]);
    assert!(!cpu2.flag(F_C), "AND1 with C=1 and bit=0 should clear C");
}

#[test]
fn e06_24_and1_inverted() {
    // AND1 C,/m.b: inverted
    let (cpu, _, _) = run(|c, b| {
        c.psw = F_C;
        b.poke(0x0005, &[0x02]); // bit 1 set -> /bit=0
    }, &[0x6A, 0x05, 0x20]);
    assert!(!cpu.flag(F_C), "AND1 with inverted set bit should clear C");

    let (cpu2, _, _) = run(|c, b| {
        c.psw = F_C;
        b.poke(0x0005, &[0x00]); // bit 1 clear -> /bit=1
    }, &[0x6A, 0x05, 0x20]);
    assert!(cpu2.flag(F_C), "AND1 with inverted clear bit should keep C");
}

#[test]
fn e06_24_eor1() {
    // EOR1 C,m.b: toggle C based on bit
    let (cpu, _, cyc) = run(|c, b| {
        c.psw = F_C;
        b.poke(0x0005, &[0x02]); // bit 1 set -> C^1=0
    }, &[0x8A, 0x05, 0x20]);
    assert!(!cpu.flag(F_C), "EOR1 with C=1 and bit=1 should clear C");
    assert_eq!(cyc, 5);

    let (cpu2, _, _) = run(|c, b| {
        c.psw = 0;
        b.poke(0x0005, &[0x02]); // bit 1 set -> C^1=1
    }, &[0x8A, 0x05, 0x20]);
    assert!(cpu2.flag(F_C), "EOR1 with C=0 and bit=1 should set C");
}

#[test]
fn e06_24_mov1_to_c() {
    // MOV1 C,m.b: copy bit to carry
    let (cpu, _, cyc) = run(|c, b| {
        c.psw = 0;
        b.poke(0x0005, &[0x02]); // bit 1 set
    }, &[0xAA, 0x05, 0x20]);
    assert!(cpu.flag(F_C), "MOV1 C,m.b with bit=1 should set C");
    assert_eq!(cyc, 4);

    let (cpu2, _, _) = run(|c, b| {
        c.psw = F_C;
        b.poke(0x0005, &[0x00]); // bit 1 clear
    }, &[0xAA, 0x05, 0x20]);
    assert!(!cpu2.flag(F_C), "MOV1 C,m.b with bit=0 should clear C");
}

#[test]
fn e06_24_mov1_to_mem() {
    // MOV1 m.b,C: copy carry to memory bit
    let (_cpu, bus, cyc) = run(|c, b| {
        c.psw = F_C;
        b.poke(0x0005, &[0x00]);
    }, &[0xCA, 0x05, 0x20]);
    assert_eq!(bus.peek(0x0005), 0x02, "MOV1 m.b,C with C=1 should set bit");
    assert_eq!(cyc, 6);

    let (_cpu2, bus2, _) = run(|c, b| {
        c.psw = 0;
        b.poke(0x0005, &[0xFF]);
    }, &[0xCA, 0x05, 0x20]);
    assert_eq!(bus2.peek(0x0005), 0xFD, "MOV1 m.b,C with C=0 should clear bit");
}

#[test]
fn e06_24_not1() {
    // NOT1 m.b: invert memory bit
    let (_cpu, bus, cyc) = run(|c, b| {
        c.psw = 0;
        b.poke(0x0005, &[0x00]);
    }, &[0xEA, 0x05, 0x20]);
    assert_eq!(bus.peek(0x0005), 0x02, "NOT1 should set bit");
    assert_eq!(cyc, 5);

    // Second execution: invert back
    let (_cpu2, bus2, _) = run(|c, b| {
        c.psw = 0;
        b.poke(0x0005, &[0x02]);
    }, &[0xEA, 0x05, 0x20]);
    assert_eq!(bus2.peek(0x0005), 0x00, "NOT1 again should clear bit");
}

#[test]
fn e06_24_bit_seven_and_address() {
    // v=0xE010: addr=0x0010, bit=7
    let (cpu, _, _) = run(|c, b| {
        c.psw = 0;
        b.poke(0x0010, &[0x80]); // bit 7 set
    }, &[0xAA, 0x10, 0xE0]);
    assert!(cpu.flag(F_C), "MOV1 with v=0xE010 should read bit 7 of addr 0x0010");

    // v=0x3FFF: addr=0x1FFF, bit=1
    let (cpu2, _, _) = run(|c, b| {
        c.psw = 0;
        b.poke(0x1FFF, &[0x02]); // bit 1 set at max address
    }, &[0xAA, 0xFF, 0x3F]);
    assert!(cpu2.flag(F_C), "MOV1 with v=0x3FFF should read bit 1 of addr 0x1FFF");
}

#[test]
fn e06_24_direct_page_ignored() {
    // P flag should not affect addressing for these instructions
    let (cpu, _, _) = run(|c, b| {
        c.psw = F_C; // set P flag -> dp=0x0100
        b.poke(0x0005, &[0x02]); // bit at 0x0005
        b.poke(0x0105, &[0x00]); // different value at 0x0105 (dp+5)
    }, &[0xAA, 0x05, 0x20]);
    assert!(cpu.flag(F_C), "Should read from absolute 0x0005, not dp-relative 0x0105");
}

#[test]
fn e06_24_no_flag_change_mem_ops() {
    // NOT1 and MOV1 m.b,C should not change any flags except C (for MOV1 to mem)
    let (cpu, _, _) = run(|c, b| {
        c.psw = 0xC3; // N,V,P,B,H,I,Z,C all set
        b.poke(0x0005, &[0x00]);
    }, &[0xEA, 0x05, 0x20]);
    assert_eq!(cpu.psw, 0xC3, "NOT1 should not change any flags");
}
