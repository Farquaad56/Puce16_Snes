//! E03-28: SysBus::write_inner routing (spec 09 section 6).

mod common;
use common::*;
use puce16_core::bus::SysBus;
use puce16_core::cart::Cart;
use puce16_core::Region;

fn mk(code: &[u8]) -> SysBus {
    SysBus::new(Cart::from_bytes(&make_rom(code, Map::LoRom)).unwrap(), Region::Ntsc)
}

#[test]
fn e03_28_wram_writes() {
    let mut bus = mk(&[]);
    bus.write_inner(0x7E0020, 0x12);
    assert_eq!(bus.wram.read(0x20), 0x12);

    bus.write_inner(0x000020, 0x34);
    assert_eq!(bus.wram.read(0x20), 0x34);

    bus.write_inner(0x3F0021, 0x56);
    assert_eq!(bus.wram.read(0x21), 0x56);

    bus.write_inner(0x7F0000, 5);
    assert_eq!(bus.wram.read(0x10000), 5);

    bus.write_inner(0x7FFFFF, 9);
    assert_eq!(bus.wram.read(0x1FFFF), 9);
}

#[test]
fn e03_28_wmadd_port() {
    let mut bus = mk(&[]);
    bus.write_inner(0x002181, 0x34);
    bus.write_inner(0x002182, 0x12);
    bus.write_inner(0x002183, 0x01);
    bus.write_inner(0x002180, 0xCC);

    assert_eq!(bus.wram.read(0x11234), 0xCC);
    assert_eq!(bus.wram.addr, 0x11235);
}

#[test]
fn e03_28_nmitimen_effects() {
    let mut bus = mk(&[]);
    bus.write_inner(0x004200, 0x81);
    assert_eq!(bus.mmio.nmitimen, 0x81);

    bus.write_inner(0x00420C, 0x05);
    assert_eq!(bus.mmio.hdmaen, 5);

    bus.write_inner(0x00420D, 0x01);
    assert!(bus.memsel);

    bus.write_inner(0x00420D, 0x00);
    assert!(!bus.memsel);
}

#[test]
fn e03_28_mdmaen_pending() {
    let mut bus = mk(&[]);
    bus.write_inner(0x00420B, 0x05);
    assert_eq!(bus.pending_dma(), 5);

    bus.write_inner(0x00420B, 0x00);
    assert_eq!(bus.pending_dma(), 0);
}

#[test]
fn e03_28_wrio() {
    let mut bus = mk(&[]);
    bus.write_inner(0x004201, 0x7F);
    assert_eq!(bus.mmio.wrio, 0x7F);
}

#[test]
fn e03_28_joyout() {
    let mut bus = mk(&[]);
    bus.joy.set_buttons(0, 0x8000);

    // out0 true: first read returns start bit (bit 0 = 1)
    bus.write_inner(0x004016, 1);
    assert_eq!(bus.read_inner(0x004016) & 1, 1);
    assert_eq!(bus.read_inner(0x004016) & 1, 1);
    assert_eq!(bus.read_inner(0x004016) & 1, 1);

    // out0 false: next read returns bit 15 (button A pressed = 1), then rest are 0
    bus.write_inner(0x004016, 0);
    assert_eq!(bus.read_inner(0x004016) & 1, 1); // button A bit
    for _ in 0..14 {
        assert_eq!(bus.read_inner(0x004016) & 1, 0);
    }
}

#[test]
fn e03_28_dma_regs() {
    let mut bus = mk(&[]);
    bus.write_inner(0x004300, 0x01);
    assert_eq!(bus.dma.ch[0].dmap, 1);

    bus.write_inner(0x004372, 0x34);
    assert_eq!(bus.dma.ch[7].a1t & 0xFF, 0x34);
}

#[test]
fn e03_28_ppu_apu() {
    let mut bus = mk(&[]);
    bus.write_inner(0x002100, 0x0F);
    assert!(!bus.ppu.forced_blank());

    bus.write_inner(0x002100, 0x80);
    assert!(bus.ppu.forced_blank());

    bus.write_inner(0x002142, 0x09);
    assert_eq!(bus.apu.cpu_to_apu(2), 9);

    bus.write_inner(0x002146, 0x07);
    assert_eq!(bus.apu.cpu_to_apu(2), 7);
}

#[test]
fn e03_28_rom_and_sram() {
    // ROM without SRAM: write is ignored
    let mut bus = mk(&[0xEA]);
    bus.write_inner(0x008000, 0x77);
    assert_eq!(bus.read_inner(0x008000), 0xEA);

    // ROM with SRAM: write goes to SRAM
    let rom = make_rom_with(&[0xEA], Map::LoRom, 15, 3, 0x02, 1);
    let mut bus = SysBus::new(Cart::from_bytes(&rom).unwrap(), Region::Ntsc);
    bus.write_inner(0x700000, 0x5A);
    assert_eq!(bus.cart.read(0x70, 0), Some(0x5A));
    assert_eq!(bus.read_inner(0x700000), 0x5A);
}

#[test]
fn e03_28_ignored_zones() {
    let mut bus = mk(&[]);
    let h1 = hash_bus(&bus);

    // Writes to ignored zones should not panic or change state
    bus.write_inner(0x002000, 0xFF);
    bus.write_inner(0x0021FF, 0xFF);
    bus.write_inner(0x004100, 0xFF);
    bus.write_inner(0x004210, 0xFF);
    bus.write_inner(0x004380, 0xFF);
    bus.write_inner(0x005000, 0xFF);

    let h2 = hash_bus(&bus);
    assert_eq!(h1, h2);
}

#[test]
fn e03_28_unknown_mmio_reg() {
    let mut bus = mk(&[]);
    let h1 = hash_bus(&bus);

    // Writes to unknown MMIO regs should not change state
    bus.write_inner(0x00420E, 0xFF);
    bus.write_inner(0x00420F, 0xFF);

    let h2 = hash_bus(&bus);
    assert_eq!(h1, h2);
}

fn hash_bus(bus: &SysBus) -> u64 {
    use puce16_core::hash::Fnv64;
    let mut h = Fnv64::new();
    bus.hash_into(&mut h);
    h.finish()
}