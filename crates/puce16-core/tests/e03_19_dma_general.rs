#[path = "common/fake_dma_bus.rs"]
mod fake_dma_bus;

use puce16_core::dma::Dma;
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
fn e03_19_p0_basic() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_channel(0x00, 0x18, 0x7E, 0x1000, 3);
    dma.run_general(&mut bus, 0x01);

    // Source default: addr as u8 (low byte of address)
    assert_eq!(bus.b_writes(), vec![(0x18, 0x00), (0x18, 0x01), (0x18, 0x02)]);
    assert_eq!(dma.ch[0].das, 0);
    assert_eq!(dma.ch[0].a1t, 0x1003);
    // Time: align(0) + startup(8) + channel(8) + 3*byte(24) = 40
    assert_eq!(bus.master, 40);
}

#[test]
fn e03_19_event_order() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_channel(0x00, 0x18, 0x7E, 0x1000, 1);
    dma.run_general(&mut bus, 0x01);

    assert_eq!(bus.ev, vec![
        Ev::Elapse(8),   // startup
        Ev::Elapse(8),   // channel overhead
        Ev::Elapse(8),   // byte transfer time
        Ev::ARead(0x7E1000),
        Ev::BWrite(0x18, 0x00),
    ]);
}

#[test]
fn e03_19_p1() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_channel(0x01, 0x18, 0x7E, 0x2000, 4);
    dma.run_general(&mut bus, 0x01);

    // Pattern 1: [0, 1] -> regs 0x18, 0x19 alternating
    assert_eq!(bus.b_writes(), vec![
        (0x18, 0x00), (0x19, 0x01), (0x18, 0x02), (0x19, 0x03)
    ]);
}

#[test]
fn e03_19_p3() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_channel(0x03, 0x04, 0x7E, 0x2000, 4);
    dma.run_general(&mut bus, 0x01);

    // Pattern 3: [0, 0, 1, 1] -> regs 0x04, 0x04, 0x05, 0x05
    assert_eq!(bus.b_writes(), vec![
        (0x04, 0x00), (0x04, 0x01), (0x05, 0x02), (0x05, 0x03)
    ]);
}

#[test]
fn e03_19_p4_partial() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_channel(0x04, 0x18, 0x7E, 0x2000, 6);
    dma.run_general(&mut bus, 0x01);

    // Pattern 4: [0, 1, 2, 3] -> wraps at byte 5 (index 4 % 4 = 0)
    assert_eq!(bus.b_writes(), vec![
        (0x18, 0x00), (0x19, 0x01), (0x1A, 0x02), (0x1B, 0x03),
        (0x18, 0x04), (0x19, 0x05)
    ]);
}

#[test]
fn e03_19_p5_p2() {
    // Pattern 5: [0, 1, 0, 1]
    let mut bus = FakeBus::new(0);
    let mut dma = make_channel(0x05, 0x22, 0x7E, 0x2000, 4);
    dma.run_general(&mut bus, 0x01);
    assert_eq!(bus.b_writes(), vec![
        (0x22, 0x00), (0x23, 0x01), (0x22, 0x02), (0x23, 0x03)
    ]);

    // Pattern 2: [0, 0] -> same reg twice
    let mut bus2 = FakeBus::new(0);
    let mut dma2 = make_channel(0x02, 0x22, 0x7E, 0x2000, 2);
    dma2.run_general(&mut bus2, 0x01);
    assert_eq!(bus2.b_writes(), vec![(0x22, 0x00), (0x22, 0x01)]);
}

#[test]
fn e03_19_reg_wrap() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_channel(0x01, 0xFF, 0x7E, 0x2000, 2);
    dma.run_general(&mut bus, 0x01);

    // BBAD wraps: 0xFF + 0 = 0xFF, 0xFF + 1 = 0x00
    assert_eq!(bus.b_writes(), vec![(0xFF, 0x00), (0x00, 0x01)]);
}

#[test]
fn e03_19_fixed_mode() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_channel(0x08, 0x18, 0x7E, 0x1000, 3);
    dma.run_general(&mut bus, 0x01);

    // Fixed mode (dmap bit 4 set): A address doesn't change
    let a_reads: Vec<u32> = bus.ev.iter()
        .filter_map(|e| if let Ev::ARead(addr) = e { Some(*addr) } else { None })
        .collect();
    assert_eq!(a_reads, vec![0x7E1000, 0x7E1000, 0x7E1000]);

    // All values are the same (low byte of 0x7E1000 = 0)
    assert_eq!(bus.b_writes(), vec![(0x18, 0x00), (0x18, 0x00), (0x18, 0x00)]);
    assert_eq!(dma.ch[0].a1t, 0x1000);
}

