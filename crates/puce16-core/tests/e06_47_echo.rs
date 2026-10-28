use puce16_core::apu::dsp::{Dsp, reg};
use puce16_core::apu::dsp::voice::Mode;

fn ram() -> Box<[u8; 65536]> {
    vec![0u8; 65536].into_boxed_slice().try_into().unwrap()
}

#[test]
fn e06_47_edl0_writes_four_bytes() {
    let mut dsp = Dsp::new();
    let mut r = ram();
    dsp.write(reg::ESA, 0x10);
    dsp.write(reg::EDL, 0);
    dsp.write(reg::FLG, 0x00);
    // Voice 0: NON=1, EON=1, GAIN=0x40 (direct env), VOLL=VOLR=0x40, mode=Sustain
    dsp.write(0x3D, 0x01); // NON bit 0
    dsp.write(0x4D, 0x01); // EON bit 0
    dsp.write(reg::vreg(0, reg::GAIN) as u8, 0x40);
    dsp.write(reg::vreg(0, reg::VOLL) as u8, 0x40);
    dsp.write(reg::vreg(0, reg::VOLR) as u8, 0x40);
    dsp.voice_mut(0).mode = Mode::Sustain;

    dsp.run_sample(&mut *r);
    // Echo write: -8192 in little-endian is [0x00, 0xE0] for each channel
    assert_eq!(&r[0x1000..0x1004], &[0x00, 0xE0, 0x00, 0xE0]);

    // Second sample: EDL=0 means length=0, offset wraps immediately back to 0
    dsp.run_sample(&mut *r);
    assert_eq!(&r[0x1004..0x1008], &[0; 4]);
}

#[test]
fn e06_47_flg_bit5_blocks_write() {
    let mut dsp = Dsp::new();
    let mut r = ram();
    dsp.write(reg::ESA, 0x10);
    dsp.write(reg::EDL, 0);
    dsp.write(reg::FLG, 0x20); // Echo write blocked
    dsp.write(0x3D, 0x01);
    dsp.write(0x4D, 0x01);
    dsp.write(reg::vreg(0, reg::GAIN) as u8, 0x40);
    dsp.write(reg::vreg(0, reg::VOLL) as u8, 0x40);
    dsp.write(reg::vreg(0, reg::VOLR) as u8, 0x40);
    dsp.voice_mut(0).mode = Mode::Sustain;

    dsp.run_sample(&mut *r);
    assert_eq!(&r[0x1000..0x1004], &[0; 4]);
}

#[test]
fn e06_47_mute_does_not_stop_echo_write() {
    let mut dsp = Dsp::new();
    let mut r = ram();
    dsp.write(reg::ESA, 0x10);
    dsp.write(reg::EDL, 0);
    dsp.write(reg::FLG, 0x40); // Mute output but echo writes continue
    dsp.write(0x3D, 0x01);
    dsp.write(0x4D, 0x01);
    dsp.write(reg::vreg(0, reg::GAIN) as u8, 0x40);
    dsp.write(reg::vreg(0, reg::VOLL) as u8, 0x40);
    dsp.write(reg::vreg(0, reg::VOLR) as u8, 0x40);
    dsp.voice_mut(0).mode = Mode::Sustain;

    dsp.run_sample(&mut *r);
    let audio = dsp.take_audio();
    assert_eq!(audio, vec![0, 0]);
    assert_eq!(&r[0x1000..0x1004], &[0x00, 0xE0, 0x00, 0xE0]);
}

