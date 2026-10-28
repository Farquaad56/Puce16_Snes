use puce16_core::apu::dsp::{Dsp, reg};
use puce16_core::apu::dsp::voice::Mode;

fn ram() -> Box<[u8; 65536]> {
    vec![0u8; 65536].into_boxed_slice().try_into().unwrap()
}

fn vreg(x: usize, off: u8) -> u8 {
    reg::vreg(x, off) as u8
}

#[test]
fn e06_45_noise_voice_output() {
    let mut dsp = Dsp::new();
    let r = ram();
    // Voice 0: noise, GAIN=0x40 (direct mode m=1), volumes 0x40/0xC0, pitch 0x0800
    dsp.write(reg::NON, 0x01);
    dsp.write(vreg(0, reg::GAIN), 0x40);
    dsp.write(vreg(0, reg::VOLL), 0x40);
    dsp.write(vreg(0, reg::VOLR), 0xC0);
    dsp.write(vreg(0, reg::PITCHL), 0x00);
    dsp.write(vreg(0, reg::PITCHH), 0x08);
    let v = dsp.voice_mut(0);
    v.mode = Mode::Sustain;
    // Set noise to a known value: 0x4000 -> sample = (0x4000 << 1) as i16 = -32768
    dsp.debug_set_noise(0x4000);

    let (l, r) = dsp.debug_voice_step(0, &r, 0);
    // sample = ((0x4000 << 1) as i16) = -32768; env after GAIN direct (m=1): 0x40<<4 = 1024
    // out = ((-32768 * 1024) >> 11) & !1 = -16384
    // l = (-16384 * 64) >> 7 = -8192; r = (-16384 * -64) >> 7 = 8192
    assert_eq!(l, -8192);
    assert_eq!(r, 8192);
    assert_eq!(dsp.voice(0).out, -16384);
    assert_eq!(dsp.voice(0).env, 1024);
    assert_eq!(dsp.voice(0).interp_pos, 0x0800);
    assert_eq!(dsp.read(vreg(0, reg::ENVX)), 0x40);
    assert_eq!(dsp.read(vreg(0, reg::OUTX)), (-16384i32 >> 8) as u8);
}

#[test]
fn e06_45_release_voice_is_silent() {
    let mut dsp = Dsp::new();
    let r = ram();
    // Default mode is Release, env=0 -> out should be 0
    dsp.write(reg::NON, 1);
    dsp.write(vreg(0, reg::GAIN), 0x7F);
    dsp.write(vreg(0, reg::VOLL), 0x7F);
    dsp.write(vreg(0, reg::VOLR), 0x7F);

    let (l, r) = dsp.debug_voice_step(0, &r, 0);
    assert_eq!(l, 0);
    assert_eq!(r, 0);
    assert_eq!(dsp.voice(0).env, 0);
}

#[test]
fn e06_45_gauss_path_constant() {
    let mut dsp = Dsp::new();
    let r = ram();
    // Constant buffer of 10000, interp_pos=0x0800 (frac=0), Sustain mode, GAIN direct
    dsp.write(reg::NON, 0);
    dsp.write(vreg(0, reg::GAIN), 0x40);
    dsp.write(vreg(0, reg::VOLL), 0x7F);
    dsp.write(vreg(0, reg::VOLR), 0x7F);
    let v = dsp.voice_mut(0);
    for i in 0..12 {
        v.buf[i] = 10000;
    }
    v.interp_pos = 0x0800;
    v.mode = Mode::Sustain;

    dsp.debug_voice_step(0, &r, 0);
    // With constant samples and frac=0: gauss gives ~10000 (slightly less due to filter)
    // out = ((sample * 0x400) >> 11) & !1 = ((10000 * 1024) >> 11) & !1 = 5000 & !1 = 5000
    let out = dsp.voice(0).out;
    assert!(out >= 4985 && out <= 5010, "out={} not in range", out);
    assert_eq!(out % 2, 0, "out must be even");
    assert_eq!(dsp.voice(0).env, 0x400);
}

