pub mod voice;
pub mod envelope;
pub mod brr;
pub mod echo;
pub mod gauss;
pub mod tables;

use crate::hash::Fnv64;
use self::voice::Voice;

pub mod reg {
    pub const MVOLL: u8 = 0x0C;
    pub const MVOLR: u8 = 0x1C;
    pub const EVOLL: u8 = 0x2C;
    pub const EVOLR: u8 = 0x3C;
    pub const KON: u8 = 0x4C;
    pub const KOFF: u8 = 0x5C;
    pub const FLG: u8 = 0x6C;
    pub const ENDX: u8 = 0x7C;
    pub const EFB: u8 = 0x0D;
    pub const PMON: u8 = 0x2D;
    pub const NON: u8 = 0x3D;
    pub const EON: u8 = 0x4D;
    pub const DIR: u8 = 0x5D;
    pub const ESA: u8 = 0x6D;
    pub const EDL: u8 = 0x7D;

    pub const VOLL: u8 = 0;
    pub const VOLR: u8 = 1;
    pub const PITCHL: u8 = 2;
    pub const PITCHH: u8 = 3;
    pub const SRCN: u8 = 4;
    pub const ADSR1: u8 = 5;
    pub const ADSR2: u8 = 6;
    pub const GAIN: u8 = 7;
    pub const ENVX: u8 = 8;
    pub const OUTX: u8 = 9;

    pub fn fir(i: usize) -> u8 {
        0x0F + 0x10 * i as u8
    }

    pub fn vreg(x: usize, off: u8) -> usize {
        x * 16 + off as usize
    }
}

pub struct Dsp {
    regs: [u8; 128],
    voices: [Voice; 8],
    counter: u16,
    noise: u16,
    every_other: bool,
    new_kon: u8,
    kon: u8,
    endx: u8,
    echo_offset: u16,
    echo_length: u16,
    hist_l: [i32; 8],
    hist_r: [i32; 8],
    audio: Vec<i16>,
    /// ENVX/OUTX values computed at the sample boundary, published to the register file
    /// at the clock where the real S-DSP writes them (see `clock`).
    pub(super) pend_envx: [u8; 8],
    pub(super) pend_outx: [u8; 8],
    /// Clock inside the current 32-clock sample (32 = nothing left to publish).
    phase: u8,
    /// ADSR1 as latched at each voice's V2 clock; used by the envelope in clocked mode.
    pub(super) lat_adsr1: [u8; 8],
    pub(super) use_lat: bool,
    /// Shared transfer buffers between the voice pipeline and the ENVX/OUTX registers.
    envx_buf: u8,
    outx_buf: u8,
    /// Configuration: output/ENVX use the envelope before the sample's update (reference order).
    pub(super) ref_order: bool,
}

impl Dsp {
    pub fn new() -> Self {
        let mut regs = [0u8; 128];
        regs[reg::FLG as usize] = 0xE0;
        Self {
            regs,
            voices: [Voice::new(); 8],
            counter: 0,
            noise: 0x4000,
            every_other: true,
            new_kon: 0,
            kon: 0,
            endx: 0,
            echo_offset: 0,
            echo_length: 0,
            hist_l: [0; 8],
            hist_r: [0; 8],
            audio: Vec::new(),
            pend_envx: [0; 8],
            pend_outx: [0; 8],
            phase: 0,
            lat_adsr1: [0; 8],
            use_lat: false,
            envx_buf: 0,
            outx_buf: 0,
            ref_order: false,
        }
    }

    /// Select the S-DSP evaluation order of the reference (see E07-11e-Q1).
    pub fn set_reference_order(&mut self, on: bool) {
        self.ref_order = on;
    }

    pub fn reset(&mut self) {
        self.regs[reg::FLG as usize] = 0xE0;
        for v in &mut self.voices {
            *v = Voice::new();
        }
        self.counter = 0;
        self.noise = 0x4000;
        self.every_other = true;
        self.new_kon = 0;
        self.kon = 0;
        self.endx = 0;
        self.echo_offset = 0;
        self.echo_length = 0;
        for h in &mut self.hist_l { *h = 0; }
        for h in &mut self.hist_r { *h = 0; }
        self.audio.clear();
        self.pend_envx = [0; 8];
        self.pend_outx = [0; 8];
        self.phase = 0;
        self.lat_adsr1 = [0; 8];
        self.use_lat = false;
        self.envx_buf = 0;
        self.outx_buf = 0;
    }

