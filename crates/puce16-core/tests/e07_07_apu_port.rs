mod common;
use common::*;
use puce16_core::Snes;

#[test]
fn e07_07_handshake_visible() {
    let rom = make_rom(&[0x80, 0xFE], Map::LoRom);
    let mut snes = Snes::new(&rom, None).unwrap();
    for _ in 0..3 {
        snes.run_frame();
    }
    assert_eq!(snes.apu_port(0), 0xAA);
    assert_eq!(snes.apu_port(1), 0xBB);
    assert_eq!(snes.apu_port(2), 0);
    assert_eq!(snes.apu_port(3), 0);
}

#[test]
fn e07_07_before_boot() {
    let rom = make_rom(&[0x80, 0xFE], Map::LoRom);
    let snes = Snes::new(&rom, None).unwrap();
    assert_eq!(snes.apu_port(0), 0);
    assert_eq!(snes.apu_port(1), 0);
    assert_eq!(snes.apu_port(2), 0);
    assert_eq!(snes.apu_port(3), 0);
}

#[test]
fn e07_07_index_masked() {
    let rom = make_rom(&[0x80, 0xFE], Map::LoRom);
    let mut snes = Snes::new(&rom, None).unwrap();
    for _ in 0..3 {
        snes.run_frame();
    }
    assert_eq!(snes.apu_port(4), snes.apu_port(0));
}
