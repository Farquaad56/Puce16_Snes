//! S-DD1 decompressor: BG (bit generators) and PEM (probability estimation module).

use super::tables;
use super::decomp::Decompressor;

impl Decompressor {
    /// BG - bit generator for instance n.
    /// Returns (bit, end_of_run).
    pub fn bg_bit<F: Fn(u32) -> u8>(&mut self, n: usize, rom: &F) -> (u8, bool) {
        if self.mps_count[n] == 0 && !self.lps_index[n] {
            let rc = self.run_count(n as u8, rom);
            self.mps_count[n] = rc.0;
            self.lps_index[n] = rc.1;
        }

        let bit;
        if self.mps_count[n] != 0 {
            bit = 0;
            self.mps_count[n] -= 1;
        } else {
            bit = 1;
            self.lps_index[n] = false;
        }

        let end_of_run = self.mps_count[n] == 0 && !self.lps_index[n];
        (bit, end_of_run)
    }

    /// PEM - probability estimation for context ctx.
    pub fn pem_bit<F: Fn(u32) -> u8>(&mut self, ctx: usize, rom: &F) -> u8 {
        let st = self.status[ctx];
        let cur_mps = self.mps[ctx];

        let (code_number, next_mps, next_lps) = tables::EVOLUTION[st as usize];
        let (bit, end) = self.bg_bit(code_number as usize, rom);

        if end {
            if bit != 0 {
                if st & 0xFE == 0 {
                    self.mps[ctx] ^= 1;
                }
                self.status[ctx] = next_lps;
            } else {
                self.status[ctx] = next_mps;
            }
        }

        bit ^ cur_mps
    }

    /// Return the PEM state for context ctx: (status, mps).
    pub fn pem_state(&self, ctx: usize) -> (u8, u8) {
        (self.status[ctx], self.mps[ctx])
    }
}