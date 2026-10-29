//! DMA channel registers ($4300-$437F). No transfer logic yet.

use crate::hash::Fnv64;
use crate::savestate::{State, StateError, StateReader, StateWriter};

pub mod hdma;

#[derive(Default, Clone)]
pub struct Channel {
    pub dmap: u8,
    pub bbad: u8,
    pub a1t: u16,
    pub a1b: u8,
    pub das: u16,
    pub dasb: u8,
    pub a2a: u16,
    pub nltr: u8,
    pub unused: u8,
    pub(crate) do_transfer: bool,
    pub(crate) done: bool,
    pub(crate) line_counter: u8,
}

#[derive(Default, Clone)]
pub struct Dma {
    pub ch: [Channel; 8],
}

impl Dma {
    pub fn new() -> Dma {
        let mut d = Dma::default();
        for ch in &mut d.ch {
            ch.dmap = 0xFF;
            ch.bbad = 0xFF;
            ch.a1t = 0xFFFF;
            ch.a1b = 0xFF;
            ch.das = 0xFFFF;
            ch.dasb = 0xFF;
            ch.a2a = 0xFFFF;
            ch.nltr = 0xFF;
            ch.unused = 0xFF;
            ch.do_transfer = false;
            ch.done = true;
            ch.line_counter = 0;
        }
        d
    }

    pub fn read(&self, reg: u16) -> Option<u8> {
        if !(0x4300..=0x437F).contains(&reg) {
            return None;
        }
        let n = ((reg >> 4) & 7) as usize;
        let off = reg & 0xF;
        let ch = &self.ch[n];
        match off {
            0 => Some(ch.dmap),
            1 => Some(ch.bbad),
            2 => Some((ch.a1t & 0xFF) as u8),
            3 => Some(((ch.a1t >> 8) & 0xFF) as u8),
            4 => Some(ch.a1b),
            5 => Some((ch.das & 0xFF) as u8),
            6 => Some(((ch.das >> 8) & 0xFF) as u8),
            7 => Some(ch.dasb),
            8 => Some((ch.a2a & 0xFF) as u8),
            9 => Some(((ch.a2a >> 8) & 0xFF) as u8),
            0xA => Some(ch.nltr),
            0xB | 0xF => Some(ch.unused),
            _ => None, // $43nC-$43nE: open bus
        }
    }

    pub fn write(&mut self, reg: u16, v: u8) {
        if !(0x4300..=0x437F).contains(&reg) {
            return;
        }
        let n = ((reg >> 4) & 7) as usize;
        let off = reg & 0xF;
        let ch = &mut self.ch[n];
        match off {
            0 => ch.dmap = v,
            1 => ch.bbad = v,
            2 => ch.a1t = (ch.a1t & 0xFF00) | v as u16,
            3 => ch.a1t = (ch.a1t & 0x00FF) | ((v as u16) << 8),
            4 => ch.a1b = v,
            5 => ch.das = (ch.das & 0xFF00) | v as u16,
            6 => ch.das = (ch.das & 0x00FF) | ((v as u16) << 8),
            7 => ch.dasb = v,
            8 => ch.a2a = (ch.a2a & 0xFF00) | v as u16,
            9 => ch.a2a = (ch.a2a & 0x00FF) | ((v as u16) << 8),
            0xA => ch.nltr = v,
            0xB | 0xF => ch.unused = v,
            _ => {} // $43nC-$43nE: ignored on write
        }
    }

    pub fn hash_into(&self, h: &mut Fnv64) {
        for ch in &self.ch {
            h.write_u8(ch.dmap);
            h.write_u8(ch.bbad);
            h.write_u16(ch.a1t);
            h.write_u8(ch.a1b);
            h.write_u16(ch.das);
            h.write_u8(ch.dasb);
            h.write_u16(ch.a2a);
            h.write_u8(ch.nltr);
            h.write_u8(ch.unused);
            if ch.do_transfer {
                h.write_u8(1);
            } else {
                h.write_u8(0);
            }
            if ch.done {
                h.write_u8(1);
            } else {
                h.write_u8(0);
            }
            h.write_u8(ch.line_counter);
        }
    }
}

pub trait DmaBus {
    fn a_read(&mut self, addr: u32) -> u8;
    fn a_write(&mut self, addr: u32, v: u8);
    fn b_read(&mut self, reg: u8) -> u8;
    fn b_write(&mut self, reg: u8, v: u8);
    fn elapse(&mut self, clocks: u32);
    fn master(&self) -> u64;
}

pub fn pattern_offsets(p: u8) -> &'static [u8] {
    match p & 7 {
        0 => &[0],
        1 => &[0, 1],
        2 => &[0, 0],
        3 => &[0, 0, 1, 1],
        4 => &[0, 1, 2, 3],
        5 => &[0, 1, 0, 1],
        6 => &[0, 0],
        _ => &[0, 0, 1, 1],
    }
}

pub fn a_step(dmap: u8) -> i32 {
    match (dmap >> 3) & 3 {
        0 => 1,
        2 => -1,
        _ => 0,
    }
}

