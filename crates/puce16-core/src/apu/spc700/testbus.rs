use super::SmpBus;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CycleKind { Read, Write, Idle }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cycle { pub addr: u16, pub val: u8, pub kind: CycleKind }

pub struct FlatSmpBus {
    mem: Vec<u8>,
    touched: Vec<u16>,
    pub log: Vec<Cycle>,
}

impl FlatSmpBus {
    pub fn new() -> Self {
        FlatSmpBus {
            mem: vec![0u8; 65536],
            touched: Vec::new(),
            log: Vec::new(),
        }
    }

    pub fn poke(&mut self, addr: u16, bytes: &[u8]) {
        let mut a = addr as usize;
        for &b in bytes {
            if !self.touched.contains(&(a as u16)) {
                self.touched.push(a as u16);
            }
            self.mem[a] = b;
            a = (a + 1) % 65536;
        }
    }

    pub fn peek(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    pub fn clear_log(&mut self) {
        self.log.clear();
    }

    pub fn reset_mem(&mut self) {
        for &a in &self.touched {
            self.mem[a as usize] = 0;
        }
        self.touched.clear();
        self.log.clear();
    }
}

impl Default for FlatSmpBus {
    fn default() -> Self {
        Self::new()
    }
}

impl SmpBus for FlatSmpBus {
    fn read(&mut self, addr: u16) -> u8 {
        let v = self.mem[addr as usize];
        self.log.push(Cycle { addr, val: v, kind: CycleKind::Read });
        v
    }

    fn write(&mut self, addr: u16, v: u8) {
        if !self.touched.contains(&addr) {
            self.touched.push(addr);
        }
        self.mem[addr as usize] = v;
        self.log.push(Cycle { addr, val: v, kind: CycleKind::Write });
    }

    fn idle(&mut self) {
        self.log.push(Cycle { addr: 0, val: 0, kind: CycleKind::Idle });
    }
}
