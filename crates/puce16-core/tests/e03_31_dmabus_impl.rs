mod common;
use common::*;
use puce16_core::bus::SysBus;
use puce16_core::cart::Cart;
use puce16_core::dma::{Dma, DmaBus};
use puce16_core::Region;

fn mk(code: &[u8]) -> SysBus {
    let rom = make_rom(code, Map::LoRom);
    let cart = Cart::from_bytes(&rom).unwrap();
    SysBus::new(cart, Region::Ntsc)
}

#[test]
fn e03_31_a_read_ok_zones() {
    // WRAM read via A-bus
    let mut bus = mk(&[]);
    bus.wram.write(0x30, 0x9A);
    assert_eq!(bus.a_read(0x7E0030), 0x9A);
    assert_eq!(bus.a_read(0x000030), 0x9A); // mirror

    // ROM read via A-bus
    let mut bus2 = mk(&[0xA9]);
    assert_eq!(bus2.a_read(0x008000), 0xA9);
    assert_eq!(bus2.mdr, 0xA9);
}

#[test]
fn e03_31_a_read_forbidden() {
    let mut bus = mk(&[]);
    bus.mdr = 0x77;

    // All forbidden zones return mdr (open bus)
    assert_eq!(bus.a_read(0x002100), 0x77);
    assert_eq!(bus.a_read(0x0021FF), 0x77);
    assert_eq!(bus.a_read(0x004000), 0x77);
    assert_eq!(bus.a_read(0x0041FF), 0x77);
    assert_eq!(bus.a_read(0x004200), 0x77);
    assert_eq!(bus.a_read(0x00421F), 0x77);
    assert_eq!(bus.a_read(0x004300), 0x77);
    assert_eq!(bus.a_read(0x00437F), 0x77);
    assert_eq!(bus.a_read(0x802100), 0x77); // bank 80 mirror

    // $4016 is in the JOYSER zone (forbidden)
    assert_eq!(bus.a_read(0x004016), 0x77);

    // $2000 is NOT forbidden - goes through read_inner (open bus via mdr)
    let v = bus.a_read(0x002000);
    assert_eq!(v, bus.mdr);
}

#[test]
fn e03_31_a_write() {
    // Write to WRAM works
    let mut bus = mk(&[]);
    bus.a_write(0x7E0040, 9);
    assert_eq!(bus.wram.read(0x40), 9);

    // Write to forbidden zone is ignored (PPU register)
    let mut bus2 = mk(&[]);
    bus2.write_inner(0x002100, 0x80); // set forced blank via PPU reg write
    assert!(bus2.ppu.forced_blank());
    bus2.a_write(0x002100, 0x00);     // DMA A-bus write to same reg should be ignored
    assert!(bus2.ppu.forced_blank());

    // Write to MDMAEN is ignored (no DMA triggered)
    let mut bus3 = mk(&[]);
    bus3.a_write(0x00420B, 0x01);
    assert_eq!(bus3.pending_dma(), 0);

    // Write to DMA channel register is ignored
    let mut bus4 = mk(&[]);
    bus4.a_write(0x004300, 5);
    assert_eq!(bus4.dma.ch[0].dmap, 0xFF);
}

#[test]
fn e03_31_b_bus() {
    let mut bus = mk(&[]);

    // B-bus write to $2180 (WRAM data port)
    bus.b_write(0x80, 0xAB);
    assert_eq!(bus.wram.read(0), 0xAB);
    assert_eq!(bus.wram.addr, 1);

    // B-bus read from $2180 reads next byte and increments
    let v = bus.b_read(0x80);
    assert_eq!(v, bus.mdr); // open bus value at addr 1 (uninitialized)

    // B-bus write to PPU register works
    bus.b_write(0x00, 0x00);
    assert!(!bus.ppu.forced_blank());

    // B-bus write to APU port works
    bus.b_write(0x42, 7);
    assert_eq!(bus.apu.cpu_to_apu(2), 7);

    // Write to high register doesn't panic
    bus.b_write(0xFF, 1);
}

#[test]
fn e03_31_b_read_updates_mdr() {
    let mut bus = mk(&[]);
    bus.apu.debug_set_from_apu(3, 0x5E);
    let v = bus.b_read(0x43);
    assert_eq!(v, 0x5E);
    assert_eq!(bus.mdr, 0x5E);
}

#[test]
fn e03_31_no_time_except_elapse() {
    let mut bus = mk(&[]);

    // A-bus and B-bus operations don't advance time
    bus.wram.write(0x10, 0xAA);
    bus.a_read(0x7E0010);
    bus.a_write(0x7E0011, 0xBB);
    bus.b_read(0x80);
    bus.b_write(0x80, 0xCC);
    assert_eq!(bus.master, 0);

    // elapse advances time
    bus.elapse(10);
    assert_eq!(bus.master(), 10);

    // elapse drives PPU (1364 clocks = 1 scanline)
    bus.elapse(1364);
    assert_eq!(bus.ppu.v(), 1);
}

#[test]
fn e03_31_dma_through_trait() {
    let mut bus = mk(&[1, 2, 3]); // ROM at $8000 contains [1,2,3]

    // Configure DMA channel 0: A->B, source $8000, dest $2180 (WRAM data), count=3
    let mut d = std::mem::take(&mut bus.dma);
    d.ch[0].dmap = 0;      // A->B, increment
    d.ch[0].bbad = 0x80;   // $2180 (WRAM data port)
    d.ch[0].a1b = 0;       // bank 00
    d.ch[0].a1t = 0x8000;  // source address
    d.ch[0].das = 3;       // transfer 3 bytes

    bus.wram.set_addr_h(0);
    bus.wram.set_addr_m(1);   // address 0x100 = high bit 0, mid byte 1, low byte 0
    bus.wram.set_addr_l(0);

    d.run_general(&mut bus, 1);
    std::mem::swap(&mut d, &mut bus.dma);

    // Verify data was transferred to WRAM at $7E:0100-0102
    assert_eq!(bus.wram.read(0x100), 1);
    assert_eq!(bus.wram.read(0x101), 2);
    assert_eq!(bus.wram.read(0x102), 3);

    // Verify timing: align(0) + startup(8) + channel(8) + 3*byte(8) = 40 clocks
    assert_eq!(bus.master(), 40);
}