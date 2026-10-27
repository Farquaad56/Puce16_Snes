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
fn e03_22_repeat_free_entry() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_hdma_channel(0x00, 0x26, 0x7E, 0x2000);
    // Table: [0x03, 0xAA, 0x82, 0xBB, 0xCC, 0x00]
    bus.poke(0x7E2000, 0x03);
    bus.poke(0x7E2001, 0xAA);
    bus.poke(0x7E2002, 0x82);
    bus.poke(0x7E2003, 0xBB);
    bus.poke(0x7E2004, 0xCC);
    bus.poke(0x7E2005, 0x00);

    dma.hdma_init(&mut bus, 0x01);

    // Run 6 scanlines
    for _ in 0..6 {
        dma.hdma_run_line(&mut bus, 0x01);
    }

    let writes = bus.b_writes();
    assert_eq!(writes.len(), 3, "should have exactly 3 B-bus writes");
    assert_eq!(writes[0], (0x26, 0xAA));
    assert_eq!(writes[1], (0x26, 0xBB));
    assert_eq!(writes[2], (0x26, 0xCC));

    // After the 5th run_line call, channel should be done
    let state = dma.hdma_state(0);
    assert_eq!(state.3, true, "channel should be done after reading 0x00");
}

#[test]
fn e03_22_states_walk() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_hdma_channel(0x00, 0x26, 0x7E, 0x2000);
    // Table: [0x03, 0xAA, 0x82, 0xBB, 0xCC, 0x00]
    bus.poke(0x7E2000, 0x03);
    bus.poke(0x7E2001, 0xAA);
    bus.poke(0x7E2002, 0x82);
    bus.poke(0x7E2003, 0xBB);
    bus.poke(0x7E2004, 0xCC);
    bus.poke(0x7E2005, 0x00);

    dma.hdma_init(&mut bus, 0x01);

    // After init: a2a=0x2001, line_counter=3, do_transfer=true, done=false
    let s = dma.hdma_state(0);
    assert_eq!(s, (0x2001, 3, true, false), "state after init");

    // Run line 0: transfer AA, counter->2
    dma.hdma_run_line(&mut bus, 0x01);
    let s = dma.hdma_state(0);
    assert_eq!(s.0, 0x2002, "a2a after first run");
    assert_eq!(s.1, 2, "line_counter after first run");
    assert_eq!(s.2, false, "do_transfer=false (no repeat)");
    assert_eq!(s.3, false, "not done yet");

    // Run line 1: no transfer, counter->1
    dma.hdma_run_line(&mut bus, 0x01);
    let s = dma.hdma_state(0);
    assert_eq!(s.0, 0x2002, "a2a unchanged (no transfer)");
    assert_eq!(s.1, 1, "line_counter after second run");

    // Run line 2: counter->0, read next byte (0x82), transfer BB
    dma.hdma_run_line(&mut bus, 0x01);
    let s = dma.hdma_state(0);
    assert_eq!(s.0, 0x2003, "a2a after reading 0x82");
    assert_eq!(s.1, 2, "line_counter reset to 2 (from 0x82)");
    assert_eq!(s.2, true, "do_transfer=true (repeat flag set in 0x82)");

    // Run line 3: transfer CC, counter->1
    dma.hdma_run_line(&mut bus, 0x01);
    let s = dma.hdma_state(0);
    assert_eq!(s.0, 0x2004, "a2a after transferring CC");
    assert_eq!(s.1, 1, "line_counter after fourth run");

    // Run line 4: counter->0, read next byte (0x00), done
    dma.hdma_run_line(&mut bus, 0x01);
    let s = dma.hdma_state(0);
    assert_eq!(s.0, 0x2006, "a2a after reading 0x00");
    assert_eq!(s.1, 0, "line_counter is 0");
    assert_eq!(s.3, true, "done after reading 0x00");
}

#[test]
fn e03_22_timing_first_call() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_hdma_channel(0x00, 0x26, 0x7E, 0x2000);
    // Table: [0x03, 0xAA, 0x82, 0xBB, 0xCC, 0x00]
    bus.poke(0x7E2000, 0x03);
    bus.poke(0x7E2001, 0xAA);
    bus.poke(0x7E2002, 0x82);
    bus.poke(0x7E2003, 0xBB);
    bus.poke(0x7E2004, 0xCC);
    bus.poke(0x7E2005, 0x00);

    dma.hdma_init(&mut bus, 0x01);
    let after_init = bus.master; // 26 clocks

    // First run_line: transfer AA (1 byte) -> 18 + 8 + 8 = 34 clocks
    dma.hdma_run_line(&mut bus, 0x01);
    assert_eq!(bus.master - after_init, 34, "first run should add 34 clocks");

    // Second run_line: no transfer -> 18 + 8 = 26 clocks
    dma.hdma_run_line(&mut bus, 0x01);
    assert_eq!(bus.master - after_init, 60, "second run should add 26 more clocks");
}

