// E02-10: addr/indirect.rs - DpInd, DpIndLong, DpXInd, DpIndY, DpIndLongY, SrIndY addressing modes

use puce16_core::cpu::{addr, Cpu};
use puce16_core::cpu::table::Am;
use puce16_core::cpu::testbus::{CycleKind, FlatBus};

fn make_cpu() -> Cpu {
    let mut cpu = Cpu::new();
    cpu.e = false; // native mode for 16-bit tests
    cpu
}

#[test]
fn e02_10_dp_ind() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.d = 0x0000;
    cpu.db = 0x7E;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x10]); // operand byte
    bus.poke(0x0010, &[0x34, 0x12]); // pointer at D+dl

    let e = addr::ea(&mut cpu, &mut bus, Am::DpInd, false);
    assert_eq!(e.lo, 0x7E1234);
    assert_eq!(e.hi, 0x7E1235);
    // Log: R(operand), R(0x0010), R(0x0011) - no idle because D=0
    assert_eq!(bus.log.len(), 3);
}

#[test]
fn e02_10_dp_ind_dl() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.d = 0x0101; // DL=1, so idle cycle required
    cpu.db = 0x7E;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x10]); // operand byte
    bus.poke(0x0111, &[0x34, 0x12]); // pointer at D+dl

    let e = addr::ea(&mut cpu, &mut bus, Am::DpInd, false);
    assert_eq!(e.lo, (cpu.db as u32) << 16 | 0x1234);
    // Log: Read(operand), Idle(DL!=0), Read(ptr lo), Read(ptr hi)
    assert_eq!(bus.log.len(), 4);
}

#[test]
fn e02_10_dp_ind_emulation_wrap() {
    let mut cpu = make_cpu();
    cpu.e = true; // emulation mode
    let mut bus = FlatBus::new();
    cpu.p = 0x30; // X=1 (8-bit index)
    cpu.d = 0x1200;
    cpu.db = 0x01;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0xFF]); // operand byte
    bus.poke(0x12FF, &[0x78]); // pointer lo at D+dl (wraps within page)
    bus.poke(0x1200, &[0x56]); // pointer hi at D+dl+1 (wraps to start of page)

    let e = addr::ea(&mut cpu, &mut bus, Am::DpInd, false);
    assert_eq!(e.lo, 0x015678);
}

#[test]
fn e02_10_dp_ind_long() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.d = 0x0000;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x20]); // operand byte
    bus.poke(0x0020, &[0xFF, 0xFF, 0x12]); // long pointer at D+dl

    let e = addr::ea(&mut cpu, &mut bus, Am::DpIndLong, false);
    assert_eq!(e.lo, 0x12FFFF);
    assert_eq!(e.hi, 0x130000);
    // Log: Read(operand), Read(ptr lo), Read(ptr hi), Read(bank) - no idle because D=0
    assert_eq!(bus.log.len(), 4);
}

#[test]
fn e02_10_dp_ind_long_bank0_wrap() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.d = 0xFFFF;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x00]); // operand byte (dl=0)
    // Pointer address: D + dl = 0xFFFF + 0 = 0xFFFF
    bus.poke(0xFFFF, &[0x11]); // ptr lo at 0xFFFF
    bus.poke(0x0000, &[0x22, 0x33]); // ptr hi at 0x0000, bank at 0x0001

    let e = addr::ea(&mut cpu, &mut bus, Am::DpIndLong, false);
    assert_eq!(e.lo, 0x332211);
    // Log contains an Idle because DL=0xFF != 0
    let has_idle = bus.log.iter().any(|c| c.kind == CycleKind::Idle);
    assert!(has_idle);
}

#[test]
fn e02_10_dp_x_ind() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // X=0 (16-bit index)
    cpu.d = 0x0000;
    cpu.x = 0x0004;
    cpu.db = 2;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x10]); // operand byte
    bus.poke(0x0014, &[0x00, 0x90]); // pointer at D+dl+X

    let e = addr::ea(&mut cpu, &mut bus, Am::DpXInd, false);
    assert_eq!(e.lo, 0x029000);
    // Log: Read(operand), Idle(DL!=0? no, D=0), Idle(index cycle), Read(ptr lo), Read(ptr hi)
    // Actually D=0 so DL=0, no idle for that. But there's always an index cycle idle.
    assert_eq!(bus.log.len(), 4);
}

