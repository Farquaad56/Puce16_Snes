//! E03-30: bus/cycle.rs - DRAM refresh (40 clocks per line).

mod common;
use common::*;
use puce16_core::bus::SysBus;
use puce16_core::cart::Cart;
use puce16_core::cpu::Bus;
use puce16_core::Region;

fn mk(code: &[u8]) -> SysBus {
    SysBus::new(Cart::from_bytes(&make_rom(code, Map::LoRom)).unwrap(), Region::Ntsc)
}

#[test]
fn e03_30_refresh_once_per_line() {
    let mut bus = mk(&[]);
    // Advance to just before refresh threshold (hclock 536 < 538).
    bus.advance(536);
    assert_eq!(bus.master, 536);

    // First idle: hclock 536 < 538, no refresh. Just +6 clocks.
    Bus::idle(&mut bus);
    assert_eq!(bus.master, 542);

    // Second idle: hclock 542 >= 538, refresh due. +40 (refresh) + 6 (idle).
    Bus::idle(&mut bus);
    assert_eq!(bus.master, 588);

    // Third idle: refresh already done this line, just +6 clocks.
    Bus::idle(&mut bus);
    assert_eq!(bus.master, 594);
}

#[test]
fn e03_30_refresh_next_line() {
    let mut bus = mk(&[]);
    // Trigger refresh on line 0 (same as previous test).
    bus.advance(536);
    Bus::idle(&mut bus); // no refresh, master=542
    Bus::idle(&mut bus); // refresh, master=588

    // Advance to hclock 538 on line 1. Line length is 1364 clocks.
    // From master=588 (hclock=588), advance to next line's refresh point.
    let remaining = 1364 - 588;
    bus.advance(remaining + 538);
    assert_eq!(bus.master, 1902);

    // Refresh should be due again on this new line.
    Bus::idle(&mut bus);
    assert_eq!(bus.master, 1948); // 1902 + 40 (refresh) + 6 (idle)
}

#[test]
fn e03_30_no_refresh_early() {
    let mut bus = mk(&[]);
    // 80 idles on line 0: 80 * 6 = 480 clocks, all before hclock 538.
    for _ in 0..80 {
        Bus::idle(&mut bus);
    }
    assert_eq!(bus.master, 480);
}

#[test]
fn e03_30_refresh_crossing_line() {
    let mut bus = mk(&[]);
    // Advance to hclock 1360 on line 0 (refresh not yet consumed).
    bus.advance(1360);
    assert_eq!(bus.ppu.v(), 0);

    // Idle triggers refresh: +40 clocks crosses line boundary (1360+40=1400 > 1364),
    // then +6 for the idle itself.
    Bus::idle(&mut bus);
    assert_eq!(bus.master, 1406);
    assert_eq!(bus.ppu.v(), 1);

    // Refresh was marked BEFORE advancing, so on_dot at dot 0 of line 1 reset
    // the flag. Now hclock is 42 (1406 - 1364), which is < 538.
    assert!(!bus.timing.refresh_due(bus.master));

    // Advance to hclock 538 on line 1: refresh should be due again.
    bus.advance(538 - 42);
    assert!(bus.timing.refresh_due(bus.master));
}

#[test]
fn e03_30_read_triggers_refresh() {
    let mut bus = mk(&[0xA9]);
    // Advance to hclock 540 (past refresh threshold).
    bus.advance(540);

    // Read triggers begin_cycle, which inserts the refresh.
    Bus::read(&mut bus, 0x008000);
    assert_eq!(bus.master, 588); // 540 + 40 (refresh) + 8 (ROM read)
}

#[test]
fn e03_30_hundred_reads_with_refresh() {
    let mut bus = mk(&[0xA9]);
    // 100 ROM reads from master=0. Each read costs 8 clocks.
    // First 68 reads: 68 * 8 = 544 clocks (hclock reaches 544).
    // Read #69 starts at hclock 544 >= 538, so refresh is inserted (+40).
    // Total: 100 * 8 + 40 = 840.
    for _ in 0..100 {
        Bus::read(&mut bus, 0x008000);
    }
    assert_eq!(bus.master, 840);
    assert_eq!(bus.ppu.v(), 0); // still on line 0 (840 < 1364)
}
