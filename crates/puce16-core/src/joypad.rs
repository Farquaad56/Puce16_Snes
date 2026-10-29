// 16-bit button format shared by core and desktop (spec 08, section 1).
pub const BTN_B: u16 = 1 << 15;
pub const BTN_Y: u16 = 1 << 14;
pub const BTN_SELECT: u16 = 1 << 13;
pub const BTN_START: u16 = 1 << 12;
pub const BTN_UP: u16 = 1 << 11;
pub const BTN_DOWN: u16 = 1 << 10;
pub const BTN_LEFT: u16 = 1 << 9;
pub const BTN_RIGHT: u16 = 1 << 8;
pub const BTN_A: u16 = 1 << 7;
pub const BTN_X: u16 = 1 << 6;
pub const BTN_L: u16 = 1 << 5;
pub const BTN_R: u16 = 1 << 4;

/// If both directions of a pair are pressed, clear that whole pair.
/// All other bits pass through unchanged.
pub fn filter_opposites(b: u16) -> u16 {
    let mut out = b;
    if b & BTN_LEFT != 0 && b & BTN_RIGHT != 0 {
        out &= !(BTN_LEFT | BTN_RIGHT);
    }
    if b & BTN_UP != 0 && b & BTN_DOWN != 0 {
        out &= !(BTN_UP | BTN_DOWN);
    }
    out
}

/// Two joypads with serial shift registers (spec 02 section 6.5).
pub struct Joypads {
    state: [u16; 2],
    shift: [u16; 2],
    out0: bool,
}

impl Joypads {
    /// Fresh joypads: no buttons pressed, shift registers full of 1s.
    pub fn new() -> Joypads {
        Joypads {
            state: [0, 0],
            shift: [0xFFFF, 0xFFFF],
            out0: false,
        }
    }

    /// Set the button state for a port (format: spec 02 section 6.5).
    pub fn set_buttons(&mut self, port: usize, v: u16) {
        if port < 2 {
            self.state[port] = v;
        }
    }

    /// Read the current button state for a port.
    pub fn buttons(&self, port: usize) -> u16 {
        if port < 2 {
            self.state[port]
        } else {
            0
        }
    }

    /// Write to $4016 (JOYOUT). bit0 = OUT0 latch.
    pub fn write_out(&mut self, bit0: bool) {
        if bit0 || self.out0 {
            for p in 0..2 {
                self.shift[p] = self.state[p];
            }
        }
        self.out0 = bit0;
    }

    /// Read from $4016/$4017 (JOYSER0/JOYSER1). Returns D0 bit.
    pub fn read_port(&mut self, port: usize) -> u8 {
        if port >= 2 {
            return 1;
        }
        if self.out0 {
            ((self.state[port] >> 15) & 1) as u8
        } else {
            let b = (self.shift[port] >> 15) & 1;
            self.shift[port] = (self.shift[port] << 1) | 1;
            b as u8
        }
    }

    /// Auto-read: return [JOY1, JOY2, 0, 0] and reset shift registers.
    pub fn autoread_values(&mut self) -> [u16; 4] {
        let vals = [self.state[0], self.state[1], 0, 0];
        for p in 0..2 {
            self.shift[p] = 0xFFFF;
        }
        vals
    }

    /// Hash joypad state into the given hasher.
    pub fn hash_into(&self, h: &mut crate::hash::Fnv64) {
        for p in 0..2 {
            h.write_u16(self.state[p]);
        }
        for p in 0..2 {
            h.write_u16(self.shift[p]);
        }
        if self.out0 {
            h.write_u8(1);
        } else {
            h.write_u8(0);
        }
    }
}

impl crate::savestate::State for Joypads {
    fn save(&self, w: &mut crate::savestate::StateWriter) {
        // state[0], state[1] (u16 each), shift[0], shift[1] (u16 each), out0 (bool)
        w.u16(self.state[0]);
        w.u16(self.state[1]);
        w.u16(self.shift[0]);
        w.u16(self.shift[1]);
        w.bool(self.out0);
    }

    fn load(&mut self, r: &mut crate::savestate::StateReader) -> Result<(), crate::savestate::StateError> {
        self.state[0] = r.u16()?;
        self.state[1] = r.u16()?;
        self.shift[0] = r.u16()?;
        self.shift[1] = r.u16()?;
        self.out0 = r.bool()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::savestate::{State, StateWriter, StateReader};

    #[test]
    fn e08_03_joypads_roundtrip() {
        let mut j = Joypads::new();
        // Set all fields to distinct non-zero values
        j.state[0] = 0xAAAA;
        j.state[1] = 0xBBBB;
        j.shift[0] = 0xCCCC;
        j.shift[1] = 0xDDDD;
        j.out0 = true;

        // Save
        let mut w = StateWriter::new();
        j.save(&mut w);
        let data = w.finish();

        // Load into fresh instance
        let mut j2 = Joypads::new();
        let mut r = StateReader::new(&data);
        j2.load(&mut r).unwrap();

        // Verify fields match
        assert_eq!(j.state[0], j2.state[0]);
        assert_eq!(j.state[1], j2.state[1]);
        assert_eq!(j.shift[0], j2.shift[0]);
        assert_eq!(j.shift[1], j2.shift[1]);
        assert_eq!(j.out0, j2.out0);

        // Verify re-save produces identical bytes
        let mut w2 = StateWriter::new();
        j2.save(&mut w2);
        assert_eq!(data, w2.finish());

        // Verify hash_into matches
        let mut h1 = crate::hash::Fnv64::new();
        j.hash_into(&mut h1);
        let mut h2 = crate::hash::Fnv64::new();
        j2.hash_into(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn e08_03_joypads_truncated() {
        let mut j = Joypads::new();
        j.state[0] = 0xAAAA;
        j.state[1] = 0xBBBB;
        j.shift[0] = 0xCCCC;
        j.shift[1] = 0xDDDD;
        j.out0 = true;

        let mut w = StateWriter::new();
        j.save(&mut w);
        let data = w.finish();

        // Test truncation at various points
        for n in [0, data.len() / 2, data.len() - 1] {
            let mut r = StateReader::new(&data[..n]);
            let mut j2 = Joypads::new();
            assert!(j2.load(&mut r).is_err(), "truncated at {} should fail", n);
        }
    }
}
