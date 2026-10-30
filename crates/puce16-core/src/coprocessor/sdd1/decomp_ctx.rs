//! S-DD1 decompressor: CM (context model) and OL (output logic).

use super::decomp::Decompressor;

impl Decompressor {
    /// CM - context model bit.
    /// Determines the current bitplane, computes the context index from previous bits,
    /// calls PEM to get the decoded bit, updates state, and returns the bit.
    pub fn cm_bit<F: Fn(u32) -> u8>(&mut self, rom: &F) -> u8 {
        // Update current_bitplane based on bitplanes_info
        match self.bitplanes_info {
            0x00 => {
                self.current_bitplane ^= 1;
            }
            0x40 => {
                self.current_bitplane ^= 1;
                if self.bit_number & 0x7F == 0 {
                    self.current_bitplane = (self.current_bitplane + 2) & 7;
                }
            }
            0x80 => {
                self.current_bitplane ^= 1;
                if self.bit_number & 0x7F == 0 {
                    self.current_bitplane ^= 2;
                }
            }
            0xC0 => {
                self.current_bitplane = self.bit_number & 7;
            }
            _ => {} // should not happen
        }

        // Get previous bits for this bitplane (u16)
        let cb = self.prev[self.current_bitplane as usize];

        // Compute context index: high nibble from bitplane parity, low nibble from prev bits
        let mut ctx = ((self.current_bitplane & 1) << 4) as u8;
        match self.context_info {
            0x00 => {
                ctx |= (((cb & 0x01C0) >> 5) | (cb & 0x0001)) as u8;
            }
            0x10 => {
                ctx |= (((cb & 0x0180) >> 5) | (cb & 0x0001)) as u8;
            }
            0x20 => {
                ctx |= (((cb & 0x00C0) >> 5) | (cb & 0x0001)) as u8;
            }
            0x30 => {
                ctx |= (((cb & 0x0180) >> 5) | (cb & 0x0003)) as u8;
            }
            _ => {} // should not happen
        }

        // Get decoded bit from PEM
        let bit = self.pem_bit(ctx as usize, rom);

        // Update prev bits for this bitplane: shift left, add new bit (u16 truncation)
        self.prev[self.current_bitplane as usize] = ((cb << 1) | bit as u16) & 0xFFFF;

        // Increment bit counter (wraps at 256)
        self.bit_number = self.bit_number.wrapping_add(1);

        bit
    }

    /// OL - output logic: read one decompressed byte.
    pub fn read<F: Fn(u32) -> u8>(&mut self, rom: &F) -> u8 {
        if self.bitplanes_info != 0xC0 {
            // Non-mode7: produce pairs of bytes (r1 then r2)
            if self.r0 == 0 {
                // Second byte of the pair
                self.r0 = !self.r0; // becomes 0xFF
                return self.r2;
            }
            // First byte: decode a full byte pair
            self.r0 = 0x80;
            self.r1 = 0;
            self.r2 = 0;
            while self.r0 != 0 {
                if self.cm_bit(rom) != 0 {
                    self.r1 |= self.r0;
                }
                if self.cm_bit(rom) != 0 {
                    self.r2 |= self.r0;
                }
                self.r0 >>= 1;
            }
            // r0 is now 0, so next read() returns r2
            return self.r1;
        } else {
            // Mode7: single byte per call, LSB first
            self.r0 = 1;
            self.r1 = 0;
            while self.r0 != 0 {
                if self.cm_bit(rom) != 0 {
                    self.r1 |= self.r0;
                }
                // r0 <<= 1 wraps at u8: 0x80 << 1 = 0
                self.r0 = (self.r0 << 1) & 0xFF;
            }
            return self.r1;
        }
    }
}
