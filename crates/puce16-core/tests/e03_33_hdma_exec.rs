#![allow(dead_code)]
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
fn e03_33_init_after_dot3() {
    // ROM with HDMA table at 0x8300: [0x01, 0xA1, 0x00]
    let mut data_block = vec![0u8; 0x310];
    data_block[0x300] = 0x01; // line byte: transfer 1 time, no repeat
    data_block[0x301] = 0xA1; // data to transfer
    data_block[0x302] = 0x00; // terminator
    let rom = make_rom(&data_block, Map::LoRom);
    let cart = Cart::from_bytes(&rom).unwrap();
    let mut bus = SysBus::new(cart, Region::Ntsc);

    // Enable HDMA channel 0
    bus.write_inner(0x00420C, 0x01);

    // Set WMADD to $7E:0300 (WRAM destination)
    bus.write_inner(0x002181, 0x00);
    bus.write_inner(0x002182, 0x03);

    // Configure HDMA channel 0: direct mode, source $8300 (ROM), dest via WMADD
    bus.dma.write(0x4300, 0x00); // dmap: A->B, increment
    bus.dma.write(0x4301, 0x80); // bbad: $2180 (WMADD port)
    bus.dma.write(0x4302, 0x00); // a1t low
    bus.dma.write(0x4303, 0x83); // a1t high -> source $8300
    bus.dma.write(0x4304, 0x00); // a1b: bank 00 (ROM)

    // Advance to dot 5 (master clock 20). HDMA init should have been triggered
    // at dot 3 during catch_up, but not executed yet.
    bus.advance(20);
    let state = bus.dma.hdma_state(0);
    // $43xA is the line counter (power-on value $FF -> derived count 127), untouched.
    assert_eq!(state, (0xFFFF, 127, false, true), "HDMA not executed before begin_cycle");

    // Now execute a CPU cycle - HDMA init runs in begin_cycle
    Bus::idle(&mut bus);

    // After hdma_init: read line byte 0x01 from table at $8300
    // a2a incremented to $8301, line_counter=1, do_transfer=true, done=false
    let state = bus.dma.hdma_state(0);
    assert_eq!(state, (0x8301, 1, true, false), "HDMA init failed");

    // Timing: 20 (advance) + 26 (hdma_init: 18 base + 8 channel) + 6 (idle) = 52
    assert_eq!(bus.master, 52);
}

#[test]
fn e03_33_run_line_writes() {
    // Same setup as init test
    let mut data_block = vec![0u8; 0x310];
    data_block[0x300] = 0x01;
    data_block[0x301] = 0xA1;
    data_block[0x302] = 0x00;
    let rom = make_rom(&data_block, Map::LoRom);
    let cart = Cart::from_bytes(&rom).unwrap();
    let mut bus = SysBus::new(cart, Region::Ntsc);

    bus.write_inner(0x00420C, 0x01);
    bus.write_inner(0x002181, 0x00);
    bus.write_inner(0x002182, 0x03);
    bus.dma.write(0x4300, 0x00);
    bus.dma.write(0x4301, 0x80);
    bus.dma.write(0x4302, 0x00);
    bus.dma.write(0x4303, 0x83);
    bus.dma.write(0x4304, 0x00);

    // Advance to dot 5 and run init
    bus.advance(20);
    Bus::idle(&mut bus);
    assert_eq!(bus.master, 52);

    // Now advance to HDMA_RUN_DOT (dot 276 = master clock 1104)
    // Task says 1101, but let's use the exact dot boundary
    bus.advance((1101 - bus.master) as u32);
    assert_eq!(bus.master, 1101);

    // Execute CPU cycle: refresh + HDMA run + idle
    Bus::idle(&mut bus);

    // Expected timing: 1101 + 40 (refresh) + 34 (hdma_run: 18+8+8) + 6 (idle) = 1181
    assert_eq!(bus.master, 1181);

    // Data should have been written to WRAM at $7E:0300
    let wram = &bus.wram;
    assert_eq!(wram.read(0x300), 0xA1, "HDMA run did not write data");
    assert_eq!(wram.addr, 0x301, "WRAM address not advanced");

    // After reading next line byte (0x00), channel should be done
    let state = bus.dma.hdma_state(0);
    assert!(state.3, "Channel not marked done after terminator");
}

#[test]
fn e03_33_done_channel_no_more() {
    // Same setup
    let mut data_block = vec![0u8; 0x310];
    data_block[0x300] = 0x01;
    data_block[0x301] = 0xA1;
    data_block[0x302] = 0x00;
    let rom = make_rom(&data_block, Map::LoRom);
    let cart = Cart::from_bytes(&rom).unwrap();
    let mut bus = SysBus::new(cart, Region::Ntsc);

    bus.write_inner(0x00420C, 0x01);
    bus.write_inner(0x002181, 0x00);
    bus.write_inner(0x002182, 0x03);
    bus.dma.write(0x4300, 0x00);
    bus.dma.write(0x4301, 0x80);
    bus.dma.write(0x4302, 0x00);
    bus.dma.write(0x4303, 0x83);
    bus.dma.write(0x4304, 0x00);

    // Run init and first line transfer (same as previous test)
    bus.advance(20);
    Bus::idle(&mut bus);
    bus.advance((1101 - bus.master) as u32);
    Bus::idle(&mut bus);
    let master_after_first_line = bus.master;

    // Advance to HDMA_RUN_DOT on line 1 (dot 276 of next scanline)
    // Line length is 341 dots = 1364 clocks
    bus.advance((1364 + 1101 - master_after_first_line) as u32);

    // Execute CPU cycle: refresh only, no HDMA (channel done)
    Bus::idle(&mut bus);

    // No additional data written
    let wram = &bus.wram;
    assert_eq!(wram.addr, 0x301, "WRAM address changed after channel done");
    let state = bus.dma.hdma_state(0);
    assert!(state.3, "Channel should still be done");

    // Timing: only refresh (40) + idle (6), no HDMA overhead
    assert_eq!(bus.master, master_after_first_line + 1364 + 1101 - master_after_first_line + 40 + 6);
}

