use puce16_core::cart::{CartError, MapMode};
use puce16_core::region::Region;
use puce16_core::snes::{LoadError, Snes};

mod common;
use common::{make_rom, make_rom_with, Map};

#[test]
fn e03_34_new_ok() {
    let rom = make_rom(&[0xEA], Map::LoRom);
    let s = Snes::new(&rom, None).expect("should load");
    assert_eq!(s.region(), Region::Ntsc);
}

#[test]
fn e03_34_region_from_header() {
    // Country 2 -> PAL
    let rom_pal = make_rom_with(&[0xEA], Map::LoRom, 15, 0, 0x00, 2);
    let s = Snes::new(&rom_pal, None).expect("should load");
    assert_eq!(s.region(), Region::Pal);

    // Forced region overrides header (country 1 -> NTSC, forced to PAL)
    let rom_ntsc = make_rom_with(&[0xEA], Map::LoRom, 15, 0, 0x00, 1);
    let s2 = Snes::new(&rom_ntsc, Some(Region::Pal)).expect("should load");
    assert_eq!(s2.region(), Region::Pal);

    // Header says NTSC (country 1), forced to PAL
    let rom_forced = make_rom_with(&[0xEA], Map::LoRom, 15, 0, 0x00, 1);
    let s3 = Snes::new(&rom_forced, Some(Region::Pal)).expect("should load");
    assert_eq!(s3.region(), Region::Pal);
}

#[test]
fn e03_34_errors() {
    // Too small (less than 32 KiB)
    let too_small = vec![0u8; 100];
    let err = Snes::new(&too_small, None).err().expect("should fail");
    assert_eq!(err, LoadError::Cart(CartError::TooSmall));

    // Unsupported coprocessor (chipset 0x13 -> GSU)
    let gsu_rom = make_rom_with(&[0xEA], Map::LoRom, 15, 0, 0x13, 1);
    let err2 = Snes::new(&gsu_rom, None).err().expect("should fail");
    assert_eq!(err2, LoadError::Cart(CartError::UnsupportedCoprocessor("GSU")));
}

#[test]
fn e03_34_forced_map() {
    let rom = make_rom(&[0xEA], Map::LoRom);
    // Force HiRom on a 32 KiB LoRom ROM -> no header at 0xFFC0
    let err = Snes::new_with(&rom, None, Some(MapMode::HiRom)).err().expect("should fail");
    assert_eq!(err, LoadError::Cart(CartError::NoHeader));
}

#[test]
fn e03_34_pattern_still_works() {
    let mut s = Snes::new_test_pattern(Region::Ntsc);
    s.run_frame(); // f = 1
    assert_eq!(s.frame().pixels[0], 0xFF01_0040);

    s.reset();
    assert_eq!(s.frame_count(), 0);
}

#[test]
fn e03_34_existing_e01_06_tests() {
    // Marker: tests/e01_06_snes_test_pattern.rs must stay green unmodified.
    let s = puce16_core::snes::Snes::new_test_pattern(puce16_core::Region::Ntsc);
    assert_eq!(s.frame_count(), 0);
}