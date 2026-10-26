//! E03-29: bus/cycle.rs - time advancement, catch_up, cpu::Bus impl for SysBus.

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
fn e03_29_idle_time() {
    let mut bus = mk(&[]);
    Bus::idle(&mut bus);
    assert_eq!(bus.master, 6);
    assert_eq!(bus.ppu.dot(), 2); // dots 0 and 1 executed

    Bus::idle(&mut bus);
    assert_eq!(bus.master, 12);
    assert_eq!(bus.ppu.dot(), 3);
}

#[test]
fn e03_29_read_costs() {
    let mut bus = mk(&[0xA9]);
    let v = Bus::read(&mut bus, 0x008000);
    assert_eq!(v, 0xA9);
    assert_eq!(bus.master, 8);
    assert_eq!(bus.mdr, 0xA9);

    Bus::read(&mut bus, 0x002100);
    assert_eq!(bus.master, 14);

    Bus::read(&mut bus, 0x004016);
    assert_eq!(bus.master, 26);

    Bus::read(&mut bus, 0x000000);
    assert_eq!(bus.master, 34);
}

#[test]
fn e03_29_write_costs_and_mdr() {
    let mut bus = mk(&[]);
    Bus::write(&mut bus, 0x000010, 0x44);
    assert_eq!(bus.wram.read(0x10), 0x44);
    assert_eq!(bus.mdr, 0x44);
    assert_eq!(bus.master, 8);

    Bus::write(&mut bus, 0x002100, 0x80);
    assert_eq!(bus.master, 14);
}

#[test]
fn e03_29_memsel_fast() {
    let mut bus = mk(&[]);
    bus.memsel = true;
    Bus::read(&mut bus, 0x808000);
    assert_eq!(bus.master, 6);

    Bus::read(&mut bus, 0x008000);
    assert_eq!(bus.master, 14);
}

#[test]
fn e03_29_addr_masked() {
    let mut bus = mk(&[0xA9]);
    let v = Bus::read(&mut bus, 0x01008000);
    assert_eq!(v, 0xA9);
    assert_eq!(bus.master, 8);
}

#[test]
fn e03_29_line_boundary() {
    let mut bus = mk(&[]);
    bus.advance(1360);
    assert_eq!(bus.ppu.v(), 0);
    assert_eq!(bus.ppu.dot(), 339); // dot 339 starts at 1360, not yet executed

    bus.advance(1);
    assert_eq!(bus.ppu.v(), 1);
    assert_eq!(bus.ppu.dot(), 0); // dot 339 executes when master > 1360

    let mut bus2 = mk(&[]);
    bus2.advance(1364);
    assert_eq!(bus2.ppu.v(), 1);
    assert_eq!(bus2.ppu.dot(), 0);
}

#[test]
fn e03_29_vblank_flag_strict() {
    let mut bus = mk(&[]);
    bus.advance(306_904);
    assert!(!bus.take_vblank_flag()); // dot (225,1) starts at 306904, not yet executed

    bus.advance(1);
    assert!(bus.take_vblank_flag());
    assert!(!bus.take_vblank_flag());
    assert_eq!(bus.vblank_count(), 1);
    assert!(bus.mmio.rdnmi_flag);
}

#[test]
fn e03_29_nmi_pending_consumed() {
    let mut bus = mk(&[]);
    bus.mmio.nmitimen = 0x80;
    bus.advance(306_904);
    assert!(!Bus::nmi_pending(&mut bus));

    bus.advance(1);
    assert!(Bus::nmi_pending(&mut bus));
    assert!(!Bus::nmi_pending(&mut bus));
}

#[test]
fn e03_29_irq_level() {
    let mut bus = mk(&[]);
    bus.mmio.irq_line = true;
    assert!(Bus::irq_asserted(&mut bus));
    assert!(Bus::irq_asserted(&mut bus)); // level, not consumed

    bus.mmio.irq_line = false;
    assert!(!Bus::irq_asserted(&mut bus));
}

#[test]
fn e03_29_frame_length() {
    let mut bus = mk(&[]);
    bus.advance(357_368);
    assert_eq!(bus.ppu.v(), 0);
    assert_eq!(bus.ppu.dot(), 0);
    assert!(bus.ppu.field());
    assert_eq!(bus.vblank_count(), 1);
}

#[test]
fn e03_29_time_is_monotonic() {
    let mut bus = mk(&[]);
    for _ in 0..60 {
        Bus::read(&mut bus, 0x008000);
    }
    assert_eq!(bus.master, 480); // 480 < 538: no DRAM refresh even after E03-30
    assert_eq!(bus.ppu.v(), 0);
    assert_eq!(bus.ppu.dot(), 120);
}
