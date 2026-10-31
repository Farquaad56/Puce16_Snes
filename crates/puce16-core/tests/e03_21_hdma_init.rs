#[path = "common/fake_dma_bus.rs"]
mod fake_dma_bus;

use puce16_core::dma::Dma;
use fake_dma_bus::*;

fn make_hdma_channel(dmap: u8, bbad: u8, a1b: u8, a1t: u16) -> Dma {
    let mut d = Dma::new();
    d.ch[0].dmap = dmap;
    d.ch[0].bbad = bbad;
    d.ch[0].a1b = a1b;
    d.ch[0].a1t = a1t;
    d
}

#[test]
fn e03_21_direct_basic() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_hdma_channel(0x00, 0x26, 0x7E, 0x2000);
    // Table: line byte = 3 (transfer on lines 0,1,2)
    bus.poke(0x7E2000, 0x03);

    dma.hdma_init(&mut bus, 0x01);

    let state = dma.hdma_state(0);
    assert_eq!(state.0, 0x2001, "a2a should be incremented after reading line byte");
    assert_eq!(state.1, 3, "line_counter should be 3");
    assert_eq!(state.2, true, "do_transfer should be true");
    assert_eq!(state.3, false, "done should be false");
    assert_eq!(dma.ch[0].nltr, 3);

    // Timing: 18 (init) + 8 (channel) = 26 clocks
    assert_eq!(bus.master, 26);

    // No B-bus writes during init
    assert!(bus.b_writes().is_empty());
}

#[test]
fn e03_21_event_order() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_hdma_channel(0x00, 0x26, 0x7E, 0x2000);
    bus.poke(0x7E2000, 0x03);

    dma.hdma_init(&mut bus, 0x01);

    // Expected event sequence: Elapse(18), Elapse(8), ARead(table)
    assert_eq!(bus.ev.len(), 3);
    assert_eq!(bus.ev[0], Ev::Elapse(18));
    assert_eq!(bus.ev[1], Ev::Elapse(8));
    assert_eq!(bus.ev[2], Ev::ARead(0x7E2000));
}

#[test]
fn e03_21_zero_first_byte() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_hdma_channel(0x00, 0x26, 0x7E, 0x2000);
    // Line byte = 0 means channel is done for this frame
    bus.poke(0x7E2000, 0x00);

    dma.hdma_init(&mut bus, 0x01);

    let state = dma.hdma_state(0);
    assert_eq!(state.0, 0x2001, "a2a still incremented");
    assert_eq!(state.1, 0, "line_counter is 0");
    assert_eq!(state.2, false, "do_transfer is false");
    assert_eq!(state.3, true, "done is true");
}

#[test]
fn e03_21_repeat_flag_count() {
    // Test repeat flag (bit 7) and line count decoding
    let cases = vec![
        (0x85u8, 5),   // repeat + 5 lines
        (0x80u8, 128), // repeat + $80 means 128 lines
        (0x01u8, 1),   // no repeat, 1 line
        (0x7Fu8, 127), // no repeat, max without repeat flag
        (0xFFu8, 127), // repeat + $FF -> low 7 bits = 127
    ];

    for (byte, expected_count) in cases {
        let mut bus = FakeBus::new(0);
        let mut dma = make_hdma_channel(0x00, 0x26, 0x7E, 0x2000);
        bus.poke(0x7E2000, byte);

        dma.hdma_init(&mut bus, 0x01);

        let state = dma.hdma_state(0);
        assert_eq!(
            state.1, expected_count,
            "byte {:02X} should give line_counter {}", byte, expected_count
        );
        assert_eq!(dma.ch[0].nltr, byte);
    }
}

#[test]
fn e03_21_indirect() {
    let mut bus = FakeBus::new(0);
    // Indirect mode: dmap bit 6 set
    let mut dma = make_hdma_channel(0x40, 0x26, 0x7E, 0x3000);
    // Table: line byte=2, DAS lo=0x00, DAS hi=0x40 -> das=0x4000
    bus.poke(0x7E3000, 0x02);
    bus.poke(0x7E3001, 0x00);
    bus.poke(0x7E3002, 0x40);

    dma.hdma_init(&mut bus, 0x01);

    let state = dma.hdma_state(0);
    assert_eq!(state.0, 0x3003, "a2a should be at 0x3003 after reading line byte + DAS");
    assert_eq!(state.1, 2, "line_counter is 2");
    assert_eq!(state.2, true, "do_transfer is true");
    assert_eq!(state.3, false, "done is false");
    assert_eq!(dma.ch[0].das, 0x4000, "DAS should be loaded from table");

    // Verify A-bus reads: line byte + DAS lo + DAS hi
    let a_reads: Vec<u32> = bus.ev.iter()
        .filter_map(|e| if let Ev::ARead(addr) = e { Some(*addr) } else { None })
        .collect();
    assert_eq!(a_reads, vec![0x7E3000, 0x7E3001, 0x7E3002]);
}

