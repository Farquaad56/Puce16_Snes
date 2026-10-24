use puce16_core::cpu::{Bus, Cpu, FLAG_C, FLAG_M, FLAG_N, FLAG_V, FLAG_Z};

fn make_cpu() -> Cpu {
    let mut cpu = Cpu::new();
    cpu.e = false; // native mode by default
    cpu
}

#[test]
fn e02_26_tsb_dp_8() {
    // TSB direct page 8-bit: p=0x20 (M=1), poke(0x10,&[0xF0]), a=0x0F, `04 10` -> Z=1, peek(0x10)==0xFF; cycles=5
    let mut cpu = make_cpu();
    cpu.p = FLAG_M;
    cpu.a = 0x0F;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    bus.poke(0x10, &[0xF0]);
    cpu.pc = 0;
    bus.write(0, 0x04); // TSB d
    bus.write(1, 0x10);
    cpu.step(&mut bus);
    assert!(cpu.flag(FLAG_Z), "Z should be 1 (A & M == 0)");
    assert_eq!(bus.peek(0x10), 0xFF, "memory should have bits set: 0xF0 | 0x0F = 0xFF");
}

#[test]
fn e02_26_trb_dp_8() {
    // TRB direct page 8-bit: p=0x20 (M=1), poke(0x10,&[0xFF]), a=0x0F, `14 10` -> Z=0, peek(0x10)==0xF0
    let mut cpu = make_cpu();
    cpu.p = FLAG_M;
    cpu.a = 0x0F;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    bus.poke(0x10, &[0xFF]);
    cpu.pc = 0;
    bus.write(0, 0x14); // TRB d
    bus.write(1, 0x10);
    cpu.step(&mut bus);
    assert!(!cpu.flag(FLAG_Z), "Z should be 0 (A & M != 0)");
    assert_eq!(bus.peek(0x10), 0xF0, "memory should have bits reset: 0xFF & ~0x0F = 0xF0");
}

#[test]
fn e02_26_tsb_abs_16() {
    // TSB absolute 16-bit: p=0x00 (M=0), poke(0x2000,&[0xF0,0x00]), a=0x1234, `0C 00 20` -> Z=0, peek(0x2000)==0xF4, peek(0x2001)==0x12; cycles=8
    let mut cpu = make_cpu();
    cpu.p = 0; // M=0 (16-bit mode)
    cpu.a = 0x1234;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    bus.poke(0x2000, &[0xF0, 0x00]);
    cpu.pc = 0;
    bus.write(0, 0x0C); // TSB a
    bus.write(1, 0x00);
    bus.write(2, 0x20);
    cpu.step(&mut bus);
    assert!(!cpu.flag(FLAG_Z), "Z should be 0 (A & M != 0)");
    assert_eq!(bus.peek(0x2000), 0xF4, "low byte: 0xF0 | 0x34 = 0xF4");
    assert_eq!(bus.peek(0x2001), 0x12, "high byte: 0x00 | 0x12 = 0x12");
}

#[test]
fn e02_26_trb_abs_16() {
    // TRB absolute 16-bit: p=0x00 (M=0), poke(0x2000,&[0xFF,0xFF]), a=0x00FF, `1C 00 20` -> peek(0x2000)==0x00, peek(0x2001)==0xFF; Z=0
    let mut cpu = make_cpu();
    cpu.p = 0; // M=0 (16-bit mode)
    cpu.a = 0x00FF;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    bus.poke(0x2000, &[0xFF, 0xFF]);
    cpu.pc = 0;
    bus.write(0, 0x1C); // TRB a
    bus.write(1, 0x00);
    bus.write(2, 0x20);
    cpu.step(&mut bus);
    assert!(!cpu.flag(FLAG_Z), "Z should be 0 (A & M != 0)");
    assert_eq!(bus.peek(0x2000), 0x00, "low byte: 0xFF & ~0xFF = 0x00");
    assert_eq!(bus.peek(0x2001), 0xFF, "high byte: 0xFF & ~0x00 = 0xFF");
}

#[test]
fn e02_26_nvc_untouched() {
    // N, V, C flags should be unchanged by TSB/TRB
    let mut cpu = make_cpu();
    cpu.p = FLAG_M | FLAG_N | FLAG_V | FLAG_C; // Set all three flags
    cpu.a = 0x0F;
    let mut bus = puce16_core::cpu::testbus::FlatBus::new();
    bus.poke(0x10, &[0xF0]);
    cpu.pc = 0;
    bus.write(0, 0x04); // TSB d
    bus.write(1, 0x10);
    cpu.step(&mut bus);
    assert!(cpu.flag(FLAG_N), "N should be unchanged (still set)");
    assert!(cpu.flag(FLAG_V), "V should be unchanged (still set)");
    assert!(cpu.flag(FLAG_C), "C should be unchanged (still set)");
}