#[test]
fn e03_22_event_order_run() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_hdma_channel(0x00, 0x26, 0x7E, 0x2000);
    // Table: [0x03, 0xAA, ...]
    bus.poke(0x7E2000, 0x03);
    bus.poke(0x7E2001, 0xAA);

    dma.hdma_init(&mut bus, 0x01);
    bus.ev.clear(); // Clear init events

    // First run_line: transfer AA
    dma.hdma_run_line(&mut bus, 0x01);

    // Expected: Elapse(18), Elapse(8), Elapse(8), ARead(0x7E2001), BWrite(0x26, 0xAA)
    assert_eq!(bus.ev.len(), 5);
    assert_eq!(bus.ev[0], Ev::Elapse(18));
    assert_eq!(bus.ev[1], Ev::Elapse(8));
    assert_eq!(bus.ev[2], Ev::Elapse(8));
    assert_eq!(bus.ev[3], Ev::ARead(0x7E2001));
    assert_eq!(bus.ev[4], Ev::BWrite(0x26, 0xAA));
}

#[test]
fn e03_22_pattern_p1() {
    let mut bus = FakeBus::new(0);
    // P1 pattern: 2 bytes at offsets 0 and 1
    let mut dma = make_hdma_channel(0x01, 0x18, 0x7E, 0x2000);
    // Table: [0x01, 0x11, 0x22, 0x00]
    bus.poke(0x7E2000, 0x01);
    bus.poke(0x7E2001, 0x11);
    bus.poke(0x7E2002, 0x22);
    bus.poke(0x7E2003, 0x00);

    dma.hdma_init(&mut bus, 0x01);
    let after_init = bus.master;

    // First run: transfer both bytes of P1 pattern
    dma.hdma_run_line(&mut bus, 0x01);

    let writes = bus.b_writes();
    assert_eq!(writes.len(), 2);
    assert_eq!(writes[0], (0x18, 0x11));
    assert_eq!(writes[1], (0x19, 0x22));

    // Timing: 18 + 8 + 8*2 = 42 clocks for the run
    assert_eq!(bus.master - after_init, 42);

    // After this run, counter=0 so next byte (0x00) is read -> done
    let state = dma.hdma_state(0);
    assert_eq!(state.3, true, "channel should be done");

    // Second run produces no events
    bus.ev.clear();
    dma.hdma_run_line(&mut bus, 0x01);
    assert!(bus.ev.is_empty(), "second run should produce no events");
}

#[test]
fn e03_22_indirect_repeat() {
    let mut bus = FakeBus::new(0);
    // Indirect mode with repeat flag
    let mut dma = make_hdma_channel(0x40, 0x26, 0x7E, 0x3000);
    dma.ch[0].dasb = 0x7F;

    // Table: [0x82, 0x00, 0x40] -> repeat for 2 lines, DAS=0x4000
    bus.poke(0x7E3000, 0x82);
    bus.poke(0x7E3001, 0x00);
    bus.poke(0x7E3002, 0x40);
    // Next line byte after repeat group: 0x00 (done)
    bus.poke(0x7E3003, 0x00);

    // Data at DAS: [0x55, 0x66]
    bus.poke(0x7F4000, 0x55);
    bus.poke(0x7F4001, 0x66);

    dma.hdma_init(&mut bus, 0x01);

    let state = dma.hdma_state(0);
    assert_eq!(state.0, 0x3003, "a2a after init");
    assert_eq!(dma.ch[0].das, 0x4000, "DAS should be loaded");

    // First run: transfer 0x55 (repeat flag set)
    dma.hdma_run_line(&mut bus, 0x01);
    let writes = bus.b_writes();
    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0], (0x26, 0x55));

    // DAS should have incremented
    assert_eq!(dma.ch[0].das, 0x4001, "DAS incremented after transfer");

    // Second run: transfer 0x66, then counter=0 -> read next byte (0x00) -> done
    dma.hdma_run_line(&mut bus, 0x01);
    let writes = bus.b_writes();
    assert_eq!(writes.len(), 2);
    assert_eq!(writes[1], (0x26, 0x66));

    let state = dma.hdma_state(0);
    assert_eq!(state.3, true, "channel should be done after reading 0x00");
}

#[test]
fn e03_22_indirect_non_repeat() {
    let mut bus = FakeBus::new(0);
    // Indirect mode without repeat flag
    let mut dma = make_hdma_channel(0x40, 0x26, 0x7E, 0x3000);
    dma.ch[0].dasb = 0x7F;

    // Table: [0x02, 0x00, 0x40] -> no repeat for 2 lines, DAS=0x4000
    bus.poke(0x7E3000, 0x02);
    bus.poke(0x7E3001, 0x00);
    bus.poke(0x7E3002, 0x40);
    // Next line byte after group: 0x00 (done)
    bus.poke(0x7E3003, 0x00);

    // Data at DAS: [0x55]
    bus.poke(0x7F4000, 0x55);

    dma.hdma_init(&mut bus, 0x01);

    // First run: transfer 0x55 (do_transfer=true from init)
    dma.hdma_run_line(&mut bus, 0x01);
    let writes = bus.b_writes();
    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0], (0x26, 0x55));

    // Second run: no transfer (no repeat flag), counter=0 -> read next byte (0x00) -> done
    dma.hdma_run_line(&mut bus, 0x01);
    let writes = bus.b_writes();
    assert_eq!(writes.len(), 1, "second run should not transfer");

    let state = dma.hdma_state(0);
    assert_eq!(state.3, true, "channel should be done after reading 0x00");
}

