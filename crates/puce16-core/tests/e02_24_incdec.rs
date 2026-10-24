use puce16_core::cpu::{Bus, Cpu, FLAG_C, FLAG_M, FLAG_N, FLAG_X, FLAG_Z};

fn make_cpu() -> Cpu {
    let mut cpu = Cpu::new();
    cpu.e = false; // native mode by default
    cpu
}

#[test]
fn e02_24_inc_a_8() {
    // INC A 8-bit: p=0x20 (M=1), a=0x12FF, `1A` -> a==0x1200, Z=1; cycles=2
    let mut cpu = make_cpu();
    cpu.p = FLAG_M;
    cpu.a = 0x12FF;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    bus.write(0, 0x1A); // INC A
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x1200, "a should be 0x1200 (high byte preserved)");
    assert!(cpu.flag(FLAG_Z), "Z should be 1");
}

#[test]
fn e02_24_inc_a_16() {
    // INC A 16-bit: p=0x00 (M=0), a=0x7FFF, `1A` -> a==0x8000, N=1
    let mut cpu = make_cpu();
    cpu.p = 0;
    cpu.a = 0x7FFF;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    bus.write(0, 0x1A); // INC A
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x8000, "a should be 0x8000");
    assert!(cpu.flag(FLAG_N), "N should be 1");

    // Also test wrap-around: a=0xFFFF -> a==0, Z=1
    let mut cpu2 = make_cpu();
    cpu2.p = 0;
    cpu2.a = 0xFFFF;
    let mut bus2 = puce16_core::cpu::testbus::FlatBus::new();
    cpu2.pc = 0;
    bus2.write(0, 0x1A); // INC A
    cpu2.step(&mut bus2);
    assert_eq!(cpu2.a, 0, "a should wrap to 0");
    assert!(cpu2.flag(FLAG_Z), "Z should be 1 on wrap");
}

#[test]
fn e02_24_dec_a_16() {
    // DEC A 16-bit: p=0x00 (M=0), a=0x0000, `3A` -> a==0xFFFF, N=1
    let mut cpu = make_cpu();
    cpu.p = 0;
    cpu.a = 0;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    bus.write(0, 0x3A); // DEC A
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0xFFFF, "a should wrap to 0xFFFF");
    assert!(cpu.flag(FLAG_N), "N should be 1");
}

#[test]
fn e02_24_dec_a_8() {
    // DEC A 8-bit: p=0x20 (M=1), a=0x01, `3A` -> A.l==0, Z=1
    let mut cpu = make_cpu();
    cpu.p = FLAG_M;
    cpu.a = 0x01;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    bus.write(0, 0x3A); // DEC A
    cpu.step(&mut bus);
    assert_eq!(cpu.a & 0xFF, 0, "low byte should be 0");
    assert!(cpu.flag(FLAG_Z), "Z should be 1");
}

#[test]
fn e02_24_inc_dp_8() {
    // INC direct page 8-bit: p=0x20 (M=1), d=0, poke(0x10,&[0x7F]), `E6 10` -> peek(0x10)==0x80, N=1; cycles=5
    let mut cpu = make_cpu();
    cpu.p = FLAG_M;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    bus.poke(0x10, &[0x7F]);
    cpu.pc = 0;
    bus.write(0, 0xE6); // INC d
    bus.write(1, 0x10);
    cpu.step(&mut bus);
    assert_eq!(bus.peek(0x10), 0x80, "memory should be incremented to 0x80");
    assert!(cpu.flag(FLAG_N), "N should be 1 (0x80 has bit 7 set)");
}

#[test]
fn e02_24_inc_abs_16() {
    // INC absolute 16-bit: p=0x00 (M=0), poke(0x2000,&[0xFF,0x00]), `EE 00 20` -> peek(0x2000)==0x00, peek(0x2001)==0x01
    let mut cpu = make_cpu();
    cpu.p = 0;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    bus.poke(0x2000, &[0xFF, 0x00]);
    cpu.pc = 0;
    bus.write(0, 0xEE); // INC a
    bus.write(1, 0x00);
    bus.write(2, 0x20);
    cpu.step(&mut bus);
    assert_eq!(bus.peek(0x2000), 0x00, "low byte should wrap to 0");
    assert_eq!(bus.peek(0x2001), 0x01, "high byte should increment to 1");
}

