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
