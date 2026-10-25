use puce16_core::cart::{mapper, MapMode};

#[test]
fn e03_02_lorom_rom() {
    let m = MapMode::LoRom;
    // Some(offset) cases
    assert_eq!(mapper::rom_offset(m, 0x00, 0x8000), Some(0));
    assert_eq!(mapper::rom_offset(m, 0x00, 0xFFFF), Some(0x7FFF));
    assert_eq!(mapper::rom_offset(m, 0x01, 0x8000), Some(0x8000));
    assert_eq!(mapper::rom_offset(m, 0x3F, 0x8000), Some(0x1F8000));
    assert_eq!(mapper::rom_offset(m, 0x80, 0x8000), Some(0));
    assert_eq!(mapper::rom_offset(m, 0xBF, 0xFFFF), Some(0x1FFFFF));
    // Low-half mirror cases (addr < 0x8000)
    assert_eq!(mapper::rom_offset(m, 0x40, 0x0000), Some(0x200000));
    assert_eq!(mapper::rom_offset(m, 0xC0, 0x1234), Some(0x201234));
    assert_eq!(mapper::rom_offset(m, 0x7D, 0xFFFF), Some(0x3EFFFF));
    assert_eq!(mapper::rom_offset(m, 0xFF, 0x0000), Some(0x3F8000));
    // None cases
    assert_eq!(mapper::rom_offset(m, 0x00, 0x7FFF), None);
    assert_eq!(mapper::rom_offset(m, 0x3F, 0x0000), None);
    assert_eq!(mapper::rom_offset(m, 0x7E, 0x8000), None);
    assert_eq!(mapper::rom_offset(m, 0x7F, 0xFFFF), None);
}

#[test]
fn e03_02_hirom_rom() {
    let m = MapMode::HiRom;
    // Full bank cases (any addr)
    assert_eq!(mapper::rom_offset(m, 0xC0, 0x0000), Some(0));
    assert_eq!(mapper::rom_offset(m, 0xC0, 0xFFFF), Some(0xFFFF));
    assert_eq!(mapper::rom_offset(m, 0xC1, 0x0000), Some(0x10000));
    assert_eq!(mapper::rom_offset(m, 0xFF, 0xFFFF), Some(0x3FFFFF));
    // Mirror banks (any addr)
    assert_eq!(mapper::rom_offset(m, 0x40, 0x1234), Some(0x1234));
    assert_eq!(mapper::rom_offset(m, 0x7D, 0x0000), Some(0x3D0000));
    // High-half-only banks (addr >= 0x8000)
    assert_eq!(mapper::rom_offset(m, 0x00, 0x8000), Some(0x8000));
    assert_eq!(mapper::rom_offset(m, 0x3F, 0xFFFF), Some(0x3FFFFF));
    assert_eq!(mapper::rom_offset(m, 0x80, 0x8000), Some(0x8000));
    assert_eq!(mapper::rom_offset(m, 0xBF, 0x9000), Some(0x3F9000));
    // None cases
    assert_eq!(mapper::rom_offset(m, 0x00, 0x7FFF), None);
    assert_eq!(mapper::rom_offset(m, 0x7E, 0x0000), None);
    assert_eq!(mapper::rom_offset(m, 0x7F, 0x8000), None);
    // SRAM bank is not ROM
    assert_eq!(mapper::rom_offset(m, 0x20, 0x6000), None);
}

#[test]
fn e03_02_exhirom_rom() {
    let m = MapMode::ExHiRom;
    // Full bank cases (any addr) - lower half of ExHiROM space
    assert_eq!(mapper::rom_offset(m, 0xC0, 0x0000), Some(0));
    assert_eq!(mapper::rom_offset(m, 0xFF, 0xFFFF), Some(0x3FFFFF));
    // Upper half of ExHiROM space (bank < 0x80)
    assert_eq!(mapper::rom_offset(m, 0x40, 0x0000), Some(0x400000));
    assert_eq!(mapper::rom_offset(m, 0x7D, 0xFFFF), Some(0x7DFFFF));
    // High-half-only banks (addr >= 0x8000) - upper half
    assert_eq!(mapper::rom_offset(m, 0x00, 0x8000), Some(0x408000));
    assert_eq!(mapper::rom_offset(m, 0x3F, 0xFFFF), Some(0x7FFFFF));
    // High-half-only banks (addr >= 0x8000) - lower half
    assert_eq!(mapper::rom_offset(m, 0x80, 0x8000), Some(0x8000));
    assert_eq!(mapper::rom_offset(m, 0xBF, 0x8000), Some(0x3F8000));
    // None cases
    assert_eq!(mapper::rom_offset(m, 0x00, 0x7FFF), None);
    assert_eq!(mapper::rom_offset(m, 0x7E, 0x0000), None);
}

