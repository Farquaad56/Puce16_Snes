pub mod table;
pub mod pattern;
pub mod addr;
pub mod testbus;
mod alu;
mod mov;
mod incdec;
mod word;
mod bits;
mod flow;
mod misc;

// PSW flag bit positions (N V P B H I Z C)
pub const F_C: u8 = 0x01;
pub const F_Z: u8 = 0x02;
pub const F_I: u8 = 0x04;
pub const F_H: u8 = 0x08;
pub const F_B: u8 = 0x10;
pub const F_P: u8 = 0x20;
pub const F_V: u8 = 0x40;
pub const F_N: u8 = 0x80;

/// Bus interface for SPC700 memory access.
pub trait SmpBus {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, v: u8);
    fn idle(&mut self);
}

#[derive(Clone, Debug)]
pub struct Spc700 {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    pub psw: u8,
    pub halted: bool,
    cyc: u32,
    taken: bool,
    pat: &'static [u8],
    pos: usize,
}

impl Spc700 {
    /// Create a new SPC700 CPU with default register values.
    pub fn new() -> Self {
        Spc700 {
            a: 0, x: 0, y: 0, sp: 0, pc: 0, psw: 0x02, halted: false, cyc: 0, taken: false,
            pat: &[], pos: 0,
        }
    }

    /// Emit the internal / dummy cycles the pattern places before the next
    /// real access of kind `kind` (b'R' or b'W').
    fn align<B: SmpBus>(&mut self, bus: &mut B, kind: u8, addr: u16) {
        while self.pos < self.pat.len() {
            let c = self.pat[self.pos];
            if c == b'.' {
                bus.idle();
            } else if c == b'R' && kind == b'W' {
                // dummy read before a write hits the same address as the write
                let _ = bus.read(addr);
            } else {
                break;
            }
            self.cyc += 1;
            self.pos += 1;
        }
    }

    /// Read one byte from bus at addr; counts as one cycle.
    pub fn rd<B: SmpBus>(&mut self, bus: &mut B, addr: u16) -> u8 {
        self.align(bus, b'R', addr);
        let v = bus.read(addr);
        self.cyc += 1;
        self.pos += 1;
        v
    }

    /// Write one byte to bus at addr; counts as one cycle.
    pub fn wr<B: SmpBus>(&mut self, bus: &mut B, addr: u16, v: u8) {
        self.align(bus, b'W', addr);
        bus.write(addr, v);
        self.cyc += 1;
        self.pos += 1;
    }

    /// One idle bus cycle: consumes the next pattern slot (a dummy read if it is an R slot).
    pub fn idle<B: SmpBus>(&mut self, bus: &mut B) {
        if self.pos < self.pat.len() && self.pat[self.pos] == b'R' {
            let _ = bus.read(self.pc);
        } else {
            bus.idle();
        }
        self.cyc += 1;
        self.pos += 1;
    }

    /// Fetch one byte from PC and increment PC.
    pub fn fetch8<B: SmpBus>(&mut self, bus: &mut B) -> u8 {
        let v = self.rd(bus, self.pc);
        self.pc = self.pc.wrapping_add(1);
        v
    }

    /// Fetch two bytes (little-endian) from PC and increment PC by 2.
    pub fn fetch16<B: SmpBus>(&mut self, bus: &mut B) -> u16 {
        let lo = self.fetch8(bus);
        let hi = self.fetch8(bus);
        ((hi as u16) << 8) | (lo as u16)
    }

    /// Direct page base address (0x0000 or 0x0100 depending on P flag).
    pub fn dp(&self) -> u16 {
        if self.flag(F_P) { 0x0100 } else { 0x0000 }
    }

    /// Read a 16-bit value from two addresses (low byte first, then high).
    pub fn rd16<B: SmpBus>(&mut self, bus: &mut B, lo_addr: u16, hi_addr: u16) -> u16 {
        let lo = self.rd(bus, lo_addr);
        let hi = self.rd(bus, hi_addr);
        ((hi as u16) << 8) | (lo as u16)
    }

    /// Test a PSW flag.
    pub fn flag(&self, f: u8) -> bool {
        (self.psw & f) != 0
    }

    /// Set or clear a PSW flag.
    pub fn set_flag(&mut self, f: u8, on: bool) {
        if on {
            self.psw |= f;
        } else {
            self.psw &= !f;
        }
    }