    pub fn read(&mut self, addr: u8) -> u8 {
        self.regs[addr as usize & 0x7F]
    }

    pub fn read_peek(&self, addr: u8) -> u8 {
        self.regs[addr as usize & 0x7F]
    }

    pub fn write(&mut self, addr: u8, v: u8) {
        let a = addr as usize & 0x7F;
        self.regs[a] = v;
        // The S-DSP's ENVX/OUTX transfer buffers are loaded by CPU writes too.
        match a & 0x0F {
            8 => self.envx_buf = v,
            9 => self.outx_buf = v,
            _ => {}
        }
        if a == reg::KON as usize {
            self.new_kon = v;
        }
        if a == reg::ENDX as usize {
            self.endx = 0;
            self.regs[reg::ENDX as usize] = 0;
        }
    }


    /// Whole sample, results visible immediately (unit tests, offline use).
    pub fn run_sample(&mut self, ram: &mut [u8; 65536]) {
        self.run_sample_inner(ram, false);
        self.commit_all();
    }

    /// Publishes every pending ENVX/OUTX/ENDX value at once.
    fn commit_all(&mut self) {
        for x in 0..8 {
            self.regs[reg::vreg(x, reg::ENVX)] = self.pend_envx[x];
            self.regs[reg::vreg(x, reg::OUTX)] = self.pend_outx[x];
        }
        self.regs[reg::ENDX as usize] = self.endx;
        self.phase = 32;
    }

    /// One DSP clock after `run_sample_deferred` (spec 07b / S-DSP pipeline): voice i
    /// publishes ENDX at clock 2+3i, OUTX at 3+3i, ENVX at 4+3i.
    pub fn clock(&mut self) {
        let p = self.phase;
        if p >= 32 {
            return;
        }
        // ADSR1 is latched at the voice's V2 clock (voice 0: 21, voice x>=1: 3*(x-1)),
        // so a write after that clock only takes effect on the next sample.
        for x in 0..8usize {
            let p2 = if x == 0 { 21 } else { 3 * (x as u8 - 1) };
            if p == p2 {
                self.lat_adsr1[x] = self.regs[reg::vreg(x, reg::ADSR1)];
            }
        }
        // Shared OUTX/ENVX buffers (a CPU write to any voice's OUTX/ENVX also loads them):
        // voice x loads OUTX buf at 3x+1 (V6), ENVX buf + ENDX at 3x+2 (V7), then stores
        // OUTX at 3x+3 (V8) and ENVX at 3x+4 (V9).
        if (1..=25).contains(&p) {
            let k = (p - 1) as usize;
            let x = k / 3;
            match k % 3 {
                0 => {
                    // clock 3x+1: V9 of voice x-1 (store ENVX), then V6 of voice x (load OUTX)
                    if x > 0 {
                        self.regs[reg::vreg(x - 1, reg::ENVX)] = self.envx_buf;
                    }
                    if x < 8 {
                        self.outx_buf = self.pend_outx[x];
                    }
                }
                1 => {
                    self.envx_buf = self.pend_envx[x];
                    let bit = 1u8 << x;
                    let r = &mut self.regs[reg::ENDX as usize];
                    *r = (*r & !bit) | (self.endx & bit);
                }
                _ => self.regs[reg::vreg(x, reg::OUTX)] = self.outx_buf,
            }
        }
        self.phase = p + 1;
    }

    /// Whole sample computed at once; register publication is left to `clock`.
    pub fn run_sample_deferred(&mut self, ram: &mut [u8; 65536]) {
        self.run_sample_inner(ram, true);
    }