#[test]
fn e03_21_indirect_zero_line() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_hdma_channel(0x40, 0x26, 0x7E, 0x3000);
    // Line byte = 0: done immediately, no DAS read
    bus.poke(0x7E3000, 0x00);

    dma.hdma_init(&mut bus, 0x01);

    let state = dma.hdma_state(0);
    assert_eq!(state.0, 0x3001, "a2a incremented only for line byte");
    assert_eq!(state.3, true, "done is true");

    // Only one A-bus read (line byte), no DAS reads
    let a_reads: Vec<u32> = bus.ev.iter()
        .filter_map(|e| if let Ev::ARead(addr) = e { Some(*addr) } else { None })
        .collect();
    assert_eq!(a_reads, vec![0x7E3000]);
}

#[test]
fn e03_21_mask_selects() {
    let mut bus = FakeBus::new(0);
    // Channel 0 and channel 2 active (mask = 0x05)
    let mut dma = Dma::new();
    dma.ch[0].dmap = 0x00;
    dma.ch[0].bbad = 0x26;
    dma.ch[0].a1b = 0x7E;
    dma.ch[0].a1t = 0x2000;

    dma.ch[2].dmap = 0x00;
    dma.ch[2].bbad = 0x26;
    dma.ch[2].a1b = 0x7E;
    dma.ch[2].a1t = 0x4000;

    bus.poke(0x7E2000, 0x03);
    bus.poke(0x7E4000, 0x05);

    // Channel 1 should remain at power-on defaults (done=true)
    let ch1_state_before = dma.hdma_state(1);
    assert_eq!(ch1_state_before.3, true, "channel 1 starts done");

    dma.hdma_init(&mut bus, 0x05);

    // Channel 0 and 2 should be initialized
    assert!(!dma.hdma_state(0).3, "channel 0 not done");
    assert!(!dma.hdma_state(2).3, "channel 2 not done");

    // Channel 1 is not enabled: it is not initialised (no table read, A2A untouched), but the
    // frame-start reset (hardware hdmaReset) clears the completed flag of EVERY channel and
    // the setup sets every channel's transfer flag (E09-12).
    let ch1_state_after = dma.hdma_state(1);
    assert_eq!(ch1_state_after.0, 0xFFFF, "channel 1 A2A untouched");
    assert_eq!(ch1_state_after.3, false, "channel 1 done flag cleared by the frame-start reset");

    // Timing: 18 + 8*2 channels = 34 clocks
    assert_eq!(bus.master, 34);

    // A-bus reads in channel order: ch0 then ch2
    let a_reads: Vec<u32> = bus.ev.iter()
        .filter_map(|e| if let Ev::ARead(addr) = e { Some(*addr) } else { None })
        .collect();
    assert_eq!(a_reads, vec![0x7E2000, 0x7E4000]);
}

#[test]
fn e03_21_empty_mask() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_hdma_channel(0x00, 0x26, 0x7E, 0x2000);

    dma.hdma_init(&mut bus, 0x00);

    // No events at all
    assert!(bus.ev.is_empty());
    assert_eq!(bus.master, 0);
}

#[test]
fn e03_21_a2a_wrap() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_hdma_channel(0x00, 0x26, 0x7E, 0xFFFF);
    // Line byte at bank 7E offset FFFF
    bus.poke(0x7EFFFF, 0x02);

    dma.hdma_init(&mut bus, 0x01);

    let state = dma.hdma_state(0);
    assert_eq!(state.0, 0x0000, "a2a should wrap from FFFF to 0000");
}

#[test]
fn e03_21_reinit_resets_done() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_hdma_channel(0x00, 0x26, 0x7E, 0x2000);

    // First init: line byte = 0 -> done
    bus.poke(0x7E2000, 0x00);
    dma.hdma_init(&mut bus, 0x01);
    assert!(dma.hdma_state(0).3, "channel should be done after first init");

    // Second init: line byte = 5 -> not done (re-init resets done flag)
    let mut bus2 = FakeBus::new(0);
    dma.ch[0].a1t = 0x2000; // reset a1t for second test
    bus2.poke(0x7E2000, 0x05);
    dma.hdma_init(&mut bus2, 0x01);

    let state = dma.hdma_state(0);
    assert!(!state.3, "re-init with non-zero byte should clear done");
    assert!(state.2, "do_transfer should be true after re-init");
}
