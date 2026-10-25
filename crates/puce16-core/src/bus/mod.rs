pub mod wram;
pub mod mmio;

/// CPU memory access timing in master clocks, per spec 02 section 2.
/// addr = 24-bit address (bank << 16 | offset). memsel = MEMSEL bit0 of $420B.
pub fn access_clocks(addr: u32, memsel: bool) -> u32 {
    let a = (addr & 0xFFFF) as u16;
    if addr & 0x408000 != 0 {
        // ROM area: A15 or A22 set
        return if addr & 0x800000 != 0 && memsel { 6 } else { 8 };
    }
    if (a.wrapping_add(0x6000)) & 0x4000 != 0 {
        // $0000-$1FFF and $6000-$7FFF: WRAM/SRAM area
        return 8;
    }
    if (a.wrapping_sub(0x4000)) & 0x7E00 != 0 {
        // IO area except $4000-$41FF
        return 6;
    }
    // JOYSER area $4000-$41FF
    12
}
