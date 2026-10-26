mod common;
use common::*;
use puce16_core::bus::SysBus;
use puce16_core::cart::Cart;
use puce16_core::Region;

fn mk(code: &[u8]) -> SysBus {
    let rom = make_rom(code, Map::LoRom);
    SysBus::new(Cart::from_bytes(&rom).unwrap(), Region::Ntsc)
}

#[test]
fn e03_26_power_on() {
    let mut bus = mk(&[0xEA]);
    assert_eq!(bus.master, 0);
    assert_eq!(bus.mdr, 0);
    assert!(!bus.memsel);
    assert_eq!(bus.wram.read(0), 0x55);
    assert_eq!(bus.ppu.v(), 0);
    assert_eq!(bus.ppu.dot(), 0);
    assert_eq!(bus.vblank_count(), 0);
    assert!(!bus.take_vblank_flag());
    assert_eq!(bus.pending_dma(), 0);
    assert_eq!(bus.region, Region::Ntsc);
}

#[test]
fn e03_26_reset_keeps_wram() {
    let mut bus = mk(&[0xEA]);
    bus.wram.write(5, 1);
    bus.mmio.nmitimen = 0x80;
    bus.master = 1000;
    bus.mdr = 7;
    bus.reset();
    assert_eq!(bus.master, 0);
    assert_eq!(bus.mdr, 0);
    assert_eq!(bus.mmio.nmitimen, 0);
    assert_eq!(bus.wram.read(5), 1);
    assert_eq!(bus.wram.addr, 0);
    assert_eq!(bus.ppu.v(), 0);
    assert_eq!(bus.ppu.dot(), 0);
}

#[test]
fn e03_26_reset_keeps_cart_sram() {
    let rom = make_rom_with(&[], Map::LoRom, 15, 3, 0x02, 1);
    let mut bus = SysBus::new(Cart::from_bytes(&rom).unwrap(), Region::Ntsc);
    bus.cart.write(0x70, 0, 0x5A);
    bus.reset();
    assert_eq!(bus.cart.read(0x70, 0), Some(0x5A));
}

#[test]
fn e03_26_take_vblank_initial() {
    let mut bus = mk(&[0xEA]);
    assert_eq!(bus.vblank_count(), 0);
    assert!(!bus.take_vblank_flag());
    assert!(!bus.take_vblank_flag());
}

#[test]
fn e03_26_hash_equal() {
    let b1 = mk(&[0xEA]);
    let b2 = mk(&[0xEA]);
    let mut h1 = puce16_core::hash::Fnv64::new();
    let mut h2 = puce16_core::hash::Fnv64::new();
    b1.hash_into(&mut h1);
    b2.hash_into(&mut h2);
    assert_eq!(h1.finish(), h2.finish());

    let mut b3 = mk(&[0xEA]);
    b3.wram.write(0, 1);
    let mut h3 = puce16_core::hash::Fnv64::new();
    b3.hash_into(&mut h3);
    assert_ne!(h1.finish(), h3.finish());

    let mut b4 = mk(&[0xEA]);
    b4.master = 5;
    let mut h4 = puce16_core::hash::Fnv64::new();
    b4.hash_into(&mut h4);
    assert_ne!(h1.finish(), h4.finish());
}

#[test]
fn e03_26_pal_region() {
    let rom = make_rom(&[0xEA], Map::LoRom);
    let bus = SysBus::new(Cart::from_bytes(&rom).unwrap(), Region::Pal);
    assert_eq!(bus.ppu.last_line(), 311);
}
