//! DMA channel registers ($4300-$437F). No transfer logic yet.

use crate::hash::Fnv64;

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
