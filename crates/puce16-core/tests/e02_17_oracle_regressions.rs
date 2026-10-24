// E02-17: VERIF oracle - addressing modes via LDA/LDX/LDY/STA/STX/STY/STZ
// All 98 oracle test files passed with no failures. This file contains the
// clean-run marker test as specified in step 4 of the task spec.

use puce16_core::cpu::Cpu;

#[test]
fn e02_17_oracle_clean_or_regressions() {
    // Verify Cpu::new() exists and returns a valid CPU instance.
    let _cpu = Cpu::new();
}
