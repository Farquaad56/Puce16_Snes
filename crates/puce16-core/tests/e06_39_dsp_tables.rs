use puce16_core::apu::dsp::{gauss, tables};

#[test]
fn e06_39_period_table() {
    assert_eq!(tables::PERIOD.len(), 32);
    assert_eq!(tables::PERIOD[0], 0);
    assert_eq!(tables::PERIOD[1], 2048);
    assert_eq!(tables::PERIOD[2], 1536);
    assert_eq!(tables::PERIOD[13], 128);
    assert_eq!(tables::PERIOD[31], 1);
    for r in 1..32 {
        assert_eq!(30720 % tables::PERIOD[r] as u32, 0, "rate {}", r);
    }
}

#[test]
fn e06_39_offsets() {
    assert_eq!(tables::offset(0), 0);
    assert_eq!(tables::offset(1), 0);
    assert_eq!(tables::offset(2), 1040);
    assert_eq!(tables::offset(3), 536);
    assert_eq!(tables::offset(4), 0);
    assert_eq!(tables::offset(30), 536);
    assert_eq!(tables::offset(31), 0);
}

#[test]
fn e06_39_poll_rate0_never() {
    for counter in 0..0x7800u16 {
        assert!(!tables::poll(counter, 0));
    }
}

#[test]
fn e06_39_poll_rate31_always() {
    for counter in 0..0x7800u16 {
        assert!(tables::poll(counter, 31));
    }
}

#[test]
fn e06_39_poll_counts_full_cycle() {
    for r in 1..32u8 {
        let mut count = 0;
        for counter in 0..0x7800u16 {
            if tables::poll(counter, r) {
                count += 1;
            }
        }
        assert_eq!(count as u32, 30720 / tables::PERIOD[r as usize] as u32, "rate {}", r);
    }
}

#[test]
fn e06_39_poll_known_points() {
    // rate 2: offset=1040, period=1536; counter+1040 must be multiple of 1536
    assert!(!tables::poll(0, 2));      // 1040 % 1536 != 0
    assert!(tables::poll(496, 2));     // 496+1040=1536
    // rate 3: offset=536, period=1280; counter+536 must be multiple of 1280
    assert!(tables::poll(744, 3));     // 744+536=1280
    // rate 1: offset=0, period=2048
    assert!(tables::poll(0, 1));       // 0 % 2048 == 0
    assert!(!tables::poll(1, 1));      // 1 % 2048 != 0
}

#[test]
fn e06_39_gauss_invariants() {
    let g = &gauss::GAUSS;
    assert_eq!(g.len(), 512);
    assert_eq!(g[0], 0);
    assert_eq!(g[511], 1305);

    // non-decreasing
    for i in 1..512 {
        assert!(g[i] >= g[i - 1]);
    }

    // sum property: GAUSS[255-f] + GAUSS[511-f] + GAUSS[256+f] + GAUSS[f] in [2045, 2051]
    for f in 0..256 {
        let s = g[255 - f] + g[511 - f] + g[256 + f] + g[f];
        assert!(s >= 2045 && s <= 2051, "f={} sum={}", f, s);
    }
}