#[test]
fn e03_33_hdmaen_zero() {
    // Same setup but don't enable HDMA
    let mut data_block = vec![0u8; 0x310];
    data_block[0x300] = 0x01;
    data_block[0x301] = 0xA1;
    data_block[0x302] = 0x00;
    let rom = make_rom(&data_block, Map::LoRom);
    let cart = Cart::from_bytes(&rom).unwrap();
    let mut bus = SysBus::new(cart, Region::Ntsc);

    // Don't write to $420C (hdmaen stays 0)
    bus.write_inner(0x002181, 0x00);
    bus.write_inner(0x002182, 0x03);
    bus.dma.write(0x4300, 0x00);
    bus.dma.write(0x4301, 0x80);
    bus.dma.write(0x4302, 0x00);
    bus.dma.write(0x4303, 0x83);
    bus.dma.write(0x4304, 0x00);

    // Fill WRAM with known value
    for i in 0..0x310 {
        bus.wram.write(i, 0x55);
    }

    // Run a full frame worth of cycles in chunks
    for _ in 0..10 {
        bus.advance(35736);
        Bus::idle(&mut bus);
    }

    // No HDMA writes should have occurred
    assert_eq!(bus.wram.read(0x300), 0x55, "HDMA wrote despite hdmaen=0");
}

#[test]
fn e03_33_init_each_frame() {
    let mut data_block = vec![0u8; 0x310];
    data_block[0x300] = 0x01;
    data_block[0x301] = 0xA1;
    data_block[0x302] = 0x00;
    let rom = make_rom(&data_block, Map::LoRom);
    let cart = Cart::from_bytes(&rom).unwrap();
    let mut bus = SysBus::new(cart, Region::Ntsc);

    bus.write_inner(0x00420C, 0x01);
    bus.write_inner(0x002181, 0x00);
    bus.write_inner(0x002182, 0x03);
    bus.dma.write(0x4300, 0x00);
    bus.dma.write(0x4301, 0x80);
    bus.dma.write(0x4302, 0x00);
    bus.dma.write(0x4303, 0x83);
    bus.dma.write(0x4304, 0x00);

    // Run first frame's init and transfer
    bus.advance(20);
    Bus::idle(&mut bus);
    bus.advance((1101 - bus.master) as u32);
    Bus::idle(&mut bus);

    // Channel should be done after first line
    let state = bus.dma.hdma_state(0);
    assert!(state.3, "Channel not done after first frame");

    // Advance to start of next frame (dot 5 of frame 1)
    // Frame length: 262 lines * 341 dots = 89342 dots = 357368 clocks
    // We're at master=1181, need to reach ~357380 (dot 3 of line 0 frame 1)
    let target = 357_380u64;
    while bus.master < target {
        let step = if target - bus.master > 10000 { 10000 } else { target - bus.master };
        bus.advance(step as u32);
        Bus::idle(&mut bus);
    }

    // Channel should be re-initialized (not done)
    let state = bus.dma.hdma_state(0);
    assert!(!state.3, "Channel not re-initialized on new frame");
}

#[test]
fn e03_33_mark_refreshed_skips_refresh() {
    // Same setup as run_line_writes
    let mut data_block = vec![0u8; 0x310];
    data_block[0x300] = 0x01;
    data_block[0x301] = 0xA1;
    data_block[0x302] = 0x00;
    let rom = make_rom(&data_block, Map::LoRom);
    let cart = Cart::from_bytes(&rom).unwrap();
    let mut bus = SysBus::new(cart, Region::Ntsc);

    bus.write_inner(0x00420C, 0x01);
    bus.write_inner(0x002181, 0x00);
    bus.write_inner(0x002182, 0x03);
    bus.dma.write(0x4300, 0x00);
    bus.dma.write(0x4301, 0x80);
    bus.dma.write(0x4302, 0x00);
    bus.dma.write(0x4303, 0x83);
    bus.dma.write(0x4304, 0x00);

    // Run init
    bus.advance(20);
    Bus::idle(&mut bus);

    // Advance to HDMA_RUN_DOT
    bus.advance((1101 - bus.master) as u32);

    // Mark as refreshed before idle (so refresh is skipped)
    bus.timing.mark_refreshed();

    // Execute CPU cycle: no refresh, just HDMA run + idle
    Bus::idle(&mut bus);

    // Expected timing: 1101 + 34 (hdma_run) + 6 (idle) = 1141 (no refresh overhead)
    assert_eq!(bus.master, 1141);
}