#[test]
fn e03_22_line_count_128() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_hdma_channel(0x00, 0x26, 0x7E, 0x2000);

    // Line byte $80 means 128 lines (repeat flag set)
    bus.poke(0x7E2000, 0x80);
    // Provide data for all 128 transfers
    for i in 0..128 {
        bus.poke(0x7E2001 + i as u32, 0xAA);
    }
    // Next line byte after the group: 0x00 (done)
    bus.poke(0x7E2001 + 128, 0x00);

    dma.hdma_init(&mut bus, 0x01);

    let state = dma.hdma_state(0);
    assert_eq!(state.1, 128, "line_counter should be 128 for $80");

    // Run 127 lines (counter goes from 128 to 1)
    for _ in 0..127 {
        dma.hdma_run_line(&mut bus, 0x01);
    }

    let state = dma.hdma_state(0);
    assert_eq!(state.1, 1, "line_counter should be 1 after 127 runs");

    // 128th run: counter=0 -> read next byte from table (at a2a which is now past the data)
    dma.hdma_run_line(&mut bus, 0x01);

    let state = dma.hdma_state(0);
    assert_eq!(state.3, true, "channel should be done after reading next table entry");
}

#[test]
fn e03_22_mask_excludes() {
    let mut bus = FakeBus::new(0);
    let mut dma = make_hdma_channel(0x00, 0x26, 0x7E, 0x2000);

    // Table: [0x03, 0xAA]
    bus.poke(0x7E2000, 0x03);
    bus.poke(0x7E2001, 0xAA);

    dma.hdma_init(&mut bus, 0x01);

    // Run with mask that doesn't include channel 0
    bus.ev.clear();
    dma.hdma_run_line(&mut bus, 0x02);
    assert!(bus.ev.is_empty(), "channel not in mask should produce no events");

    // Now mark channel as done and try again
    let mut bus2 = FakeBus::new(0);
    let mut dma2 = make_hdma_channel(0x00, 0x26, 0x7E, 0x2000);
    bus2.poke(0x7E2000, 0x00); // Done immediately
    dma2.hdma_init(&mut bus2, 0x01);

    bus2.ev.clear();
    dma2.hdma_run_line(&mut bus2, 0x01);
    assert!(bus2.ev.is_empty(), "done channel should produce no events");
}

#[test]
fn e03_22_two_channels() {
    let mut bus = FakeBus::new(0);
    // Two channels with P0 pattern (1 byte each)
    let mut dma = Dma::new();

    // Channel 0
    dma.ch[0].dmap = 0x00;
    dma.ch[0].bbad = 0x26;
    dma.ch[0].a1b = 0x7E;
    dma.ch[0].a1t = 0x2000;

    // Channel 1
    dma.ch[1].dmap = 0x00;
    dma.ch[1].bbad = 0x36;
    dma.ch[1].a1b = 0x7E;
    dma.ch[1].a1t = 0x4000;

    // Tables: each has [0x01, data]
    bus.poke(0x7E2000, 0x01);
    bus.poke(0x7E2001, 0xAA);
    bus.poke(0x7E4000, 0x01);
    bus.poke(0x7E4001, 0xBB);

    dma.hdma_init(&mut bus, 0x03);
    let after_init = bus.master;

    // First run: both channels transfer
    dma.hdma_run_line(&mut bus, 0x03);

    // Timing: 18 (once) + 2*(8 channel + 8 byte) = 18 + 32 = 50 clocks
    assert_eq!(bus.master - after_init, 50, "two channels should add 50 clocks");

    let writes = bus.b_writes();
    assert_eq!(writes.len(), 2);
    assert_eq!(writes[0], (0x26, 0xAA));
    assert_eq!(writes[1], (0x36, 0xBB));
}

#[test]
fn e03_22_a2a_wrap_in_run() {
    let mut bus = FakeBus::new(0);
    // Direct mode with a2a at FFFF
    let mut dma = make_hdma_channel(0x00, 0x26, 0x7E, 0xFFFF);

    // Table: [0x01] at bank 7E offset FFFF
    bus.poke(0x7EFFFF, 0x01);
    // Data at bank 7E offset 0000 (after wrap)
    bus.poke(0x7E0000, 0xAA);

    dma.hdma_init(&mut bus, 0x01);

    let state = dma.hdma_state(0);
    assert_eq!(state.0, 0x0000, "a2a should wrap after init");

    // Run line: transfer from wrapped address
    dma.hdma_run_line(&mut bus, 0x01);

    let writes = bus.b_writes();
    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0], (0x26, 0xAA));
}
