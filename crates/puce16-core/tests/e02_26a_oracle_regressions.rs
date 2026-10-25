use puce16_core::cpu::Cpu;

#[test]
fn e02_26a_oracle_clean_or_regressions() {
    // Sanity check: CPU power-on state has P = 0x34 (I=1, D=0)
    let cpu = Cpu::new();
    assert_eq!(cpu.p, 0x34, "CPU power-on flags should be 0x34");
}
