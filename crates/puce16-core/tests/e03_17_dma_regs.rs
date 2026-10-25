use puce16_core::dma::Dma;
use puce16_core::hash::Fnv64;

#[test]
fn e03_17_power_on() {
    let d = Dma::new();
    for n in 0..8u16 {
        for off in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0xA, 0xB, 0xF] {
            let reg = 0x4300 | (n << 4) | off;
            assert_eq!(d.read(reg), Some(0xFF));
        }
    }
    assert_eq!(d.ch[3].a1t, 0xFFFF);
    assert_eq!(d.ch[3].das, 0xFFFF);
    assert_eq!(d.ch[3].a2a, 0xFFFF);
}

#[test]
fn e03_17_unmapped() {
    let d = Dma::new();
    assert_eq!(d.read(0x430C), None);
    assert_eq!(d.read(0x430D), None);
    assert_eq!(d.read(0x437E), None);
    assert_eq!(d.read(0x4380), None);
    assert_eq!(d.read(0x42FF), None);
}

#[test]
fn e03_17_a1t_a1b() {
    let mut d = Dma::new();
    d.write(0x4312, 0x34);
    d.write(0x4313, 0x12);
    d.write(0x4314, 0x7E);
    assert_eq!(d.ch[1].a1t, 0x1234);
    assert_eq!(d.ch[1].a1b, 0x7E);
    assert_eq!(d.read(0x4312), Some(0x34));
    assert_eq!(d.read(0x4313), Some(0x12));
    assert_eq!(d.read(0x4314), Some(0x7E));
}

#[test]
fn e03_17_das() {
    let mut d = Dma::new();
    d.write(0x4325, 0x00);
    d.write(0x4326, 0x01);
    d.write(0x4327, 0x7F);
    assert_eq!(d.ch[2].das, 0x0100);
    assert_eq!(d.ch[2].dasb, 0x7F);
    assert_eq!(d.read(0x4325), Some(0x00));
    assert_eq!(d.read(0x4326), Some(0x01));
    assert_eq!(d.read(0x4327), Some(0x7F));
}

#[test]
fn e03_17_dmap_bbad() {
    let mut d = Dma::new();
    d.write(0x4300, 0x01);
    d.write(0x4301, 0x18);
    assert_eq!(d.ch[0].dmap, 1);
    assert_eq!(d.ch[0].bbad, 0x18);
}

#[test]
fn e03_17_a2a_nltr() {
    let mut d = Dma::new();
    d.write(0x4348, 0xCD);
    d.write(0x4349, 0xAB);
    d.write(0x434A, 0x85);
    assert_eq!(d.ch[4].a2a, 0xABCD);
    assert_eq!(d.ch[4].nltr, 0x85);
}

#[test]
fn e03_17_unused_shared() {
    let mut d = Dma::new();
    d.write(0x435B, 0x77);
    assert_eq!(d.read(0x435F), Some(0x77));
    d.write(0x435F, 0x11);
    assert_eq!(d.read(0x435B), Some(0x11));
    d.write(0x436C, 1);
    assert_eq!(d.read(0x436C), None);
}

#[test]
fn e03_17_hash() {
    let mut h1 = Fnv64::new();
    let mut h2 = Fnv64::new();
    let d1 = Dma::new();
    let d2 = Dma::new();
    d1.hash_into(&mut h1);
    d2.hash_into(&mut h2);
    assert_eq!(h1.finish(), h2.finish());

    let mut d3 = Dma::new();
    d3.write(0x4300, 0);
    let mut h3 = Fnv64::new();
    d3.hash_into(&mut h3);
    assert_ne!(h1.finish(), h3.finish());

    let mut d4 = Dma::new();
    d4.ch[7].das = 0x1234;
    let mut h4 = Fnv64::new();
    d4.hash_into(&mut h4);
    assert_ne!(h1.finish(), h4.finish());
}
