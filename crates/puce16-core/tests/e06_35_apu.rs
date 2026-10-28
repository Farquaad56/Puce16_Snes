use puce16_core::apu::Apu;
use puce16_core::hash::Fnv64;
use puce16_core::region::Region;

fn apu_hash(a: &Apu) -> u64 {
    let mut h = Fnv64::new();
    a.hash_into(&mut h);
    h.finish()
}

#[test]
fn e06_35_new_defaults() {
    let a = Apu::new();
    for i in 0..4 {
        assert_eq!(a.port_read(i), 0);
        assert_eq!(a.cpu_to_apu(i), 0);
    }
    assert_eq!(a.debug_pc(), 0xFFC0);
}

#[test]
fn e06_35_handshake_aa_bb() {
    let mut a = Apu::new();
    a.step_to(100_000);
    assert_eq!(a.port_read(0), 0xAA);
    assert_eq!(a.port_read(1), 0xBB);
    // Mirrors
    assert_eq!(a.port_read(4), 0xAA);
}

#[test]
fn e06_35_not_yet_before_clear_loop() {
    let mut a = Apu::new();
    a.step_to(10_000);
    // IPL still clearing RAM, handshake not yet reached
    assert_eq!(a.port_read(0), 0);
}

#[test]
fn e06_35_step_to_is_incremental() {
    let mut a = Apu::new();
    a.step_to(100_000);
    let hash_a = apu_hash(&a);

    let mut b = Apu::new();
    b.step_to(50_000);
    b.step_to(100_000);
    let hash_b = apu_hash(&b);

    assert_eq!(hash_a, hash_b);
}

#[test]
fn e06_35_step_to_backwards_noop() {
    let mut a = Apu::new();
    a.step_to(100_000);
    let hash_before = apu_hash(&a);

    a.step_to(50_000);
    let hash_after = apu_hash(&a);

    assert_eq!(hash_before, hash_after);
}

#[test]
fn e06_35_ipl_clears_low_ram() {
    let mut a = Apu::new();
    a.debug_poke(0x0010, &[0x99]);
    a.step_to(100_000);
    assert_eq!(a.debug_peek(0x0010), 0);
}

#[test]
fn e06_35_port_write_visible_to_smp() {
    let mut a = Apu::new();
    a.step_to(100_000); // handshake complete
    a.port_write(0, 0xCC);
    assert_eq!(a.cpu_to_apu(0), 0xCC);
    // port_read still shows AA until SMP responds
    assert_eq!(a.port_read(0), 0xAA);
}

#[test]
fn e06_35_set_region_pal() {
    let mut a = Apu::new();
    a.set_region(Region::Pal);
    a.step_to(100_000);
    // Handshake still works with PAL timing
    assert_eq!(a.port_read(0), 0xAA);
    assert_eq!(a.port_read(1), 0xBB);
}

#[test]
fn e06_35_reset_restarts_ipl() {
    let mut a = Apu::new();
    a.step_to(100_000); // handshake complete
    assert_eq!(a.port_read(0), 0xAA);

    a.reset();
    assert_eq!(a.port_read(0), 0);
    assert_eq!(a.debug_pc(), 0xFFC0);

    a.step_to(100_000);
    assert_eq!(a.port_read(0), 0xAA);
}

#[test]
fn e06_35_debug_dsp_write() {
    let mut a = Apu::new();
    let hash_before = apu_hash(&a);

    a.debug_dsp_write(0x0C, 0x40);
    let hash_after = apu_hash(&a);

    assert_ne!(hash_before, hash_after);
}

#[test]
fn e06_35_take_audio_empty() {
    let mut a = Apu::new();
    let audio = a.take_audio();
    assert!(audio.is_empty());
}

#[test]
fn e06_35_determinism() {
    let mut a = Apu::new();
    a.step_to(100_000);
    let hash_a = apu_hash(&a);

    let mut b = Apu::new();
    b.step_to(100_000);
    let hash_b = apu_hash(&b);

    assert_eq!(hash_a, hash_b);
}
