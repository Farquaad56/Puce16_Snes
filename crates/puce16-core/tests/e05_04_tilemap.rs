use puce16_core::ppu::bg::{decode_entry, map_entry_addr};

#[test]
fn e05_04_decode_entry() {
    let e = decode_entry(0xFFFF);
    assert_eq!(e.tile, 0x3FF);
    assert_eq!(e.pal, 7);
    assert_eq!(e.prio, 1);
    assert!(e.hflip);
    assert!(e.vflip);

    let e = decode_entry(0x2000);
    assert_eq!(e.prio, 1);
    assert_eq!(e.tile, 0);
    assert_eq!(e.pal, 0);
    assert!(!e.hflip);
    assert!(!e.vflip);

    let e = decode_entry(0x0C00);
    assert_eq!(e.pal, 3);

    let e = decode_entry(0x1400);
    assert_eq!(e.pal, 5);

    let e = decode_entry(0x03FF);
    assert_eq!(e.tile, 0x3FF);
    assert_eq!(e.prio, 0);

    let e = decode_entry(0x4001);
    assert!(e.hflip);
    assert!(!e.vflip);
    assert_eq!(e.tile, 1);

    let e = decode_entry(0x8001);
    assert!(e.vflip);
    assert!(!e.hflip);
}

#[test]
fn e05_04_map32x32() {
    // base 0x1000, x2=false, y2=false
    assert_eq!(map_entry_addr(0x1000, false, false, 3, 5), 0x10A3);
    assert_eq!(map_entry_addr(0x1000, false, false, 35, 5), 0x10A3); // tx masked to 31
    assert_eq!(map_entry_addr(0x1000, false, false, 0, 0), 0x1000);
    assert_eq!(map_entry_addr(0x1000, false, false, 31, 31), 0x13FF);
    assert_eq!(map_entry_addr(0x1000, false, false, 3, 37), 0x10A3); // ty masked to 5
}

#[test]
fn e05_04_map64x32() {
    // x2=true, y2=false
    assert_eq!(map_entry_addr(0x1000, true, false, 35, 5), 0x14A3);
    assert_eq!(map_entry_addr(0x1000, true, false, 64, 5), 0x10A0); // wraps at 64 -> tx=0
    assert_eq!(map_entry_addr(0x1000, true, false, 31, 5), 0x10BF);
    assert_eq!(map_entry_addr(0x1000, true, false, 32, 5), 0x14A0); // screen 1
}

#[test]
fn e05_04_map32x64() {
    // x2=false, y2=true
    assert_eq!(map_entry_addr(0x1000, false, true, 3, 37), 0x14A3); // screen below
    assert_eq!(map_entry_addr(0x1000, false, true, 3, 5), 0x10A3);
    assert_eq!(map_entry_addr(0x1000, false, true, 35, 37), 0x14A3); // tx masked to 31
}

#[test]
fn e05_04_map64x64() {
    // x2=true, y2=true
    assert_eq!(map_entry_addr(0x1000, true, true, 35, 37), 0x1CA3); // screen 3
    assert_eq!(map_entry_addr(0x1000, true, true, 35, 5), 0x14A3);  // screen 1
    assert_eq!(map_entry_addr(0x1000, true, true, 3, 37), 0x18A3);  // screen 2
    assert_eq!(map_entry_addr(0x1000, true, true, 63, 63), 0x1FFF);
}

#[test]
fn e05_04_addr_wraps_15_bits() {
    // base 0x7C00, x2=y2=true, (63,63) -> wraps past 0x7FFF
    assert_eq!(map_entry_addr(0x7C00, true, true, 63, 63), 0x0BFF);
}

#[test]
fn e05_04_tx_beyond_127() {
    // tx can exceed 63 (pixel position / 8 up to 127)
    assert_eq!(map_entry_addr(0, false, false, 127, 0), 31);
    assert_eq!(map_entry_addr(0, true, false, 100, 0), 0x404); // (100 & 63) = 36 -> screen 1 + offset
}
