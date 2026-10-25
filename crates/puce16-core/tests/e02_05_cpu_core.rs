use std::collections::HashMap;
use std::hash::{Hasher, DefaultHasher};

use puce16_core::cpu::{Bus, Cpu, FLAG_N, FLAG_X, FLAG_Z};

// Simple test bus backed by a HashMap (sparse)
struct TestBus {
    mem: HashMap<u32, u8>,
}

impl TestBus {
    fn new() -> Self {
        TestBus { mem: HashMap::new() }
    }
    fn poke(&mut self, addr: u32, val: u8) {
        self.mem.insert(addr, val);
    }
    fn peek(&self, addr: u32) -> u8 {
        *self.mem.get(&addr).unwrap_or(&0)
    }
}

impl Bus for TestBus {
    fn read(&mut self, addr: u32) -> u8 {
        *self.mem.get(&addr).unwrap_or(&0)
    }
    fn write(&mut self, addr: u32, v: u8) {
        self.mem.insert(addr, v);
    }
    fn idle(&mut self) {}
}

fn cpu_hash(cpu: &Cpu) -> u64 {
    let mut h = DefaultHasher::new();
    cpu.hash_into(&mut h);
    h.finish()
}

#[test]
fn e02_05_new_defaults() {
    let cpu = Cpu::new();
    assert_eq!(cpu.a, 0);
    assert_eq!(cpu.x, 0);
    assert_eq!(cpu.y, 0);
    assert_eq!(cpu.s, 0x01FF);
    assert_eq!(cpu.d, 0);
    assert_eq!(cpu.db, 0);
    assert_eq!(cpu.pb, 0);
    assert_eq!(cpu.pc, 0);
    assert_eq!(cpu.p, 0x34); // M=1, X=1, I=1
    assert!(cpu.e);
    assert!(!cpu.halted);
    assert!(!cpu.waiting);
}

#[test]
fn e02_05_set_p_emulation() {
    let mut cpu = Cpu::new();
    // In emulation mode, M and X bits are forced to 1 (0x30)
    cpu.set_p(0x00);
    assert_eq!(cpu.p, 0x30);

    cpu.set_p(0xCF);
    assert_eq!(cpu.p, 0xFF); // 0xCF | 0x30 = 0xFF
}

#[test]
fn e02_05_set_p_native_x() {
    let mut cpu = Cpu::new();
    cpu.e = false;
    cpu.x = 0x1234;
    cpu.y = 0x5678;

    // Set X flag (index 8-bit mode) -> truncate x and y to low byte
    cpu.set_p(FLAG_X);
    assert_eq!(cpu.p, FLAG_X);
    assert_eq!(cpu.x, 0x0034);
    assert_eq!(cpu.y, 0x0078);

    // Clear X flag -> no further truncation
    cpu.set_p(0x00);
    assert_eq!(cpu.p, 0x00);
    assert_eq!(cpu.x, 0x0034);
    assert_eq!(cpu.y, 0x0078);
}

#[test]
fn e02_05_set_nz() {
    let mut cpu = Cpu::new();

    // set_nz8(0x80): N=1, Z=0
    cpu.set_nz8(0x80);
    assert!(cpu.flag(FLAG_N));
    assert!(!cpu.flag(FLAG_Z));

    // set_nz8(0): Z=1, N=0
    cpu.set_nz8(0);
    assert!(!cpu.flag(FLAG_N));
    assert!(cpu.flag(FLAG_Z));

    // set_nz16(0x8000): N=1
    cpu.set_nz16(0x8000);
    assert!(cpu.flag(FLAG_N));
    assert!(!cpu.flag(FLAG_Z));

    // set_nz16(0x0080): N=0, Z=0 (high bit of 16-bit value is clear)
    cpu.set_nz16(0x0080);
    assert!(!cpu.flag(FLAG_N));
    assert!(!cpu.flag(FLAG_Z));

    // set_nz(0x0100, false): only low byte matters -> Z=1
    cpu.set_nz(0x0100, false);
    assert!(cpu.flag(FLAG_Z));

    // set_nz(0x0100, true): full 16-bit -> Z=0, N=0
    cpu.set_nz(0x0100, true);
    assert!(!cpu.flag(FLAG_Z));
    assert!(!cpu.flag(FLAG_N));
}