#[test]
fn e06_45_pmon_modulates() {
    let mut dsp = Dsp::new();
    let r = ram();
    // Voice 1 with PMON enabled, noise source
    dsp.write(reg::PMON, 0x02);
    dsp.write(reg::NON, 0x02);
    dsp.write(vreg(1, reg::GAIN), 0x40);
    dsp.write(vreg(1, reg::PITCHL), 0x00);
    dsp.write(vreg(1, reg::PITCHH), 0x10);
    let v = dsp.voice_mut(1);
    v.mode = Mode::Sustain;

    // prev_out = -16384 -> modulation: pitch += ((-16384 >> 5) * 0x1000) >> 10
    // = ((-512) * 4096) >> 10 = (-2097152) >> 10 = -2048
    // pitch = 4096 + (-2048) = 2048
    dsp.debug_voice_step(1, &r, -16384);
    assert_eq!(dsp.voice(1).interp_pos, 2048);

    // Without PMON: pitch stays at 0x1000
    let mut dsp2 = Dsp::new();
    dsp2.write(reg::NON, 0x02);
    dsp2.write(vreg(1, reg::GAIN), 0x40);
    dsp2.write(vreg(1, reg::PITCHL), 0x00);
    dsp2.write(vreg(1, reg::PITCHH), 0x10);
    let v2 = dsp2.voice_mut(1);
    v2.mode = Mode::Sustain;
    dsp2.debug_voice_step(1, &r, -16384);
    assert_eq!(dsp2.voice(1).interp_pos, 0x1000);
}

#[test]
fn e06_45_pmon_ignored_for_voice0() {
    let mut dsp = Dsp::new();
    let r = ram();
    // PMON bit 0 should be ignored for voice 0
    dsp.write(reg::PMON, 0x01);
    dsp.write(reg::NON, 1);
    dsp.write(vreg(0, reg::GAIN), 0x40);
    dsp.write(vreg(0, reg::PITCHL), 0x00);
    dsp.write(vreg(0, reg::PITCHH), 0x10);
    let v = dsp.voice_mut(0);
    v.mode = Mode::Sustain;

    dsp.debug_voice_step(0, &r, -16384);
    assert_eq!(dsp.voice(0).interp_pos, 0x1000);
}

#[test]
fn e06_45_interp_pos_clamped() {
    let mut dsp = Dsp::new();
    let r = ram();
    // Voice 1 with PMON, max pitch and positive prev_out -> should clamp at 0x7FFF
    dsp.write(reg::PMON, 0x02);
    dsp.write(reg::NON, 0x02);
    dsp.write(vreg(1, reg::GAIN), 0x40);
    dsp.write(vreg(1, reg::PITCHL), 0xFF);
    dsp.write(vreg(1, reg::PITCHH), 0x3F);
    let v = dsp.voice_mut(1);
    v.mode = Mode::Sustain;
    v.interp_pos = 0x3FFF;

    // pitch = 0x3FFF = 16383
    // modulation: ((0x7FFE >> 5) * 16383) >> 10 = (249 * 16383) >> 10 = 4079367 >> 10 = 3985
    // total pitch = 16383 + 3985 = 20368
    // interp_pos = min(0x3FFF + 20368, 0x7FFF) = min(4095+20368, 32767) = 32767 = 0x7FFF
    dsp.debug_voice_step(1, &r, 0x7FFE);
    assert_eq!(dsp.voice(1).interp_pos, 0x7FFF);
}

#[test]
fn e06_45_decode_when_position_wraps() {
    let mut dsp = Dsp::new();
    let r = ram();
    // Common BRR setup: DIR=0x02, voice 0 entry at 0x0204 with SRCN=1
    dsp.write(reg::DIR, 0x02);
    let mut r_mut = *r;
    r_mut[0x0204] = 0x00; // start addr low
    r_mut[0x0205] = 0x10; // start addr high -> 0x1000
    r_mut[0x0206] = 0x10; // loop addr low
    r_mut[0x0207] = 0x10; // loop addr high -> 0x1010

    dsp.write(vreg(0, reg::SRCN), 1);
    r_mut[0x1000] = 0xC3; // block header: shift=12, filter=3, not last, no loop
    for i in 1..9 {
        r_mut[0x1000 + i] = 0x77;
    }

    {
        let v = dsp.voice_mut(0);
        v.addr = 0x1000;
        v.group_in_block = 0;
        v.buf_pos = 0;
        v.interp_pos = 0x4010; // >= 0x4000, should decode one group then subtract
        v.mode = Mode::Sustain;
    }

    dsp.write(reg::NON, 0);
    dsp.write(vreg(0, reg::GAIN), 0x40);
    dsp.write(vreg(0, reg::PITCHL), 0x00);
    dsp.write(vreg(0, reg::PITCHH), 0x10);

    dsp.debug_voice_step(0, &r_mut, 0);
    let v = dsp.voice(0);
    // After decode: buf[0..4] should be decoded samples (all same with 0x77 data)
    assert_eq!(v.buf[0], v.buf[1]);
    assert_eq!(v.buf[1], v.buf[2]);
    assert_eq!(v.buf[2], v.buf[3]);
    // With shift=12, filter=3, all 0x77: samples should be large positive values
    assert!(v.buf[0] > 0);
    assert_eq!(v.buf_pos, 4);
    assert_eq!(v.group_in_block, 1);
    // interp_pos = 0x4010 - 0x4000 + pitch(0x1000) = 0x1010
    assert_eq!(v.interp_pos, 0x1010);
}