#[test]
fn e02_24_dec_abs_x() {
    // DEC absolute,X: p=0x20 (M=1), X=0 so index is 16 bits, x=1, poke(0x2001,&[0x01]), `DE 00 20` -> peek(0x2001)==0, Z=1; cycles=7
    let mut cpu = make_cpu();
    cpu.p = FLAG_M;
    cpu.x = 1;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    bus.poke(0x2001, &[0x01]);
    cpu.pc = 0;
    bus.write(0, 0xDE); // DEC a,x
    bus.write(1, 0x00);
    bus.write(2, 0x20);
    cpu.step(&mut bus);
    assert_eq!(bus.peek(0x2001), 0, "memory should be decremented to 0");
    assert!(cpu.flag(FLAG_Z), "Z should be 1");
}

#[test]
fn e02_24_inx_8() {
    // INX 8-bit: p=0x30 (M=1,X=1), x=0x00FF, `E8` -> x==0, Z=1; cycles=2
    let mut cpu = make_cpu();
    cpu.p = FLAG_M | FLAG_X;
    cpu.x = 0xFF;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    bus.write(0, 0xE8); // INX
    cpu.step(&mut bus);
    assert_eq!(cpu.x, 0, "x should wrap to 0");
    assert!(cpu.flag(FLAG_Z), "Z should be 1");
}

#[test]
fn e02_24_inx_16() {
    // INX 16-bit: p=0x00 (M=0,X=0), x=0xFFFF, `E8` -> x==0, Z=1
    let mut cpu = make_cpu();
    cpu.p = 0;
    cpu.x = 0xFFFF;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    bus.write(0, 0xE8); // INX
    cpu.step(&mut bus);
    assert_eq!(cpu.x, 0, "x should wrap to 0");
    assert!(cpu.flag(FLAG_Z), "Z should be 1");
}

#[test]
fn e02_24_dey_16() {
    // DEY 16-bit: p=0x00 (M=0,X=0), y=0, `88` -> y==0xFFFF, N=1
    let mut cpu = make_cpu();
    cpu.p = 0;
    cpu.y = 0;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    bus.write(0, 0x88); // DEY
    cpu.step(&mut bus);
    assert_eq!(cpu.y, 0xFFFF, "y should wrap to 0xFFFF");
    assert!(cpu.flag(FLAG_N), "N should be 1");
}

#[test]
fn e02_24_carry_untouched() {
    // C flag not modified by INC/DEC/INX: set C=1, execute INC A, verify C still 1
    let mut cpu = make_cpu();
    cpu.p = FLAG_C;
    cpu.a = 0x50;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    cpu.pc = 0;
    bus.write(0, 0x1A); // INC A
    cpu.step(&mut bus);
    assert!(cpu.flag(FLAG_C), "C should remain 1 after INC");

    // Also test DEC and INX
    let mut cpu2 = make_cpu();
    cpu2.p = FLAG_C;
    cpu2.a = 0x50;
    let mut bus2 = puce16_core::cpu::testbus::FlatBus::new();
    cpu2.pc = 0;
    bus2.write(0, 0x3A); // DEC A
    cpu2.step(&mut bus2);
    assert!(cpu2.flag(FLAG_C), "C should remain 1 after DEC");

    let mut cpu3 = make_cpu();
    cpu3.p = FLAG_C;
    cpu3.x = 0x50;
    let mut bus3 = puce16_core::cpu::testbus::FlatBus::new();
    cpu3.pc = 0;
    bus3.write(0, 0xE8); // INX
    cpu3.step(&mut bus3);
    assert!(cpu3.flag(FLAG_C), "C should remain 1 after INX");
}
