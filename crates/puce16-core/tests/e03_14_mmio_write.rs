use puce16_core::bus::mmio::{Mmio, MmioEffect};
use puce16_core::hash::Fnv64;

#[test]
fn e03_14_power_on() {
    let m = Mmio::new();
    assert_eq!(m.nmitimen, 0);
    assert_eq!(m.wrio, 0xFF);
    assert_eq!(m.htime, 0x1FF);
    assert_eq!(m.vtime, 0x1FF);
    assert_eq!(m.hdmaen, 0);
    assert!(!m.memsel);
    assert!(!m.rdnmi_flag);
    assert!(!m.timeup_flag);
    assert!(!m.irq_line);
    assert_eq!(m.joy, [0; 4]);
}

#[test]
fn e03_14_nmitimen() {
    let mut m = Mmio::new();
    let eff = m.write(0x4200, 0x81, 0);
    assert_eq!(eff, MmioEffect::None);
    assert_eq!(m.nmitimen, 0x81);
}

#[test]
fn e03_14_wrio_effect() {
    let mut m = Mmio::new();
    let eff = m.write(0x4201, 0x7F, 0);
    assert_eq!(eff, MmioEffect::Wrio { old: 0xFF, new: 0x7F });
    assert_eq!(m.wrio, 0x7F);
    let eff2 = m.write(0x4201, 0xFF, 0);
    assert_eq!(eff2, MmioEffect::Wrio { old: 0x7F, new: 0xFF });
}

#[test]
fn e03_14_htime() {
    let mut m = Mmio::new();
    m.write(0x4207, 0x34, 0);
    assert_eq!(m.htime, 0x134);
    m.write(0x4208, 0x00, 0);
    assert_eq!(m.htime, 0x034);
    m.write(0x4208, 0x03, 0);
    assert_eq!(m.htime, 0x134);
    m.write(0x4207, 0xFF, 0);
    assert_eq!(m.htime, 0x1FF);
}

#[test]
fn e03_14_vtime() {
    let mut m = Mmio::new();
    m.write(0x4209, 0xE0, 0);
    m.write(0x420A, 0x01, 0);
    assert_eq!(m.vtime, 0x1E0);
    m.write(0x420A, 0x00, 0);
    assert_eq!(m.vtime, 0x0E0);
}

#[test]
fn e03_14_dma_effects() {
    let mut m = Mmio::new();
    let eff = m.write(0x420B, 0x05, 0);
    assert_eq!(eff, MmioEffect::Mdmaen(5));
    let eff2 = m.write(0x420C, 0x81, 0);
    assert_eq!(eff2, MmioEffect::Hdmaen(0x81));
    assert_eq!(m.hdmaen, 0x81);
    let eff3 = m.write(0x420D, 0x01, 0);
    assert_eq!(eff3, MmioEffect::Memsel(true));
    assert!(m.memsel);
    let eff4 = m.write(0x420D, 0xFE, 0);
    assert_eq!(eff4, MmioEffect::Memsel(false));
    assert!(!m.memsel);
}

#[test]
fn e03_14_clear_irq_on_off() {
    let mut m = Mmio::new();
    m.timeup_flag = true;
    m.irq_line = true;
    m.write(0x4200, 0x80, 0);
    assert!(!m.timeup_flag);
    assert!(!m.irq_line);
    m.timeup_flag = true;
    m.irq_line = true;
    m.write(0x4200, 0x10, 0);
    assert!(m.timeup_flag);
    assert!(m.irq_line);
    m.write(0x4200, 0x31, 0);
    assert!(m.timeup_flag);
    assert!(m.irq_line);
}

#[test]
fn e03_14_other_regs_ignored() {
    let mut m = Mmio::new();
    let mut h1 = Fnv64::new();
    m.hash_into(&mut h1);
    let hash_before = h1.finish();
    assert_eq!(m.write(0x4210, 0xFF, 0), MmioEffect::None);
    assert_eq!(m.write(0x4300, 1, 0), MmioEffect::None);
    let mut h2 = Fnv64::new();
    m.hash_into(&mut h2);
    assert_eq!(h2.finish(), hash_before);
}

#[test]
fn e03_14_reset() {
    let mut m = Mmio::new();
    m.nmitimen = 0x81;
    m.hdmaen = 0xFF;
    m.memsel = true;
    m.rdnmi_flag = true;
    m.timeup_flag = true;
    m.irq_line = true;
    m.htime = 0x55;
    m.wrio = 0x12;
    m.reset();
    assert_eq!(m.nmitimen, 0);
    assert_eq!(m.hdmaen, 0);
    assert!(!m.memsel);
    assert!(!m.rdnmi_flag);
    assert!(!m.timeup_flag);
    assert!(!m.irq_line);
    assert_eq!(m.htime, 0x55);
    assert_eq!(m.wrio, 0x12);
}

#[test]
fn e03_14_hash() {
    let m1 = Mmio::new();
    let mut h1 = Fnv64::new();
    m1.hash_into(&mut h1);
    let hash1 = h1.finish();

    let m2 = Mmio::new();
    let mut h2 = Fnv64::new();
    m2.hash_into(&mut h2);
    assert_eq!(h2.finish(), hash1);

    let mut m3 = Mmio::new();
    m3.write(0x4202, 3, 0);
    let mut h3 = Fnv64::new();
    m3.hash_into(&mut h3);
    assert_ne!(h3.finish(), hash1);

    let mut m4 = Mmio::new();
    m4.write(0x4204, 1, 0);
    let mut h4 = Fnv64::new();
    m4.hash_into(&mut h4);
    assert_ne!(h4.finish(), hash1);
}
