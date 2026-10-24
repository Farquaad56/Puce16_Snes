use puce16_core::cpu::{Bus, Cpu, FLAG_C, FLAG_D, FLAG_M, FLAG_V};

fn setup_cpu(a: u16, flags: u8) -> Cpu {
    let mut cpu = Cpu::new();
    cpu.a = a;
    cpu.set_p(flags);
    cpu
}

#[test]
fn e02_19_bcd_12_34() {
    // D=1 (p=0x28: M=1, D=1), a=0x12, C=0, 69 34 -> A.l==0x46, C=0, V=0
    let mut cpu = setup_cpu(0x12, FLAG_M | FLAG_D);
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    // ADC #immediate (opcode 0x69)
    cpu.pc = 0;
    let bytes = [0x69, 0x34];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert_eq!(cpu.a as u8, 0x46, "A.l should be 0x46");
    assert!(!cpu.flag(FLAG_C), "C should be 0");
    assert!(!cpu.flag(FLAG_V), "V should be 0");
}

#[test]
fn e02_19_bcd_58_46() {
    // D=1, a=0x58, 69 46 -> A.l==0x04, C=1, V=1 (reference algorithm result)
    let mut cpu = setup_cpu(0x58, FLAG_M | FLAG_D);
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    let bytes = [0x69, 0x46];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert_eq!(cpu.a as u8, 0x04, "A.l should be 0x04");
    assert!(cpu.flag(FLAG_C), "C should be 1");
    assert!(cpu.flag(FLAG_V), "V should be 1 (reference algorithm)");
}

#[test]
fn e02_19_bcd_99_01() {
    // D=1, a=0x99, 69 01 -> A.l==0x00, C=1, Z=1, V=0
    let mut cpu = setup_cpu(0x99, FLAG_M | FLAG_D);
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    let bytes = [0x69, 0x01];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert_eq!(cpu.a as u8, 0x00, "A.l should be 0x00");
    assert!(cpu.flag(FLAG_C), "C should be 1");
    assert!(cpu.flag(puce16_core::cpu::FLAG_Z), "Z should be 1");
    assert!(!cpu.flag(FLAG_V), "V should be 0");
}

#[test]
fn e02_19_bcd_carry_in() {
    // D=1, C=1, a=0x09, 69 01 -> A.l==0x11, C=0
    let mut cpu = setup_cpu(0x09, FLAG_M | FLAG_D);
    cpu.set_flag(FLAG_C, true);
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    let bytes = [0x69, 0x01];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert_eq!(cpu.a as u8, 0x11, "A.l should be 0x11");
    assert!(!cpu.flag(FLAG_C), "C should be 0");
}

#[test]
fn e02_19_bcd_16_1234_4321() {
    // D=1, M=0 (p=0x08), a=0x1234, 69 21 43 -> a==0x5555, C=0; cycles=3
    let mut cpu = setup_cpu(0x1234, FLAG_D);
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    let bytes = [0x69, 0x21, 0x43];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x5555, "A should be 0x5555");
    assert!(!cpu.flag(FLAG_C), "C should be 0");
}

#[test]
fn e02_19_bcd_16_0999_0001() {
    // D=1, M=0, a=0x0999, 69 01 00 -> a==0x1000, C=0, N=0
    let mut cpu = setup_cpu(0x0999, FLAG_D);
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    let bytes = [0x69, 0x01, 0x00];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x1000, "A should be 0x1000");
    assert!(!cpu.flag(FLAG_C), "C should be 0");
    assert!(!cpu.flag(puce16_core::cpu::FLAG_N), "N should be 0");
}

#[test]
fn e02_19_bcd_16_9999_0001() {
    // D=1, M=0, a=0x9999, 69 01 00 -> a==0x0000, C=1, Z=1
    let mut cpu = setup_cpu(0x9999, FLAG_D);
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    let bytes = [0x69, 0x01, 0x00];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x0000, "A should be 0x0000");
    assert!(cpu.flag(FLAG_C), "C should be 1");
    assert!(cpu.flag(puce16_core::cpu::FLAG_Z), "Z should be 1");
}

#[test]
fn e02_19_binary_still_works() {
    // D=0: 69 05 with a=0x10 gives 0x15 (non-regression from E02-18)
    let mut cpu = setup_cpu(0x10, FLAG_M);
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    let bytes = [0x69, 0x05];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert_eq!(cpu.a as u8, 0x15, "A.l should be 0x15");
}
