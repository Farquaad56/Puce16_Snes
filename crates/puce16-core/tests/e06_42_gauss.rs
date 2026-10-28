use puce16_core::apu::dsp::voice::gauss_interp;
use puce16_core::apu::dsp::gauss::GAUSS;

fn clamp16(v: i32) -> i32 {
    if v > 32767 { 32767 } else if v < -32768 { -32768 } else { v }
}

#[test]
fn e06_42_zero_window() {
    for f in [0, 1, 100, 255] {
        assert_eq!(gauss_interp(0, 0, 0, 0, f), 0);
    }
}

#[test]
fn e06_42_newest_has_zero_weight_at_f0() {
    // GAUSS[0] == 0, so d has zero weight at frac=0
    assert_eq!(gauss_interp(0, 0, 0, 12345, 0), 0);
}

#[test]
fn e06_42_constant_positive() {
    for f in [0, 1, 64, 127, 128, 200, 255] {
        let r = gauss_interp(10000, 10000, 10000, 10000, f);
        assert!(9980 <= r && r <= 10020, "f={} r={}", f, r);
        assert_eq!(r % 2, 0, "f={} r={} not even", f, r);
    }
}

#[test]
fn e06_42_constant_negative() {
    for f in [0, 1, 64, 127, 128, 200, 255] {
        let r = gauss_interp(-10000, -10000, -10000, -10000, f);
        assert!(-10020 <= r && r <= -9980, "f={} r={}", f, r);
        assert_eq!(r & 1, 0, "f={} r={} not even", f, r);
    }
}

#[test]
fn e06_42_matches_reference_formula() {
    // Reference implementation using i64 arithmetic and explicit as i16 wrap
    fn gauss_ref(a: i32, b: i32, c: i32, d: i32, f: usize) -> i32 {
        let mut o = ((GAUSS[255 - f] as i64) * (a as i64)) >> 11;
        o += ((GAUSS[511 - f] as i64) * (b as i64)) >> 11;
        o += ((GAUSS[256 + f] as i64) * (c as i64)) >> 11;
        let wrapped = (o as i32) as i16 as i32;
        let mut result = wrapped;
        let d_term = (((GAUSS[f] as i64) * (d as i64)) >> 11) as i32;
        result += d_term;
        clamp16(result) & !1
    }

    let windows = [
        (1000, -2000, 3000, -4000),
        (32767, 32767, 32767, 32767),
        (-32768, -32768, -32768, 0),
        (-32768, -32768, -32768, -32768),
    ];

    for (a, b, c, d) in windows {
        for f in 0..=255 {
            let got = gauss_interp(a, b, c, d, f);
            let expected = gauss_ref(a, b, c, d, f);
            assert_eq!(got, expected, "window=({}, {}, {}, {}) f={} got={} expected={}", a, b, c, d, f, got, expected);
        }
    }
}

#[test]
fn e06_42_clamps_to_16_bits() {
    for f in 0..=255 {
        let r = gauss_interp(32767, 32767, 32767, 32767, f);
        assert!(r >= -32768 && r <= 32767, "f={} r={}", f, r);
    }
}

#[test]
fn e06_42_frac_masked() {
    let a = gauss_interp(100, 200, 300, 400, 256);
    let b = gauss_interp(100, 200, 300, 400, 0);
    assert_eq!(a, b);
}
