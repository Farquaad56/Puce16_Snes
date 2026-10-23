use puce16_core::consts::*;
use puce16_core::Region;

#[test]
fn e01_03_master_hz() {
    assert_eq!(Region::Ntsc.master_hz(), 21_477_272);
    assert_eq!(Region::Pal.master_hz(), 21_281_370);
}

#[test]
fn e01_03_last_line() {
    assert_eq!(Region::Ntsc.last_line(), 261);
    assert_eq!(Region::Pal.last_line(), 311);
    assert!(!Region::Ntsc.is_pal());
    assert!(Region::Pal.is_pal());
}

#[test]
fn e01_03_line_sums() {
    assert_eq!(CLOCKS_SHORT_LINE, 1360);
    assert_eq!(CLOCKS_PER_LINE, 1364);
    assert_eq!(CLOCKS_LONG_LINE, 1368);
    assert_eq!((LINES_NTSC as u64) * (CLOCKS_PER_LINE as u64), 357_368);
    assert_eq!((LINES_PAL as u64) * (CLOCKS_PER_LINE as u64), 425_568);
}

#[test]
fn e01_03_visible() {
    assert_eq!(VISIBLE_CLOCK_FIRST, 88);
    assert_eq!(VISIBLE_DOT_FIRST * 4, VISIBLE_CLOCK_FIRST);
    assert_eq!(VISIBLE_CLOCK_END - VISIBLE_CLOCK_FIRST, 1024);
}