pub fn is_wram_addr(addr: u32) -> bool {
    let bank = (addr >> 16) as u8;
    if bank >= 0x7E && bank <= 0x7F {
        return true;
    }
    if (bank & 0x7F) <= 0x3F && (addr & 0xFFFF) < 0x2000 {
        return true;
    }
    false
}

pub fn step_a(a1t: u16, step: i32) -> u16 {
    let result = a1t as i32 + step;
    (result & 0xFFFF) as u16
}

impl Dma {
    pub fn run_general<B: DmaBus>(&mut self, bus: &mut B, mask: u8) {
        // Align to next multiple of 8 clocks
        let align = ((8 - bus.master() % 8) % 8) as u32;
        if align > 0 {
            bus.elapse(align);
        }
        // Startup overhead
        bus.elapse(8);

        for n in 0..8 {
            if (mask & (1 << n)) == 0 {
                continue;
            }
            let ch = &mut self.ch[n];
            // Channel overhead
            bus.elapse(8);

            let count = if ch.das == 0 { 65536 } else { ch.das as u32 };
            let offs = pattern_offsets(ch.dmap & 7);
            let step = a_step(ch.dmap);
            let mut i: u32 = 0;

            for _ in 0..count {
                bus.elapse(8);
                let addr_a = ((ch.a1b as u32) << 16) | (ch.a1t as u32);
                let idx = i as usize % offs.len();
                let reg_b = ch.bbad.wrapping_add(offs[idx]);

                // WRAM <-> $2180 skip rule: when regB is 0x80 and addr_a is in WRAM,
                // the transfer has no effect (no read/write on either side), but time
                // passes and counters advance. UNKNOWN - to confirm exact behavior.
                let wram_skip = reg_b == 0x80 && is_wram_addr(addr_a);

                if (ch.dmap & 0x80) == 0 {
                    // A -> B direction
                    if !wram_skip {
                        let v = bus.a_read(addr_a);
                        bus.b_write(reg_b, v);
                    }
                } else {
                    // B -> A direction
                    if !wram_skip {
                        let v = bus.b_read(reg_b);
                        bus.a_write(addr_a, v);
                    }
                }

                ch.a1t = step_a(ch.a1t, step);
                i += 1;
            }

            // After channel: das = 0
            ch.das = 0;
        }
    }
}

impl State for Dma {
    fn save(&self, w: &mut StateWriter) {
        for ch in &self.ch {
            w.u8(ch.dmap);
            w.u8(ch.bbad);
            w.u16(ch.a1t);
            w.u8(ch.a1b);
            w.u16(ch.das);
            w.u8(ch.dasb);
            w.u16(ch.a2a);
            w.u8(ch.nltr);
            w.u8(ch.unused);
            w.bool(ch.do_transfer);
            w.bool(ch.done);
            w.u8(ch.line_counter);
        }
    }

