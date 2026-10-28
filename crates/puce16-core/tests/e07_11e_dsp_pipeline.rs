// E07-11e: S-DSP register pipeline timing (reference: blargg SPC_DSP.cpp), clocked mode.
// A "sample" = `run_sample_deferred` (computes everything), then `clock()` once per SMP tick;
// ENVX/OUTX/ENDX reach the register file at the clock where the real DSP stores them.

use puce16_core::apu::dsp::voice::Mode;
use puce16_core::apu::dsp::{reg, Dsp};

fn ram() -> Box<[u8; 65536]> {
    vec![0u8; 65536].into_boxed_slice().try_into().unwrap()
}

fn envx0(dsp: &mut Dsp) -> u8 {
    dsp.read(reg::vreg(0, reg::ENVX) as u8)
}

/// Voice 0: noise source, GAIN direct $40 (env = $400), sustain, ADSR off.
fn dsp_gain_40() -> Dsp {
    let mut dsp = Dsp::new();
    dsp.set_reference_order(true);
    dsp.write(reg::FLG, 0x00); // no soft reset, no mute
    dsp.write(reg::NON, 0x01);
    dsp.write(reg::vreg(0, reg::GAIN) as u8, 0x40);
    dsp.voice_mut(0).mode = Mode::Sustain;
    dsp.debug_set_noise(0x4000);
    dsp
}

fn clocks(dsp: &mut Dsp, n: usize) {
    for _ in 0..n {
        dsp.clock();
    }
}

#[test]
fn e07_11e_envx_published_at_clock_4() {
    let mut r = ram();
    let mut dsp = dsp_gain_40();
    // Sample 1: ENVX of this sample is the envelope BEFORE the update (0), env becomes $400.
    dsp.run_sample_deferred(&mut r);
    assert_eq!(dsp.voice(0).env, 0x400);
    clocks(&mut dsp, 31);
    assert_eq!(envx0(&mut dsp), 0x00);
    // Sample 2: ENVX = $40 but only visible once clock 4 (V9 of voice 0) has run.
    dsp.run_sample_deferred(&mut r);
    assert_eq!(envx0(&mut dsp), 0x00);
    clocks(&mut dsp, 4); // clocks 0..3
    assert_eq!(envx0(&mut dsp), 0x00);
    clocks(&mut dsp, 1); // clock 4
    assert_eq!(envx0(&mut dsp), 0x40);
}

#[test]
fn e07_11e_envx_write_goes_through_shared_buffer() {
    let mut r = ram();
    let mut dsp = dsp_gain_40();
    dsp.run_sample_deferred(&mut r);
    clocks(&mut dsp, 31);
    dsp.run_sample_deferred(&mut r);
    clocks(&mut dsp, 4); // after V7 (clock 2) loaded the buffer, before V9 (clock 4)
    dsp.write(reg::vreg(0, reg::ENVX) as u8, 0x88);
    clocks(&mut dsp, 1);
    assert_eq!(envx0(&mut dsp), 0x88, "write before V9 is stored back by V9");
    clocks(&mut dsp, 26);
    // Next sample: a write BEFORE V7 (clock 2) is overwritten by the reload.
    dsp.run_sample_deferred(&mut r);
    clocks(&mut dsp, 1); // clock 0
    dsp.write(reg::vreg(0, reg::ENVX) as u8, 0x88);
    assert_eq!(envx0(&mut dsp), 0x88);
    clocks(&mut dsp, 4); // clocks 1..4
    assert_eq!(envx0(&mut dsp), 0x40);
}

#[test]
fn e07_11e_adsr1_latched_at_clock_21() {
    let adsr1 = reg::vreg(0, reg::ADSR1) as u8;
    // ADSR1 = $FF (ADSR on, sustain rate 0 = never changes) written BEFORE the latch clock:
    // the envelope sees ADSR and stays 0.
    let mut r = ram();
    let mut dsp = dsp_gain_40();
    clocks(&mut dsp, 21); // clocks 0..20
    dsp.write(adsr1, 0xFF);
    clocks(&mut dsp, 1); // clock 21: latch
    dsp.run_sample_deferred(&mut r);
    assert_eq!(dsp.voice(0).env, 0);
    // Same write AFTER the latch clock: this sample still runs in GAIN mode (env = $400).
    let mut dsp = dsp_gain_40();
    clocks(&mut dsp, 22); // clocks 0..21
    dsp.write(adsr1, 0xFF);
    dsp.run_sample_deferred(&mut r);
    assert_eq!(dsp.voice(0).env, 0x400);
}

#[test]
fn e07_11e_output_uses_envelope_before_update() {
    let mut r = ram();
    let mut dsp = dsp_gain_40();
    // Sample 1: env 0 -> output 0, then env = $400.
    dsp.run_sample_deferred(&mut r);
    assert_eq!(dsp.voice(0).out, 0);
    assert_eq!(dsp.voice(0).env, 0x400);
    // Sample 2: noise sample -32768 * $400 >> 11 = -16384.
    dsp.run_sample_deferred(&mut r);
    assert_eq!(dsp.voice(0).out, -16384);
}

#[test]
fn e07_11e_gain_e0_threshold_regression() {
    // GAIN mode 7 (bent line): +32 per step, +8 once hidden_env >= $600.
    let gain = reg::vreg(0, reg::GAIN) as u8;
    // $FF (rate 31: env follows every step).
    let mut dsp = Dsp::new();
    dsp.write(gain, 0xFF);
    dsp.voice_mut(0).mode = Mode::Sustain;
    dsp.voice_mut(0).env = 0x5F0;
    dsp.voice_mut(0).hidden_env = 0x5F0;
    dsp.debug_run_envelope(0); // below threshold: +32
    assert_eq!((dsp.voice(0).env, dsp.voice(0).hidden_env), (0x610, 0x610));
    dsp.debug_run_envelope(0); // hidden_env >= $600: +8
    assert_eq!((dsp.voice(0).env, dsp.voice(0).hidden_env), (0x618, 0x618));
    // $E0 (rate 0: the visible env never moves, only hidden_env, computed from env).
    let mut dsp = Dsp::new();
    dsp.write(gain, 0xE0);
    dsp.voice_mut(0).mode = Mode::Sustain;
    dsp.voice_mut(0).env = 0x5F0;
    dsp.voice_mut(0).hidden_env = 0x5F0;
    dsp.debug_run_envelope(0);
    assert_eq!((dsp.voice(0).env, dsp.voice(0).hidden_env), (0x5F0, 0x610));
    dsp.debug_run_envelope(0); // hidden_env >= $600: env + 8
    assert_eq!((dsp.voice(0).env, dsp.voice(0).hidden_env), (0x5F0, 0x5F8));
    dsp.debug_run_envelope(0); // hidden_env < $600 again: env + 32
    assert_eq!((dsp.voice(0).env, dsp.voice(0).hidden_env), (0x5F0, 0x610));
}
