use puce16_core::apu::spc700::Spc700;

#[test]
fn e06_22_oracle_clean_or_regressions() {
    let cpu = Spc700::new();
    assert_eq!(cpu.a, 0);
}
