use puce16_core::apu::spc700::{Spc700, testbus::FlatSmpBus};

#[test]
fn e06_18_oracle_clean() {
    let cpu = Spc700::new();
    assert_eq!(cpu.a, 0);
}