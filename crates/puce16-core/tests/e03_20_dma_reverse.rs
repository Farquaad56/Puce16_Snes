#[path = "common/fake_dma_bus.rs"]
mod fake_dma_bus;

use puce16_core::dma::{Dma, DmaBus};
use fake_dma_bus::*;

fn make_channel(dmap: u8, bbad: u8, a1b: u8, a1t: u16, das: u16) -> Dma {
    let mut d = Dma::new();
    d.ch[0].dmap = dmap;
    d.ch[0].bbad = bbad;
    d.ch[0].a1b = a1b;
    d.ch[0].a1t = a1t;
    d.ch[0].das = das;
    d
}

#[test]
fn e03_20_b_to_a() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_channel(0x80, 0x39, 0x7E, 0x3000, 2);
    dma.run_general(&mut bus, 0x01);

    // B->A direction: read from B-bus (reg 0x39), write to A-bus
    assert_eq!(bus.a_writes(), vec![(0x7E3000, 1), (0x7E3001, 2)]);
    assert_eq!(bus.count_b_reads(), 2);
    assert_eq!(dma.ch[0].a1t, 0x3002);
    assert_eq!(dma.ch[0].das, 0);
}

#[test]
fn e03_20_b_to_a_p1() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_channel(0x81, 0x39, 0x7E, 0x3000, 4);
    dma.run_general(&mut bus, 0x01);

    // Pattern 1: [0, 1] -> regs 0x39, 0x3A alternating
    let b_reads: Vec<u8> = bus.ev.iter()
        .filter_map(|e| if let Ev::BRead(reg) = e { Some(*reg) } else { None })
        .collect();
    assert_eq!(b_reads, vec![0x39, 0x3A, 0x39, 0x3A]);

    // Values written to A: 1, 2, 3, 4 (incrementing b_counter)
    let a_writes = bus.a_writes();
    assert_eq!(a_writes.len(), 4);
    assert_eq!(a_writes[0], (0x7E3000, 1));
    assert_eq!(a_writes[1], (0x7E3001, 2));
    assert_eq!(a_writes[2], (0x7E3002, 3));
    assert_eq!(a_writes[3], (0x7E3003, 4));
}

#[test]
fn e03_20_b_to_a_decrement() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_channel(0x90, 0x39, 0x7E, 0x3001, 2);
    dma.run_general(&mut bus, 0x01);

    // Decrement mode: writes to A at 0x7E3001 then 0x7E3000
    let a_writes = bus.a_writes();
    assert_eq!(a_writes[0], (0x7E3001, 1));
    assert_eq!(a_writes[1], (0x7E3000, 2));
}

#[test]
fn e03_20_wram_skip_a_to_b() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_channel(0x00, 0x80, 0x7E, 0x1000, 3);
    dma.run_general(&mut bus, 0x01);

    // WRAM source + reg 0x80 (WMADD) -> skip all transfers
    assert_eq!(bus.b_writes().len(), 0);
    let a_reads: Vec<u32> = bus.ev.iter()
        .filter_map(|e| if let Ev::ARead(addr) = e { Some(*addr) } else { None })
        .collect();
    assert_eq!(a_reads.len(), 0);

    // But time passes and counters advance
    assert_eq!(dma.ch[0].a1t, 0x1003);
    assert_eq!(bus.master, 40); // align(0) + startup(8) + channel(8) + 3*byte(24)
}

#[test]
fn e03_20_wram_skip_mirror() {
    // Mirror WRAM: bank 00, addr < 0x2000
    let mut bus = FakeBus::new(0);
    let mut dma = make_channel(0x00, 0x80, 0x00, 0x0100, 3);
    dma.run_general(&mut bus, 0x01);
    assert_eq!(bus.b_writes().len(), 0);

    // Mirror WRAM: bank 01, addr < 0x2000
    let mut bus2 = FakeBus::new(0);
    let mut dma2 = make_channel(0x00, 0x80, 0x01, 0x1000, 3);
    dma2.run_general(&mut bus2, 0x01);
    assert_eq!(bus2.b_writes().len(), 0);
}

#[test]
fn e03_20_rom_not_skipped() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_channel(0x00, 0x80, 0xC0, 0x8000, 3);
    dma.run_general(&mut bus, 0x01);

    // ROM source (bank C0) is not WRAM -> transfers happen
    assert_eq!(bus.b_writes().len(), 3);
    for w in &bus.b_writes() {
        assert_eq!(w.0, 0x80);
    }
}

#[test]
fn e03_20_wram_skip_b_to_a() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_channel(0x80, 0x80, 0x7E, 0x1000, 2);
    dma.run_general(&mut bus, 0x01);

    // B->A direction: WRAM destination + reg 0x80 -> skip all transfers
    assert_eq!(bus.count_b_reads(), 0);
    assert_eq!(bus.a_writes().len(), 0);

    // Time passes and counters advance
    assert_eq!(dma.ch[0].a1t, 0x1002);
    assert_eq!(bus.master, 32); // align(0) + startup(8) + channel(8) + 2*byte(16)
}

#[test]
fn e03_20_skip_per_transfer_reg() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_channel(0x01, 0x7F, 0x7E, 0x1000, 4);
    dma.run_general(&mut bus, 0x01);

    // Pattern 1: [0, 1] -> regs 0x7F, 0x80 alternating
    // Transfers to reg 0x80 are skipped (WRAM source), only 0x7F writes happen
    let b_writes = bus.b_writes();
    assert_eq!(b_writes.len(), 2);
    for w in &b_writes {
        assert_eq!(w.0, 0x7F);
    }

    // All 4 transfers advance the address counter
    assert_eq!(dma.ch[0].a1t, 0x1004);
}

#[test]
fn e03_20_a_to_b_regression() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_channel(0x00, 0x18, 0x7E, 0x1000, 3);
    dma.run_general(&mut bus, 0x01);

    // Same as E03-19 p0_basic: A->B direction still works
    assert_eq!(bus.b_writes(), vec![(0x18, 0x00), (0x18, 0x01), (0x18, 0x02)]);
    assert_eq!(dma.ch[0].das, 0);
    assert_eq!(dma.ch[0].a1t, 0x1003);
    assert_eq!(bus.master, 40);
}
