use puce16_core::apu::Apu;
use puce16_core::hash::Fnv64;

#[test]
fn e03_12_power_on() {
    let a = Apu::new();
    for i in 0..4 {
        assert_eq!(a.port_read(i), 0);
        assert_eq!(a.cpu_to_apu(i), 0);
    }
}

#[test]
fn e03_12_write_goes_to_apu() {
    let mut a = Apu::new();
    a.port_write(1, 0x55);
    assert_eq!(a.cpu_to_apu(1), 0x55);
    assert_eq!(a.port_read(1), 0);
}

#[test]
fn e03_12_mirror() {
    let mut a = Apu::new();
    a.port_write(5, 0x77);
    assert_eq!(a.cpu_to_apu(1), 0x77);

    a.debug_set_from_apu(2, 0x99);
    assert_eq!(a.port_read(6), 0x99);
    assert_eq!(a.port_read(2), 0x99);
}

#[test]
fn e03_12_reset() {
    let mut a = Apu::new();
    a.port_write(0, 1);
    a.port_write(1, 2);
    a.debug_set_from_apu(2, 3);
    a.debug_set_from_apu(3, 4);

    a.reset();

    for i in 0..4 {
        assert_eq!(a.cpu_to_apu(i), 0);
        assert_eq!(a.port_read(i), 0);
    }
}

#[test]
fn e03_12_hash() {
    let mut a = Apu::new();

    let mut h_base = Fnv64::new();
    a.hash_into(&mut h_base);
    let base = h_base.finish();

    a.port_write(0, 1);
    let mut h_to = Fnv64::new();
    a.hash_into(&mut h_to);
    assert_ne!(h_to.finish(), base);

    a.reset();
    a.debug_set_from_apu(0, 1);
    let mut h_from = Fnv64::new();
    a.hash_into(&mut h_from);
    assert_ne!(h_from.finish(), base);
}
