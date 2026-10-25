// E02-27: oracle regression tests (SBC decimal rewritten from docs/specs/data/bcd_reference.md).
use puce16_core::cpu::testbus::FlatBus;
use puce16_core::cpu::{Bus, Cpu, FLAG_C, FLAG_D};

#[test]
fn e02_27_oracle_clean_or_regressions() {
    assert_eq!(Cpu::new().p, 0x34);
}

#[test]
fn e02_27_sbc_decimal_16bit_borrow_chain() {
    // native, 16-bit A, D=1, C=1: 0x0100 - 0x0001 = 0x0099 (BCD), C stays 1
    let mut cpu = Cpu::new();
    cpu.e = false;
    cpu.p = FLAG_D | FLAG_C;
    cpu.a = 0x0100;
    cpu.pc = 0;
    let mut bus = FlatBus::new();
    for (i, b) in [0xE9u8, 0x01, 0x00].iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x0099);
    assert!(cpu.p & FLAG_C != 0);
}
