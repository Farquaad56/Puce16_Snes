use puce16_core::apu::dsp::{Dsp, voice};
use puce16_core::hash::Fnv64;

#[test]
fn e06_31_power_on() {
    let mut dsp = Dsp::new();
    assert_eq!(dsp.read(0x6C), 0xE0);
    for a in 0..0x80 {
        if a != 0x6C {
            assert_eq!(dsp.read(a as u8), 0, "reg {} should be 0", a);
        }
    }
    assert_eq!(dsp.new_kon(), 0);
    assert_eq!(dsp.endx(), 0);
    assert!(dsp.take_audio().is_empty());
}

#[test]
fn e06_31_write_read() {
    let mut dsp = Dsp::new();
    dsp.write(0x00, 0x55);
    assert_eq!(dsp.read(0x00), 0x55);
    dsp.write(0x0F, 0x7F);
    assert_eq!(dsp.read(0x0F), 0x7F);
    dsp.write(0x1D, 0x9A);
    assert_eq!(dsp.read(0x1D), 0x9A);
}

#[test]
fn e06_31_address_mask() {
    let mut dsp = Dsp::new();
    dsp.write(0x81, 0x07);
    assert_eq!(dsp.read(0x01), 0x07);
    assert_eq!(dsp.read(0x81), 0x07);
}

#[test]
fn e06_31_kon_write() {
    let mut dsp = Dsp::new();
    dsp.write(0x4C, 0x03);
    assert_eq!(dsp.read(0x4C), 0x03);
    assert_eq!(dsp.new_kon(), 0x03);
}

#[test]
fn e06_31_endx_write_clears() {
    let mut dsp = Dsp::new();
    dsp.debug_set_endx(0xFF);
    assert_eq!(dsp.endx(), 0xFF);
    dsp.write(0x7C, 0x12);
    assert_eq!(dsp.endx(), 0);
    assert_eq!(dsp.read(0x7C), 0);
}

#[test]
fn e06_31_reset_keeps_registers() {
    let mut dsp = Dsp::new();
    dsp.write(0x00, 1);
    dsp.write(0x6C, 0x20);
    dsp.write(0x4C, 1);
    dsp.reset();
    assert_eq!(dsp.read(0x6C), 0xE0);
    assert_eq!(dsp.read(0x00), 1);
    assert_eq!(dsp.new_kon(), 0);
}

#[test]
fn e06_31_reg_constants() {
    use puce16_core::apu::dsp::reg;
    assert_eq!(reg::MVOLL, 0x0C);
    assert_eq!(reg::FLG, 0x6C);
    assert_eq!(reg::EDL, 0x7D);
    assert_eq!(reg::fir(0), 0x0F);
    assert_eq!(reg::fir(7), 0x7F);
    assert_eq!(reg::vreg(3, reg::GAIN), 0x37);
    assert_eq!(reg::vreg(7, reg::OUTX), 0x79);
}

#[test]
fn e06_31_voice_new() {
    let v = voice::Voice::new();
    assert_eq!(v.kon_delay, 0);
    assert_eq!(v.mode, voice::Mode::Release);
    assert_eq!(v.env, 0);
    assert_eq!(v.hidden_env, 0);
    assert_eq!(v.buf, [0; 12]);
    assert_eq!(v.interp_pos, 0);
    assert_eq!(v.p1, 0);
    assert_eq!(v.p2, 0);
}

#[test]
fn e06_31_hash_changes() {
    let mut dsp1 = Dsp::new();
    let mut dsp2 = Dsp::new();
    let mut h1 = Fnv64::new();
    let mut h2 = Fnv64::new();
    dsp1.hash_into(&mut h1);
    dsp2.hash_into(&mut h2);
    assert_eq!(h1.finish(), h2.finish());

    dsp1.write(0x00, 1);
    h1 = Fnv64::new();
    h2 = Fnv64::new();
    dsp1.hash_into(&mut h1);
    dsp2.hash_into(&mut h2);
    assert_ne!(h1.finish(), h2.finish());

    let mut dsp3 = Dsp::new();
    dsp3.debug_set_endx(1);
    h1 = Fnv64::new();
    h2 = Fnv64::new();
    dsp3.hash_into(&mut h1);
    dsp2.hash_into(&mut h2);
    assert_ne!(h1.finish(), h2.finish());
}

#[test]
fn e06_31_take_audio_drains() {
    let mut dsp = Dsp::new();
    assert!(dsp.take_audio().is_empty());
    assert!(dsp.take_audio().is_empty());
}
