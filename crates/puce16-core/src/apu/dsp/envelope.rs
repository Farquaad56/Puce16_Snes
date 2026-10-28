use super::{Dsp, reg, tables::poll};
use super::voice::Mode;

impl Dsp {
    /// Run envelope for voice x (spec 07b section 4.2).
    pub(super) fn run_envelope(&mut self, x: usize) {
        let v = &mut self.voices[x & 7];
        if v.mode == Mode::Release {
            v.env = i32::max(v.env - 8, 0);
            v.hidden_env = v.env;
            return;
        }

        let adsr1 = if self.use_lat {
            self.lat_adsr1[x & 7]
        } else {
            self.regs[reg::vreg(x, reg::ADSR1)]
        };
        let adsr2 = self.regs[reg::vreg(x, reg::ADSR2)];
        let gain = self.regs[reg::vreg(x, reg::GAIN)];

        let rate: i32;
        let mut e = v.env;
        let mut data = adsr2 as i32;

        if (adsr1 & 0x80) != 0 {
            // ADSR mode: Decay/Sustain decrease, Attack increases
            let is_decay_or_sustain = v.mode == Mode::Decay || v.mode == Mode::Sustain;
            if is_decay_or_sustain {
                e -= 1;
                e -= e >> 8;
                rate = if v.mode == Mode::Decay {
                    (((adsr1 >> 4) & 7) * 2 + 16) as i32
                } else {
                    (adsr2 & 0x1F) as i32
                };
            } else {
                rate = ((adsr1 & 0xF) * 2 + 1) as i32;
                e += if rate < 31 { 32 } else { 1024 };
            }
        } else {
            // GAIN mode
            data = gain as i32;
            let m = (gain >> 5) & 7;
            if m < 4 {
                e = data << 4;
                rate = 31;
            } else {
                rate = (gain & 0x1F) as i32;
                if m == 4 {
                    e -= 32;
                } else if m < 6 {
                    e -= 1;
                    e -= e >> 8;
                } else {
                    e += 32;
                    if m > 6 && v.hidden_env >= 0x600 {
                        e += 8 - 32;
                    }
                }
            }
        }

        // Decay -> Sustain transition check
        if (e >> 8) == (data >> 5) && v.mode == Mode::Decay {
            v.mode = Mode::Sustain;
        }

        v.hidden_env = e;

        // Clamp and Attack -> Decay transition
        if e < 0 || e > 0x7FF {
            e = if e < 0 { 0 } else { 0x7FF };
            if v.mode == Mode::Attack {
                v.mode = Mode::Decay;
            }
        }

        // Apply envelope update based on poll result
        if poll(self.counter, rate as u8) {
            v.env = e;
        }
    }

    /// Debug wrapper to run envelope for voice x.
    pub fn debug_run_envelope(&mut self, x: usize) {
        self.run_envelope(x);
    }

    /// Read the period counter value.
    pub fn counter(&self) -> u16 {
        self.counter
    }

    /// Set the period counter for testing slow envelope rates.
    pub fn debug_set_counter(&mut self, c: u16) {
        self.counter = c;
    }

    /// Step shared counter and noise LFSR (spec 07b sections 2 and 5).
    pub(super) fn step_counter_noise(&mut self) {
        // Counter decrement with wrap at 0x77FF
        if self.counter == 0 {
            self.counter = 0x77FF;
        } else {
            self.counter -= 1;
        }

        // Noise LFSR update based on FLG rate bits (lower 5)
        let flg_rate = self.regs[reg::FLG as usize] & 0x1F;
        if poll(self.counter, flg_rate) {
            let fb = (self.noise << 13) ^ (self.noise << 14);
            self.noise = (fb & 0x4000) ^ (self.noise >> 1);
        }
    }

    /// Debug wrapper to step counter and noise.
    pub fn debug_step_counter_noise(&mut self) {
        self.step_counter_noise();
    }

    /// Read current noise value.
    pub fn noise(&self) -> u16 {
        self.noise
    }

    /// Set noise value for testing.
    pub fn debug_set_noise(&mut self, n: u16) {
        self.noise = n;
    }
}
