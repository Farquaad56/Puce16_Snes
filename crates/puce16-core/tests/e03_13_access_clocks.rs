use puce16_core::bus::access_clocks;

#[test]
fn e03_13_wram_low() {
    assert_eq!(access_clocks(0x000000, false), 8);
    assert_eq!(access_clocks(0x001FFF, false), 8);
    assert_eq!(access_clocks(0x801234, false), 8);
    assert_eq!(access_clocks(0x3F0000, false), 8);
}

#[test]
fn e03_13_io_6() {
    assert_eq!(access_clocks(0x002000, false), 6);
    assert_eq!(access_clocks(0x002100, false), 6);
    assert_eq!(access_clocks(0x00213F, false), 6);
    assert_eq!(access_clocks(0x002140, false), 6);
    assert_eq!(access_clocks(0x002180, false), 6);
    assert_eq!(access_clocks(0x003FFF, false), 6);
    assert_eq!(access_clocks(0x004200, false), 6);
    assert_eq!(access_clocks(0x00420B, false), 6);
    assert_eq!(access_clocks(0x004300, false), 6);
    assert_eq!(access_clocks(0x005FFF, false), 6);
}

#[test]
fn e03_13_joyser_12() {
    assert_eq!(access_clocks(0x004000, false), 12);
    assert_eq!(access_clocks(0x004016, false), 12);
    assert_eq!(access_clocks(0x004017, false), 12);
    assert_eq!(access_clocks(0x0041FF, false), 12);
    assert_eq!(access_clocks(0x804016, false), 12);
}

#[test]
fn e03_13_sram_area() {
    assert_eq!(access_clocks(0x006000, false), 8);
    assert_eq!(access_clocks(0x007FFF, false), 8);
    assert_eq!(access_clocks(0x706000, false), 8);
}

#[test]
fn e03_13_rom_slow() {
    assert_eq!(access_clocks(0x008000, false), 8);
    assert_eq!(access_clocks(0x00FFFF, false), 8);
    assert_eq!(access_clocks(0x018000, false), 8);
    assert_eq!(access_clocks(0x400000, false), 8);
    assert_eq!(access_clocks(0x7E0000, false), 8);
    assert_eq!(access_clocks(0x7FFFFF, false), 8);
    assert_eq!(access_clocks(0xC00000, false), 8);
}

#[test]
fn e03_13_memsel() {
    // memsel=true: ROM banks >= $80 get 6 clocks
    assert_eq!(access_clocks(0x808000, true), 6);
    assert_eq!(access_clocks(0xC00000, true), 6);
    assert_eq!(access_clocks(0xFFFFFF, true), 6);
    // memsel=true: non-ROM areas unchanged
    assert_eq!(access_clocks(0x008000, true), 8);
    assert_eq!(access_clocks(0x400000, true), 8);
    assert_eq!(access_clocks(0x7E0000, true), 8);
    // IO still 6 with memsel
    assert_eq!(access_clocks(0x802100, true), 6);
    // WRAM mirror at $800000 is not ROM (A22=1 but A15=0)
    assert_eq!(access_clocks(0x800000, true), 8);
}

#[test]
fn e03_13_exhaustive_banks() {
    for bank in 0x00..=0x3F {
        let addr = (bank << 16) | 0x4016;
        assert_eq!(access_clocks(addr, false), 12);
        assert_eq!(access_clocks(addr, true), 12);
    }
    for bank in 0x80..=0xBF {
        let addr = (bank << 16) | 0x4016;
        assert_eq!(access_clocks(addr, false), 12);
        assert_eq!(access_clocks(addr, true), 12);
    }
    for bank in 0x00..=0x3F {
        let addr = (bank << 16) | 0x2100;
        assert_eq!(access_clocks(addr, false), 6);
        assert_eq!(access_clocks(addr, true), 6);
    }
    for bank in 0x80..=0xBF {
        let addr = (bank << 16) | 0x2100;
        assert_eq!(access_clocks(addr, false), 6);
        assert_eq!(access_clocks(addr, true), 6);
    }
}