#[test]
fn e06_47_offset_advances_edl1() {
    let mut dsp = Dsp::new();
    let mut r = ram();
    dsp.write(reg::ESA, 0x10);
    dsp.write(reg::EDL, 1); // Length = 0x800 bytes
    dsp.write(reg::FLG, 0x00);
    dsp.write(0x3D, 0x01);
    dsp.write(0x4D, 0x01);
    dsp.write(reg::vreg(0, reg::GAIN) as u8, 0x40);
    dsp.write(reg::vreg(0, reg::VOLL) as u8, 0x40);
    dsp.write(reg::vreg(0, reg::VOLR) as u8, 0x40);
    dsp.voice_mut(0).mode = Mode::Sustain;

    for _ in 0..3 {
        dsp.run_sample(&mut *r);
    }
    // Writes at 0x1000, 0x1004, 0x1008
    assert_eq!(&r[0x1000..0x1004], &[0x00, 0xE0, 0x00, 0xE0]);
    assert_eq!(&r[0x1004..0x1008], &[0x00, 0xE0, 0x00, 0xE0]);
    assert_eq!(&r[0x1008..0x100C], &[0x00, 0xE0, 0x00, 0xE0]);
    // Next slot should be untouched
    assert_eq!(&r[0x100C..0x1010], &[0; 4]);
}

#[test]
fn e06_47_offset_wraps_at_length() {
    let mut dsp = Dsp::new();
    let mut r = ram();
    dsp.write(reg::ESA, 0x10);
    dsp.write(reg::EDL, 1); // Length = 0x800 bytes = 512 samples * 4 bytes
    dsp.write(reg::FLG, 0x00);
    dsp.write(0x3D, 0x01);
    dsp.write(0x4D, 0x01);
    dsp.write(reg::vreg(0, reg::GAIN) as u8, 0x40);
    dsp.write(reg::vreg(0, reg::VOLL) as u8, 0x40);
    dsp.write(reg::vreg(0, reg::VOLR) as u8, 0x40);
    dsp.voice_mut(0).mode = Mode::Sustain;

    // Fill the buffer with 512 samples
    for _ in 0..512 {
        dsp.run_sample(&mut *r);
    }
    // Last write was at offset 0x7FC (sample 511)
    assert_eq!(&r[0x17FC..0x1800], &[0x00, 0xE0, 0x00, 0xE0]);

    // Clear the start of buffer
    for i in 0x1000..0x1004 {
        r[i] = 0;
    }

    // One more sample should wrap back to offset 0
    dsp.run_sample(&mut *r);
    assert_eq!(&r[0x1000..0x1004], &[0x00, 0xE0, 0x00, 0xE0]);
}

#[test]
fn e06_47_fir_delays_by_seven() {
    let mut dsp = Dsp::new();
    let mut r = ram();
    dsp.write(reg::ESA, 0x10);
    dsp.write(reg::EDL, 1);
    dsp.write(reg::FLG, 0x00);
    // FIR0 = 0x40 (64), all others 0
    dsp.write(reg::fir(0), 0x40);
    // EVOLL=EVOLR=0x40, MVOLL=MVOLR=0
    dsp.write(reg::MVOLL, 0);
    dsp.write(reg::MVOLR, 0);
    dsp.write(reg::EVOLL, 0x40);
    dsp.write(reg::EVOLR, 0x40);

    // Write known values into echo buffer at successive positions
    for k in 0..9 {
        let offset = 0x1000 + 4 * k;
        r[offset] = 0x00;
        r[offset + 1] = (k + 1) as u8;
        r[offset + 2] = 0x00;
        r[offset + 3] = (2 * (k + 1)) as u8;
    }

    // Run 9 samples: first 7 should be silent (FIR delay), then values appear
    let mut audio = Vec::new();
    for _ in 0..9 {
        dsp.run_sample(&mut *r);
        audio.extend(dsp.take_audio());
    }

    // First 14 values (7 pairs) should be 0
    for i in 0..14 {
        assert_eq!(audio[i], 0, "sample {} should be 0", i);
    }
    // Pair 7 (indices 14,15): FIR0 * hist[0] >> 6 = 64 * 1 >> 6 = 1 for L, 2 for R
    assert_eq!(audio[14], 0x40, "pair 7 L");
    assert_eq!(audio[15], 0x80, "pair 7 R");
    // Pair 8 (indices 16,17): FIR0 * hist[0] >> 6 = 64 * 2 >> 6 = 2 for L, 4 for R
    assert_eq!(audio[16], 0x80, "pair 8 L");
    assert_eq!(audio[17], 0x100, "pair 8 R");
}

