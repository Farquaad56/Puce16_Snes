use puce16_core::apu::dsp::{Dsp, reg, voice::Mode};

fn ram() -> Box<[u8; 65536]> {
    vec![0u8; 65536].into_boxed_slice().try_into().unwrap()
}

#[test]
fn e06_46_soft_reset_flag() {
    let mut dsp = Dsp::new();
    // FLG=0xE0 has bit 7 set (soft reset)
    dsp.voice_mut(0).mode = Mode::Sustain;
    dsp.voice_mut(0).env = 0x500;
    let mut r = ram();
    dsp.run_sample(&mut *r);
    assert_eq!(dsp.voice(0).mode, Mode::Release);
    assert_eq!(dsp.voice(0).env, 0);
}

#[test]
fn e06_46_kon_polled_every_other_sample() {
    let mut dsp = Dsp::new();
    dsp.write(reg::DIR, 0x02);
    let mut r = ram();
    // Voice 0 directory entry: start=0x1000, loop=0x1010
    r[0x0204] = 0x00; r[0x0205] = 0x10;
    r[0x0206] = 0x10; r[0x0207] = 0x10;
    dsp.write(reg::FLG, 0x20);
    dsp.write(reg::vreg(0, reg::SRCN) as u8, 1);
    // BRR block at 0x1000: header=0xC3 (loop bit set), all samples same
    r[0x1000] = 0xC3;
    for i in 1..9 { r[0x1000 + i] = 0x77; }

    dsp.write(reg::KON, 0x01);
    // After 1 sample: every_other toggled to false, KON not yet polled
    dsp.run_sample(&mut *r);
    assert_eq!(dsp.new_kon(), 1);
    assert_eq!(dsp.voice(0).kon_delay, 0);
    assert_eq!(dsp.voice(0).mode, Mode::Release);

    // After 2 samples: every_other toggled to true, KON polled
    dsp.run_sample(&mut *r);
    assert_eq!(dsp.new_kon(), 0);
    assert_eq!(dsp.voice(0).kon_delay, 4);
    assert_eq!(dsp.voice(0).mode, Mode::Attack);
    assert_eq!(dsp.voice(0).addr, 0x1000);
}

#[test]
fn e06_46_kon_timeline_endx() {
    let mut dsp = Dsp::new();
    dsp.write(reg::DIR, 0x02);
    let mut r = ram();
    r[0x0204] = 0x00; r[0x0205] = 0x10;
    r[0x0206] = 0x10; r[0x0207] = 0x10;
    dsp.write(reg::FLG, 0x20);
    dsp.write(reg::vreg(0, reg::SRCN) as u8, 1);
    r[0x1000] = 0xC3;
    for i in 1..9 { r[0x1000 + i] = 0x77; }
    dsp.write(reg::VOLL, 0x7F);
    dsp.write(reg::VOLR, 0x7F);
    dsp.write(reg::MVOLL, 0x7F);
    dsp.write(reg::MVOLR, 0x7F);
    dsp.write(reg::GAIN, 0x7F);
    dsp.write(reg::ADSR1, 0);
    dsp.write(reg::PITCHL, 0x00);
    dsp.write(reg::PITCHH, 0x10);

    dsp.write(reg::KON, 0x01);
    // After 6 samples: kon_delay should be 0 (5 delay steps + 1 for polling)
    for _ in 0..6 { dsp.run_sample(&mut *r); }
    assert_eq!(dsp.voice(0).kon_delay, 0);
    let audio = dsp.take_audio();
    for &v in &audio { assert_eq!(v, 0); }

    // After 10 samples: ENDX should be 0 (still playing)
    for _ in 0..4 { dsp.run_sample(&mut *r); }
    assert_eq!(dsp.endx(), 0);

    // After 11 samples: ENDX should be set (block finished, loop bit set)
    dsp.run_sample(&mut *r);
    assert_eq!(dsp.endx(), 1);
    assert_eq!(dsp.read(0x7C), 1);
    assert_eq!(dsp.voice(0).addr, 0x1010);
}

