use std::collections::HashMap;
use puce16_core::dma::DmaBus;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ev {
    Elapse(u32),
    ARead(u32),
    AWrite(u32, u8),
    BRead(u8),
    BWrite(u8, u8),
}

pub struct FakeBus {
    pub mem: HashMap<u32, u8>,
    pub ev: Vec<Ev>,
    pub master: u64,
    pub b_counter: u8,
}

impl FakeBus {
    pub fn new(master: u64) -> FakeBus {
        FakeBus {
            mem: HashMap::new(),
            ev: Vec::new(),
            master,
            b_counter: 0,
        }
    }

    pub fn poke(&mut self, addr: u32, v: u8) {
        self.mem.insert(addr, v);
    }

    pub fn b_writes(&self) -> Vec<(u8, u8)> {
        let mut result = Vec::new();
        for e in &self.ev {
            if let Ev::BWrite(reg, val) = e {
                result.push((*reg, *val));
            }
        }
        result
    }

    pub fn a_writes(&self) -> Vec<(u32, u8)> {
        let mut result = Vec::new();
        for e in &self.ev {
            if let Ev::AWrite(addr, val) = e {
                result.push((*addr, *val));
            }
        }
        result
    }

    pub fn count_b_reads(&self) -> usize {
        self.ev.iter().filter(|e| matches!(e, Ev::BRead(_))).count()
    }
}

impl DmaBus for FakeBus {
    fn a_read(&mut self, addr: u32) -> u8 {
        self.ev.push(Ev::ARead(addr));
        match self.mem.get(&addr) {
            Some(v) => *v,
            None => addr as u8,
        }
    }

    fn a_write(&mut self, addr: u32, v: u8) {
        self.ev.push(Ev::AWrite(addr, v));
        self.mem.insert(addr, v);
    }

    fn b_read(&mut self, reg: u8) -> u8 {
        self.ev.push(Ev::BRead(reg));
        self.b_counter = self.b_counter.wrapping_add(1);
        self.b_counter
    }

    fn b_write(&mut self, reg: u8, v: u8) {
        self.ev.push(Ev::BWrite(reg, v));
    }

    fn elapse(&mut self, clocks: u32) {
        self.ev.push(Ev::Elapse(clocks));
        self.master = self.master + clocks as u64;
    }

    fn master(&self) -> u64 {
        self.master
    }
}
