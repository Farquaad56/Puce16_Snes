use puce16_core::cpu::{Bus, Cpu, FLAG_C, FLAG_D, FLAG_M};

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
fn e02_21_bcd_46_12() {
    // D=1, C=1 (p=0x29: M=1, D=1, C=1), a=0x46, E9 12 -> A.l==0x34, C=1
    let mut cpu = setup_cpu(0x46, FLAG_M | FLAG_D | FLAG_C);
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    // SBC #immediate (opcode 0xE9)
    cpu.pc = 0;
    let bytes = [0xE9, 0x12];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert_eq!(cpu.a as u8, 0x34, "A.l should be 0x34");
    assert!(cpu.flag(FLAG_C), "C should be 1");
}

#[test]
fn e02_21_bcd_12_34() {
    // D=1, C=1, a=0x12, E9 34 -> A.l==0x78, C=0 (borrow)
    let mut cpu = setup_cpu(0x12, FLAG_M | FLAG_D | FLAG_C);
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    let bytes = [0xE9, 0x34];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert_eq!(cpu.a as u8, 0x78, "A.l should be 0x78");
    assert!(!cpu.flag(FLAG_C), "C should be 0 (borrow)");
}

#[test]
fn e02_21_bcd_00_01() {
    // D=1, C=1, a=0x00, E9 01 -> A.l==0x99, C=0 (full borrow)
    let mut cpu = setup_cpu(0x00, FLAG_M | FLAG_D | FLAG_C);
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    let bytes = [0xE9, 0x01];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert_eq!(cpu.a as u8, 0x99, "A.l should be 0x99");
    assert!(!cpu.flag(FLAG_C), "C should be 0 (borrow)");
}

#[test]
fn e02_21_bcd_borrow_in() {
    // D=1, C=0 (borrow in), a=0x50, E9 20 -> A.l==0x29, C=1
    let mut cpu = setup_cpu(0x50, FLAG_M | FLAG_D);
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    let bytes = [0xE9, 0x20];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert_eq!(cpu.a as u8, 0x29, "A.l should be 0x29");
    assert!(cpu.flag(FLAG_C), "C should be 1 (no borrow out)");
}

#[test]
fn e02_21_bcd_16_1000_0001() {
    // D=1, M=0 (p=0x09: D=1, C=1), a=0x1000, E9 01 00 -> a==0x0999, C=1
    let mut cpu = setup_cpu(0x1000, FLAG_D | FLAG_C);
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    let bytes = [0xE9, 0x01, 0x00];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x0999, "A should be 0x0999");
    assert!(cpu.flag(FLAG_C), "C should be 1 (no borrow)");
}

#[test]
fn e02_21_bcd_16_0000_0001() {
    // D=1, M=0, C=1, a=0x0000, E9 01 00 -> a==0x9999, C=0 (full borrow)
    let mut cpu = setup_cpu(0x0000, FLAG_D | FLAG_C);
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    let bytes = [0xE9, 0x01, 0x00];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x9999, "A should be 0x9999");
    assert!(!cpu.flag(FLAG_C), "C should be 0 (borrow)");
}

#[test]
fn e02_21_binary_still_works() {
    // D=0: non-regression from E02-20 (a=0x10, C=1, E9 05 -> 0x0B)
    let mut cpu = setup_cpu(0x10, FLAG_M | FLAG_C);
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    let bytes = [0xE9, 0x05];
    for (i, b) in bytes.iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert_eq!(cpu.a as u8, 0x0B, "A.l should be 0x0B");
}
