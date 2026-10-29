use crate::hash::Fnv64;
use super::{Dsp, reg};
use super::brr;
use super::gauss::GAUSS;

/// Gaussian interpolation of 4 samples (a=oldest ... d=newest), frac in [0,255].
pub fn gauss_interp(a: i32, b: i32, c: i32, d: i32, frac: usize) -> i32 {
    let f = frac & 255;
    let mut o = (GAUSS[255 - f] * a) >> 11;
    o += (GAUSS[511 - f] * b) >> 11;
    o += (GAUSS[256 + f] * c) >> 11;
    o = o as i16 as i32;
    o += (GAUSS[f] * d) >> 11;
    brr::clamp16(o) & !1
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Clone, Copy, Debug)]
pub struct Voice {
    pub kon_delay: u8,
    pub mode: Mode,
    pub env: i32,
    pub hidden_env: i32,
    pub addr: u16,
    pub block_header: u8,
    pub group_in_block: u8,
    pub buf: [i32; 12],
    pub buf_pos: usize,
    pub interp_pos: u32,
    pub out: i32,
    pub p1: i32,
    pub p2: i32,
}

impl Voice {
    pub fn new() -> Self {
        Self {
            kon_delay: 0,
            mode: Mode::Release,
            env: 0,
            hidden_env: 0,
            addr: 0,
            block_header: 0,
            group_in_block: 0,
            buf: [0; 12],
            buf_pos: 0,
            interp_pos: 0,
            out: 0,
            p1: 0,
            p2: 0,
        }
    }

    pub fn hash_into(&self, h: &mut Fnv64) {
        h.write_u8(self.kon_delay);
        let mode_val = match self.mode {
            Mode::Attack => 0u8,
            Mode::Decay => 1u8,
            Mode::Sustain => 2u8,
            Mode::Release => 3u8,
        };
        h.write_u8(mode_val);
        h.write_u32(self.env as u32);
        h.write_u32(self.hidden_env as u32);
        h.write_u16(self.addr);
        h.write_u8(self.block_header);
        h.write_u8(self.group_in_block);
        for &v in &self.buf {
            h.write_u32(v as u32);
        }
        h.write_u8(self.buf_pos as u8);
        h.write_u32(self.interp_pos);
        h.write_u32(self.out as u32);
        h.write_u32(self.p1 as u32);
        h.write_u32(self.p2 as u32);
    }
}

use crate::savestate::{State, StateWriter, StateReader, StateError};

impl State for Voice {
    fn save(&self, w: &mut StateWriter) {
        // Save all fields in declaration order. All are covered by hash_into.
        w.u8(self.kon_delay);
        let mode_val = match self.mode {
            Mode::Attack => 0u8,
            Mode::Decay => 1u8,
            Mode::Sustain => 2u8,
            Mode::Release => 3u8,
        };
        w.u8(mode_val);
        w.i32(self.env);
        w.i32(self.hidden_env);
        w.u16(self.addr);
        w.u8(self.block_header);
        w.u8(self.group_in_block);
        for &v in &self.buf {
            w.i32(v);
        }
        w.u8(self.buf_pos as u8);
        w.u32(self.interp_pos);
        w.i32(self.out);
        w.i32(self.p1);
        w.i32(self.p2);
    }

    fn load(&mut self, r: &mut StateReader) -> Result<(), StateError> {
        // Load all fields in declaration order.
        self.kon_delay = r.u8()?;
        let mode_val = r.u8()?;
        match mode_val {
            0 => self.mode = Mode::Attack,
            1 => self.mode = Mode::Decay,
            2 => self.mode = Mode::Sustain,
            3 => self.mode = Mode::Release,
            _ => return Err(StateError::Mismatch("voice mode")),
        }
        self.env = r.i32()?;
        self.hidden_env = r.i32()?;
        self.addr = r.u16()?;
        self.block_header = r.u8()?;
        self.group_in_block = r.u8()?;
        for slot in &mut self.buf {
            *slot = r.i32()?;
        }
        self.buf_pos = r.u8()? as usize;
        self.interp_pos = r.u32()?;
        self.out = r.i32()?;
        self.p1 = r.i32()?;
        self.p2 = r.i32()?;
        Ok(())
    }
}

impl Dsp {
    pub fn voice(&self, x: usize) -> &Voice {
        &self.voices[x & 7]
    }

    pub fn voice_mut(&mut self, x: usize) -> &mut Voice {
        &mut self.voices[x & 7]
    }

    pub fn debug_voice_begin(&mut self, x: usize, ram: &[u8; 65536]) {
        self.voice_begin(x, ram);
    }

    pub fn debug_voice_decode_group(&mut self, x: usize, ram: &[u8; 65536]) {
        self.voice_decode_group(x, ram);
    }

