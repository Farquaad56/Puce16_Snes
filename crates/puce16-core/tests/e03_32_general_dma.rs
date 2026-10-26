mod common;
use common::*;
use puce16_core::bus::SysBus;
use puce16_core::cart::Cart;
use puce16_core::cpu::Bus;
use puce16_core::Region;

fn mk(code: &[u8]) -> SysBus {
    let rom = make_rom(code, Map::LoRom);
    let cart = Cart::from_bytes(&rom).unwrap();
    SysBus::new(cart, Region::Ntsc)
}

#[test]
fn e03_32_dma_after_next_cycle() {
    // Build ROM with data at $8200 (offset 0x200 in LoRom = bank 01 offset 0x200)
    let mut data_block = vec![0u8; 0x210];
    data_block[0x200] = 0x11;
    data_block[0x201] = 0x22;
    data_block[0x202] = 0x33;
    data_block[0x203] = 0x44;
    let rom = make_rom(&data_block, Map::LoRom);
    let cart = Cart::from_bytes(&rom).unwrap();
    let mut bus = SysBus::new(cart, Region::Ntsc);

    // Fill WRAM with known value so we can detect transfer
    for i in 0..0x210 {
        bus.wram.write(i, 0x55);
    }

    // Set WMADD to $7E:0200 (WRAM destination)
    bus.write_inner(0x002181, 0x00);
    bus.write_inner(0x002182, 0x02);

    // Configure DMA channel 0: A->B, source $8200 (ROM), dest $2180 (WRAM data port)
    bus.dma.write(0x4300, 0x00); // dmap: A->B, increment
    bus.dma.write(0x4301, 0x80); // bbad: $2180
    bus.dma.write(0x4302, 0x00); // a1t low
    bus.dma.write(0x4303, 0x82); // a1t high -> source $8200
    bus.dma.write(0x4304, 0x00); // a1b: bank 00 (ROM)
    bus.dma.write(0x4305, 0x04); // das low: transfer 4 bytes
    bus.dma.write(0x4306, 0x00); // das high

    // Write MDMAEN=0x01 to trigger channel 0
    Bus::write(&mut bus, 0x00420B, 0x01);

    // After the write cycle: master should be 6 (access_clocks for $420B)
    assert_eq!(bus.master, 6);
    // DMA is pending but hasn't run yet (dma_wait=true)
    assert_eq!(bus.pending_dma(), 1);
    // No data copied yet - WRAM still has fill value
    assert_eq!(bus.wram.read(0x200), 0x55);
}

#[test]
fn e03_32_dma_runs_on_next_cycle() {
    let mut data_block = vec![0u8; 0x210];
    data_block[0x200] = 0x11;
    data_block[0x201] = 0x22;
    data_block[0x202] = 0x33;
    data_block[0x203] = 0x44;
    let rom = make_rom(&data_block, Map::LoRom);
    let cart = Cart::from_bytes(&rom).unwrap();
    let mut bus = SysBus::new(cart, Region::Ntsc);

    for i in 0..0x210 {
        bus.wram.write(i, 0x55);
    }

    bus.write_inner(0x002181, 0x00);
    bus.write_inner(0x002182, 0x02);

    bus.dma.write(0x4300, 0x00);
    bus.dma.write(0x4301, 0x80);
    bus.dma.write(0x4302, 0x00);
    bus.dma.write(0x4303, 0x82);
    bus.dma.write(0x4304, 0x00);
    bus.dma.write(0x4305, 0x04);
    bus.dma.write(0x4306, 0x00);

    Bus::write(&mut bus, 0x00420B, 0x01);
    assert_eq!(bus.master, 6);

    // Next CPU access triggers the DMA in end_cycle
    Bus::read(&mut bus, 0x008000);

    // DMA should have completed
    assert_eq!(bus.pending_dma(), 0);
    // Data was copied to WRAM at $7E:0200-0203
    let dest = [bus.wram.read(0x200), bus.wram.read(0x201), bus.wram.read(0x202), bus.wram.read(0x203)];
    assert_eq!(dest, [0x11, 0x22, 0x33, 0x44]);
    // WRAM address advanced past the transfer
    assert_eq!(bus.wram.addr, 0x204);
    // Channel das is now 0 (transfer complete)
    assert_eq!(bus.dma.ch[0].das, 0);
    // Source address advanced by 4 bytes
    assert_eq!(bus.dma.ch[0].a1t, 0x8204);
    // Timing: 6 + 8 (read cycle) = 14; align +2 -> 16; startup +8 -> 24; channel +8 -> 32; 4*8=32 -> 64
    assert_eq!(bus.master, 64);
    // PPU dot should be at 16 (master/4)
    assert_eq!(bus.ppu.dot(), 16);
}