#[test]
fn e02_05_fetch_wrap() {
    let mut bus = TestBus::new();
    let mut cpu = Cpu::new();

    // fetch8 at PC wrap: pb=0x12, pc=0xFFFF -> reads 12:FFFF, wraps to pc=0x0000
    cpu.pb = 0x12;
    cpu.pc = 0xFFFF;
    bus.poke(0x12FFFF, 0xAA);
    let v8 = cpu.fetch8(&mut bus);
    assert_eq!(v8, 0xAA);
    assert_eq!(cpu.pc, 0x0000);
    assert_eq!(cpu.pb, 0x12);

    // fetch16 at pc=0x8000 with [0x34, 0x12] -> 0x1234, pc becomes 0x8002
    cpu.pc = 0x8000;
    bus.poke(0x128000, 0x34);
    bus.poke(0x128001, 0x12);
    let v16 = cpu.fetch16(&mut bus);
    assert_eq!(v16, 0x1234);
    assert_eq!(cpu.pc, 0x8002);

    // fetch24 with [0x56, 0x34, 0x12] -> 0x123456
    cpu.pc = 0x9000;
    bus.poke(0x129000, 0x56);
    bus.poke(0x129001, 0x34);
    bus.poke(0x129002, 0x12);
    let v24 = cpu.fetch24(&mut bus);
    assert_eq!(v24, 0x123456);
}

#[test]
fn e02_05_push_pull_emulation() {
    let mut bus = TestBus::new();
    let mut cpu = Cpu::new();
    // Emulation mode: stack wraps in page 1 (0x0100-0x01FF)
    cpu.e = true;
    cpu.s = 0x0100;

    // push8(0xAB): write at s=0x0100, then s decrements and wraps to 0x01FF
    cpu.push8(&mut bus, 0xAB);
    assert_eq!(bus.peek(0x000100), 0xAB);
    assert_eq!(cpu.s, 0x01FF);

    // pull8: s increments (wraps to 0x0100), read at that address
    let v = cpu.pull8(&mut bus);
    assert_eq!(v, 0xAB);
    assert_eq!(cpu.s, 0x0100);
}

#[test]
fn e02_05_push_pull_native() {
    let mut bus = TestBus::new();
    let mut cpu = Cpu::new();

    // New-style stack in native mode: full 16-bit range, no page wrapping
    cpu.e = false;
    cpu.s = 0x0000;

    // push8n(0x11): write at s=0x0000, then s wraps to 0xFFFF
    cpu.push8n(&mut bus, 0x11);
    assert_eq!(bus.peek(0x000000), 0x11);
    assert_eq!(cpu.s, 0xFFFF);

    // pull8n: s increments (wraps to 0x0000), read at that address
    let v = cpu.pull8n(&mut bus);
    assert_eq!(v, 0x11);
    assert_eq!(cpu.s, 0x0000);

    // New-style stack in emulation mode: still no page wrapping during push/pull
    cpu.e = true;
    cpu.s = 0x0100;
    cpu.push8n(&mut bus, 5);
    assert_eq!(bus.peek(0x000100), 5);
    assert_eq!(cpu.s, 0x00FF);

    // fix_stack_e restores page wrapping after new-style instruction in emulation mode
    cpu.fix_stack_e();
    assert_eq!(cpu.s, 0x01FF);
}

#[test]
fn e02_05_hash_into() {
    let cpu1 = Cpu::new();
    let cpu2 = Cpu::new();
    assert_eq!(cpu_hash(&cpu1), cpu_hash(&cpu2));

    // Different accumulator -> different hash
    let mut cpu3 = Cpu::new();
    cpu3.a = 1;
    assert_ne!(cpu_hash(&cpu1), cpu_hash(&cpu3));

    // Different e flag -> different hash
    let mut cpu4 = Cpu::new();
    cpu4.e = false;
    assert_ne!(cpu_hash(&cpu1), cpu_hash(&cpu4));
}
