use super::Bus;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CycleKind { Read, Write, Idle }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cycle { pub addr: u32, pub val: u8, pub kind: CycleKind }

pub struct FlatBus {
    mem: Vec<u8>,
    touched: Vec<u32>,
    pub log: Vec<Cycle>,
    pub nmi: bool,
    pub irq: bool,
}

impl FlatBus {
    pub fn new() -> FlatBus {
        FlatBus {
            mem: vec![0u8; 1 << 24],
            touched: Vec::new(),
            log: Vec::new(),
            nmi: false,
            irq: false,
        }
    }

    pub fn poke(&mut self, addr: u32, bytes: &[u8]) {
        let mut a = addr & 0xFFFFFF;
        for b in bytes {
            if !self.touched.contains(&a) {
                self.touched.push(a);
            }
            self.mem[a as usize] = *b;
            a = (a + 1) & 0xFFFFFF;
        }
    }

    pub fn peek(&self, addr: u32) -> u8 {
        self.mem[(addr & 0xFFFFFF) as usize]
    }

    pub fn clear_log(&mut self) {
        self.log.clear();
    }

    pub fn reset_mem(&mut self) {
        for a in &self.touched {
            self.mem[*a as usize] = 0;
        }
        self.touched.clear();
        self.log.clear();
    }
}

impl Default for FlatBus {
    fn default() -> Self {
        FlatBus::new()
    }
}

impl Bus for FlatBus {
    fn read(&mut self, addr: u32) -> u8 {
        let a = addr & 0xFFFFFF;
        let v = self.mem[a as usize];
        self.log.push(Cycle { addr: a, val: v, kind: CycleKind::Read });
        v
    }

    fn write(&mut self, addr: u32, v: u8) {
        let a = addr & 0xFFFFFF;
        if !self.touched.contains(&a) {
            self.touched.push(a);
        }
        self.mem[a as usize] = v;
        self.log.push(Cycle { addr: a, val: v, kind: CycleKind::Write });
    }

    fn idle(&mut self) {
        self.log.push(Cycle { addr: 0, val: 0, kind: CycleKind::Idle });
    }

    fn nmi_pending(&mut self) -> bool {
        let p = self.nmi;
        self.nmi = false;
        p
    }

    fn irq_asserted(&mut self) -> bool {
        self.irq
    }
}