    pub(super) fn voice_begin(&mut self, x: usize, ram: &[u8; 65536]) {
        let v = &mut self.voices[x & 7];
        let srcn = self.regs[reg::vreg(x, reg::SRCN)];
        let entry = ((self.regs[reg::DIR as usize] as u16) << 8).wrapping_add((srcn as u16) * 4);
        let start = ram16(ram, entry);
        v.addr = start;
        v.group_in_block = 0;
        v.block_header = ram[start as usize];
        self.endx &= !(1 << x);
        v.interp_pos = 0;
        v.buf_pos = 0;
    }

    pub(super) fn voice_decode_group(&mut self, x: usize, ram: &[u8; 65536]) {
        let v = &mut self.voices[x & 7];
        if v.group_in_block == 0 {
            v.block_header = ram[v.addr as usize];
        }
        let shift = (v.block_header >> 4) & 15;
        let filter = (v.block_header >> 2) & 3;
        let offset = 1 + 2 * (v.group_in_block as u16);
        let b0_addr = v.addr.wrapping_add(offset);
        let b1_addr = v.addr.wrapping_add(offset).wrapping_add(1);
        let b0 = ram[b0_addr as usize];
        let b1 = ram[b1_addr as usize];
        let (samples, p1, p2) = brr::decode_group(b0, b1, shift, filter, v.p1, v.p2);
        for i in 0..4 {
            v.buf[v.buf_pos + i] = samples[i];
        }
        v.buf_pos = (v.buf_pos + 4) % 12;
        v.p1 = p1;
        v.p2 = p2;
        v.group_in_block += 1;
        if v.group_in_block == 4 {
            v.group_in_block = 0;
            self.voice_advance_block(x, ram);
        }
    }

    fn voice_advance_block(&mut self, x: usize, ram: &[u8; 65536]) {
        let v = &mut self.voices[x & 7];
        let h = v.block_header;
        if (h & 1) != 0 {
            self.endx |= 1 << x;
            let srcn = self.regs[reg::vreg(x, reg::SRCN)];
            let entry = ((self.regs[reg::DIR as usize] as u16) << 8).wrapping_add((srcn as u16) * 4);
            v.addr = ram16(ram, entry.wrapping_add(2));
            if (h & 2) == 0 {
                v.mode = Mode::Release;
                v.env = 0;
                v.hidden_env = 0;
            }
        } else {
            v.addr = v.addr.wrapping_add(9);
        }
    }
}

fn ram16(ram: &[u8; 65536], addr: u16) -> u16 {
    let lo = ram[addr as usize];
    let hi_addr = addr.wrapping_add(1);
    let hi = ram[hi_addr as usize];
    (lo as u16) | ((hi as u16) << 8)
}

impl Dsp {
    /// Process one sample for voice x. Returns (left, right) contributions after volume.
    pub(super) fn voice_step(&mut self, x: usize, ram: &[u8; 65536], prev_out: i32) -> (i32, i32) {
        // Step 1: Key-on delay sequence
        if self.voices[x].kon_delay > 0 {
            if self.voices[x].kon_delay == 5 {
                self.voice_begin(x, ram);
            }
            self.voices[x].env = 0;
            self.voices[x].hidden_env = 0;
            self.voices[x].kon_delay -= 1;
            let kd = self.voices[x].kon_delay;
            if kd == 3 || kd == 2 || kd == 1 {
                self.voice_decode_group(x, ram);
            }
            self.voices[x].out = 0;
            self.pend_envx[x & 7] = 0;
            self.pend_outx[x & 7] = 0;
            return (0, 0);
        }

        // Step 2: Decode groups as needed when position wraps
        while self.voices[x].interp_pos >= 0x4000 {
            self.voice_decode_group(x, ram);
            self.voices[x].interp_pos -= 0x4000;
        }

        // Step 3: Get sample (noise or interpolated)
        let non = self.regs[reg::NON as usize];
        let interp_pos = self.voices[x].interp_pos;
        let buf_pos = self.voices[x].buf_pos;
        let buf = self.voices[x].buf;
        let sample: i32;
        if (non >> x) & 1 != 0 {
            sample = ((self.noise as i32) << 1) as i16 as i32;
        } else {
            let base = (interp_pos >> 12) as usize;
            let frac = ((interp_pos >> 4) & 0xFF) as usize;
            let a = buf[(buf_pos + base) % 12];
            let b = buf[(buf_pos + base + 1) % 12];
            let c = buf[(buf_pos + base + 2) % 12];
            let d = buf[(buf_pos + base + 3) % 12];
            sample = gauss_interp(a, b, c, d, frac);
        }

        // Step 4: envelope. Default keeps spec 07b order (run_envelope first). With
        // `ref_order` the S-DSP order of the reference is used (output and ENVX use the
        // envelope BEFORE this sample's update); see docs/qa/inbox/E07-11e-Q1.md.
        let env;
        let out;
        if self.ref_order {
            env = self.voices[x].env;
            out = ((sample * env) >> 11) & !1;
            self.run_envelope(x);
        } else {
            self.run_envelope(x);
            env = self.voices[x].env;
            out = ((sample * env) >> 11) & !1;
        }
        self.voices[x].out = out;

        // Step 5: ENVX and OUTX values (published to the register file by `clock`)
        self.pend_envx[x & 7] = (env >> 4) as u8;
        self.pend_outx[x & 7] = (out >> 8) as u8;

        // Step 6: Compute pitch with PMON modulation and advance interp_pos
        let pitchl = self.regs[reg::vreg(x, reg::PITCHL)];
        let pitchh = self.regs[reg::vreg(x, reg::PITCHH)];
        let mut pitch = (pitchl as i32) | (((pitchh & 0x3F) as i32) << 8);
        if x >= 1 {
            let pmon = self.regs[reg::PMON as usize];
            if (pmon >> x) & 1 != 0 {
                pitch += ((prev_out >> 5) * pitch) >> 10;
            }
        }
        let new_pos = std::cmp::min(interp_pos + pitch as u32, 0x7FFF);
        self.voices[x].interp_pos = new_pos;

        // Step 7: Apply volume and return
        let voll = self.regs[reg::vreg(x, reg::VOLL)] as i8 as i32;
        let volr = self.regs[reg::vreg(x, reg::VOLR)] as i8 as i32;
        let l = (out * voll) >> 7;
        let r = (out * volr) >> 7;
        (l, r)
    }

