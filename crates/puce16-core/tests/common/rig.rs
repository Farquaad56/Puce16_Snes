#![allow(dead_code)]
//! Test harness: reproduces the catch_up loop for one dot at a time.

use puce16_core::bus::mmio::Mmio;
use puce16_core::joypad::Joypads;
use puce16_core::ppu::Ppu;
use puce16_core::region::Region;
use puce16_core::timing::{Timing, TimingOut};

pub struct Rig {
    pub ppu: Ppu,
    pub timing: Timing,
    pub mmio: Mmio,
    pub joy: Joypads,
    pub t: u64,
}

impl Rig {
    /// Create a fresh rig at master clock 0, position (0,0), without calling on_dot.
    pub fn new(region: Region) -> Rig {
        Rig {
            ppu: Ppu::new(region),
            timing: Timing::new(),
            mmio: Mmio::new(),
            joy: Joypads::new(),
            t: 0,
        }
    }

    /// Run the catch_up loop for exactly one dot. Returns the TimingOut from on_dot.
    pub fn step(&mut self) -> TimingOut {
        let c = self.ppu.dot_clocks();
        self.ppu.step_dot();
        self.t += c as u64;
        self.timing.on_dot(&mut self.mmio, &mut self.joy, &self.ppu, self.t)
    }

    /// Step until reaching the given (v, dot) position. Panics if more than 200_000 steps.
    pub fn run_until(&mut self, v: u16, dot: u16) {
        let mut count = 0;
        while self.ppu.v() != v || self.ppu.dot() != dot {
            self.step();
            count += 1;
            if count > 200_000 {
                panic!("run_until({},{}) exceeded 200k steps, now at ({},{}))",
                    v, dot, self.ppu.v(), self.ppu.dot());
            }
        }
    }

    /// Current PPU position.
    pub fn pos(&self) -> (u16, u16) {
        (self.ppu.v(), self.ppu.dot())
    }
}