    fn run_sample_inner(&mut self, ram: &mut [u8; 65536], latched: bool) {
        self.use_lat = latched;
        self.phase = 0;
        self.step_counter_noise();
        self.scan_kon_koff();
        let (mut sum_l, mut sum_r, mut echo_l, mut echo_r, mut prev_out) = (0i32, 0i32, 0i32, 0i32, 0i32);
        for x in 0..8 {
            let (l, r) = self.voice_step(x, &*ram, prev_out);
            prev_out = self.voices[x].out;
            sum_l = brr::clamp16(sum_l + l);
            sum_r = brr::clamp16(sum_r + r);
            if (self.regs[reg::EON as usize] >> x) & 1 != 0 {
                echo_l = brr::clamp16(echo_l + l);
                echo_r = brr::clamp16(echo_r + r);
            }
        }
        let (ml, mr) = self.run_echo(ram, sum_l, sum_r, echo_l, echo_r);
        self.audio.push(ml as i16);
        self.audio.push(mr as i16);
        self.use_lat = false;
    }

    fn scan_kon_koff(&mut self) {
        self.every_other = !self.every_other;
        if self.every_other {
            self.kon = self.new_kon;
            self.new_kon = 0;
            let koff = self.regs[reg::KOFF as usize];
            for x in 0..8 {
                if (self.kon >> x) & 1 != 0 {
                    self.voices[x].kon_delay = 5;
                    self.voices[x].mode = voice::Mode::Attack;
                }
                if (koff >> x) & 1 != 0 {
                    self.voices[x].mode = voice::Mode::Release;
                }
            }
        }
        if self.regs[reg::FLG as usize] & 0x80 != 0 {
            for x in 0..8 {
                self.voices[x].mode = voice::Mode::Release;
                self.voices[x].env = 0;
            }
        }
    }

    pub fn take_audio(&mut self) -> Vec<i16> {
        std::mem::take(&mut self.audio)
    }

    pub fn new_kon(&self) -> u8 {
        self.new_kon
    }

    pub fn endx(&self) -> u8 {
        self.endx
    }

    pub fn debug_set_endx(&mut self, v: u8) {
        self.endx = v;
    }

    pub fn hash_into(&self, h: &mut Fnv64) {
        for &b in &self.regs {
            h.write_u8(b);
        }
        h.write_u16(self.counter);
        h.write_u16(self.noise);
        h.write_u8(if self.every_other { 1 } else { 0 });
        h.write_u8(self.new_kon);
        h.write_u8(self.kon);
        h.write_u8(self.endx);
        h.write_u16(self.echo_offset);
        h.write_u16(self.echo_length);
        // Pipeline timing state (phase, pend_*, lat_adsr1, envx_buf/outx_buf) is derived from
        // the SMP tick (ApuIo::dsp_div, hashed) and the register history; it is deliberately
        // not hashed so the frozen state baselines keep their meaning (E07-11e).
        for &v in &self.hist_l {
            h.write_u32(v as u32);
        }
        for &v in &self.hist_r {
            h.write_u32(v as u32);
        }
        for v in &self.voices {
            v.hash_into(h);
        }
    }
}

use crate::savestate::{State, StateWriter, StateReader, StateError};

impl State for Dsp {
    fn save(&self, w: &mut StateWriter) {
        // Save all fields in declaration order. The pipeline-timing fields (pend_*, phase,
        // lat_adsr1, use_lat, envx_buf, outx_buf) are not hashed but they steer the next
        // clocks, so they ARE saved.
        // not saved: ref_order (configuration flag set by the owner, not machine state)
        w.bytes(&self.regs);
        for v in &self.voices {
            v.save(w);
        }
        w.u16(self.counter);
        w.u16(self.noise);
        w.bool(self.every_other);
        w.u8(self.new_kon);
        w.u8(self.kon);
        w.u8(self.endx);
        w.u16(self.echo_offset);
        w.u16(self.echo_length);
        for &v in &self.hist_l {
            w.i32(v);
        }
        for &v in &self.hist_r {
            w.i32(v);
        }
        // audio: Vec<i16> -> write as vec_u16 with conversion.
        let audio_u16: Vec<u16> = self.audio.iter().map(|&x| x as u16).collect();
        w.vec_u16(&audio_u16);
        w.bytes(&self.pend_envx);
        w.bytes(&self.pend_outx);
        w.u8(self.phase);
        w.bytes(&self.lat_adsr1);
        w.bool(self.use_lat);
        w.u8(self.envx_buf);
        w.u8(self.outx_buf);
    }