#[test]
fn e06_46_first_audible_sample() {
    let mut dsp = Dsp::new();
    dsp.write(reg::DIR, 0x02);
    let mut r = ram();
    r[0x0204] = 0x00; r[0x0205] = 0x10;
    r[0x0206] = 0x10; r[0x0207] = 0x10;
    dsp.write(reg::FLG, 0x20);
    dsp.write(reg::vreg(0, reg::SRCN) as u8, 1);
    r[0x1000] = 0xC3;
    for i in 1..9 { r[0x1000 + i] = 0x77; }
    dsp.write(reg::VOLL, 0x7F);
    dsp.write(reg::VOLR, 0x7F);
    dsp.write(reg::MVOLL, 0x7F);
    dsp.write(reg::MVOLR, 0x7F);
    dsp.write(reg::GAIN, 0x7F);
    dsp.write(reg::ADSR1, 0);
    dsp.write(reg::PITCHL, 0x00);
    dsp.write(reg::PITCHH, 0x10);

    dsp.write(reg::KON, 0x01);
    // First audible sample at sample 7 (after 5 delay + 1 poll + 1 first output)
    for _ in 0..7 { dsp.run_sample(&mut *r); }
    let audio = dsp.take_audio();
    assert_eq!(audio.len(), 14);
    for i in 0..12 { assert_eq!(audio[i], 0); }
    assert_ne!(audio[12], 0);
    assert_eq!(audio[12], audio[13]);
    assert!((27000..=29000).contains(&audio[12]));
}

#[test]
fn e06_46_kon_and_koff_same_voice_is_silent() {
    let mut dsp = Dsp::new();
    dsp.write(reg::DIR, 0x02);
    let mut r = ram();
    r[0x0204] = 0x00; r[0x0205] = 0x10;
    r[0x0206] = 0x10; r[0x0207] = 0x10;
    dsp.write(reg::FLG, 0x20);
    dsp.write(reg::vreg(0, reg::SRCN) as u8, 1);
    r[0x1000] = 0xC3;
    for i in 1..9 { r[0x1000 + i] = 0x77; }
    dsp.write(reg::VOLL, 0x7F);
    dsp.write(reg::VOLR, 0x7F);
    dsp.write(reg::MVOLL, 0x7F);
    dsp.write(reg::MVOLR, 0x7F);
    dsp.write(reg::GAIN, 0x7F);

    // KOFF before KON for same voice
    dsp.write(reg::KOFF, 0x01);
    dsp.write(reg::KON, 0x01);
    for _ in 0..20 { dsp.run_sample(&mut *r); }
    let audio = dsp.take_audio();
    for &v in &audio { assert_eq!(v, 0); }
    assert_eq!(dsp.voice(0).mode, Mode::Release);
}

#[test]
fn e06_46_koff_releases_a_sounding_voice() {
    let mut dsp = Dsp::new();
    dsp.write(reg::FLG, 0x20);
    dsp.write(reg::GAIN, 0x40);
    dsp.voice_mut(0).mode = Mode::Sustain;
    dsp.voice_mut(0).env = 0x400;

    let mut r = ram();
    // After 1 sample: still sustain (KOFF not yet polled)
    dsp.run_sample(&mut *r);
    assert_eq!(dsp.voice(0).mode, Mode::Sustain);
    assert_eq!(dsp.voice(0).env, 0x400);

    dsp.write(reg::KOFF, 0x01);
    // After 2 samples: KOFF polled, voice released
    dsp.run_sample(&mut *r);
    assert_eq!(dsp.voice(0).mode, Mode::Release);
    assert_eq!(dsp.voice(0).env, 0x3F8);
}

#[test]
fn e06_46_mix_noise_voice() {
    let mut dsp = Dsp::new();
    dsp.write(reg::FLG, 0x20);
    dsp.write(reg::NON, 1);
    dsp.write(reg::GAIN, 0x40);
    dsp.write(reg::VOLL, 0x40);
    dsp.write(reg::VOLR, 0x40);
    dsp.write(reg::MVOLL, 0x40);
    dsp.write(reg::MVOLR, 0x40);
    dsp.voice_mut(0).mode = Mode::Sustain;

    let mut r = ram();
    for _ in 0..3 { dsp.run_sample(&mut *r); }
    let audio = dsp.take_audio();
    // Noise output: (-8192 * 0x40) >> 7 = -4096
    for &v in &audio { assert_eq!(v, -4096); }
}