#[test]
fn e06_47_fir_newest_tap_and_feedback() {
    let mut dsp = Dsp::new();
    let mut r = ram();
    dsp.write(reg::ESA, 0x10);
    dsp.write(reg::EDL, 1);
    dsp.write(reg::FLG, 0x00);
    // FIR7 = 0x40 (64), EFB = 0x40 (64)
    dsp.write(reg::fir(7), 0x40);
    dsp.write(reg::EFB, 0x40);
    // EVOL=0, MVOL=0 so output is silent
    dsp.write(reg::MVOLL, 0);
    dsp.write(reg::MVOLR, 0);
    dsp.write(reg::EVOLL, 0);
    dsp.write(reg::EVOLR, 0);

    // Write known value into echo buffer: L=1, R=2 (after >>1 shift)
    r[0x1000] = 0x00;
    r[0x1001] = 0x01;
    r[0x1002] = 0x00;
    r[0x1003] = 0x02;

    dsp.run_sample(&mut *r);

    // l = 0x0100 >> 1 = 128 -> fir_l = (128 * 64) >> 6 = 0x80 ; feedback = (0x80 * 64) >> 7 = 0x40
    // r = 0x0200 >> 1 = 256 -> fir_r = 0x100 ; feedback = 0x80
    assert_eq!(&r[0x1000..0x1004], &[0x40, 0x00, 0x80, 0x00]);
    assert_eq!(dsp.take_audio(), vec![0i16, 0]);
}

#[test]
fn e06_47_fir_coefficient_is_signed() {
    let mut dsp = Dsp::new();
    let mut r = ram();
    dsp.write(reg::ESA, 0x10);
    dsp.write(reg::EDL, 1);
    dsp.write(reg::FLG, 0x00);
    // FIR7 = 0xC0 (-64), EFB=0
    dsp.write(reg::fir(7), 0xC0);
    dsp.write(reg::EFB, 0);
    // EVOLL=EVOLR=0x40 so we can hear the echo output
    dsp.write(reg::MVOLL, 0);
    dsp.write(reg::MVOLR, 0);
    dsp.write(reg::EVOLL, 0x40);
    dsp.write(reg::EVOLR, 0x40);

    // Write known value: L=128 (after >>1), R=256 (after >>1)
    r[0x1000] = 0x00;
    r[0x1001] = 0x01;
    r[0x1002] = 0x00;
    r[0x1003] = 0x02;

    dsp.run_sample(&mut *r);
    let audio = dsp.take_audio();

    // fir_l = (128 * -64) >> 6 = -128 -> clamp & !1 = -128
    // main_l = ((fir_l * EVOLL) >> 7) = (-128 * 64) >> 7 = -64
    assert_eq!(audio[0], -64, "main_l");
    // fir_r = (256 * -64) >> 6 = -256 -> clamp & !1 = -256
    // main_r = ((fir_r * EVOLR) >> 7) = (-256 * 64) >> 7 = -128
    assert_eq!(audio[1], -128, "main_r");
}

#[test]
fn e06_47_esa_changes_buffer() {
    let mut dsp = Dsp::new();
    let mut r = ram();
    dsp.write(reg::ESA, 0x20); // Different buffer location
    dsp.write(reg::EDL, 0);
    dsp.write(reg::FLG, 0x00);
    dsp.write(0x3D, 0x01);
    dsp.write(0x4D, 0x01);
    dsp.write(reg::vreg(0, reg::GAIN) as u8, 0x40);
    dsp.write(reg::vreg(0, reg::VOLL) as u8, 0x40);
    dsp.write(reg::vreg(0, reg::VOLR) as u8, 0x40);
    dsp.voice_mut(0).mode = Mode::Sustain;

    dsp.run_sample(&mut *r);
    // Write should go to 0x2000..0x2004, not 0x1000
    assert_eq!(&r[0x2000..0x2004], &[0x00, 0xE0, 0x00, 0xE0]);
}
