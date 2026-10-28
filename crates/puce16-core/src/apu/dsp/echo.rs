use super::{Dsp, reg};
use super::brr::clamp16;

impl Dsp {
    pub(super) fn run_echo(&mut self, ram: &mut [u8; 65536], sum_l: i32, sum_r: i32, echo_l: i32, echo_r: i32) -> (i32, i32) {
        // Step 1: compute echo length when offset is at start
        if self.echo_offset == 0 {
            self.echo_length = ((self.regs[reg::EDL as usize] & 15) as u16) << 11;
        }

        // Step 2: compute pointer into echo buffer
        let ptr = ((self.regs[reg::ESA as usize] as u16) << 8).wrapping_add(self.echo_offset);

        // Step 3: read previous echo samples (little-endian i16, shifted right by 1)
        let l = {
            let lo = ram[ptr as usize];
            let hi = ram[(ptr.wrapping_add(1)) as usize];
            ((hi as u16) << 8 | lo as u16) as i16 >> 1
        };
        let r = {
            let lo = ram[(ptr.wrapping_add(2)) as usize];
            let hi = ram[(ptr.wrapping_add(3)) as usize];
            ((hi as u16) << 8 | lo as u16) as i16 >> 1
        };

        // Step 4: shift history down, push new sample at index 7 (newest)
        for i in 0..7 {
            self.hist_l[i] = self.hist_l[i + 1];
            self.hist_r[i] = self.hist_r[i + 1];
        }
        self.hist_l[7] = l as i32;
        self.hist_r[7] = r as i32;

        // Step 5: FIR filter (8 taps, signed coefficients)
        let fir_l = {
            let mut acc = 0i32;
            for i in 0..6 {
                let coeff = self.regs[reg::fir(i) as usize] as i8 as i32;
                acc += (self.hist_l[i] * coeff) >> 6;
            }
            // Clip to 16-bit before adding last tap (hardware quirk)
            let fir = acc as i16 as i32;
            let coeff7 = self.regs[reg::fir(7) as usize] as i8 as i32;
            let fir = fir + ((self.hist_l[7] * coeff7) >> 6);
            clamp16(fir) & !1
        };

        let fir_r = {
            let mut acc = 0i32;
            for i in 0..6 {
                let coeff = self.regs[reg::fir(i) as usize] as i8 as i32;
                acc += (self.hist_r[i] * coeff) >> 6;
            }
            let fir = acc as i16 as i32;
            let coeff7 = self.regs[reg::fir(7) as usize] as i8 as i32;
            let fir = fir + ((self.hist_r[7] * coeff7) >> 6);
            clamp16(fir) & !1
        };

        // Step 6: mix dry signal with echo output
        let mvoll = self.regs[reg::MVOLL as usize] as i8 as i32;
        let mvolr = self.regs[reg::MVOLR as usize] as i8 as i32;
        let evoll = self.regs[reg::EVOLL as usize] as i8 as i32;
        let evolr = self.regs[reg::EVOLR as usize] as i8 as i32;

        let main_l = clamp16(((sum_l * mvoll) >> 7) + ((fir_l * evoll) >> 7));
        let main_r = clamp16(((sum_r * mvolr) >> 7) + ((fir_r * evolr) >> 7));

        // Step 7: feedback and write to echo buffer (before mute check)
        let efb = self.regs[reg::EFB as usize] as i8 as i32;
        let wl = clamp16(echo_l + ((fir_l * efb) >> 7)) & !1;
        let wr = clamp16(echo_r + ((fir_r * efb) >> 7)) & !1;

        // Echo write blocked by FLG bit 5
        if self.regs[reg::FLG as usize] & 0x20 == 0 {
            ram[ptr as usize] = (wl as i16) as u8;
            ram[(ptr.wrapping_add(1)) as usize] = ((wl as i16) >> 8) as u8;
            ram[(ptr.wrapping_add(2)) as usize] = (wr as i16) as u8;
            ram[(ptr.wrapping_add(3)) as usize] = ((wr as i16) >> 8) as u8;
        }

        // Step 8: advance offset, wrap at length
        self.echo_offset += 4;
        if self.echo_offset >= self.echo_length {
            self.echo_offset = 0;
        }

        // Mute flag (FLG bit 6): zero output but echo writes already done
        if self.regs[reg::FLG as usize] & 0x40 != 0 {
            return (0, 0);
        }

        (main_l, main_r)
    }
}