#[test]
fn e03_02_lorom_sram() {
    let m = MapMode::LoRom;
    // sram_len = 0x2000 (8 KiB)
    assert_eq!(mapper::sram_offset(m, 0x70, 0x0000, 0x2000), Some(0));
    assert_eq!(mapper::sram_offset(m, 0x70, 0x1234, 0x2000), Some(0x1234));
    // Wrap around at sram_len boundary
    assert_eq!(mapper::sram_offset(m, 0x70, 0x2000, 0x2000), Some(0));
    // Bank 0x71 maps to same SRAM (bank & 0x0F = 1)
    assert_eq!(mapper::sram_offset(m, 0x71, 0x0000, 0x2000), Some(0));
    // Mirror bank 0xF0
    assert_eq!(mapper::sram_offset(m, 0xF0, 0x7FFF, 0x2000), Some(0x1FFF));

    // sram_len = 0x8000 (32 KiB)
    assert_eq!(mapper::sram_offset(m, 0x71, 0x0010, 0x8000), Some(0x10));

    // sram_len = 0x20000 (128 KiB) - bank 0x73 -> offset 0x18000
    assert_eq!(mapper::sram_offset(m, 0x73, 0x0100, 0x20000), Some(0x18100));

    // None cases: addr >= 0x8000
    assert_eq!(mapper::sram_offset(m, 0x70, 0x8000, 0x2000), None);
    // None cases: wrong bank range
    assert_eq!(mapper::sram_offset(m, 0x6F, 0x0000, 0x2000), None);
    assert_eq!(mapper::sram_offset(m, 0x00, 0x6000, 0x2000), None);
    // WRAM bank is not SRAM
    assert_eq!(mapper::sram_offset(m, 0x7E, 0x0000, 0x2000), None);

    // sram_len = 0 -> always None
    assert_eq!(mapper::sram_offset(m, 0x70, 0x0000, 0), None);
}

#[test]
fn e03_02_hirom_sram() {
    let m = MapMode::HiRom;
    // sram_len = 0x2000 (8 KiB)
    assert_eq!(mapper::sram_offset(m, 0x20, 0x6000, 0x2000), Some(0));
    assert_eq!(mapper::sram_offset(m, 0x20, 0x7FFF, 0x2000), Some(0x1FFF));
    // Bank 0x21 maps to same SRAM (bank & 0x1F = 1)
    assert_eq!(mapper::sram_offset(m, 0x21, 0x6000, 0x2000), Some(0));

    // sram_len = 0x8000 (32 KiB) - bank 0x21 -> offset 0x2000
    assert_eq!(mapper::sram_offset(m, 0x21, 0x6000, 0x8000), Some(0x2000));
    // Bank 0x3F -> offset 0x7E000 (wraps to 0x7FFF with sram_len=0x8000)
    assert_eq!(mapper::sram_offset(m, 0x3F, 0x7FFF, 0x8000), Some(0x7FFF));
    // Mirror bank 0xA0
    assert_eq!(mapper::sram_offset(m, 0xA0, 0x6010, 0x8000), Some(0x10));

    // None cases: addr < 0x6000
    assert_eq!(mapper::sram_offset(m, 0x20, 0x5FFF, 0x2000), None);
    // None cases: addr > 0x7FFF
    assert_eq!(mapper::sram_offset(m, 0x20, 0x8000, 0x2000), None);
    // None cases: wrong bank range
    assert_eq!(mapper::sram_offset(m, 0x40, 0x6000, 0x2000), None);
    assert_eq!(mapper::sram_offset(m, 0x00, 0x6000, 0x2000), None);
}

#[test]
fn e03_02_exhirom_sram() {
    // ExHiROM uses the same SRAM mapping as HiROM
    let m = MapMode::ExHiRom;
    assert_eq!(mapper::sram_offset(m, 0x20, 0x6000, 0x2000), Some(0));
    assert_eq!(mapper::sram_offset(m, 0x20, 0x7FFF, 0x2000), Some(0x1FFF));
    assert_eq!(mapper::sram_offset(m, 0xA0, 0x6010, 0x8000), Some(0x10));
}

#[test]
fn e03_02_no_overlap_lorom_sram_rom() {
    // LoROM: bank 0x70 addr 0x0000 maps to BOTH ROM and SRAM.
    // rom_offset should return Some(0x380000) (the ROM formula applies).
    // sram_offset should also return Some(0) for sram_len=0x2000.
    // Cart::read (E03-06) must check SRAM before ROM to resolve this overlap.
    let m = MapMode::LoRom;
    assert_eq!(mapper::rom_offset(m, 0x70, 0x0000), Some(0x380000));
    assert_eq!(mapper::sram_offset(m, 0x70, 0x0000, 0x2000), Some(0));
}
