// E02-09: addr/direct.rs - Dp, DpX, DpY, Sr addressing modes

use puce16_core::cpu::{addr, Cpu};
use puce16_core::cpu::table::Am;
use puce16_core::cpu::testbus::FlatBus;

fn make_cpu() -> Cpu {
    let mut cpu = Cpu::new();
    cpu.e = false; // native mode for 16-bit tests
    cpu
}

#[test]
fn e02_09_dp() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.d = 0x0000;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x10]);

    let e = addr::ea(&mut cpu, &mut bus, Am::Dp, false);
    assert_eq!(e.lo, 0x0010);
    assert_eq!(e.hi, 0x0011);
    // Log: 1 read (operand byte), no idle because D=0
    assert_eq!(bus.log.len(), 1);
}

#[test]
fn e02_09_dp_dl_nonzero() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.d = 0x0001;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x10]);

    let e = addr::ea(&mut cpu, &mut bus, Am::Dp, false);
    assert_eq!(e.lo, 0x0011); // D + dl = 1 + 16 = 17
    // Log: 1 read (operand) + 1 idle (DL != 0)
    assert_eq!(bus.log.len(), 2);
}

#[test]
fn e02_09_dp_wrap_16() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.d = 0xFFF0;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x20]);

    let e = addr::ea(&mut cpu, &mut bus, Am::Dp, false);
    assert_eq!(e.lo, 0x0010); // (0xFFF0 + 0x20) & 0xFFFF = 0x0010
    assert_eq!(e.hi, 0x0011);
}

#[test]
fn e02_09_dpx_native() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // X=0 (16-bit index)
    cpu.d = 0x2000;
    cpu.x = 0x0003;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x05]);

    let e = addr::ea(&mut cpu, &mut bus, Am::DpX, false);
    assert_eq!(e.lo, 0x2008); // D + dl + X = 0x2000 + 5 + 3 = 0x2008
    // Log: 1 read (operand) + 1 idle (index cycle), no DL idle because D=0x2000 has DL=0
    assert_eq!(bus.log.len(), 2);
}

#[test]
fn e02_09_dpx_native_dl() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // X=0 (16-bit index)
    cpu.d = 0x2001;
    cpu.x = 3;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x05]);

    let e = addr::ea(&mut cpu, &mut bus, Am::DpX, false);
    assert_eq!(e.lo, 0x2009); // D + dl + X = 0x2001 + 5 + 3 = 0x2009
    // Log: 1 read (operand) + 1 idle (DL != 0) + 1 idle (index cycle)
    assert_eq!(bus.log.len(), 3);
}

#[test]
fn e02_09_dpx_native_16bit_index() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // X=0 (16-bit index)
    cpu.d = 0;
    cpu.x = 0x0120;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0xF0]);

    let e = addr::ea(&mut cpu, &mut bus, Am::DpX, false);
    assert_eq!(e.lo, 0x0210); // D + dl + X = 0 + 0xF0 + 0x0120 = 0x0210 (no 8-bit wrap in native)
}

#[test]
fn e02_09_dpx_emulation_page_wrap() {
    let mut cpu = make_cpu();
    cpu.e = true; // emulation mode
    let mut bus = FlatBus::new();
    cpu.p = 0x30; // X=1 (8-bit index)
    cpu.d = 0x1200;
    cpu.x = 0x20;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0xF0]);

    let e = addr::ea(&mut cpu, &mut bus, Am::DpX, false);
    // Emulation page wrap: (dl + X) wraps within page D.h
    assert_eq!(e.lo, 0x1210); // (0xF0 + 0x20) & 0xFF = 0x10, page stays 0x12

    // Test with x=0 and operand that wraps past 0xFF
    cpu.x = 0;
    cpu.pc = 0x8000;
    bus.clear_log();
    bus.poke(0x8000, &[0xFF]);
    let e2 = addr::ea(&mut cpu, &mut bus, Am::DpX, false);
    assert_eq!(e2.lo, 0x12FF); // (0xFF + 0) & 0xFF = 0xFF
    assert_eq!(e2.hi, 0x1200); // wraps within page: (0xFF + 1) & 0xFF = 0x00
}

#[test]
fn e02_09_dpx_emulation_dl_nonzero() {
    let mut cpu = make_cpu();
    cpu.e = true; // emulation mode
    let mut bus = FlatBus::new();
    cpu.p = 0x30; // X=1 (8-bit index)
    cpu.d = 0x1201;
    cpu.x = 0x20;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0xF0]);

    let e = addr::ea(&mut cpu, &mut bus, Am::DpX, false);
    // DL != 0: no page wrap, full 16-bit addition
    assert_eq!(e.lo, 0x1311); // D + dl + X = 0x1201 + 0xF0 + 0x20 = 0x1311
}

#[test]
fn e02_09_dpy() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.p = 0x00; // X=0 (16-bit index)
    cpu.d = 0;
    cpu.y = 0x0004;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x10]);

    let e = addr::ea(&mut cpu, &mut bus, Am::DpY, false);
    assert_eq!(e.lo, 0x0014); // D + dl + Y = 0 + 0x10 + 4 = 0x14
    // Log: 1 read (operand) + 1 idle (index cycle)
    assert_eq!(bus.log.len(), 2);
}

#[test]
fn e02_09_sr() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.s = 0x01FD;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x03]);

    let e = addr::ea(&mut cpu, &mut bus, Am::Sr, false);
    assert_eq!(e.lo, 0x0200); // S + sr = 0x01FD + 3 = 0x0200
    assert_eq!(e.hi, 0x0201);
    // Log: 1 read (operand) + 1 idle
    assert_eq!(bus.log.len(), 2);
}

#[test]
fn e02_09_sr_wrap() {
    let mut cpu = make_cpu();
    let mut bus = FlatBus::new();
    cpu.s = 0xFFFF;
    cpu.pc = 0x8000;
    bus.poke(0x8000, &[0x02]);

    let e = addr::ea(&mut cpu, &mut bus, Am::Sr, false);
    assert_eq!(e.lo, 0x0001); // (0xFFFF + 2) & 0xFFFF = 0x0001
}
