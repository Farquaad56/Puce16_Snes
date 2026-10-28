use puce16_core::apu::dsp::{Dsp, reg};
use puce16_core::apu::dsp::voice::{Voice, Mode};

fn ram() -> Box<[u8; 65536]> {
    vec![0u8; 65536].into_boxed_slice().try_into().unwrap()
}

#[test]
fn e06_41_begin_reads_directory() {
    let mut dsp = Dsp::new();
    let mut r = ram();
    dsp.write(reg::DIR, 0x02);
    r[0x0204..0x0208].copy_from_slice(&[0x00, 0x10, 0x10, 0x10]);
    dsp.write(0x04, 1);
    r[0x1000] = 0xC3;
    dsp.debug_set_endx(0xFF);
    dsp.debug_voice_begin(0, &r);
    assert_eq!(dsp.voice(0).addr, 0x1000);
    assert_eq!(dsp.voice(0).group_in_block, 0);
    assert_eq!(dsp.voice(0).buf_pos, 0);
    assert_eq!(dsp.voice(0).interp_pos, 0);
    assert_eq!(dsp.voice(0).block_header, 0xC3);
    assert_eq!(dsp.endx(), 0xFE);
}

#[test]
fn e06_41_directory_wraps() {
    let mut dsp = Dsp::new();
    let mut r = ram();
    dsp.write(reg::DIR, 0xFF);
    dsp.write(0x04, 255);
    let entry = ((0xFF as u16) << 8).wrapping_add(255u16 * 4);
    assert_eq!(entry, 0x02FC);
    r[0x02FC] = 0x34;
    r[0x02FD] = 0x12;
    dsp.debug_voice_begin(0, &r);
    assert_eq!(dsp.voice(0).addr, 0x1234);
}

#[test]
fn e06_41_begin_keeps_history() {
    let mut dsp = Dsp::new();
    let mut r = ram();
    dsp.write(reg::DIR, 0x02);
    r[0x0204..0x0208].copy_from_slice(&[0x00, 0x10, 0x10, 0x10]);
    dsp.write(0x04, 1);
    r[0x1000] = 0xC3;
    dsp.voice_mut(0).p1 = 77;
    dsp.voice_mut(0).p2 = -5;
    dsp.debug_voice_begin(0, &r);
    assert_eq!(dsp.voice(0).p1, 77);
    assert_eq!(dsp.voice(0).p2, -5);
}

#[test]
fn e06_41_decode_first_group() {
    let mut dsp = Dsp::new();
    let mut r = ram();
    dsp.write(reg::DIR, 0x02);
    r[0x0204..0x0208].copy_from_slice(&[0x00, 0x10, 0x10, 0x10]);
    dsp.write(0x04, 1);
    r[0x1000] = 0xC3;
    for i in 1..9 {
        r[0x1000 + i] = 0x77;
    }
    dsp.debug_voice_begin(0, &r);
    dsp.debug_voice_decode_group(0, &r);
    let v = dsp.voice(0);
    assert_eq!(v.buf[0..4], [28672, 28672, 28672, 28672]);
    assert_eq!(v.buf_pos, 4);
    assert_eq!(v.group_in_block, 1);
    assert_eq!(v.p1, 28672);
    assert_eq!(v.p2, 14336);
    assert_eq!(v.addr, 0x1000);
}

#[test]
fn e06_41_group_byte_offsets() {
    let mut dsp = Dsp::new();
    let mut r = ram();
    dsp.write(reg::DIR, 0x02);
    r[0x0204..0x0208].copy_from_slice(&[0x00, 0x10, 0x10, 0x10]);
    dsp.write(0x04, 1);
    r[0x1000] = 0x40;
    let data = [0x12, 0x34, 0x56, 0x70, 0x9A, 0xBC, 0xDE, 0xF1];
    for (i, &b) in data.iter().enumerate() {
        r[0x1001 + i] = b;
    }
    dsp.debug_voice_begin(0, &r);
    dsp.debug_voice_decode_group(0, &r);
    dsp.debug_voice_decode_group(0, &r);
    dsp.debug_voice_decode_group(0, &r);
    let v = dsp.voice(0);
    assert_eq!(v.buf[0..4], [16, 32, 48, 64]);
    assert_eq!(v.buf[4..8], [80, 96, 112, 0]);
    assert_eq!(v.buf[8..12], [-112, -96, -80, -64]);
    assert_eq!(v.buf_pos, 0);
}

#[test]
fn e06_41_fourth_group_wraps_ring() {
    let mut dsp = Dsp::new();
    let mut r = ram();
    dsp.write(reg::DIR, 0x02);
    r[0x0204..0x0208].copy_from_slice(&[0x00, 0x10, 0x10, 0x10]);
    dsp.write(0x04, 1);
    r[0x1000] = 0x40;
    let data = [0x12, 0x34, 0x56, 0x70, 0x9A, 0xBC, 0xDE, 0xF1];
    for (i, &b) in data.iter().enumerate() {
        r[0x1001 + i] = b;
    }
    dsp.debug_voice_begin(0, &r);
    dsp.debug_voice_decode_group(0, &r);
    dsp.debug_voice_decode_group(0, &r);
    dsp.debug_voice_decode_group(0, &r);
    dsp.debug_voice_decode_group(0, &r);
    let v = dsp.voice(0);
    assert_eq!(v.buf[0..4], [-48, -32, -16, 16]);
    assert_eq!(v.buf[4..8], [80, 96, 112, 0]);
    assert_eq!(v.buf[8..12], [-112, -96, -80, -64]);
    assert_eq!(v.buf_pos, 4);
    assert_eq!(v.p1, 16);
    assert_eq!(v.p2, -8);
}

