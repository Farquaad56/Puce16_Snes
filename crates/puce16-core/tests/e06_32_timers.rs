use puce16_core::apu::timer::Timer;
use puce16_core::hash::Fnv64;

#[test]
fn e06_32_period_timer2() {
    let mut t = Timer::new(16);
    t.set_target(3);
    t.set_enable(true);
    for _ in 0..47 { t.tick(); }
    assert_eq!(t.out(), 0);
    t.tick();
    assert_eq!(t.out(), 1);
    assert_eq!(t.stage1(), 0);
    for _ in 0..47 { t.tick(); }
    assert_eq!(t.out(), 1);
    t.tick();
    assert_eq!(t.out(), 2);
}

#[test]
fn e06_32_read_clears() {
    let mut t = Timer::new(16);
    t.set_target(3);
    t.set_enable(true);
    for _ in 0..48 { t.tick(); }
    assert_eq!(t.read_out(), 1);
    assert_eq!(t.read_out(), 0);
}

#[test]
fn e06_32_target_zero_is_256() {
    let mut t = Timer::new(16);
    t.set_target(0);
    t.set_enable(true);
    for _ in 0..16*255 { t.tick(); }
    assert_eq!(t.out(), 0);
    for _ in 0..16 { t.tick(); }
    assert_eq!(t.out(), 1);
}

#[test]
fn e06_32_disabled_runs_stage0_only() {
    let mut t = Timer::new(16);
    for _ in 0..20 { t.tick(); }
    assert_eq!(t.stage0(), 4);
    assert_eq!(t.stage1(), 0);
    assert_eq!(t.out(), 0);
}

#[test]
fn e06_32_enable_midway() {
    let mut t = Timer::new(16);
    t.set_target(1);
    for _ in 0..8 { t.tick(); }
    t.set_enable(true);
    for _ in 0..8 { t.tick(); }
    assert_eq!(t.out(), 1);
}

#[test]
fn e06_32_enable_transition_resets() {
    let mut t = Timer::new(16);
    t.set_target(1);
    t.set_enable(true);
    for _ in 0..16 { t.tick(); }
    assert_eq!(t.out(), 1);
    t.set_enable(false);
    t.set_enable(true);
    assert_eq!(t.out(), 0);
    assert_eq!(t.stage1(), 0);
    let mut u = Timer::new(16);
    u.set_target(1);
    u.set_enable(true);
    for _ in 0..16 { u.tick(); }
    u.set_enable(true);
    assert_eq!(u.out(), 1);
}

#[test]
fn e06_32_out_wraps_4_bits() {
    let mut t = Timer::new(1);
    t.set_target(1);
    t.set_enable(true);
    for _ in 0..15 { t.tick(); }
    assert_eq!(t.out(), 15);
    t.tick();
    assert_eq!(t.out(), 0);
}

#[test]
fn e06_32_divider_128() {
    let mut t = Timer::new(128);
    t.set_target(1);
    t.set_enable(true);
    for _ in 0..127 { t.tick(); }
    assert_eq!(t.out(), 0);
    t.tick();
    assert_eq!(t.out(), 1);
}

#[test]
fn e06_32_hash() {
    let mut a = Timer::new(16);
    let mut b = Timer::new(16);
    let mut ha = Fnv64::new();
    let mut hb = Fnv64::new();
    a.hash_into(&mut ha);
    b.hash_into(&mut hb);
    assert_eq!(ha.finish(), hb.finish());
    a.tick();
    let mut hc = Fnv64::new();
    a.hash_into(&mut hc);
    assert_ne!(hc.finish(), hb.finish());
}
