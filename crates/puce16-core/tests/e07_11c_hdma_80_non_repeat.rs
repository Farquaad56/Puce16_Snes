#[path = "common/fake_dma_bus.rs"]
mod fake_dma_bus;

use fake_dma_bus::*;
use puce16_core::dma::Dma;

fn channel() -> Dma {
    let mut d = Dma::new();
    d.ch[0].dmap = 0x00;
    d.ch[0].bbad = 0x01;
    d.ch[0].a1b = 0x7E;
    d.ch[0].a1t = 0x2000;
    d
}

/// NLTR $80 = 128 lines, ONE write (non-repeat); the next table byte is the next NLTR, not data.
/// Table: [$80, $A2, $01, $A3, $00]: write $A2 on line 0, nothing for 127 lines,
/// then NLTR $01 (non-repeat, 1 line): write $A3 on line 128, then $00 ends.
#[test]
fn e07_11c_hdma_80_non_repeat() {
    let mut bus = FakeBus::new(0);
    let mut dma = channel();
    for (i, b) in [0x80u8, 0xA2, 0x01, 0xA3, 0x00].iter().enumerate() {
        bus.poke(0x7E2000 + i as u32, *b);
    }
    dma.hdma_init(&mut bus, 0x01);
    for _ in 0..130 {
        dma.hdma_run_line(&mut bus, 0x01);
    }
    let writes = bus.b_writes();
    assert_eq!(writes.len(), 2, "exactly two writes: {:?}", writes);
    assert_eq!(writes[0], (0x01, 0xA2));
    assert_eq!(writes[1], (0x01, 0xA3));
    assert!(dma.hdma_state(0).3, "channel done after the $00 terminator");
}

/// $81 stays repeat mode with a count of 1 line.
#[test]
fn e07_11c_hdma_81_is_repeat() {
    let mut bus = FakeBus::new(0);
    let mut dma = channel();
    for (i, b) in [0x82u8, 0x11, 0x22, 0x00].iter().enumerate() {
        bus.poke(0x7E2000 + i as u32, *b);
    }
    dma.hdma_init(&mut bus, 0x01);
    for _ in 0..4 {
        dma.hdma_run_line(&mut bus, 0x01);
    }
    let writes = bus.b_writes();
    assert_eq!(writes, vec![(0x01, 0x11), (0x01, 0x22)]);
}