#[test]
fn e03_32_dma_regs_restored() {
    let mut data_block = vec![0u8; 0x210];
    data_block[0x200] = 0xAA;
    let rom = make_rom(&data_block, Map::LoRom);
    let cart = Cart::from_bytes(&rom).unwrap();
    let mut bus = SysBus::new(cart, Region::Ntsc);

    // Set up channel 0 for transfer
    bus.dma.write(0x4300, 0x00);
    bus.dma.write(0x4301, 0x80);
    bus.dma.write(0x4302, 0x00);
    bus.dma.write(0x4303, 0x82);
    bus.dma.write(0x4304, 0x00);
    bus.dma.write(0x4305, 0x01);
    bus.dma.write(0x4306, 0x00);

    // Set channel 3 dmap to a known value (not touched by transfer)
    bus.dma.ch[3].dmap = 0xFF;

    Bus::write(&mut bus, 0x00420B, 0x01);
    Bus::read(&mut bus, 0x008000); // triggers DMA

    // After DMA, channel 0 dmap should still be what we set (not reset to default)
    assert_eq!(bus.dma.ch[0].dmap, 0x00);
    // Channel 3 untouched
    assert_eq!(bus.dma.ch[3].dmap, 0xFF);
}

#[test]
fn e03_32_mdmaen_zero() {
    let mut bus = mk(&[]);

    // Write MDMAEN=0 (no channels)
    Bus::write(&mut bus, 0x00420B, 0x00);
    assert_eq!(bus.master, 6);

    // Next cycle: no DMA should run
    Bus::read(&mut bus, 0x008000);
    // master = 6 + 8 (read) = 14; no DMA overhead
    assert_eq!(bus.master, 14);
}

#[test]
fn e03_32_two_channels_order() {
    let mut data_block = vec![0u8; 0x210];
    data_block[0x200] = 0xAA;
    data_block[0x201] = 0xBB;
    data_block[0x202] = 0xCC;
    data_block[0x203] = 0xDD;
    let rom = make_rom(&data_block, Map::LoRom);
    let cart = Cart::from_bytes(&rom).unwrap();
    let mut bus = SysBus::new(cart, Region::Ntsc);

    for i in 0..0x310 {
        bus.wram.write(i, 0x55);
    }

    // Set WMADD to $7E:0300
    bus.write_inner(0x002181, 0x00);
    bus.write_inner(0x002182, 0x03);

    // Channel 1: transfer 2 bytes from ROM to WRAM
    bus.dma.write(0x4310, 0x00);
    bus.dma.write(0x4311, 0x80);
    bus.dma.write(0x4312, 0x00);
    bus.dma.write(0x4313, 0x82);
    bus.dma.write(0x4314, 0x00);
    bus.dma.write(0x4315, 0x02);
    bus.dma.write(0x4316, 0x00);

    // Channel 2: transfer 2 bytes from ROM to WRAM (continues at WMADD)
    bus.dma.write(0x4320, 0x00);
    bus.dma.write(0x4321, 0x80);
    bus.dma.write(0x4322, 0x02); // source offset +2
    bus.dma.write(0x4323, 0x82);
    bus.dma.write(0x4324, 0x00);
    bus.dma.write(0x4325, 0x02);
    bus.dma.write(0x4326, 0x00);

    // Trigger channels 1 and 2 (mask = 0x06)
    Bus::write(&mut bus, 0x00420B, 0x06);
    Bus::read(&mut bus, 0x008000); // triggers DMA

    // Both channels should have transferred in order: AA BB CC DD at $7E:0300-0303
    assert_eq!(bus.wram.read(0x300), 0xAA);
    assert_eq!(bus.wram.read(0x301), 0xBB);
    assert_eq!(bus.wram.read(0x302), 0xCC);
    assert_eq!(bus.wram.read(0x303), 0xDD);
}

