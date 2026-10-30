//! S-DD1 decompressor: struct, init, IM (input manager), GCD (Golomb code decoder).

use super::tables;

#[derive(Clone, Debug)]
pub struct Decompressor {
    // IM - input manager
    pub(super) im_offset: u32,
    pub(super) im_bit_count: u8,
    // BG - bit generators (8 instances)
    pub(super) mps_count: [u8; 8],
    pub(super) lps_index: [bool; 8],
    // PEM - probability estimation (32 contexts)
    pub(super) status: [u8; 32],
    pub(super) mps: [u8; 32],
    // CM - context model
    pub(super) bitplanes_info: u8,
    pub(super) context_info: u8,
    pub(super) bit_number: u8,
    pub(super) current_bitplane: u8,
    pub(super) prev: [u16; 8],
    // OL - output logic
    pub(super) r0: u8,
    pub(super) r1: u8,
    pub(super) r2: u8,
}

impl Default for Decompressor {
    fn default() -> Self {
        Self::new()
    }
}

impl Decompressor {
    /// Create a new decompressor in null (not started) state.
    pub fn new() -> Decompressor {
        Decompressor {
            im_offset: 0,
            im_bit_count: 0,
            mps_count: [0; 8],
            lps_index: [false; 8],
            status: [0; 32],
            mps: [0; 32],
            bitplanes_info: 0,
            context_info: 0,
            bit_number: 0,
            current_bitplane: 0,
            prev: [0; 8],
            r0: 0,
            r1: 0,
            r2: 0,
        }
    }

    /// Initialize the decompressor for a new stream starting at `start`.
    /// Reads the header byte and sets up all state per spec section 6.
    pub fn init<F: Fn(u32) -> u8>(&mut self, start: u32, rom: &F) {
        // IM: offset = start, bit_count = 4 (first nibble is header)
        self.im_offset = start;
        self.im_bit_count = 4;

        // BG: all zeros
        for i in 0..8 {
            self.mps_count[i] = 0;
            self.lps_index[i] = false;
        }

        // PEM: all status and mps to 0
        for i in 0..32 {
            self.status[i] = 0;
            self.mps[i] = 0;
        }

        // CM: read header byte, extract bitplanes_info and context_info
        let h = rom(start);
        self.bitplanes_info = h & 0xC0;
        self.context_info = h & 0x30;
        self.bit_number = 0;
        for i in 0..8 {
            self.prev[i] = 0;
        }

        // current_bitplane from bitplanes_info table: 0x00->1, 0x40->7, 0x80->3, 0xC0->0
        match self.bitplanes_info {
            0x00 => self.current_bitplane = 1,
            0x40 => self.current_bitplane = 7,
            0x80 => self.current_bitplane = 3,
            0xC0 => self.current_bitplane = 0,
            _ => {} // should not happen
        }

        // OL: r0 = 1, r1 = r2 = 0
        self.r0 = 1;
        self.r1 = 0;
        self.r2 = 0;
    }

    /// IM - code_word: read a Golomb-Rice codeword of given length.
    pub fn code_word<F: Fn(u32) -> u8>(&mut self, code_length: u8, rom: &F) -> u8 {
        // Compute in u16 to avoid premature truncation, then truncate to u8
        let mut cw = ((rom(self.im_offset) as u16) << self.im_bit_count) as u8;
        self.im_bit_count += 1;

        if cw & 0x80 != 0 {
            // Need second byte: bit_count already incremented
            let shift = 9 - self.im_bit_count;
            let second = (rom(self.im_offset + 1) as u16) >> shift;
            cw |= second as u8;
            self.im_bit_count += code_length;
        }

        if self.im_bit_count & 0x08 != 0 {
            self.im_offset += 1;
            self.im_bit_count &= 0x07;
        }

        cw
    }

    /// GCD - run_count: decode a Golomb-Rice codeword into (mps_count, lps_index).
    pub fn run_count<F: Fn(u32) -> u8>(&mut self, code_number: u8, rom: &F) -> (u8, bool) {
        let cw = self.code_word(code_number, rom);

        if cw & 0x80 != 0 {
            // LPS case: look up in RUNCOUNT table
            let index = (cw >> (code_number ^ 7)) as usize;
            (tables::RUNCOUNT[index], true)
        } else {
            // MPS case: power of two
            (1 << code_number, false)
        }
    }

    /// Return the current IM position (offset, bit_count).
    pub fn im_position(&self) -> (u32, u8) {
        (self.im_offset, self.im_bit_count)
    }
}
