//! SPC700 I/O bus: 64K RAM, $F0-$FF registers, ports, timers, IPL overlay.

use crate::apu::dsp::Dsp;
use crate::apu::ipl::IPL_ROM;
use crate::apu::spc700::SmpBus;
use crate::apu::timer::Timer;
use crate::hash::Fnv64;
use crate::savestate::{State, StateWriter, StateReader, StateError};

pub struct ApuIo {
    pub ram: Box<[u8; 65536]>,
    pub dsp: Dsp,
    pub timers: [Timer; 3],
    pub port_in: [u8; 4],
    pub port_out: [u8; 4],
    pub control: u8,
    pub dspaddr: u8,
    dsp_div: u8,
}

impl ApuIo {
    pub fn new() -> ApuIo {
        let mut ram = Box::new([0u8; 65536]);
        // UNKNOWN - to confirm: RAM power-on value (assuming 0x00)
        for b in &mut *ram {
            *b = 0;
        }
        ApuIo {
            ram,
            dsp: Dsp::new(),
            timers: [Timer::new(128), Timer::new(128), Timer::new(16)],
            port_in: [0, 0, 0, 0],
            port_out: [0, 0, 0, 0],
            control: 0xB0,
            dspaddr: 0,
            dsp_div: 0,
        }
    }

    pub fn reset(&mut self) {
        for i in 0..4 {
            self.port_in[i] = 0;
            self.port_out[i] = 0;
        }
        self.control = 0xB0;
        self.dspaddr = 0;
        self.dsp_div = 0;
        self.timers[0] = Timer::new(128);
        self.timers[1] = Timer::new(128);
        self.timers[2] = Timer::new(16);
        self.dsp.reset();
    }

    fn tick(&mut self) {
        for t in &mut self.timers {
            t.tick();
        }
        self.dsp_div += 1;
        if self.dsp_div == 32 {
            self.dsp_div = 0;
            self.dsp.run_sample_deferred(&mut *self.ram);
        } else {
            self.dsp.clock();
        }
    }

    pub fn hash_into(&self, h: &mut Fnv64) {
        for b in &*self.ram {
            h.write_u8(*b);
        }
        self.dsp.hash_into(h);
        for t in &self.timers {
            t.hash_into(h);
        }
        for i in 0..4 {
            h.write_u8(self.port_in[i]);
            h.write_u8(self.port_out[i]);
        }
        h.write_u8(self.control);
        h.write_u8(self.dspaddr);
        h.write_u8(self.dsp_div);
    }
}

impl State for ApuIo {
    fn save(&self, w: &mut StateWriter) {
        // Save all fields in declaration order. All are covered by hash_into.
        w.bytes(&*self.ram);
        w.u8(self.control);
        w.u8(self.dspaddr);
        w.u8(self.dsp_div);
        for i in 0..4 {
            w.u8(self.port_in[i]);
        }
        for i in 0..4 {
            w.u8(self.port_out[i]);
        }
        for t in &self.timers {
            t.save(w);
        }
        self.dsp.save(w);
    }

    fn load(&mut self, r: &mut StateReader) -> Result<(), StateError> {
        // Load all fields in declaration order.
        r.bytes_into(&mut *self.ram)?;
        self.control = r.u8()?;
        self.dspaddr = r.u8()?;
        self.dsp_div = r.u8()?;
        for i in 0..4 {
            self.port_in[i] = r.u8()?;
        }
        for i in 0..4 {
            self.port_out[i] = r.u8()?;
        }
        for t in &mut self.timers {
            t.load(r)?;
        }
        self.dsp.load(r)?;
        Ok(())
    }
}


