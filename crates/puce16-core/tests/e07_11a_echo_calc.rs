// E07-11a: regression tests for the S-DSP echo FIR (tap 6 was skipped: loop `0..6` instead of `0..=6`).
// Found with blargg spc_dsp6 ("Echo/echo calc", Failed 03).
use puce16_core::apu::dsp::{Dsp, reg};

fn ram() -> Box<[u8; 65536]> {
    vec![0u8; 65536].into_boxed_slice().try_into().unwrap()
}

fn setup(dsp: &mut Dsp) {
    dsp.write(reg::ESA, 0x10);
    dsp.write(reg::EDL, 1);
    dsp.write(reg::FLG, 0x00);
    dsp.write(reg::EFB, 0);
    dsp.write(reg::MVOLL, 0);
    dsp.write(reg::MVOLR, 0);
    dsp.write(reg::EVOLL, 0x40);
    dsp.write(reg::EVOLR, 0x40);
}

#[test]
fn e07_11a_echo_calc_regression() {
    // Only FIR6 (the second newest history sample) is non-zero: output is delayed by ONE sample.
    let mut dsp = Dsp::new();
    let mut r = ram();
    setup(&mut dsp);
    dsp.write(reg::fir(6), 0x40);
    // Echo buffer slot k: L raw = 0x0100 * (k + 1), R raw = 0x0200 * (k + 1); history stores raw >> 1.
    for k in 0..4usize {
        let o = 0x1000 + 4 * k;
        r[o] = 0x00;
        r[o + 1] = (k + 1) as u8;
        r[o + 2] = 0x00;
        r[o + 3] = (2 * (k + 1)) as u8;
    }
    let mut audio = Vec::new();
    for _ in 0..3 {
        dsp.run_sample(&mut *r);
        audio.extend(dsp.take_audio());
    }
    // fir_l = (128 * (k + 1) * 64) >> 6 = 128 * (k + 1); main_l = (fir_l * 0x40) >> 7 = 64 * (k + 1)
    // sample n uses history slot k = n - 1, so sample 0 is silent.
    assert_eq!(audio[0], 0, "sample 0 L");
    assert_eq!(audio[1], 0, "sample 0 R");
    assert_eq!(audio[2], 64, "sample 1 L");
    assert_eq!(audio[3], 128, "sample 1 R");
    assert_eq!(audio[4], 128, "sample 2 L");
    assert_eq!(audio[5], 256, "sample 2 R");
}

#[test]
fn e07_11a_fir_all_eight_taps() {
    // Constant echo buffer, all 8 taps = 0x10: every tap contributes (128 * 16) >> 6 = 32 (L).
    let mut dsp = Dsp::new();
    let mut r = ram();
    setup(&mut dsp);
    for i in 0..8 {
        dsp.write(reg::fir(i), 0x10);
    }
    for k in 0..16usize {
        let o = 0x1000 + 4 * k;
        r[o] = 0x00;
        r[o + 1] = 0x01; // L raw 0x0100 -> 128
        r[o + 2] = 0x00;
        r[o + 3] = 0x02; // R raw 0x0200 -> 256
    }
    let mut audio = Vec::new();
    for _ in 0..9 {
        dsp.run_sample(&mut *r);
        audio.extend(dsp.take_audio());
    }
    // From the 8th sample the history is full: fir_l = 8 * 32 = 256, main_l = (256 * 0x40) >> 7 = 128.
    // With the skipped tap 6 it was 7 * 32 = 224 -> 112.
    assert_eq!(audio[14], 128, "sample 7 L");
    assert_eq!(audio[15], 256, "sample 7 R");
    assert_eq!(audio[16], 128, "sample 8 L");
    assert_eq!(audio[17], 256, "sample 8 R");
}
