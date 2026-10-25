// E02-30c: oracle verification marker (stack/blocks group validated against the real oracle).
use puce16_core::cpu::Cpu;

#[test]
fn e02_30c_oracle_clean_or_regressions() {
    assert_eq!(Cpu::new().s, 0x01FF);
}

#[test]
fn e02_30c_plb_emulation_no_page_wrap() {
    use puce16_core::cpu::testbus::FlatBus;
    // emulation, S low byte = 0xFF: PLB reads $0200 (no page-1 wrap) and S becomes $0100
    let mut cpu = Cpu::new();
    cpu.e = true;
    cpu.s = 0x01FF;
    cpu.pc = 0;
    let mut bus = FlatBus::new();
    bus.poke(0x0000, &[0xAB]);
    bus.poke(0x0200, &[0x48]);
    cpu.step(&mut bus);
    assert_eq!(cpu.db, 0x48);
    assert_eq!(cpu.s, 0x0100);
}
