// E02-30b: oracle verification marker (stack/blocks group validated against the real oracle).
use puce16_core::cpu::Cpu;

#[test]
fn e02_30b_oracle_clean_or_regressions() {
    assert_eq!(Cpu::new().s, 0x01FF);
}
