pub mod spc700;
pub mod dsp;

/// SPC700 CPU communication ports (skeleton, will be completed in E06).
/// to_apu: bytes written by 65816 CPU destined for SPC700.
/// from_apu: bytes written by SPC700 that the 65816 CPU can read.
pub struct Apu {
    to_apu: [u8; 4],
    from_apu: [u8; 4],
}

impl Apu {
    /// Fresh APU with all ports zeroed.
    pub fn new() -> Apu {
        Apu {
            to_apu: [0, 0, 0, 0],
            from_apu: [0, 0, 0, 0],
        }
    }

    /// Reset all ports to zero.
    pub fn reset(&mut self) {
        for i in 0..4 {
            self.to_apu[i] = 0;
            self.from_apu[i] = 0;
        }
    }

    /// Read from port $2140-$2143 (mirrored every 4). Returns byte written by SPC700.
    pub fn port_read(&self, i: usize) -> u8 {
        self.from_apu[i & 3]
    }

    /// Write to port $2140-$2143 (mirrored every 4). Sends byte to SPC700.
    pub fn port_write(&mut self, i: usize, v: u8) {
        self.to_apu[i & 3] = v;
    }

    /// Advance APU clock (no-op until E06 when SPC700 is implemented).
    pub fn step_to(&mut self, _master: u64) {}

    /// Debug/test access to bytes written by CPU destined for SPC700.
    pub fn cpu_to_apu(&self, i: usize) -> u8 {
        self.to_apu[i & 3]
    }

    /// Debug/test: set a byte that the CPU will read from this port.
    pub fn debug_set_from_apu(&mut self, i: usize, v: u8) {
        self.from_apu[i & 3] = v;
    }

    /// Hash APU state into the given hasher.
    pub fn hash_into(&self, h: &mut crate::hash::Fnv64) {
        for i in 0..4 {
            h.write_u8(self.to_apu[i]);
        }
        for i in 0..4 {
            h.write_u8(self.from_apu[i]);
        }
    }
}
