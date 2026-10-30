use puce16_core::coprocessor::sdd1::tables::{RUNCOUNT, EVOLUTION};

#[test]
fn e09_01_runcount_spot_values() {
    assert_eq!(RUNCOUNT[2], 1);
    assert_eq!(RUNCOUNT[3], 0);
    assert_eq!(RUNCOUNT[4], 3);
    assert_eq!(RUNCOUNT[7], 0);
    assert_eq!(RUNCOUNT[8], 7);
    assert_eq!(RUNCOUNT[0x80], 0x7F);
    assert_eq!(RUNCOUNT[0xFA], 0x50);
    assert_eq!(RUNCOUNT[0xFF], 0);
}

#[test]
fn e09_01_runcount_block_range() {
    assert_eq!(RUNCOUNT[0], 0);
    assert_eq!(RUNCOUNT[1], 0);
    for n in 1..=7 {
        let start = 1 << n;
        let end = 1 << (n + 1);
        let max_val = 1 << n;
        for idx in start..end {
            assert!(RUNCOUNT[idx] < max_val, "idx={}", idx);
        }
    }
}

#[test]
fn e09_01_evolution_spot_values() {
    assert_eq!(EVOLUTION[0], (0, 25, 25));
    assert_eq!(EVOLUTION[1], (0, 2, 1));
    assert_eq!(EVOLUTION[24], (7, 24, 23));
    assert_eq!(EVOLUTION[25], (0, 26, 1));
    assert_eq!(EVOLUTION[32], (7, 24, 22));
}

#[test]
fn e09_01_evolution_next_states_in_range() {
    for entry in EVOLUTION.iter() {
        let (code_number, next_if_mps, next_if_lps) = *entry;
        assert!(code_number <= 7);
        assert!(next_if_mps < 33);
        assert!(next_if_lps < 33);
    }
}