#[test]
fn e06_46_mute_flag() {
    let mut dsp = Dsp::new();
    dsp.write(reg::FLG, 0x60); // bit 6 set (mute)
    dsp.write(reg::NON, 1);
    dsp.write(reg::GAIN, 0x40);
    dsp.write(reg::VOLL, 0x40);
    dsp.write(reg::VOLR, 0x40);
    dsp.write(reg::MVOLL, 0x40);
    dsp.write(reg::MVOLR, 0x40);
    dsp.voice_mut(0).mode = Mode::Sustain;

    let mut r = ram();
    for _ in 0..3 { dsp.run_sample(&mut *r); }
    let audio = dsp.take_audio();
    for &v in &audio { assert_eq!(v, 0); }
}

#[test]
fn e06_46_master_volume_signed() {
    let mut dsp = Dsp::new();
    dsp.write(reg::FLG, 0x20);
    dsp.write(reg::NON, 1);
    dsp.write(reg::GAIN, 0x40);
    dsp.write(reg::VOLL, 0x40);
    dsp.write(reg::VOLR, 0x40);
    dsp.write(reg::MVOLL, 0xC0); // i8 = -64
    dsp.write(reg::MVOLR, 0x40);
    dsp.voice_mut(0).mode = Mode::Sustain;

    let mut r = ram();
    dsp.run_sample(&mut *r);
    let audio = dsp.take_audio();
    // (-8192 * 0x40) >> 7 = -4096, then * -64 >> 7 = +4096 for left
    // (-8192 * 0x40) >> 7 = -4096, then * 64 >> 7 = -4096 for right
    assert_eq!(audio[0], 4096);
    assert_eq!(audio[1], -4096);
}

#[test]
fn e06_46_endx_mirrored() {
    let mut dsp = Dsp::new();
    dsp.debug_set_endx(0x81);
    dsp.write(reg::FLG, 0x20);

    let mut r = ram();
    dsp.run_sample(&mut *r);
    assert_eq!(dsp.read(0x7C), 0x81);
}

#[test]
fn e06_46_counter_and_noise_advance() {
    let mut dsp = Dsp::new();
    dsp.write(reg::FLG, 0x3F); // max noise rate

    let mut r = ram();
    dsp.run_sample(&mut *r);
    assert_eq!(dsp.counter(), 0x77FF);
    assert_eq!(dsp.noise(), 0x2000);

    dsp.run_sample(&mut *r);
    assert_eq!(dsp.noise(), 0x1000);
}

#[test]
fn e06_46_two_values_per_sample() {
    let mut dsp = Dsp::new();
    dsp.write(reg::FLG, 0x20);

    let mut r = ram();
    for _ in 0..5 { dsp.run_sample(&mut *r); }
    let audio = dsp.take_audio();
    assert_eq!(audio.len(), 10);
    let empty = dsp.take_audio();
    assert_eq!(empty.len(), 0);
}

#[test]
fn e06_46_determinism() {
    fn make_dsp() -> Dsp {
        let mut d = Dsp::new();
        d.write(reg::FLG, 0x20);
        d.write(reg::NON, 1);
        d.write(reg::GAIN, 0x40);
        d.write(reg::VOLL, 0x40);
        d.write(reg::VOLR, 0x40);
        d.write(reg::MVOLL, 0x7F);
        d.write(reg::MVOLR, 0x7F);
        d.voice_mut(0).mode = Mode::Sustain;
        d
    }

    let mut dsp1 = make_dsp();
    let mut dsp2 = make_dsp();

    let mut r1 = ram();
    let mut r2 = ram();

    for _ in 0..5 {
        dsp1.run_sample(&mut *r1);
        dsp2.run_sample(&mut *r2);
    }

    let a1 = dsp1.take_audio();
    let a2 = dsp2.take_audio();
    assert_eq!(a1, a2);

    use puce16_core::hash::Fnv64;
    let mut h1 = Fnv64::new();
    let mut h2 = Fnv64::new();
    dsp1.hash_into(&mut h1);
    dsp2.hash_into(&mut h2);
    assert_eq!(h1.finish(), h2.finish());
}
