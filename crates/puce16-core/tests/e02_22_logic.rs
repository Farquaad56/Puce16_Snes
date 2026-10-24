use puce16_core::cpu::{Bus, Cpu, FLAG_M, FLAG_N, FLAG_V, FLAG_X, FLAG_Z};

fn make_cpu() -> Cpu {
    let mut cpu = Cpu::new();
    cpu.e = false; // native mode by default
    cpu
}

fn setup_cpu(a: u16, flags: u8) -> Cpu {
    let mut cpu = make_cpu();
    cpu.a = a;
    cpu.p = flags;
    cpu
}

#[test]
fn e02_22_and_imm_8() {
    // AND #immediate 8-bit: p=0x20 (M=1), a=0xABF0, `29 3C` -> a==0xAB30, N=0,Z=0; cycles=2
    let mut cpu = setup_cpu(0xABF0, FLAG_M);
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    let bytes = [0x29, 0x3C];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0xAB30, "A should be 0xAB30");
    assert!(!cpu.flag(FLAG_N), "N should be 0");
    assert!(!cpu.flag(FLAG_Z), "Z should be 0");
}

#[test]
fn e02_22_and_zero() {
    // AND #immediate that results in zero: p=0x20, a=0x0F, `29 F0` -> A.l==0, Z=1
    let mut cpu = setup_cpu(0x0F, FLAG_M);
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    let bytes = [0x29, 0xF0];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert_eq!(cpu.a as u8, 0, "A.l should be 0");
    assert!(cpu.flag(FLAG_Z), "Z should be 1");
}

#[test]
fn e02_22_ora_imm_8() {
    // ORA #immediate 8-bit: p=0x20, a=0x80, `09 01` -> A.l==0x81, N=1
    let mut cpu = setup_cpu(0x80, FLAG_M);
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    let bytes = [0x09, 0x01];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert_eq!(cpu.a as u8, 0x81, "A.l should be 0x81");
    assert!(cpu.flag(FLAG_N), "N should be 1");
}

#[test]
fn e02_22_eor_zero() {
    // EOR #immediate that results in zero: p=0x20, a=0xFF, `49 FF` -> A.l==0, Z=1
    let mut cpu = setup_cpu(0xFF, FLAG_M);
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    let bytes = [0x49, 0xFF];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert_eq!(cpu.a as u8, 0, "A.l should be 0");
    assert!(cpu.flag(FLAG_Z), "Z should be 1");
}

#[test]
fn e02_22_and_imm_16() {
    // AND #immediate 16-bit: p=0x00 (M=0), a=0xFF00, `29 F0 0F` -> a==0x0F00; cycles=3
    let mut cpu = setup_cpu(0xFF00, 0);
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    let bytes = [0x29, 0xF0, 0x0F];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x0F00, "A should be 0x0F00");
}

#[test]
fn e02_22_ora_dp_16() {
    // ORA direct page 16-bit: p=0x00 (M=0), d=0, poke(0x10,&[0x0F,0xF0]), a=0x0101, `05 10` -> a==0xF10F, N=1; cycles=4
    let mut cpu = setup_cpu(0x0101, 0);
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    bus.poke(0x10, &[0x0F, 0xF0]);
    cpu.pc = 0;
    let bytes = [0x05, 0x10];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0xF10F, "A should be 0xF10F");
    assert!(cpu.flag(FLAG_N), "N should be 1");
}

#[test]
fn e02_22_bit_imm_only_z() {
    // BIT #immediate: only Z is affected; N and V unchanged. p=0xE0 (M=1,N=1,V=1), a=0x0F, `89 F0` -> Z=1, N=1, V=1 preserved
    let mut cpu = setup_cpu(0x0F, FLAG_M | FLAG_N | FLAG_V);
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    let bytes = [0x89, 0xF0];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert!(cpu.flag(FLAG_Z), "Z should be 1");
    assert!(cpu.flag(FLAG_N), "N should be preserved (1)");
    assert!(cpu.flag(FLAG_V), "V should be preserved (1)");
}

#[test]
fn e02_22_bit_dp_nv() {
    // BIT direct page: Z=(A&M)==0, N=bit7 of M, V=bit6 of M. p=0x20, d=0, poke(0x10,&[0xC0]), a=0x01, `24 10` -> Z=1, N=1, V=1; A unchanged; cycles=3
    let mut cpu = setup_cpu(0x01, FLAG_M);
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    bus.poke(0x10, &[0xC0]);
    cpu.pc = 0;
    let bytes = [0x24, 0x10];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert!(cpu.flag(FLAG_Z), "Z should be 1");
    assert!(cpu.flag(FLAG_N), "N should be 1 (bit7 of M)");
    assert!(cpu.flag(FLAG_V), "V should be 1 (bit6 of M)");
    assert_eq!(cpu.a, 0x01, "A should be unchanged");
}

#[test]
fn e02_22_bit_dp_16() {
    // BIT direct page 16-bit: p=0x00 (M=0), poke(0x10,&[0x00,0x40]), a=0x4000, `24 10` -> Z=0, V=1, N=0; cycles=4
    let mut cpu = setup_cpu(0x4000, 0);
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    bus.poke(0x10, &[0x00, 0x40]);
    cpu.pc = 0;
    let bytes = [0x24, 0x10];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert!(!cpu.flag(FLAG_Z), "Z should be 0");
    assert!(cpu.flag(FLAG_V), "V should be 1 (bit14 of M)");
    assert!(!cpu.flag(FLAG_N), "N should be 0 (bit15 of M)");
}

#[test]
fn e02_22_bit_abs_x() {
    // BIT absolute,X: p=0x30 (M=1,X=1), x=1, poke(0x2001,&[0x80]), a=0x80, `3C 00 20` -> N=1, Z=0, V=0; cycles=4
    let mut cpu = setup_cpu(0x80, FLAG_M | FLAG_X);
    cpu.x = 1;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    bus.poke(0x2001, &[0x80]);
    cpu.pc = 0;
    let bytes = [0x3C, 0x00, 0x20];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert!(cpu.flag(FLAG_N), "N should be 1");
    assert!(!cpu.flag(FLAG_Z), "Z should be 0");
    assert!(!cpu.flag(FLAG_V), "V should be 0");
}