    /// Set N and Z flags based on an 8-bit value.
    pub fn set_nz(&mut self, v: u8) {
        self.set_flag(F_N, (v & 0x80) != 0);
        self.set_flag(F_Z, v == 0);
    }

    /// Read the YA pair as a 16-bit value ((Y << 8) | A).
    pub fn ya(&self) -> u16 {
        ((self.y as u16) << 8) | (self.a as u16)
    }

    /// Write to the YA pair from a 16-bit value.
    pub fn set_ya(&mut self, v: u16) {
        self.y = (v >> 8) as u8;
        self.a = v as u8;
    }

    /// Push an 8-bit value onto the stack.
    pub fn push8<B: SmpBus>(&mut self, bus: &mut B, v: u8) {
        let addr = 0x0100u16 | self.sp as u16;
        self.wr(bus, addr, v);
        self.sp = self.sp.wrapping_sub(1);
    }

    /// Pull an 8-bit value from the stack.
    pub fn pull8<B: SmpBus>(&mut self, bus: &mut B) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let addr = 0x0100u16 | self.sp as u16;
        self.rd(bus, addr)
    }

    /// Take a branch with an 8-bit signed relative offset.
    pub fn branch(&mut self, rel: u8) {
        let off = rel as i8 as i16 as u16;
        self.pc = self.pc.wrapping_add(off);
        self.taken = true;
    }

    /// Execute one instruction and return the number of cycles consumed.
    pub fn step<B: SmpBus>(&mut self, bus: &mut B) -> u32 {
        if self.halted {
            self.idle(bus);
            return 1;
        }
        self.cyc = 0;
        self.taken = false;
        self.pat = &[];
        self.pos = 0;
        let op = self.fetch8(bus);
        self.pat = pattern::PATTERN[op as usize].as_bytes();
        self.pos = 1;
        let exec = mov::exec(self, bus, op)
            || alu::exec(self, bus, op)
            || incdec::exec(self, bus, op)
            || word::exec(self, bus, op)
            || bits::exec(self, bus, op)
            || flow::exec(self, bus, op)
            || misc::exec(self, bus, op);
        if !exec {
            panic!("unimplemented opcode {:02X} ({})", op, table::NAME[op as usize]);
        }
        while self.pos < self.pat.len() {
            self.idle(bus);
        }
        self.pat = &[];
        let base = table::CYCLES[op as usize] as u32;
        let extra = if self.taken { table::BRANCH_EXTRA[op as usize] as u32 } else { 0 };
        let target = base + extra;
        while self.cyc < target {
            self.idle(bus);
        }
        if self.cyc > target { self.cyc } else { target }
    }

    /// Reset the CPU by reading the reset vector from $FFFE/$FFFF.
    pub fn reset<B: SmpBus>(&mut self, bus: &mut B) {
        let vec = self.rd16(bus, 0xFFFE, 0xFFFF);
        self.pc = vec;
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0;
        self.psw = 0x02;
        self.halted = false;
    }
}

impl Default for Spc700 {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::savestate::State for Spc700 {
    fn save(&self, w: &mut crate::savestate::StateWriter) {
        // Save all fields in declaration order. All are covered by hash_into or influence future behavior.
        w.u8(self.a);
        w.u8(self.x);
        w.u8(self.y);
        w.u8(self.sp);
        w.u16(self.pc);
        w.u8(self.psw);
        w.bool(self.halted);
        w.u32(self.cyc);
        w.bool(self.taken);
        // not saved: pat (instruction pattern, reset each step)
        // not saved: pos (pattern position, reset each step)
    }

