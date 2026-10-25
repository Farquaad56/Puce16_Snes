use puce16_core::cpu::Cpu;

#[test]
fn e02_26d_oracle_clean_or_regressions() {
    let cpu = Cpu::new();
    assert_eq!(cpu.p, 0x34);
}
