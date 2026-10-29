pub mod addr;
pub mod alu;
pub mod mem;
pub mod flow;
pub mod misc;
pub mod interrupts;
pub mod table;
pub mod testbus;

use std::hash::{Hash, Hasher};

// Flag bits in P register
pub const FLAG_C: u8 = 0x01;
pub const FLAG_Z: u8 = 0x02;
pub const FLAG_I: u8 = 0x04;
pub const FLAG_D: u8 = 0x08;
pub const FLAG_X: u8 = 0x10;
pub const FLAG_M: u8 = 0x20;
pub const FLAG_V: u8 = 0x40;
pub const FLAG_N: u8 = 0x80;

/// Bus interface for CPU memory access. Each call represents one bus cycle.
pub trait Bus {
    fn read(&mut self, addr: u32) -> u8;
    fn write(&mut self, addr: u32, v: u8);
    fn idle(&mut self);
    fn nmi_pending(&mut self) -> bool { false }
    fn irq_asserted(&mut self) -> bool { false }
}

/// 65816 CPU state.
#[derive(Clone, Debug)]
pub struct Cpu {
    pub a: u16,
    pub x: u16,
    pub y: u16,
    pub s: u16,
    pub d: u16,
    pub db: u8,
    pub pb: u8,
    pub pc: u16,
    pub p: u8,
    pub e: bool,
    pub halted: bool,
    pub waiting: bool,
}

impl Cpu {
    /// Create a new CPU in emulation mode with default register values.
    pub fn new() -> Cpu {
        Cpu {
            a: 0, x: 0, y: 0, s: 0x01FF, d: 0, db: 0, pb: 0, pc: 0,
            p: 0x34, e: true, halted: false, waiting: false,
        }
    }

    /// Read a flag from P.
    pub fn flag(&self, f: u8) -> bool { (self.p & f) != 0 }

    /// Set or clear a flag in P.
    pub fn set_flag(&mut self, f: u8, v: bool) {
        if v { self.p |= f; } else { self.p &= !f; }
    }

    /// Is the accumulator in 8-bit mode?
    pub fn m8(&self) -> bool { self.flag(FLAG_M) }

    /// Are index registers in 8-bit mode?
    pub fn x8(&self) -> bool { self.flag(FLAG_X) }

    /// Write P register, enforcing emulation-mode invariant and X flag side effects.
    pub fn set_p(&mut self, v: u8) {
        if self.e {
            self.p = v | 0x30; // M and X forced in emulation mode
        } else {
            self.p = v;
        }
        if self.flag(FLAG_X) {
            self.x &= 0xFF;
            self.y &= 0xFF;
        }
    }

    /// Set N and Z flags from an 8-bit value.
    pub fn set_nz8(&mut self, v: u8) {
        self.set_flag(FLAG_N, (v & 0x80) != 0);
        self.set_flag(FLAG_Z, v == 0);
    }

    /// Set N and Z flags from a 16-bit value.
    pub fn set_nz16(&mut self, v: u16) {
        self.set_flag(FLAG_N, (v & 0x8000) != 0);
        self.set_flag(FLAG_Z, v == 0);
    }

    /// Set N and Z flags from a value; if wide is false only the low byte matters.
    pub fn set_nz(&mut self, v: u16, wide: bool) {
        if wide {
            self.set_nz16(v);
        } else {
            self.set_nz8(v as u8);
        }
    }

    /// Compute the 24-bit address from program bank and PC.
    pub fn pc24(&self) -> u32 { ((self.pb as u32) << 16) | (self.pc as u32) }

    /// Fetch one byte, incrementing PC.
    pub fn fetch8<B: Bus>(&mut self, bus: &mut B) -> u8 {
        let v = bus.read(self.pc24());
        self.pc = self.pc.wrapping_add(1);
        v
    }

    /// Fetch two bytes (little-endian), incrementing PC.
    pub fn fetch16<B: Bus>(&mut self, bus: &mut B) -> u16 {
        let lo = self.fetch8(bus) as u16;
        let hi = self.fetch8(bus) as u16;
        (hi << 8) | lo
    }

    /// Fetch three bytes (bank, then address), incrementing PC. Returns full 24-bit address.
    pub fn fetch24<B: Bus>(&mut self, bus: &mut B) -> u32 {
        let a = self.fetch16(bus);
        let b = self.fetch8(bus);
        ((b as u32) << 16) | (a as u32)
    }

