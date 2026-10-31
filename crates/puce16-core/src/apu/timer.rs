//! SPC700 timers (three instances, divisors 128/128/16).

#[derive(Clone, Debug)]
pub struct Timer {
    divider: u16,
    enable: bool,
    target: u8,
    stage0: u16,
    stage1: u8,
    out: u8,
}

impl Timer {
    pub fn new(divider: u16) -> Timer {
        Timer { divider, enable: false, target: 0, stage0: 0, stage1: 0, out: 0 }
    }

    pub fn tick(&mut self) {
        self.stage0 += 1;
        if self.stage0 == self.divider {
            self.stage0 = 0;
            if self.enable {
                self.stage1 = self.stage1.wrapping_add(1);
                if self.stage1 == self.target {
                    self.stage1 = 0;
                    self.out = (self.out + 1) & 0x0F;
                }
            }
        }
    }

    pub fn set_enable(&mut self, on: bool) {
        if !self.enable && on {
            self.stage1 = 0;
            self.out = 0;
        }
        self.enable = on;
    }

    pub fn set_target(&mut self, t: u8) {
        self.target = t;
    }

    pub fn read_out(&mut self) -> u8 {
        let r = self.out & 0x0F;
        self.out = 0;
        r
    }

    pub fn target(&self) -> u8 { self.target }
    pub fn out(&self) -> u8 { self.out }
    pub fn stage1(&self) -> u8 { self.stage1 }
    pub fn stage0(&self) -> u16 { self.stage0 }
    pub fn enabled(&self) -> bool { self.enable }

    pub fn hash_into(&self, h: &mut crate::hash::Fnv64) {
        h.write_u16(self.divider);
        h.write_u8(if self.enable { 1 } else { 0 });
        h.write_u8(self.target);
        h.write_u16(self.stage0);
        h.write_u8(self.stage1);
        h.write_u8(self.out);
    }
}

impl crate::savestate::State for Timer {
    fn save(&self, w: &mut crate::savestate::StateWriter) {
        // Save all fields in declaration order. All are covered by hash_into.
        w.u16(self.divider);
        w.bool(self.enable);
        w.u8(self.target);
        w.u16(self.stage0);
        w.u8(self.stage1);
        w.u8(self.out);
    }

    fn load(&mut self, r: &mut crate::savestate::StateReader) -> Result<(), crate::savestate::StateError> {
        // Load all fields in declaration order. Verify divider matches.
        let saved_divider = r.u16()?;
        if saved_divider != self.divider {
            return Err(crate::savestate::StateError::Mismatch("timer divider"));
        }
        self.enable = r.bool()?;
        self.target = r.u8()?;
        self.stage0 = r.u16()?;
        self.stage1 = r.u8()?;
        self.out = r.u8()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::savestate::{State, StateWriter, StateReader};
    use crate::hash::Fnv64;

    fn timer_hash(t: &Timer) -> u64 {
        let mut h = Fnv64::new();
        t.hash_into(&mut h);
        h.finish()
    }

    #[test]
    fn e08_09_timer_roundtrip() {
        // Set every saved field to a distinct non-zero value.
        let mut timer = Timer::new(16);
        timer.enable = true;
        timer.target = 0x7F;
        timer.stage0 = 54321;
        timer.stage1 = 0xAB;
        timer.out = 0xCD;

        // Save.
        let mut writer = StateWriter::new();
        timer.save(&mut writer);
        let data = writer.finish();

        // Load into fresh instance with same divider.
        let mut loaded = Timer::new(16);
        let mut reader = StateReader::new(&data);
        loaded.load(&mut reader).unwrap();

        // Verify bytes match on re-save.
        let mut writer2 = StateWriter::new();
        loaded.save(&mut writer2);
        let data2 = writer2.finish();
        assert_eq!(data, data2, "re-saved bytes must match original");

        // Verify hash matches.
        assert_eq!(timer_hash(&timer), timer_hash(&loaded), "hashes must match after roundtrip");
    }

    #[test]
    fn e08_09_timer_truncated() {
        let mut timer = Timer::new(16);
        timer.enable = true;
        timer.target = 0x7F;
        timer.stage0 = 54321;
        timer.stage1 = 0xAB;
        timer.out = 0xCD;

        let mut writer = StateWriter::new();
        timer.save(&mut writer);
        let data = writer.finish();

        // Test truncation at various points.
        for n in [0, data.len() / 2, data.len() - 1] {
            let mut loaded = Timer::new(16);
            let mut reader = StateReader::new(&data[..n]);
            let result = loaded.load(&mut reader);
            assert!(matches!(result, Err(crate::savestate::StateError::Truncated)),
                "truncation at {} should fail with Truncated", n);
        }
    }

    #[test]
    fn e08_09_timer_divider_mismatch() {
        // Save a timer with divider 16, load into one with divider 128 -> Mismatch.
        let mut timer = Timer::new(16);
        timer.enable = true;
        timer.target = 0x7F;

        let mut writer = StateWriter::new();
        timer.save(&mut writer);
        let data = writer.finish();

        let mut loaded = Timer::new(128);
        let mut reader = StateReader::new(&data);
        let result = loaded.load(&mut reader);
        assert!(matches!(result, Err(crate::savestate::StateError::Mismatch("timer divider"))),
            "divider mismatch should fail with Mismatch(\"timer divider\")");
    }

    #[test]
    fn e08_09_timer_behaves_identically() {
        // After restoring mid-counting, 100 ticks give same out() and stage1().
        let mut timer = Timer::new(16);
        timer.set_enable(true);
        timer.set_target(5);

        // Run some ticks to get into non-trivial state.
        for _ in 0..80 {
            timer.tick();
        }

        // Save.
        let mut writer = StateWriter::new();
        timer.save(&mut writer);
        let data = writer.finish();

        // Load into fresh instance with same divider.
        let mut loaded = Timer::new(16);
        let mut reader = StateReader::new(&data);
        loaded.load(&mut reader).unwrap();

        // Run both for 100 more ticks.
        for _ in 0..100 {
            timer.tick();
            loaded.tick();
        }

        assert_eq!(timer.out(), loaded.out(), "out() must match after 100 ticks");
        assert_eq!(timer.stage1(), loaded.stage1(), "stage1() must match after 100 ticks");
    }
}
