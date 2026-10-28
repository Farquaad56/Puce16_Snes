use puce16_core::apu::dsp::{Dsp, reg};

#[test]
fn e06_44_counter_wraps() {
    let mut dsp = Dsp::new();
    // counter starts at 0; first step wraps to 0x77FF
    dsp.debug_step_counter_noise();
    assert_eq!(dsp.counter(), 0x77FF);

    // second step decrements normally
    dsp.debug_step_counter_noise();
    assert_eq!(dsp.counter(), 0x77FE);

    // set counter to 1, step -> 0
    dsp.debug_set_counter(1);
    dsp.debug_step_counter_noise();
    assert_eq!(dsp.counter(), 0);

    // step again -> wraps to 0x77FF
    dsp.debug_step_counter_noise();
    assert_eq!(dsp.counter(), 0x77FF);
}

#[test]
fn e06_44_counter_cycle_length() {
    let mut dsp = Dsp::new();
    dsp.debug_set_counter(0x77FF);

    // After 0x77FF steps, counter should be 0
    for _ in 0..0x77FF {
        dsp.debug_step_counter_noise();
    }
    assert_eq!(dsp.counter(), 0);

    // One more step wraps to 0x77FF (full cycle = 0x7800)
    dsp.debug_step_counter_noise();
    assert_eq!(dsp.counter(), 0x77FF);
}

#[test]
fn e06_44_noise_rate0_frozen() {
    let mut dsp = Dsp::new();
    // Rate 0 means poll never fires (PERIOD[0] = 0)
    dsp.write(reg::FLG, 0x20);
    for _ in 0..100 {
        dsp.debug_step_counter_noise();
    }
    assert_eq!(dsp.noise(), 0x4000);
}

#[test]
fn e06_44_noise_rate31_sequence() {
    let mut dsp = Dsp::new();
    // Rate 31 means poll fires every sample (PERIOD[31] = 1)
    dsp.write(reg::FLG, 0x3F);

    dsp.debug_step_counter_noise();
    assert_eq!(dsp.noise(), 0x2000);

    dsp.debug_step_counter_noise();
    assert_eq!(dsp.noise(), 0x1000);

    dsp.debug_step_counter_noise();
    assert_eq!(dsp.noise(), 0x0800);

    dsp.debug_step_counter_noise();
    assert_eq!(dsp.noise(), 0x0400);
}

#[test]
fn e06_44_noise_feedback() {
    let mut dsp = Dsp::new();
    dsp.write(reg::FLG, 0x3F);
    dsp.debug_set_noise(0x4000);

    // Step 13 times to reach noise=0x0002
    for _ in 0..13 {
        dsp.debug_step_counter_noise();
    }
    assert_eq!(dsp.noise(), 0x0002);

    // 14th step -> 0x4001 (feedback kicks in)
    dsp.debug_step_counter_noise();
    assert_eq!(dsp.noise(), 0x4001);

    // 15th step -> 0x6000
    dsp.debug_step_counter_noise();
    assert_eq!(dsp.noise(), 0x6000);

    // 16th step -> 0x3000
    dsp.debug_step_counter_noise();
    assert_eq!(dsp.noise(), 0x3000);
}

#[test]
fn e06_44_noise_rate5_cadence() {
    let mut dsp = Dsp::new();
    // Rate 5: PERIOD[5] = 768, so updates every 768 samples
    dsp.write(reg::FLG, 0x25);
    dsp.debug_set_counter(0);
    dsp.debug_set_noise(0x4000);

    // 30720 / 768 = 40 noise updates expected
    for _ in 0..30720 {
        dsp.debug_step_counter_noise();
    }

    assert_eq!(dsp.noise(), 0x0014);
    assert_eq!(dsp.counter(), 0);
}

#[test]
fn e06_44_noise_ignores_other_flg_bits() {
    let mut dsp = Dsp::new();
    // FLG=0xFF has rate bits 0x1F (31), same as 0x1F
    dsp.write(reg::FLG, 0xFF);
    dsp.debug_set_noise(0x4000);

    dsp.debug_step_counter_noise();
    assert_eq!(dsp.noise(), 0x2000);
}

#[test]
fn e06_44_noise_accessors() {
    let mut dsp = Dsp::new();
    assert_eq!(dsp.noise(), 0x4000);

    dsp.debug_set_noise(7);
    assert_eq!(dsp.noise(), 7);
}