    /// Push byte to old-style stack (page 0 in emulation mode).
    pub fn push8<B: Bus>(&mut self, bus: &mut B, v: u8) {
        bus.write(self.s as u32, v);
        self.s = self.s.wrapping_sub(1);
        if self.e {
            self.s = 0x0100 | (self.s & 0xFF);
        }
    }

    /// Pull byte from old-style stack.
    pub fn pull8<B: Bus>(&mut self, bus: &mut B) -> u8 {
        self.s = self.s.wrapping_add(1);
        if self.e {
            self.s = 0x0100 | (self.s & 0xFF);
        }
        bus.read(self.s as u32)
    }

    /// Push byte to new-style stack (full 16-bit range).
    pub fn push8n<B: Bus>(&mut self, bus: &mut B, v: u8) {
        bus.write(self.s as u32, v);
        self.s = self.s.wrapping_sub(1);
    }

    /// Pull byte from new-style stack.
    pub fn pull8n<B: Bus>(&mut self, bus: &mut B) -> u8 {
        self.s = self.s.wrapping_add(1);
        bus.read(self.s as u32)
    }

    /// Fix stack pointer after a new-style instruction in emulation mode.
    pub fn fix_stack_e(&mut self) {
        if self.e {
            self.s = 0x0100 | (self.s & 0xFF);
        }
    }

    /// Hash CPU state into an FNV-64 hasher for oracle comparison.
    pub fn hash_into<H: Hasher>(&self, h: &mut H) {
        self.a.hash(h);
        self.x.hash(h);
        self.y.hash(h);
        self.s.hash(h);
        self.d.hash(h);
        self.db.hash(h);
        self.pb.hash(h);
        self.pc.hash(h);
        self.p.hash(h);
        (if self.e { 1u8 } else { 0u8 }).hash(h);
        (if self.halted { 1u8 } else { 0u8 }).hash(h);
        (if self.waiting { 1u8 } else { 0u8 }).hash(h);
    }

    /// Execute one instruction or an interrupt sequence.
    pub fn step<B: Bus>(&mut self, bus: &mut B) {
        if interrupts::poll(self, bus) { return; }
        let op = self.fetch8(bus);
        let mn = table::MN[op as usize];
        let am = table::AM[op as usize];
        if alu::exec(self, bus, mn, am) || mem::exec(self, bus, mn, am)
            || flow::exec(self, bus, mn, am) || misc::exec(self, bus, mn, am) {
            return;
        }
        panic!("unimplemented opcode {:02X} ({})", op, mn.name());
    }

    /// Reset the CPU.
    pub fn reset<B: Bus>(&mut self, bus: &mut B) {
        interrupts::reset(self, bus);
    }
}

impl crate::savestate::State for Cpu {
    fn save(&self, w: &mut crate::savestate::StateWriter) {
        // Save all CPU fields in declaration order. All are covered by hash_into.
        w.u16(self.a);
        w.u16(self.x);
        w.u16(self.y);
        w.u16(self.s);
        w.u16(self.d);
        w.u8(self.db);
        w.u8(self.pb);
        w.u16(self.pc);
        w.u8(self.p);
        w.bool(self.e);
        w.bool(self.halted);
        w.bool(self.waiting);
    }