#[test]
fn e03_19_decrement_mode() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_channel(0x10, 0x18, 0x7E, 0x1002, 3);
    dma.run_general(&mut bus, 0x01);

    // Decrement mode (dmap bit 5 set): A address decreases
    let a_reads: Vec<u32> = bus.ev.iter()
        .filter_map(|e| if let Ev::ARead(addr) = e { Some(*addr) } else { None })
        .collect();
    assert_eq!(a_reads, vec![0x7E1002, 0x7E1001, 0x7E1000]);

    // Values are low bytes: 2, 1, 0
    assert_eq!(bus.b_writes(), vec![(0x18, 0x02), (0x18, 0x01), (0x18, 0x00)]);
    // After 3 decrements from 0x1002: wraps to 0x0FFF
    assert_eq!(dma.ch[0].a1t, 0x0FFF);
}

#[test]
fn e03_19_a1t_wrap_in_bank() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_channel(0x00, 0x18, 0x7E, 0xFFFE, 3);
    dma.run_general(&mut bus, 0x01);

    // A address wraps within bank (bank never changes)
    let a_reads: Vec<u32> = bus.ev.iter()
        .filter_map(|e| if let Ev::ARead(addr) = e { Some(*addr) } else { None })
        .collect();
    assert_eq!(a_reads, vec![0x7EFFFE, 0x7EFFFF, 0x7E0000]);

    // After wrapping: a1t = 0x0001
    assert_eq!(dma.ch[0].a1t, 0x0001);
}

#[test]
fn e03_19_das_zero_65536() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_channel(0x00, 0x18, 0x7E, 0x0000, 0);
    dma.run_general(&mut bus, 0x01);

    // das=0 means 65536 bytes
    assert_eq!(bus.b_writes().len(), 65536);
    // Time: align(0) + startup(8) + channel(8) + 65536*byte(8) = 524304
    assert_eq!(bus.master, 524_304);
    // a1t wraps back to 0 after 65536 increments
    assert_eq!(dma.ch[0].a1t, 0x0000);
    assert_eq!(dma.ch[0].das, 0);
}

#[test]
fn e03_19_two_channels() {
    let mut bus = FakeBus::new(0);
    let mut dma = Dma::new();
    // Channel 0: das=1
    dma.ch[0].dmap = 0x00;
    dma.ch[0].bbad = 0x18;
    dma.ch[0].a1b = 0x7E;
    dma.ch[0].a1t = 0x1000;
    dma.ch[0].das = 1;
    // Channel 2: das=2
    dma.ch[2].dmap = 0x00;
    dma.ch[2].bbad = 0x20;
    dma.ch[2].a1b = 0x7E;
    dma.ch[2].a1t = 0x2000;
    dma.ch[2].das = 2;

    // Run channels 0 and 2 (mask = 0b101)
    dma.run_general(&mut bus, 0b101);

    let writes = bus.b_writes();
    // Channel 0 first (1 byte), then channel 2 (2 bytes)
    assert_eq!(writes[0], (0x18, 0x00));
    assert_eq!(writes[1], (0x20, 0x00));
    assert_eq!(writes[2], (0x20, 0x01));

    // Time: align(0) + startup(8) + ch0(8+8) + ch2(8+16) = 48
    assert_eq!(bus.master, 48);

    // Channel 1 unchanged (das stays at power-on value)
    assert_eq!(dma.ch[1].das, 0xFFFF);
}

#[test]
fn e03_19_alignment() {
    let mut bus = FakeBus::new(3);
    let mut dma = make_channel(0x00, 0x18, 0x7E, 0x1000, 1);
    dma.run_general(&mut bus, 0x01);

    // First event should be Elapse(5) to align from 3 to 8
    assert_eq!(bus.ev[0], Ev::Elapse(5));
    // Then startup(8), channel(8), byte(8)
    assert_eq!(bus.ev[1], Ev::Elapse(8));
    assert_eq!(bus.ev[2], Ev::Elapse(8));
    assert_eq!(bus.ev[3], Ev::Elapse(8));

    // Total: 3 + 5 + 24 = 32
    assert_eq!(bus.master, 32);
}

#[test]
fn e03_19_empty_mask() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_channel(0x00, 0x18, 0x7E, 0x1000, 1);
    dma.run_general(&mut bus, 0);

    // No channels run, but startup time is consumed
    assert_eq!(bus.b_writes().len(), 0);
    // Time: align(0) + startup(8) = 8
    assert_eq!(bus.master, 8);
}
