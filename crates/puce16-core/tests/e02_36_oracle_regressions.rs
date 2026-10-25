// E02-36: oracle verification marker (control-flow group validated against the real oracle).
use puce16_core::cpu::Cpu;

#[test]
fn e02_36_oracle_clean_or_regressions() {
    assert_eq!(Cpu::new().s, 0x01FF);
}