    /// Debug wrapper for voice_step.
    pub fn debug_voice_step(&mut self, x: usize, ram: &[u8; 65536], prev_out: i32) -> (i32, i32) {
        let r = self.voice_step(x, ram, prev_out);
        self.regs[reg::vreg(x, reg::ENVX)] = self.pend_envx[x & 7];
        self.regs[reg::vreg(x, reg::OUTX)] = self.pend_outx[x & 7];
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::savestate::{State, StateWriter, StateReader};
    use crate::hash::Fnv64;

    fn voice_hash(v: &Voice) -> u64 {
        let mut h = Fnv64::new();
        v.hash_into(&mut h);
        h.finish()
    }

    #[test]
    fn e08_10_voice_roundtrip() {
        // Set every saved field to a distinct non-zero value. Mode: Sustain (2).
        let mut voice = Voice::new();
        voice.kon_delay = 5;
        voice.mode = Mode::Sustain;
        voice.env = 1234567890;
        voice.hidden_env = 987654321;
        voice.addr = 0xABCD;
        voice.block_header = 0x4F;
        voice.group_in_block = 3;
        for i in 0..12 {
            voice.buf[i] = (i as i32) * 100 + 7;
        }
        voice.buf_pos = 9;
        voice.interp_pos = 0x5678;
        voice.out = -42;
        voice.p1 = 111111;
        voice.p2 = 222222;

        // Save.
        let mut writer = StateWriter::new();
        voice.save(&mut writer);
        let data = writer.finish();

        // Load into fresh instance.
        let mut loaded = Voice::new();
        let mut reader = StateReader::new(&data);
        loaded.load(&mut reader).unwrap();

        // Verify bytes match on re-save.
        let mut writer2 = StateWriter::new();
        loaded.save(&mut writer2);
        let data2 = writer2.finish();
        assert_eq!(data, data2, "re-saved bytes must match original");

        // Verify hash matches.
        assert_eq!(voice_hash(&voice), voice_hash(&loaded), "hashes must match after roundtrip");
    }

    #[test]
    fn e08_10_voice_bad_mode() {
        let mut writer = StateWriter::new();
        writer.u8(5); // kon_delay
        writer.u8(9); // bad mode value
        let data = writer.finish();

        let mut voice = Voice::new();
        let mut reader = StateReader::new(&data);
        let result = voice.load(&mut reader);
        assert!(matches!(result, Err(StateError::Mismatch("voice mode"))),
            "bad mode byte should fail with Mismatch(\"voice mode\")");
    }

    #[test]
    fn e08_10_truncated_voice() {
        let mut voice = Voice::new();
        voice.kon_delay = 5;
        voice.mode = Mode::Sustain;
        voice.env = 1234567890;

        let mut writer = StateWriter::new();
        voice.save(&mut writer);
        let data = writer.finish();

        // Test truncation at various points.
        for n in [0, data.len() / 2, data.len() - 1] {
            let mut loaded = Voice::new();
            let mut reader = StateReader::new(&data[..n]);
            let result = loaded.load(&mut reader);
            assert!(matches!(result, Err(StateError::Truncated)),
                "truncation at {} should fail with Truncated", n);
        }
    }
}