impl SmpBus for ApuIo {
    fn read(&mut self, addr: u16) -> u8 {
        let result = match addr {
            0xF0 => 0,
            0xF1 => 0,
            0xF2 => self.dspaddr & 0x7F,
            0xF3 => self.dsp.read(self.dspaddr & 0x7F),
            0xF4..=0xF7 => self.port_in[(addr - 0xF4) as usize],
            0xF8 | 0xF9 => self.ram[addr as usize],
            0xFA..=0xFC => 0,
            0xFD..=0xFF => self.timers[(addr - 0xFD) as usize].read_out(),
            0xFFC0..=0xFFFF if (self.control & 0x80) != 0 => IPL_ROM[(addr - 0xFFC0) as usize],
            _ => self.ram[addr as usize],
        };
        self.tick();
        result
    }

    fn write(&mut self, addr: u16, v: u8) {
        match addr {
            0xF0 => {} // ignore
            0xF1 => {
                self.control = v;
                for i in 0..3 {
                    let on = (v >> i) & 1 != 0;
                    self.timers[i].set_enable(on);
                }
                if v & 0x10 != 0 {
                    self.port_in[0] = 0;
                    self.port_in[1] = 0;
                }
                if v & 0x20 != 0 {
                    self.port_in[2] = 0;
                    self.port_in[3] = 0;
                }
            }
            0xF2 => {
                self.dspaddr = v;
            }
            0xF3 => {
                if (self.dspaddr & 0x80) == 0 {
                    self.dsp.write(self.dspaddr, v);
                }
            }
            0xF4..=0xF7 => {
                self.port_out[(addr - 0xF4) as usize] = v;
            }
            0xF8 | 0xF9 => {
                self.ram[addr as usize] = v;
            }
            0xFA..=0xFC => {
                self.timers[(addr - 0xFA) as usize].set_target(v);
            }
            0xFD..=0xFF => {} // ignore
            _ => {
                self.ram[addr as usize] = v;
            }
        }
        self.tick();
    }

    fn idle(&mut self) {
        self.tick();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::savestate::{State, StateWriter, StateReader};
    use crate::hash::Fnv64;

    fn io_hash(io: &ApuIo) -> u64 {
        let mut h = Fnv64::new();
        io.hash_into(&mut h);
        h.finish()
    }

    #[test]
    fn e08_11_io_roundtrip() {
        // Set up ApuIo with non-trivial state.
        let mut io = ApuIo::new();
        // Fill RAM with a pattern.
        for i in 0..65536 {
            io.ram[i] = (i & 0xFF) as u8;
        }
        io.control = 0xB1;
        io.dspaddr = 0x42;
        io.dsp_div = 17;
        for i in 0..4 {
            io.port_in[i] = (i + 1) as u8 * 10;
            io.port_out[i] = (i + 1) as u8 * 20;
        }

        // Save.
        let mut writer = StateWriter::new();
        io.save(&mut writer);
        let data = writer.finish();

        // Load into fresh instance.
        let mut loaded = ApuIo::new();
        let mut reader = StateReader::new(&data);
        loaded.load(&mut reader).unwrap();

        // Verify bytes match on re-save.
        let mut writer2 = StateWriter::new();
        loaded.save(&mut writer2);
        let data2 = writer2.finish();
        assert_eq!(data, data2, "re-saved bytes must match original");

        // Verify hash matches.
        assert_eq!(io_hash(&io), io_hash(&loaded), "hashes must match after roundtrip");
    }

    #[test]
    fn e08_11_truncated_io() {
        let mut io = ApuIo::new();
        for i in 0..65536 {
            io.ram[i] = (i & 0xFF) as u8;
        }

        let mut writer = StateWriter::new();
        io.save(&mut writer);
        let data = writer.finish();

        // Test truncation at various points.
        for n in [0, data.len() / 2, data.len() - 1] {
            let mut loaded = ApuIo::new();
            let mut reader = StateReader::new(&data[..n]);
            let result = loaded.load(&mut reader);
            assert!(matches!(result, Err(StateError::Truncated)),
                "truncation at {} should fail with Truncated", n);
        }
    }
}
