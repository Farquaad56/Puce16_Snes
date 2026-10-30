//! E09-02 tests: S-DD1 decompressor struct, init, IM (code_word), GCD (run_count).

use puce16_core::coprocessor::sdd1::Decompressor;

/// LCG byte generator matching docs/tools/sdd1_ref.py
fn lcg_bytes(seed: u32, n: usize) -> Vec<u8> {
    let mut x = seed;
    (0..n)
        .map(|_| {
            x = x.wrapping_mul(1664525).wrapping_add(1013904223);
            (x >> 24) as u8
        })
        .collect()
}

/// Cyclic ROM closure for tests
fn make_rom(data: Vec<u8>) -> impl Fn(u32) -> u8 {
    move |a: u32| data[(a as usize) % data.len()]
}

#[test]
fn e09_02_im_code_word_sequence() {
    let data = vec![0xB6, 0x4D, 0x00, 0xFF, 0x5A, 0xC3, 0x11, 0x22];
    let rom = make_rom(data);

    let mut d = Decompressor::new();
    d.init(0, &rom);

    // Expected: (code_word_result, offset_after, bit_count_after)
    let expected = [
        (96u8, 0u32, 5u8),
        (201, 1, 1),
        (154, 1, 2),
        (52, 1, 3),
        (104, 1, 4),
        (208, 2, 4),
        (0, 2, 5),
        (0, 2, 6),
        (0, 2, 7),
        (0, 3, 0),
    ];

    let code_lengths = [3u8, 3, 0, 1, 2, 7, 4, 5, 0, 6];

    for (i, &cl) in code_lengths.iter().enumerate() {
        let cw = d.code_word(cl, &rom);
        let (off, bc) = d.im_position();
        assert_eq!(cw, expected[i].0, "code_word[{}] result", i);
        assert_eq!(off, expected[i].1, "code_word[{}] offset", i);
        assert_eq!(bc, expected[i].2, "code_word[{}] bit_count", i);
    }
}

#[test]
fn e09_02_init_reads_header_at_start() {
    // ROM that returns 0xC0 only at address 0xC00100
    let rom = |a: u32| if a == 0xC00100 { 0xC0 } else { 0 };

    let mut d = Decompressor::new();
    d.init(0xC00100, &rom);

    // IM should be at start with bit_count=4 (header nibble consumed)
    assert_eq!(d.im_position(), (0xC00100, 4));
}

#[test]
fn e09_02_run_count_lps_cases() {
    // First set: [0x0F, 0xA5, 0x5A, 0x3C] + 60 zeros
    let mut data1 = vec![0x0Fu8, 0xA5, 0x5A, 0x3C];
    data1.extend(vec![0u8; 60]);
    let rom1 = make_rom(data1);

    let expected1: [(u8, bool); 8] = [
        (0, true), (0, true), (0, true), (0, true),
        (0, true), (16, true), (16, true), (80, true),
    ];

    for n in 0..8u8 {
        let mut d = Decompressor::new();
        d.init(0, &rom1);
        let result = d.run_count(n, &rom1);
        assert_eq!(result, expected1[n as usize], "n={} LPS case 1", n);
    }

    // Second set: [0x08, 0x40, 0x00, 0x00] + 60 zeros
    let mut data2 = vec![0x08u8, 0x40, 0x00, 0x00];
    data2.extend(vec![0u8; 60]);
    let rom2 = make_rom(data2);

    let expected2: [(u8, bool); 8] = [
        (0, true), (1, true), (3, true), (7, true),
        (15, true), (15, true), (47, true), (111, true),
    ];

    for n in 0..8u8 {
        let mut d = Decompressor::new();
        d.init(0, &rom2);
        let result = d.run_count(n, &rom2);
        assert_eq!(result, expected2[n as usize], "n={} LPS case 2", n);
    }
}

#[test]
fn e09_02_run_count_mps_cases() {
    // MPS cases: all zeros in stream means code_word returns values < 128
    let data = vec![0u8; 64];
    let rom = make_rom(data);

    for n in 0..8u8 {
        let mut d = Decompressor::new();
        d.init(0, &rom);
        let (count, is_lps) = d.run_count(n, &rom);
        assert!(!is_lps, "n={} should be MPS", n);
        assert_eq!(count, 1 << n, "n={} count should be {}", n, 1 << n);
    }

    // Verify IM position after one run_count call (code_word consumes 5 bits)
    let mut d = Decompressor::new();
    d.init(0, &rom);
    d.run_count(0, &rom);
    assert_eq!(d.im_position(), (0, 5));
}
