use puce16_core::cpu::{Bus, Cpu, FLAG_C, FLAG_M, FLAG_N, FLAG_Z};

fn make_cpu() -> Cpu {
    let mut cpu = Cpu::new();
    cpu.e = false; // native mode by default
    cpu
}

#[test]
fn e02_25_asl_a_8() {
    // ASL A 8-bit: p=0x20 (M=1), a=0x1281, `0A` -> A.l==0x02, C=1, N=0, Z=0, a==0x1202; cycles=2
    let mut cpu = make_cpu();
    cpu.p = FLAG_M;
    cpu.a = 0x1281;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    bus.write(0, 0x0A); // ASL A
    cpu.step(&mut bus);
    assert_eq!(cpu.a & 0xFF, 0x02, "low byte should be 0x02");
    assert_eq!(cpu.a, 0x1202, "a should be 0x1202 (high byte preserved)");
    assert!(cpu.flag(FLAG_C), "C should be 1 (bit 7 was set)");
    assert!(!cpu.flag(FLAG_N), "N should be 0");
    assert!(!cpu.flag(FLAG_Z), "Z should be 0");
}

#[test]
fn e02_25_asl_a_16() {
    // ASL A 16-bit: p=0x00 (M=0), a=0x4000, `0A` -> a==0x8000, N=1, C=0
    let mut cpu = make_cpu();
    cpu.p = 0;
    cpu.a = 0x4000;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    bus.write(0, 0x0A); // ASL A
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x8000, "a should be 0x8000");
    assert!(cpu.flag(FLAG_N), "N should be 1 (bit 15 set)");
    assert!(!cpu.flag(FLAG_C), "C should be 0 (bit 15 was not set before shift)");
}

#[test]
fn e02_25_lsr_a_8() {
    // LSR A 8-bit: p=0x20 (M=1), a=0x01, `4A` -> A.l==0, C=1, Z=1, N=0
    let mut cpu = make_cpu();
    cpu.p = FLAG_M;
    cpu.a = 0x01;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    bus.write(0, 0x4A); // LSR A
    cpu.step(&mut bus);
    assert_eq!(cpu.a & 0xFF, 0, "low byte should be 0");
    assert!(cpu.flag(FLAG_C), "C should be 1 (bit 0 was set)");
    assert!(cpu.flag(FLAG_Z), "Z should be 1 (result is zero)");
    assert!(!cpu.flag(FLAG_N), "N should be 0 (LSR always clears N)");
}

#[test]
fn e02_25_lsr_a_16() {
    // LSR A 16-bit: p=0x00 (M=0), a=0x0003, `4A` -> a==1, C=1
    let mut cpu = make_cpu();
    cpu.p = 0;
    cpu.a = 0x0003;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    bus.write(0, 0x4A); // LSR A
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 1, "a should be 1");
    assert!(cpu.flag(FLAG_C), "C should be 1 (bit 0 was set)");
}

#[test]
fn e02_25_rol_a_8() {
    // ROL A 8-bit: p=0x21 (M=1,C=1), a=0x80, `2A` -> A.l==0x01, C=1
    let mut cpu = make_cpu();
    cpu.p = FLAG_M | FLAG_C;
    cpu.a = 0x80;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    bus.write(0, 0x2A); // ROL A
    cpu.step(&mut bus);
    assert_eq!(cpu.a & 0xFF, 0x01, "low byte should be 0x01");
    assert!(cpu.flag(FLAG_C), "C should be 1 (bit 7 was set before rotate)");
}

#[test]
fn e02_25_rol_a_16() {
    // ROL A 16-bit: p=0x00 (M=0,C=0), a=0x8001, `2A` -> a==0x0002, C=1
    let mut cpu = make_cpu();
    cpu.p = 0; // M=0, C=0
    cpu.a = 0x8001;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    bus.write(0, 0x2A); // ROL A
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x0002, "a should be 0x0002");
    assert!(cpu.flag(FLAG_C), "C should be 1 (bit 15 was set before rotate)");
}

#[test]
fn e02_25_ror_a_8() {
    // ROR A 8-bit: p=0x21 (M=1,C=1), a=0x01, `6A` -> A.l==0x80, C=1, N=1
    let mut cpu = make_cpu();
    cpu.p = FLAG_M | FLAG_C;
    cpu.a = 0x01;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    bus.write(0, 0x6A); // ROR A
    cpu.step(&mut bus);
    assert_eq!(cpu.a & 0xFF, 0x80, "low byte should be 0x80");
    assert!(cpu.flag(FLAG_C), "C should be 1 (bit 0 was set before rotate)");
    assert!(cpu.flag(FLAG_N), "N should be 1 (bit 7 is now set)");
}

#[test]
fn e02_25_ror_a_16() {
    // ROR A 16-bit: p=0x01 (M=0,C=1), a=0, `6A` -> a==0x8000, C=0, N=1
    let mut cpu = make_cpu();
    cpu.p = FLAG_C; // M=0, C=1
    cpu.a = 0;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    bus.write(0, 0x6A); // ROR A
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x8000, "a should be 0x8000");
    assert!(!cpu.flag(FLAG_C), "C should be 0 (bit 0 was not set before rotate)");
    assert!(cpu.flag(FLAG_N), "N should be 1 (bit 15 is now set)");
}

#[test]
fn e02_25_asl_dp() {
    // ASL direct page: p=0x20 (M=1), d=0, poke(0x10,&[0x40]), `06 10` -> peek(0x10)==0x80, N=1; cycles=5
    let mut cpu = make_cpu();
    cpu.p = FLAG_M;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    bus.poke(0x10, &[0x40]);
    cpu.pc = 0;
    bus.write(0, 0x06); // ASL d
    bus.write(1, 0x10);
    cpu.step(&mut bus);
    assert_eq!(bus.peek(0x10), 0x80, "memory should be shifted to 0x80");
    assert!(cpu.flag(FLAG_N), "N should be 1 (bit 7 is now set)");
}

#[test]
fn e02_25_ror_abs_x() {
    // ROR absolute,X: p=0x20 (M=1, X=0 so index is 16 bits), x=1, poke(0x2001,&[0x02]), C=0, `7E 00 20` -> peek(0x2001)==0x01; cycles=7
    let mut cpu = make_cpu();
    cpu.p = FLAG_M; // M=1, X=0 (index is 16 bits)
    cpu.x = 1;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    bus.poke(0x2001, &[0x02]);
    cpu.pc = 0;
    bus.write(0, 0x7E); // ROR a,x
    bus.write(1, 0x00);
    bus.write(2, 0x20);
    cpu.step(&mut bus);
    assert_eq!(bus.peek(0x2001), 0x01, "memory should be rotated to 0x01");
}

#[test]
fn e02_25_rol_abs_16() {
    // ROL absolute 16-bit: p=0x00 (M=0), C=1, poke(0x2000,&[0x00,0x80]), `2E 00 20` -> peek(0x2000)==0x01, peek(0x2001)==0x00, C=1; cycles=8
    let mut cpu = make_cpu();
    cpu.p = FLAG_C; // M=0, C=1
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    bus.poke(0x2000, &[0x00, 0x80]);
    cpu.pc = 0;
    bus.write(0, 0x2E); // ROL a
    bus.write(1, 0x00);
    bus.write(2, 0x20);
    cpu.step(&mut bus);
    assert_eq!(bus.peek(0x2000), 0x01, "low byte should be 0x01 (carry rotated in)");
    assert_eq!(bus.peek(0x2001), 0x00, "high byte should be 0x00");
    assert!(cpu.flag(FLAG_C), "C should be 1 (bit 15 was set before rotate)");
}