    fn load(&mut self, r: &mut crate::savestate::StateReader) -> Result<(), crate::savestate::StateError> {
        // Load all fields in declaration order.
        self.a = r.u8()?;
        self.x = r.u8()?;
        self.y = r.u8()?;
        self.sp = r.u8()?;
        self.pc = r.u16()?;
        self.psw = r.u8()?;
        self.halted = r.bool()?;
        self.cyc = r.u32()?;
        self.taken = r.bool()?;
        // pat and pos are reset at the start of each step, so no need to restore them.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::savestate::{State, StateWriter, StateReader};
    use crate::hash::Fnv64;

    fn spc_hash(spc: &Spc700) -> u64 {
        let mut h = Fnv64::new();
        h.write_u8(spc.a);
        h.write_u8(spc.x);
        h.write_u8(spc.y);
        h.write_u8(spc.sp);
        h.write_u16(spc.pc);
        h.write_u8(spc.psw);
        h.write_u8(if spc.halted { 1 } else { 0 });
        h.write_u32(spc.cyc);
        h.write_u8(if spc.taken { 1 } else { 0 });
        h.finish()
    }

    #[test]
    fn e08_09_spc_roundtrip() {
        // Set every saved field to a distinct non-zero value.
        let mut spc = Spc700::new();
        spc.a = 0xA1;
        spc.x = 0xB2;
        spc.y = 0xC3;
        spc.sp = 0xD4;
        spc.pc = 0x5678;
        spc.psw = 0xE9;
        spc.halted = true;
        spc.cyc = 12345;
        spc.taken = true;

        // Save.
        let mut writer = StateWriter::new();
        spc.save(&mut writer);
        let data = writer.finish();

        // Load into fresh instance.
        let mut loaded = Spc700::new();
        let mut reader = StateReader::new(&data);
        loaded.load(&mut reader).unwrap();

        // Verify bytes match on re-save.
        let mut writer2 = StateWriter::new();
        loaded.save(&mut writer2);
        let data2 = writer2.finish();
        assert_eq!(data, data2, "re-saved bytes must match original");

        // Verify hash matches.
        assert_eq!(spc_hash(&spc), spc_hash(&loaded), "hashes must match after roundtrip");
    }

    #[test]
    fn e08_09_spc_truncated() {
        let mut spc = Spc700::new();
        spc.a = 0xA1;
        spc.x = 0xB2;
        spc.y = 0xC3;
        spc.sp = 0xD4;
        spc.pc = 0x5678;
        spc.psw = 0xE9;
        spc.halted = true;
        spc.cyc = 12345;
        spc.taken = true;

        let mut writer = StateWriter::new();
        spc.save(&mut writer);
        let data = writer.finish();

        // Test truncation at various points.
        for n in [0, data.len() / 2, data.len() - 1] {
            let mut loaded = Spc700::new();
            let mut reader = StateReader::new(&data[..n]);
            let result = loaded.load(&mut reader);
            assert!(matches!(result, Err(crate::savestate::StateError::Truncated)),
                "truncation at {} should fail with Truncated", n);
        }
    }

    #[test]
    fn e08_09_spc_behaves_identically() {
        // Same short program on original and restored CPU: 20 steps -> identical registers.
        use crate::apu::spc700::testbus::FlatSmpBus;

        // Program: MOV A,#$10 ; INC A ; BRA -2 (infinite loop incrementing A)
        let prog = [0xE8, 0x10, 0xBC, 0x2F, 0xFE];

        // Original CPU.
        let mut bus1 = FlatSmpBus::new();
        for (i, b) in prog.iter().enumerate() {
            bus1.write(0x8000 + i as u16, *b);
        }
        let mut spc1 = Spc700::new();
        spc1.pc = 0x8000;
        for _ in 0..20 {
            spc1.step(&mut bus1);
        }

        // Save original state.
        let mut writer = StateWriter::new();
        spc1.save(&mut writer);
        let data = writer.finish();

        // Restored CPU.
        let mut bus2 = FlatSmpBus::new();
        for (i, b) in prog.iter().enumerate() {
            bus2.write(0x8000 + i as u16, *b);
        }
        let mut spc2 = Spc700::new();
        let mut reader = StateReader::new(&data);
        spc2.load(&mut reader).unwrap();

        // Run both for 20 more steps.
        for _ in 0..20 {
            spc1.step(&mut bus1);
            spc2.step(&mut bus2);
        }

        assert_eq!(spc1.a, spc2.a, "A must match");
        assert_eq!(spc1.x, spc2.x, "X must match");
        assert_eq!(spc1.y, spc2.y, "Y must match");
        assert_eq!(spc1.sp, spc2.sp, "SP must match");
        assert_eq!(spc1.pc, spc2.pc, "PC must match");
        assert_eq!(spc1.psw, spc2.psw, "PSW must match");
    }
}