    fn load(&mut self, r: &mut StateReader) -> Result<(), StateError> {
        for ch in &mut self.ch {
            ch.dmap = r.u8()?;
            ch.bbad = r.u8()?;
            ch.a1t = r.u16()?;
            ch.a1b = r.u8()?;
            ch.das = r.u16()?;
            ch.dasb = r.u8()?;
            ch.a2a = r.u16()?;
            ch.nltr = r.u8()?;
            ch.unused = r.u8()?;
            ch.do_transfer = r.bool()?;
            ch.done = r.bool()?;
            ch.line_counter = r.u8()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::savestate::{StateWriter, StateReader};

    fn make_dma_with_values() -> Dma {
        let mut d = Dma::new();
        for n in 0..8 {
            let ch = &mut d.ch[n];
            ch.dmap = 0x10 + n as u8;
            ch.bbad = 0x20 + n as u8;
            ch.a1t = 0x3000 + (n * 10) as u16;
            ch.a1b = 0x40 + n as u8;
            ch.das = 0x5000 + (n * 20) as u16;
            ch.dasb = 0x60 + n as u8;
            ch.a2a = 0x7000 + (n * 30) as u16;
            ch.nltr = 0x80 + n as u8;
            ch.unused = 0x90 + n as u8;
            ch.do_transfer = n % 2 == 0;
            ch.done = n % 3 != 0;
            ch.line_counter = (n * 5) as u8;
        }
        d
    }

    #[test]
    fn e08_05_roundtrip_all_fields() {
        let original = make_dma_with_values();

        // Save
        let mut w = StateWriter::new();
        original.save(&mut w);
        let data = w.finish();

        // Load into fresh instance
        let mut restored = Dma::new();
        let mut r = StateReader::new(&data);
        restored.load(&mut r).unwrap();

        // Verify all fields match
        for n in 0..8 {
            assert_eq!(original.ch[n].dmap, restored.ch[n].dmap);
            assert_eq!(original.ch[n].bbad, restored.ch[n].bbad);
            assert_eq!(original.ch[n].a1t, restored.ch[n].a1t);
            assert_eq!(original.ch[n].a1b, restored.ch[n].a1b);
            assert_eq!(original.ch[n].das, restored.ch[n].das);
            assert_eq!(original.ch[n].dasb, restored.ch[n].dasb);
            assert_eq!(original.ch[n].a2a, restored.ch[n].a2a);
            assert_eq!(original.ch[n].nltr, restored.ch[n].nltr);
            assert_eq!(original.ch[n].unused, restored.ch[n].unused);
            assert_eq!(original.ch[n].do_transfer, restored.ch[n].do_transfer);
            assert_eq!(original.ch[n].done, restored.ch[n].done);
            assert_eq!(original.ch[n].line_counter, restored.ch[n].line_counter);
        }

        // Verify hash is identical after roundtrip
        let mut h1 = crate::hash::Fnv64::new();
        original.hash_into(&mut h1);
        let mut h2 = crate::hash::Fnv64::new();
        restored.hash_into(&mut h2);
        assert_eq!(h1.finish(), h2.finish());

        // Verify re-save produces identical bytes
        let mut w2 = StateWriter::new();
        restored.save(&mut w2);
        let data2 = w2.finish();
        assert_eq!(data, data2);
    }

    #[test]
    fn e08_05_fresh_dma_roundtrip() {
        // Save a fresh Dma (all 0xFF / done=true defaults)
        let original = Dma::new();
        let mut w = StateWriter::new();
        original.save(&mut w);
        let data = w.finish();

        // Load into a modified instance - should restore to defaults
        let mut modified = make_dma_with_values();
        let mut r = StateReader::new(&data);
        modified.load(&mut r).unwrap();

        // Verify restored to fresh defaults
        for n in 0..8 {
            assert_eq!(modified.ch[n].dmap, 0xFF);
            assert_eq!(modified.ch[n].bbad, 0xFF);
            assert_eq!(modified.ch[n].a1t, 0xFFFF);
            assert_eq!(modified.ch[n].a1b, 0xFF);
            assert_eq!(modified.ch[n].das, 0xFFFF);
            assert_eq!(modified.ch[n].dasb, 0xFF);
            assert_eq!(modified.ch[n].a2a, 0xFFFF);
            assert_eq!(modified.ch[n].nltr, 0xFF);
            assert_eq!(modified.ch[n].unused, 0xFF);
            assert!(!modified.ch[n].do_transfer);
            assert!(modified.ch[n].done);
            assert_eq!(modified.ch[n].line_counter, 0);
        }
    }

    #[test]
    fn e08_05_truncated() {
        let original = make_dma_with_values();
        let mut w = StateWriter::new();
        original.save(&mut w);
        let data = w.finish();

        // Test truncation at 0, len/2, and len-1
        for n in [0, data.len() / 2, data.len() - 1] {
            let mut restored = Dma::new();
            let mut r = StateReader::new(&data[..n]);
            let result = restored.load(&mut r);
            assert!(result.is_err(), "expected error at truncation point {}", n);
        }
    }

    // Minimal inline bus for HDMA test (no external dependency)
    struct TestBus {
        mem: std::collections::BTreeMap<u32, u8>,
        master: u64,
    }

    impl TestBus {
        fn new() -> Self {
            TestBus {
                mem: std::collections::BTreeMap::new(),
                master: 0,
            }
        }

        fn poke(&mut self, addr: u32, v: u8) {
            self.mem.insert(addr, v);
        }
    }

    impl DmaBus for TestBus {
        fn a_read(&mut self, addr: u32) -> u8 {
            match self.mem.get(&addr) {
                Some(v) => *v,
                None => 0,
            }
        }

        fn a_write(&mut self, _addr: u32, _v: u8) {}

        fn b_read(&mut self, _reg: u8) -> u8 {
            0
        }

        fn b_write(&mut self, _reg: u8, _v: u8) {}

        fn elapse(&mut self, clocks: u32) {
            self.master += clocks as u64;
        }

        fn master(&self) -> u64 {
            self.master
        }
    }

    #[test]
    fn e08_05_hdma_state_preserved() {
        // Set up HDMA on channel 0: direct mode, 3 lines
        let mut dma = Dma::new();
        let ch = &mut dma.ch[0];
        ch.dmap = 0x40; // indirect bit set for table reads
        ch.bbad = 0x80;
        ch.a1t = 0x1000;
        ch.a1b = 0x7E;

        let mut bus = TestBus::new();
        // Line byte: $83 means transfer on line, count=3 (low 7 bits)
        bus.poke(0x7E1000, 0x83);
        // DAS for indirect mode
        bus.poke(0x7E1001, 0x00);
        bus.poke(0x7E1002, 0x80);

        dma.hdma_init(&mut bus, 1);

        let state_before = dma.hdma_state(0);

        // Save and restore
        let mut w = StateWriter::new();
        dma.save(&mut w);
        let data = w.finish();

        let mut restored = Dma::new();
        let mut r = StateReader::new(&data);
        restored.load(&mut r).unwrap();

        let state_after = restored.hdma_state(0);

        assert_eq!(state_before, state_after, "HDMA state must be preserved through save/load");
    }
}
