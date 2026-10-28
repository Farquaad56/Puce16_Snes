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
        }
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
    }

    pub fn read(&mut self, addr: u8) -> u8 {
        self.regs[addr as usize & 0x7F]
    }

    pub fn write(&mut self, addr: u8, v: u8) {
        let a = addr as usize & 0x7F;
        self.regs[a] = v;
        if a == reg::KON as usize {
            self.new_kon = v;
        }
        if a == reg::ENDX as usize {
            self.endx = 0;
            self.regs[reg::ENDX as usize] = 0;
        }
    }

    pub fn run_sample(&mut self, ram: &mut [u8; 65536]) {
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
        self.regs[reg::ENDX as usize] = self.endx;
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

impl Default for Dsp {
    fn default() -> Self {
        Self::new()
    }
}