#[test]
fn e02_10_dp_x_ind_emulation_wrap() {
    let mut cpu = make_cpu();
    cpu.e = true; // emulation mode
    let mut bus = FlatBus::new();
    cpu.p = 0x30; // X=1 (8-bit index)
    cpu.d = 0x0000;
    cpu.x = 0x01;
    cpu.db = 0;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0xFE]); // operand byte (dl=0xFE)
    // Pointer address: D + dl + X = 0 + 0xFE + 1 = 0xFF (wraps within page in emulation)
    bus.poke(0xFF, &[0x34]); // ptr lo at 0xFF
    bus.poke(0x00, &[0x12]); // ptr hi at 0x00 (wraps to start of page)

    let e = addr::ea(&mut cpu, &mut bus, Am::DpXInd, false);
    assert_eq!(e.lo, 0x001234);
}

#[test]
fn e02_10_dp_ind_y_no_cross() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x10; // X=1 (8-bit index)
    cpu.d = 0x0000;
    cpu.db = 5;
    cpu.y = 0x10;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x30]); // operand byte
    bus.poke(0x0030, &[0x00, 0x20]); // pointer at D+dl

    let e = addr::ea(&mut cpu, &mut bus, Am::DpIndY, false);
    assert_eq!(e.lo, 0x052010);
    // Log: Read(operand), Read(ptr lo), Read(ptr hi) - no idle (no page cross, X=1, store=false)
    assert_eq!(bus.log.len(), 3);
}

#[test]
fn e02_10_dp_ind_y_cross() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x10; // X=1 (8-bit index)
    cpu.d = 0x0000;
    cpu.db = 5;
    cpu.y = 0x10;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x30]); // operand byte
    bus.poke(0x0030, &[0xF8, 0x20]); // pointer at D+dl (crosses page when Y added)

    let e = addr::ea(&mut cpu, &mut bus, Am::DpIndY, false);
    assert_eq!(e.lo, 0x052108);
    // Log: Read(operand), Read(ptr lo), Read(ptr hi), Idle(page cross)
    assert_eq!(bus.log.len(), 4);
}

#[test]
fn e02_10_dp_ind_y_store() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x10; // X=1 (8-bit index)
    cpu.d = 0x0000;
    cpu.db = 5;
    cpu.y = 0x10;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x30]); // operand byte
    bus.poke(0x0030, &[0x00, 0x20]); // pointer at D+dl

    let e = addr::ea(&mut cpu, &mut bus, Am::DpIndY, true);
    assert_eq!(e.lo, 0x052010);
    // Log: Read(operand), Read(ptr lo), Read(ptr hi), Idle(store)
    assert_eq!(bus.log.len(), 4);
}

#[test]
fn e02_10_dp_ind_y_16bit_index() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // X=0 (16-bit index)
    cpu.d = 0x0000;
    cpu.db = 5;
    cpu.y = 0x0001;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x30]); // operand byte
    bus.poke(0x0030, &[0x00, 0x20]); // pointer at D+dl

    let e = addr::ea(&mut cpu, &mut bus, Am::DpIndY, false);
    assert_eq!(e.lo, (cpu.db as u32) << 16 | 0x2001);
    // Log: Read(operand), Read(ptr lo), Read(ptr hi), Idle(16-bit index)
    assert_eq!(bus.log.len(), 4);
}

#[test]
fn e02_10_dp_ind_long_y() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // X=0 (16-bit index)
    cpu.d = 0x0000;
    cpu.y = 0x0020;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x40]); // operand byte
    bus.poke(0x0040, &[0xF0, 0xFF, 0x01]); // long pointer at D+dl

    let e = addr::ea(&mut cpu, &mut bus, Am::DpIndLongY, false);
    assert_eq!(e.lo, 0x020010);
    assert_eq!(e.hi, 0x020011);
    // Log: Read(operand), Read(ptr lo), Read(ptr hi), Read(bank) - no idle for long Y mode
    assert_eq!(bus.log.len(), 4);
}

#[test]
fn e02_10_sr_ind_y() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x10; // X=1 (8-bit index)
    cpu.s = 0x01F0;
    cpu.db = 0;
    cpu.y = 0x02;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x04]); // operand byte (sr=4)
    bus.poke(0x01F4, &[0x00, 0x30]); // pointer at S+sr

    let e = addr::ea(&mut cpu, &mut bus, Am::SrIndY, false);
    assert_eq!(e.lo, 0x003002);
    // Log: Read(operand), Idle, Read(ptr lo), Read(ptr hi), Idle
    assert_eq!(bus.log.len(), 5);
}
