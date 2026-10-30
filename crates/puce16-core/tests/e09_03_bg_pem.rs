//! E09-03 tests: BG (bit generators) and PEM (probability estimation module).

use puce16_core::coprocessor::sdd1::Decompressor;

fn lcg_bytes(seed: u32, n: usize) -> Vec<u8> {
    let mut x = seed;
    (0..n)
        .map(|_| {
            x = x.wrapping_mul(1664525).wrapping_add(1013904223);
            (x >> 24) as u8
        })
        .collect()
}

#[test]
fn e09_03_bg_bit_n1_sequence() {
    let mut data = lcg_bytes(0x4444, 64);
    data[0] = 0x0f;
    let rom = |a: u32| data[(a as usize) % data.len()];

    let mut d = Decompressor::new();
    d.init(0, &rom);

    // Expected: (1,t) (1,t) (0,f) (0,t) (1,t) (0,f) (0,t) (0,f) (1,t) (0,f) (1,t) (1,t) (0,f) (0,t) (0,f) (0,t)
    let expected = [
        (1, true),  (1, true),  (0, false), (0, true),
        (1, true),  (0, false), (0, true),  (0, false),
        (1, true),  (0, false), (1, true),  (1, true),
        (0, false), (0, true),  (0, false), (0, true),
    ];

    for i in 0..16 {
        let result = d.bg_bit(1, &rom);
        assert_eq!(result, expected[i], "bg_bit n=1 call {}", i + 1);
    }
}

#[test]
fn e09_03_bg_bit_n0_sequence() {
    let mut data = lcg_bytes(0x4444, 64);
    data[0] = 0x0f;
    let rom = |a: u32| data[(a as usize) % data.len()];

    let mut d = Decompressor::new();
    d.init(0, &rom);

    // Expected: (1,t) (1,t) (1,t) (1,t) (0,t) (1,t) (1,t) (0,t) (1,t) (0,t) (1,t) (0,t) (1,t) (1,t) (0,t) (0,t)
    let expected = [
        (1, true),  (1, true),  (1, true),  (1, true),
        (0, true),  (1, true),  (1, true),  (0, true),
        (1, true),  (0, true),  (1, true),  (0, true),
        (1, true),  (1, true),  (0, true),  (0, true),
    ];

    for i in 0..16 {
        let result = d.bg_bit(0, &rom);
        assert_eq!(result, expected[i], "bg_bit n=0 call {}", i + 1);
    }
}

#[test]
fn e09_03_pem_bit_ctx5_sequence() {
    let mut data = lcg_bytes(0x7777, 64);
    data[0] = 0x0f;
    let rom = |a: u32| data[(a as usize) % data.len()];

    let mut d = Decompressor::new();
    d.init(0, &rom);

    // Expected: 1001101110101011111111111001000110000001
    let expected_bits = [
        1, 0, 0, 1, 1, 0, 1, 1, 1, 0, 1, 0, 1, 0, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 0, 1,
        1, 0, 0, 0, 0, 0, 0, 1,
    ];

    for i in 0..40 {
        let bit = d.pem_bit(5, &rom);
        assert_eq!(bit, expected_bits[i], "pem_bit ctx=5 call {}", i + 1);
    }

    // Final state: status[5] == 1, mps[5] == 0
    let (status, mps) = d.pem_state(5);
    assert_eq!(status, 1, "pem_state ctx=5 status");
    assert_eq!(mps, 0, "pem_state ctx=5 mps");
}

#[test]
fn e09_03_pem_bit_ctx17_sequence() {
    let mut data = lcg_bytes(0x8888, 64);
    data[0] = 0x0f;
    let rom = |a: u32| data[(a as usize) % data.len()];

    let mut d = Decompressor::new();
    d.init(0, &rom);

    // Expected: 1001000011101011010001111000101110001001
    let expected_bits = [
        1, 0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 0, 1, 0, 1, 1,
        0, 1, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1, 0, 1, 1,
        1, 0, 0, 0, 1, 0, 0, 1,
    ];

    for i in 0..40 {
        let bit = d.pem_bit(17, &rom);
        assert_eq!(bit, expected_bits[i], "pem_bit ctx=17 call {}", i + 1);
    }

    // Final state: status[17] == 2, mps[17] == 0
    let (status, mps) = d.pem_state(17);
    assert_eq!(status, 2, "pem_state ctx=17 status");
    assert_eq!(mps, 0, "pem_state ctx=17 mps");
}