#[test]
fn e06_45_key_on_delay_sequence() {
    let mut dsp = Dsp::new();
    let r = ram();
    // Common BRR setup
    dsp.write(reg::DIR, 0x02);
    let mut r_mut = *r;
    r_mut[0x0204] = 0x00;
    r_mut[0x0205] = 0x10;
    r_mut[0x0206] = 0x10;
    r_mut[0x0207] = 0x10;

    dsp.write(vreg(0, reg::SRCN), 1);
    r_mut[0x1000] = 0xC3;
    for i in 1..9 {
        r_mut[0x1000 + i] = 0x77;
    }

    {
        let v = dsp.voice_mut(0);
        v.kon_delay = 5;
        v.env = 0x500; // Should be reset to 0 during key-on delay
    }

    // Step 1: kon_delay=5 -> voice_begin, env=0, kon_delay=4, no decode
    let (l1, r1) = dsp.debug_voice_step(0, &r_mut, 0);
    assert_eq!((l1, r1), (0, 0));
    {
        let v = dsp.voice(0);
        assert_eq!(v.kon_delay, 4);
        assert_eq!(v.addr, 0x1000);
        assert_eq!(v.env, 0);
        assert_eq!(v.group_in_block, 0);
    }

    // Step 2: kon_delay=4 -> env=0, kon_delay=3, decode group
    let (l2, r2) = dsp.debug_voice_step(0, &r_mut, 0);
    assert_eq!((l2, r2), (0, 0));
    {
        let v = dsp.voice(0);
        assert_eq!(v.kon_delay, 3);
        assert_eq!(v.group_in_block, 1);
    }

    // Step 3: kon_delay=3 -> env=0, kon_delay=2, decode group
    let (l3, r3) = dsp.debug_voice_step(0, &r_mut, 0);
    assert_eq!((l3, r3), (0, 0));
    {
        let v = dsp.voice(0);
        assert_eq!(v.kon_delay, 2);
        assert_eq!(v.group_in_block, 2);
    }

    // Step 4: kon_delay=2 -> env=0, kon_delay=1, decode group
    let (l4, r4) = dsp.debug_voice_step(0, &r_mut, 0);
    assert_eq!((l4, r4), (0, 0));
    {
        let v = dsp.voice(0);
        assert_eq!(v.kon_delay, 1);
        assert_eq!(v.group_in_block, 3);
    }

    // Step 5: kon_delay=1 -> env=0, kon_delay=0, no decode
    let (l5, r5) = dsp.debug_voice_step(0, &r_mut, 0);
    assert_eq!((l5, r5), (0, 0));
    {
        let v = dsp.voice(0);
        assert_eq!(v.kon_delay, 0);
        assert_eq!(v.group_in_block, 3);
        // interp_pos should be 0 throughout key-on delay
        assert_eq!(v.interp_pos, 0);
    }
}

#[test]
fn e06_45_other_voice_registers_untouched() {
    let mut dsp = Dsp::new();
    let r = ram();
    // Run voice 0 step and verify voice 1's ENVX/OUTX are untouched (still 0)
    dsp.write(reg::NON, 0x01);
    dsp.write(vreg(0, reg::GAIN), 0x40);
    dsp.write(vreg(0, reg::VOLL), 0x40);
    dsp.write(vreg(0, reg::VOLR), 0xC0);
    let v = dsp.voice_mut(0);
    v.mode = Mode::Sustain;

    dsp.debug_voice_step(0, &r, 0);
    assert_eq!(dsp.read(vreg(1, reg::ENVX)), 0);
    assert_eq!(dsp.read(vreg(1, reg::OUTX)), 0);
}