#[test]
fn e06_41_normal_block_advances() {
    let mut dsp = Dsp::new();
    let mut r = ram();
    dsp.write(reg::DIR, 0x02);
    r[0x0204..0x0208].copy_from_slice(&[0x00, 0x10, 0x10, 0x10]);
    dsp.write(0x04, 1);
    r[0x1000] = 0x40;
    for i in 1..9 {
        r[0x1000 + i] = 0x00;
    }
    dsp.debug_voice_begin(0, &r);
    dsp.debug_voice_decode_group(0, &r);
    dsp.debug_voice_decode_group(0, &r);
    dsp.debug_voice_decode_group(0, &r);
    dsp.debug_voice_decode_group(0, &r);
    assert_eq!(dsp.voice(0).addr, 0x1009);
    assert_eq!(dsp.voice(0).group_in_block, 0);
    assert_eq!(dsp.endx(), 0);
    r[0x1009] = 0xA0;
    dsp.debug_voice_decode_group(0, &r);
    assert_eq!(dsp.voice(0).block_header, 0xA0);
}

#[test]
fn e06_41_end_with_loop() {
    let mut dsp = Dsp::new();
    let mut r = ram();
    dsp.write(reg::DIR, 0x02);
    r[0x0204..0x0208].copy_from_slice(&[0x00, 0x10, 0x10, 0x10]);
    dsp.write(0x04, 1);
    r[0x1000] = 0xC3;
    for i in 1..9 {
        r[0x1000 + i] = 0x00;
    }
    dsp.voice_mut(0).mode = Mode::Sustain;
    dsp.voice_mut(0).env = 0x500;
    dsp.debug_voice_begin(0, &r);
    dsp.debug_voice_decode_group(0, &r);
    dsp.debug_voice_decode_group(0, &r);
    dsp.debug_voice_decode_group(0, &r);
    dsp.debug_voice_decode_group(0, &r);
    assert_eq!(dsp.endx(), 1);
    assert_eq!(dsp.voice(0).addr, 0x1010);
    assert_eq!(dsp.voice(0).mode, Mode::Sustain);
    assert_eq!(dsp.voice(0).env, 0x500);
}

#[test]
fn e06_41_end_without_loop() {
    let mut dsp = Dsp::new();
    let mut r = ram();
    dsp.write(reg::DIR, 0x02);
    r[0x0204..0x0208].copy_from_slice(&[0x00, 0x10, 0x10, 0x10]);
    dsp.write(0x04, 1);
    r[0x1000] = 0xC1;
    for i in 1..9 {
        r[0x1000 + i] = 0x00;
    }
    dsp.voice_mut(0).mode = Mode::Sustain;
    dsp.voice_mut(0).env = 0x500;
    dsp.debug_voice_begin(0, &r);
    dsp.debug_voice_decode_group(0, &r);
    dsp.debug_voice_decode_group(0, &r);
    dsp.debug_voice_decode_group(0, &r);
    dsp.debug_voice_decode_group(0, &r);
    assert_eq!(dsp.endx(), 1);
    assert_eq!(dsp.voice(0).addr, 0x1010);
    assert_eq!(dsp.voice(0).mode, Mode::Release);
    assert_eq!(dsp.voice(0).env, 0);
    assert_eq!(dsp.voice(0).hidden_env, 0);
}

#[test]
fn e06_41_filter_header_used() {
    let mut dsp = Dsp::new();
    let mut r = ram();
    dsp.write(reg::DIR, 0x02);
    r[0x0204..0x0208].copy_from_slice(&[0x00, 0x10, 0x10, 0x10]);
    dsp.write(0x04, 1);
    r[0x1000] = 0x14;
    let data = [0x12, 0x34, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    for (i, &b) in data.iter().enumerate() {
        r[0x1001 + i] = b;
    }
    dsp.debug_voice_begin(0, &r);
    dsp.debug_voice_decode_group(0, &r);
    let v = dsp.voice(0);
    assert_eq!(v.buf[0..4], [2, 4, 8, 14]);
    dsp.debug_voice_decode_group(0, &r);
    dsp.debug_voice_decode_group(0, &r);
    let v = dsp.voice(0);
    assert_eq!(v.buf[0..4], [2, 4, 8, 14]);
    assert_eq!(v.buf[4..8], [12, 10, 8, 6]);
    assert_eq!(v.buf[8..12], [4, 2, 0, 0]);
    dsp.debug_voice_decode_group(0, &r);
    let v = dsp.voice(0);
    assert_eq!(v.buf[0..4], [0, 0, 0, 0]);
    assert_eq!(v.buf[4..8], [12, 10, 8, 6]);
    assert_eq!(v.buf[8..12], [4, 2, 0, 0]);
}

#[test]
fn e06_41_voice_index_masked() {
    let mut dsp = Dsp::new();
    let v0 = dsp.voice(0) as *const Voice;
    let v8 = dsp.voice(8) as *const Voice;
    assert_eq!(v0, v8);
    dsp.voice_mut(9).env = 5;
    assert_eq!(dsp.voice(1).env, 5);
}
