use puce16_core::coprocessor::sdd1::Sdd1;

#[test]
fn e09_05_reset_values() {
    let s = Sdd1::new();
    assert_eq!(s.io_read(0x4800), Some(0));
    assert_eq!(s.io_read(0x4801), Some(0));
    assert_eq!(s.io_read(0x4804), Some(0));
    assert_eq!(s.io_read(0x4805), Some(1));
    assert_eq!(s.io_read(0x4806), Some(2));
    assert_eq!(s.io_read(0x4807), Some(3));
    // Unused registers return None (bus open)
    assert_eq!(s.io_read(0x4802), None);
    assert_eq!(s.io_read(0x4803), None);
    assert_eq!(s.io_read(0x4808), None);
    assert_eq!(s.io_read(0x480F), None);
}

#[test]
fn e09_05_bank_registers_mask_8f() {
    let mut s = Sdd1::new();
    // Bank registers mask to 0x8F on write
    s.io_write(0x4804, 0xFF);
    assert_eq!(s.io_read(0x4804), Some(0x8F));

    // DMA activation registers accept full byte
    s.io_write(0x4800, 0xFF);
    s.io_write(0x4801, 0xA5);
    assert_eq!(s.io_read(0x4800), Some(0xFF));
    assert_eq!(s.io_read(0x4801), Some(0xA5));
}

#[test]
fn e09_05_mmc_offset_defaults() {
    let s = Sdd1::new();
    // Default MMC banks: [0, 1, 2, 3]
    assert_eq!(s.mmc_offset(0xC00000), 0x000000);
    assert_eq!(s.mmc_offset(0xD01234), 0x101234);
    assert_eq!(s.mmc_offset(0xE0ABCD), 0x20ABCD);
    assert_eq!(s.mmc_offset(0xFFFFFF), 0x3FFFFF);

    // After changing bank register
    let mut s = Sdd1::new();
    s.io_write(0x4804, 0x03);
    assert_eq!(s.mmc_offset(0xC00010), 0x300010);

    // Bit 7 of bank register is ignored by MMC window
    let mut s = Sdd1::new();
    s.io_write(0x4804, 0x8F);
    assert_eq!(s.mmc_offset(0xC00000), 0xF00000);
}

#[test]
fn e09_05_lorom_offset_values() {
    let s = Sdd1::new();
    // Default: no bit 21 clearing
    assert_eq!(s.lorom_offset(0x008000), 0x000000);
    assert_eq!(s.lorom_offset(0x018004), 0x008004);
    assert_eq!(s.lorom_offset(0x3F8000), 0x1F8000);
    assert_eq!(s.lorom_offset(0x3FFFFF), 0x1FFFFF);
    assert_eq!(s.lorom_offset(0x808000), 0x000000);
    assert_eq!(s.lorom_offset(0xBFFFFF), 0x1FFFFF);
    // Banks 20-3F and A0-BF map to same offset without bit 21 clearing
    assert_eq!(s.lorom_offset(0x208000), 0x100000);
    assert_eq!(s.lorom_offset(0xA08000), 0x100000);
}

#[test]
fn e09_05_lorom_offset_bit21_cleared() {
    let mut s = Sdd1::new();
    // Enable bit 21 clearing for banks 20-3F (via r4805) and A0-BF (via r4807)
    s.io_write(0x4805, 0x80);
    s.io_write(0x4807, 0x83);

    // Banks 20-3F: bit 21 cleared -> maps to lower half
    assert_eq!(s.lorom_offset(0x208000), 0x000000);
    assert_eq!(s.lorom_offset(0xA08000), 0x000000);
    assert_eq!(s.lorom_offset(0x3F8000), 0x0F8000);
    assert_eq!(s.lorom_offset(0xBFFFFF), 0x0FFFFF);

    // Banks < 20 unchanged
    assert_eq!(s.lorom_offset(0x018004), 0x008004);
}