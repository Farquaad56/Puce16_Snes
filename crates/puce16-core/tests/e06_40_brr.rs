use puce16_core::apu::dsp::brr::{clamp16, decode_group};

#[test]
fn e06_40_clamp16() {
    assert_eq!(clamp16(40000), 32767);
    assert_eq!(clamp16(-40000), -32768);
    assert_eq!(clamp16(123), 123);
    assert_eq!(clamp16(-32768), -32768);
}

#[test]
fn e06_40_filter0_shift4() {
    let (samples, p1, p2) = decode_group(0x12, 0x34, 4, 0, 0, 0);
    assert_eq!(samples, [16, 32, 48, 64]);
    assert_eq!(p1, 64);
    assert_eq!(p2, 24);
}

#[test]
fn e06_40_filter0_shift2_shift0() {
    let (samples, p1, p2) = decode_group(0x12, 0x34, 2, 0, 0, 0);
    assert_eq!(samples, [4, 8, 12, 16]);
    assert_eq!(p1, 16);
    assert_eq!(p2, 6);

    let (samples, p1, p2) = decode_group(0x12, 0x34, 0, 0, 0, 0);
    assert_eq!(samples, [0, 2, 2, 4]);
    assert_eq!(p1, 4);
    assert_eq!(p2, 1);
}

#[test]
fn e06_40_negative_nibbles() {
    let (samples, p1, p2) = decode_group(0x8F, 0x01, 0, 0, 0, 0);
    assert_eq!(samples, [-8, -2, 0, 0]);
    assert_eq!(p1, 0);
    assert_eq!(p2, 0);
}

#[test]
fn e06_40_shift12_full_scale() {
    let (samples, p1, p2) = decode_group(0x77, 0x77, 12, 0, 0, 0);
    assert_eq!(samples, [28672, 28672, 28672, 28672]);
    assert_eq!(p1, 28672);
    assert_eq!(p2, 14336);

    let (samples, _, _) = decode_group(0x80, 0x00, 12, 0, 0, 0);
    assert_eq!(samples[0], -32768);
}

#[test]
fn e06_40_shift13_to_15() {
    let (samples, p1, p2) = decode_group(0x7F, 0x80, 13, 0, 0, 0);
    assert_eq!(samples, [0, -4096, -4096, 0]);
    assert_eq!(p1, 0);
    assert_eq!(p2, -2048);

    // Same result for shift 14 and 15
    let (s14, _, _) = decode_group(0x7F, 0x80, 14, 0, 0, 0);
    assert_eq!(samples, s14);
    let (s15, _, _) = decode_group(0x7F, 0x80, 15, 0, 0, 0);
    assert_eq!(samples, s15);
}

#[test]
fn e06_40_filter1() {
    let (samples, p1, p2) = decode_group(0, 0, 0, 1, 16000, 0);
    assert_eq!(samples, [15000, 14062, 13182, 12358]);
    assert_eq!(p1, 12358);
    assert_eq!(p2, 6591);

    let (samples, p1, p2) = decode_group(0x12, 0x34, 1, 1, 0, 0);
    assert_eq!(samples, [2, 4, 8, 14]);
    assert_eq!(p1, 14);
    assert_eq!(p2, 4);
}

#[test]
fn e06_40_filter2() {
    let (samples, p1, p2) = decode_group(0, 0, 0, 2, 1000, 500);
    assert_eq!(samples, [968, 906, 818, 708]);
    assert_eq!(p1, 708);
    assert_eq!(p2, 409);
}

#[test]
fn e06_40_filter3() {
    let (samples, p1, p2) = decode_group(0, 0, 0, 3, 1000, 500);
    assert_eq!(samples, [982, 950, 908, 858]);
    assert_eq!(p1, 858);
    assert_eq!(p2, 454);
}

#[test]
fn e06_40_clamp_then_wrap() {
    let (samples, p1, p2) = decode_group(0x70, 0x00, 12, 2, 30000, 0);
    assert_eq!(samples, [-2, -28130, 11912, -16460]);
    assert_eq!(p1, -16460);
    assert_eq!(p2, 5956);
}

#[test]
fn e06_40_history_continues() {
    let (a, p1, p2) = decode_group(0x12, 0x34, 4, 0, 0, 0);
    assert_eq!(a, [16, 32, 48, 64]);

    let (b, _, _) = decode_group(0, 0, 0, 1, p1, p2);
    assert_eq!(b, [60, 56, 52, 48]);
}

#[test]
fn e06_40_filter_bits_masked() {
    let (s5, _, _) = decode_group(0, 0, 0, 5, 1000, 500);
    let (s1, _, _) = decode_group(0, 0, 0, 1, 1000, 500);
    assert_eq!(s5, s1);
}