    fn load(&mut self, r: &mut StateReader) -> Result<(), StateError> {
        // Load all fields in declaration order.
        r.bytes_into(&mut self.regs)?;
        for v in &mut self.voices {
            v.load(r)?;
        }
        self.counter = r.u16()?;
        self.noise = r.u16()?;
        self.every_other = r.bool()?;
        self.new_kon = r.u8()?;
        self.kon = r.u8()?;
        self.endx = r.u8()?;
        self.echo_offset = r.u16()?;
        self.echo_length = r.u16()?;
        for slot in &mut self.hist_l {
            *slot = r.i32()?;
        }
        for slot in &mut self.hist_r {
            *slot = r.i32()?;
        }
        // audio: read as vec_u16 and convert back to i16.
        let audio_u16 = r.vec_u16()?;
        self.audio = audio_u16.into_iter().map(|x| x as i16).collect();
        r.bytes_into(&mut self.pend_envx)?;
        r.bytes_into(&mut self.pend_outx)?;
        self.phase = r.u8()?;
        r.bytes_into(&mut self.lat_adsr1)?;
        self.use_lat = r.bool()?;
        self.envx_buf = r.u8()?;
        self.outx_buf = r.u8()?;
        Ok(())
    }
}

impl Default for Dsp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::savestate::{State, StateWriter, StateReader};
    use crate::hash::Fnv64;
    use crate::apu::Apu;

    fn dsp_hash(d: &Dsp) -> u64 {
        let mut h = Fnv64::new();
        d.hash_into(&mut h);
        h.finish()
    }

    #[test]
    fn e08_10_dsp_roundtrip() {
        // Set up a Dsp with non-trivial state including audio output.
        let mut apu = Apu::new();
        // Directory at 0x0200: start=loop=0x0300
        apu.debug_poke(0x0200, &[0x00, 0x03, 0x00, 0x03]);
        // BRR block at 0x0300: shift 12, filter 0, loop+end flags, all nibbles 7 (constant)
        apu.debug_poke(0x0300, &[0xC3, 0x77, 0x77, 0x77, 0x77, 0x77, 0x77, 0x77, 0x77]);
        // Infinite loop program at 0x0400: BRA -2
        apu.debug_poke(0x0400, &[0x2F, 0xFE]);
        apu.debug_set_pc(0x0400);

        let pairs = [
            (0x5D, 0x02), // DIR: directory at 0x0200
            (0x04, 0x00), // SRCN: sample channel 0
            (0x02, 0x00), // PITCHL
            (0x03, 0x10), // PITCHH: pitch = 0x1000
            (0x05, 0x00), // ADSR1: direct gain, no envelope
            (0x07, 0x7F), // GAIN: max -> env value 2032
            (0x00, 0x7F), // VOLL: full left volume
            (0x01, 0x7F), // VOLR: full right volume
            (0x0C, 0x7F), // MVOLL: master left volume
            (0x1C, 0x7F), // MVOLR: master right volume
            (0x6C, 0x20), // FLG: flags
        ];
        for &(r, v) in &pairs {
            apu.debug_dsp_write(r, v);
        }
        apu.debug_dsp_write(0x4C, 1); // KON: key on

        // Run some samples to get non-trivial state with audio.
        apu.step_to(50_000);
        let _audio = apu.take_audio();

        // Save DSP state directly via the Apu's internal Dsp.
        let dsp = &apu.io.dsp;
        let mut writer = StateWriter::new();
        dsp.save(&mut writer);
        let data = writer.finish();

        // Load into fresh Dsp instance.
        let mut loaded_dsp = Dsp::new();
        let mut reader = StateReader::new(&data);
        loaded_dsp.load(&mut reader).unwrap();

        // Verify bytes match on re-save.
        let mut writer2 = StateWriter::new();
        loaded_dsp.save(&mut writer2);
        let data2 = writer2.finish();
        assert_eq!(data, data2, "re-saved bytes must match original");

        // Verify hash matches.
        assert_eq!(dsp_hash(dsp), dsp_hash(&loaded_dsp), "hashes must match after roundtrip");
    }

    #[test]
    fn e08_10_dsp_behaves_identically() {
        // BRR note (DIR $0200 -> sample $0300 looping on itself), KON, 100 samples, then
        // save/restore into a fresh Dsp and run 200 more samples on both with identical RAM.
        let mut ram = [0u8; 65536];
        ram[0x200..0x204].copy_from_slice(&[0x00, 0x03, 0x00, 0x03]);
        ram[0x300..0x309].copy_from_slice(&[0xC3, 0x77, 0x77, 0x77, 0x77, 0x77, 0x77, 0x77, 0x77]);

        let mut a = Dsp::new();
        let pairs = [
            (0x5D, 0x02), (0x04, 0x00), (0x02, 0x00), (0x03, 0x10),
            (0x05, 0x00), (0x07, 0x7F), (0x00, 0x7F), (0x01, 0x7F),
            (0x0C, 0x7F), (0x1C, 0x7F), (0x6C, 0x20),
        ];
        for &(r, v) in &pairs {
            a.write(r, v);
        }
        a.write(0x4C, 1); // KON
        for _ in 0..100 {
            a.run_sample(&mut ram);
        }
        let _ = a.take_audio();

        let mut writer = StateWriter::new();
        a.save(&mut writer);
        let data = writer.finish();
        let mut b = Dsp::new();
        b.load(&mut StateReader::new(&data)).unwrap();
        assert_eq!(dsp_hash(&a), dsp_hash(&b));

        let mut ram_b = ram;
        for _ in 0..200 {
            a.run_sample(&mut ram);
            b.run_sample(&mut ram_b);
        }
        let audio_a = a.take_audio();
        assert!(audio_a.iter().any(|&x| x != 0), "the note must produce sound");
        assert_eq!(audio_a, b.take_audio(), "audio must match after save/restore");
        assert_eq!(dsp_hash(&a), dsp_hash(&b));
    }

    /// Truncation for both E08-10 types (also covered by `e08_10_truncated_voice` / `_dsp`).
    #[test]
    fn e08_10_truncated_each() {
        use crate::apu::dsp::voice::Voice;
        use crate::savestate::StateError;

        let mut w = StateWriter::new();
        Voice::new().save(&mut w);
        let data = w.finish();
        for n in [0, data.len() / 2, data.len() - 1] {
            let mut v = Voice::new();
            assert_eq!(v.load(&mut StateReader::new(&data[..n])), Err(StateError::Truncated), "voice n={n}");
        }

        let mut d = Dsp::new();
        d.write(0x6C, 0x20);
        let mut w = StateWriter::new();
        d.save(&mut w);
        let data = w.finish();
        for n in [0, data.len() / 2, data.len() - 1] {
            let mut d2 = Dsp::new();
            assert_eq!(d2.load(&mut StateReader::new(&data[..n])), Err(StateError::Truncated), "dsp n={n}");
        }
    }

    #[test]
    fn e08_10_truncated_dsp() {
        let mut apu = Apu::new();
        apu.debug_poke(0x0200, &[0x00, 0x03, 0x00, 0x03]);
        apu.debug_poke(0x0300, &[0xC3, 0x77, 0x77, 0x77, 0x77, 0x77, 0x77, 0x77, 0x77]);
        apu.debug_poke(0x0400, &[0x2F, 0xFE]);
        apu.debug_set_pc(0x0400);

        let pairs = [
            (0x5D, 0x02), (0x04, 0x00), (0x02, 0x00), (0x03, 0x10),
            (0x05, 0x00), (0x07, 0x7F), (0x00, 0x7F), (0x01, 0x7F),
            (0x0C, 0x7F), (0x1C, 0x7F), (0x6C, 0x20),
        ];
        for &(r, v) in &pairs {
            apu.debug_dsp_write(r, v);
        }

        let dsp = &apu.io.dsp;
        let mut writer = StateWriter::new();
        dsp.save(&mut writer);
        let data = writer.finish();

        // Test truncation at various points.
        for n in [0, data.len() / 2, data.len() - 1] {
            let mut loaded_dsp = Dsp::new();
            let mut reader = StateReader::new(&data[..n]);
            let result = loaded_dsp.load(&mut reader);
            assert!(matches!(result, Err(StateError::Truncated)),
                "truncation at {} should fail with Truncated", n);
        }
    }
}