    fn load(&mut self, r: &mut crate::savestate::StateReader) -> Result<(), crate::savestate::StateError> {
        // Load all CPU fields in declaration order.
        self.a = r.u16()?;
        self.x = r.u16()?;
        self.y = r.u16()?;
        self.s = r.u16()?;
        self.d = r.u16()?;
        self.db = r.u8()?;
        self.pb = r.u8()?;
        self.pc = r.u16()?;
        self.p = r.u8()?;
        self.e = r.bool()?;
        self.halted = r.bool()?;
        self.waiting = r.bool()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::savestate::{State, StateWriter, StateReader};
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;

    fn cpu_hash(cpu: &Cpu) -> u64 {
        let mut h = DefaultHasher::new();
        cpu.hash_into(&mut h);
        h.finish()
    }

    #[test]
    fn e08_02_roundtrip_all_fields() {
        // Set each field to a distinct non-zero value
        let mut cpu = Cpu::new();
        cpu.a = 0x1111;
        cpu.x = 0x2222;
        cpu.y = 0x3333;
        cpu.s = 0x4444;
        cpu.d = 0x5555;
        cpu.db = 0x66;
        cpu.pb = 0x77;
        cpu.pc = 0x8888;
        cpu.p = 0x99;
        cpu.e = false;
        cpu.halted = true;
        cpu.waiting = true;

        // Save
        let mut writer = StateWriter::new();
        cpu.save(&mut writer);
        let data = writer.finish();

        // Load into fresh instance
        let mut loaded = Cpu::new();
        let mut reader = StateReader::new(&data);
        loaded.load(&mut reader).unwrap();

        // Verify bytes match on re-save
        let mut writer2 = StateWriter::new();
        loaded.save(&mut writer2);
        let data2 = writer2.finish();
        assert_eq!(data, data2, "re-saved bytes must match original");

        // Verify hash matches
        assert_eq!(cpu_hash(&cpu), cpu_hash(&loaded), "hashes must match after roundtrip");
    }

    #[test]
    fn e08_02_truncated() {
        let mut cpu = Cpu::new();
        cpu.a = 0x1234;
        cpu.x = 0x5678;
        cpu.y = 0x9ABC;
        cpu.s = 0xDEF0;
        cpu.d = 0x1111;
        cpu.db = 0x22;
        cpu.pb = 0x33;
        cpu.pc = 0x4444;
        cpu.p = 0x55;
        cpu.e = false;
        cpu.halted = true;
        cpu.waiting = true;

        let mut writer = StateWriter::new();
        cpu.save(&mut writer);
        let data = writer.finish();
        let len = data.len();

        // Test truncation at offsets 0, len/2, len-1
        for n in [0, len / 2, len - 1] {
            let mut loaded = Cpu::new();
            let mut reader = StateReader::new(&data[..n]);
            let result = loaded.load(&mut reader);
            assert!(result.is_err(), "load on truncated data at offset {} should fail", n);
        }
    }

    #[test]
    fn e08_02_behaves_identically_after_load() {
        // Build a simple program: LDA #12, CLC, ADC #1, INC A, DEC A, PHA, PLA, RTS
        let program = [
            0xA9, 0x12, // LDA #12
            0x18,       // CLC
            0x69, 0x01, // ADC #1
            0xEB,       // SBC #0 (no borrow)
            0x80, 0xFE, // BIT #$FE (set N flag)
            0x4A,       // LSR A
            0x0A,       // ASL A
            0x29, 0xFF, // AND #$FF
            0x65, 0x00, // ADC $00 (read from memory)
            0x4C, 0x00, 0x80, // JMP $8000 (loop back to start of program area)
        ];

        let mut bus = testbus::FlatBus::new();
        bus.poke(0x008000, &program);

        // First CPU: run normally
        let mut cpu1 = Cpu::new();
        cpu1.pb = 0;
        cpu1.pc = 0x8000;
        for _ in 0..10 {
            cpu1.step(&mut bus);
        }

        // Save state after running
        let mut writer = StateWriter::new();
        cpu1.save(&mut writer);
        let data = writer.finish();

        // Second CPU: load from saved state and run same instructions
        let mut bus2 = testbus::FlatBus::new();
        bus2.poke(0x008000, &program);

        let mut cpu2 = Cpu::new();
        let mut reader = StateReader::new(&data);
        cpu2.load(&mut reader).unwrap();

        // Run 10 more steps on both
        for _ in 0..10 {
            cpu1.step(&mut bus);
            cpu2.step(&mut bus2);
        }

        assert_eq!(cpu_hash(&cpu1), cpu_hash(&cpu2), "hashes must match after executing same instructions");
    }

    #[test]
    fn e08_02_wai_stp_state_kept() {
        // CPU in WAI state: waiting=true, halted=false
        let mut cpu = Cpu::new();
        cpu.waiting = true;
        cpu.pc = 0x1000;

        // Save and restore
        let mut writer = StateWriter::new();
        cpu.save(&mut writer);
        let data = writer.finish();

        let mut loaded = Cpu::new();
        let mut reader = StateReader::new(&data);
        loaded.load(&mut reader).unwrap();

        // Verify waiting state is preserved
        assert!(loaded.waiting, "waiting flag must be preserved after load");

        // Step without interrupt: PC should not advance (WAI waits for IRQ/NMI)
        let mut bus = testbus::FlatBus::new();
        loaded.step(&mut bus);
        assert_eq!(loaded.pc, 0x1000, "PC must not advance while waiting in WAI state");
    }
}
