use puce16_core::cpu::{Bus, Cpu, FLAG_C, FLAG_M, FLAG_N, FLAG_V, FLAG_X, FLAG_Z};

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
fn e02_23_cmp_equal() {
    // CMP #immediate 8-bit equal: p=0x20 (M=1), a=0x10, `C9 10` -> Z=1, C=1, N=0; A unchanged; cycles=2
    let mut cpu = setup_cpu(0x10, FLAG_M);
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    let bytes = [0xC9, 0x10];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert!(cpu.flag(FLAG_Z), "Z should be 1");
    assert!(cpu.flag(FLAG_C), "C should be 1");
    assert!(!cpu.flag(FLAG_N), "N should be 0");
    assert_eq!(cpu.a, 0x10, "A should be unchanged");
}

#[test]
fn e02_23_cmp_less() {
    // CMP #immediate 8-bit less: p=0x20 (M=1), a=0x10, `C9 20` -> C=0, N=1, Z=0
    let mut cpu = setup_cpu(0x10, FLAG_M);
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    let bytes = [0xC9, 0x20];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert!(!cpu.flag(FLAG_C), "C should be 0");
    assert!(cpu.flag(FLAG_N), "N should be 1");
    assert!(!cpu.flag(FLAG_Z), "Z should be 0");
}

#[test]
fn e02_23_cmp_greater() {
    // CMP #immediate 8-bit greater: p=0x20 (M=1), a=0x20, `C9 10` -> C=1, N=0, Z=0
    let mut cpu = setup_cpu(0x20, FLAG_M);
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    let bytes = [0xC9, 0x10];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert!(cpu.flag(FLAG_C), "C should be 1");
    assert!(!cpu.flag(FLAG_N), "N should be 0");
    assert!(!cpu.flag(FLAG_Z), "Z should be 0");
}

#[test]
fn e02_23_cmp_v_untouched() {
    // CMP does not affect V: p=0x60 (V=1), a=0x10, `C9 20` -> V remains 1
    let mut cpu = setup_cpu(0x10, FLAG_M | FLAG_V);
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    let bytes = [0xC9, 0x20];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert!(cpu.flag(FLAG_V), "V should remain 1");
}

#[test]
fn e02_23_cmp_16() {
    // CMP #immediate 16-bit: p=0x00 (M=0), a=0x8000, `C9 01 00` -> C=1, N=0, Z=0 (result 0x7FFF); cycles=3
    let mut cpu = setup_cpu(0x8000, 0);
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    let bytes = [0xC9, 0x01, 0x00];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert!(cpu.flag(FLAG_C), "C should be 1");
    assert!(!cpu.flag(FLAG_N), "N should be 0 (result 0x7FFF)");
    assert!(!cpu.flag(FLAG_Z), "Z should be 0");
}

#[test]
fn e02_23_cpx_imm_16() {
    // CPX #immediate 16-bit: p=0x00 (M=0,X=0), x=0x1234, `E0 34 12` -> Z=1, C=1; cycles=3
    let mut cpu = make_cpu();
    cpu.x = 0x1234;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    let bytes = [0xE0, 0x34, 0x12];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert!(cpu.flag(FLAG_Z), "Z should be 1");
    assert!(cpu.flag(FLAG_C), "C should be 1");
}

#[test]
fn e02_23_cpx_imm_8() {
    // CPX #immediate 8-bit: p=0x30 (M=1,X=1), x=0x10, `E0 11` -> C=0, N=1, Z=0; cycles=2
    let mut cpu = make_cpu();
    cpu.p = FLAG_M | FLAG_X;
    cpu.x = 0x10;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    let bytes = [0xE0, 0x11];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert!(!cpu.flag(FLAG_C), "C should be 0");
    assert!(cpu.flag(FLAG_N), "N should be 1");
    assert!(!cpu.flag(FLAG_Z), "Z should be 0");
}

#[test]
fn e02_23_cpy_dp() {
    // CPY direct page: p=0x30 (M=1,X=1), d=0, poke(0x10,&[0x05]), y=0x05, `C4 10` -> Z=1, C=1; cycles=3
    let mut cpu = make_cpu();
    cpu.p = FLAG_M | FLAG_X;
    cpu.y = 0x05;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    bus.poke(0x10, &[0x05]);
    cpu.pc = 0;
    let bytes = [0xC4, 0x10];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert!(cpu.flag(FLAG_Z), "Z should be 1");
    assert!(cpu.flag(FLAG_C), "C should be 1");
}

#[test]
fn e02_23_cmp_abs_x() {
    // CMP absolute,X: p=0x30 (M=1,X=1), x=1, poke(0x2001,&[0x40]), a=0x40, `DD 00 20` -> Z=1; cycles=4
    let mut cpu = make_cpu();
    cpu.p = FLAG_M | FLAG_X;
    cpu.a = 0x40;
    cpu.x = 1;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    bus.poke(0x2001, &[0x40]);
    cpu.pc = 0;
    let bytes = [0xDD, 0x00, 0x20];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert!(cpu.flag(FLAG_Z), "Z should be 1");
}