#[test]
fn e03_32_no_double_run() {
    let mut data_block = vec![0u8; 0x210];
    data_block[0x200] = 0xAA;
    let rom = make_rom(&data_block, Map::LoRom);
    let cart = Cart::from_bytes(&rom).unwrap();
    let mut bus = SysBus::new(cart, Region::Ntsc);

    for i in 0..0x210 {
        bus.wram.write(i, 0x55);
    }

    bus.write_inner(0x002181, 0x00);
    bus.write_inner(0x002182, 0x02);

    bus.dma.write(0x4300, 0x00);
    bus.dma.write(0x4301, 0x80);
    bus.dma.write(0x4302, 0x00);
    bus.dma.write(0x4303, 0x82);
    bus.dma.write(0x4304, 0x00);
    bus.dma.write(0x4305, 0x01);
    bus.dma.write(0x4306, 0x00);

    Bus::write(&mut bus, 0x00420B, 0x01);
    Bus::read(&mut bus, 0x008000); // triggers DMA
    let master_after_dma = bus.master;

    // Second read should NOT trigger another DMA (pending_dma is now 0)
    Bus::read(&mut bus, 0x008001);
    // Only the read cycle overhead: +8 clocks
    assert_eq!(bus.master, master_after_dma + 8);
}

#[test]
fn e03_32_write_then_idle() {
    let mut data_block = vec![0u8; 0x210];
    data_block[0x200] = 0xAA;
    let rom = make_rom(&data_block, Map::LoRom);
    let cart = Cart::from_bytes(&rom).unwrap();
    let mut bus = SysBus::new(cart, Region::Ntsc);

    for i in 0..0x210 {
        bus.wram.write(i, 0x55);
    }

    bus.write_inner(0x002181, 0x00);
    bus.write_inner(0x002182, 0x02);

    bus.dma.write(0x4300, 0x00);
    bus.dma.write(0x4301, 0x80);
    bus.dma.write(0x4302, 0x00);
    bus.dma.write(0x4303, 0x82);
    bus.dma.write(0x4304, 0x00);
    bus.dma.write(0x4305, 0x01);
    bus.dma.write(0x4306, 0x00);

    Bus::write(&mut bus, 0x00420B, 0x01);
    assert_eq!(bus.master, 6);

    // Idle cycle should also trigger DMA in its end_cycle
    Bus::idle(&mut bus);

    // DMA completed during idle's end_cycle
    assert_eq!(bus.pending_dma(), 0);
    assert_eq!(bus.wram.read(0x200), 0xAA);
}

#[test]
fn e03_32_rewrite_before_start() {
    let mut data_block = vec![0u8; 0x210];
    data_block[0x200] = 0xAA;
    data_block[0x201] = 0xBB;
    let rom = make_rom(&data_block, Map::LoRom);
    let cart = Cart::from_bytes(&rom).unwrap();
    let mut bus = SysBus::new(cart, Region::Ntsc);

    for i in 0..0x210 {
        bus.wram.write(i, 0x55);
    }

    bus.write_inner(0x002181, 0x00);
    bus.write_inner(0x002182, 0x02);

    // Channel 0: transfer 1 byte
    bus.dma.write(0x4300, 0x00);
    bus.dma.write(0x4301, 0x80);
    bus.dma.write(0x4302, 0x00);
    bus.dma.write(0x4303, 0x82);
    bus.dma.write(0x4304, 0x00);
    bus.dma.write(0x4305, 0x01);
    bus.dma.write(0x4306, 0x00);

    // Channel 1: transfer 2 bytes (different source offset)
    bus.dma.write(0x4310, 0x00);
    bus.dma.write(0x4311, 0x80);
    bus.dma.write(0x4312, 0x01); // source +1
    bus.dma.write(0x4313, 0x82);
    bus.dma.write(0x4314, 0x00);
    bus.dma.write(0x4315, 0x02);
    bus.dma.write(0x4316, 0x00);

    // First MDMAEN write: channel 0
    Bus::write(&mut bus, 0x00420B, 0x01);
    assert_eq!(bus.master, 6);

    // Second MDMAEN write before any CPU cycle: overrides to channel 1 only
    Bus::write(&mut bus, 0x00420B, 0x02);
    assert_eq!(bus.master, 12);

    // Now trigger with a read - should run channel 1 only (mask=0x02)
    Bus::read(&mut bus, 0x008000);

    // Channel 1 transferred 2 bytes: BB at $7E:0200 and $7E:0201
    assert_eq!(bus.wram.read(0x200), 0xBB);
    assert_eq!(bus.wram.read(0x201), 0x00);
    // Channel 0 did NOT transfer (its byte AA should not be at $7E:0200)
}
