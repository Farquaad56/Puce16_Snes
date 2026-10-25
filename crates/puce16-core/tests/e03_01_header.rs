use puce16_core::cart::{header, MapMode};
use puce16_core::region::Region;

#[test]
fn e03_01_pos() {
    assert_eq!(header::header_pos(MapMode::LoRom), 0x7FC0);
    assert_eq!(header::header_pos(MapMode::HiRom), 0xFFC0);
    assert_eq!(header::header_pos(MapMode::ExHiRom), 0x40FFC0);
}

#[test]
fn e03_01_parse_fields() {
    let mut buf = vec![0u8; 0x8000];
    // Write title "HELLO WORLD" (21 bytes, padded with spaces)
    let title_bytes: Vec<u8> = b"HELLO WORLD          ".to_vec();
    for i in 0..21 {
        buf[0x7FC0 + i] = title_bytes[i];
    }
    // Write header fields at correct offsets from 0x7FC0
    buf[0x7FD5] = 0x20; // map_byte
    buf[0x7FD6] = 0x02; // chipset
    buf[0x7FD7] = 0x0A; // rom_log2 (10)
    buf[0x7FD8] = 0x03; // ram_log2 (3)
    buf[0x7FD9] = 0x01; // country
    buf[0x7FDA] = 0x33; // dev_id
    buf[0x7FDB] = 0x02; // version
    // complement at +0x1C (u16 LE) -> 0x1234
    buf[0x7FDC] = 0x34;
    buf[0x7FDD] = 0x12;
    // checksum at +0x1E (u16 LE) -> 0xEDCB
    buf[0x7FDE] = 0xCB;
    buf[0x7FDF] = 0xED;

    let h = header::parse_header(&buf, 0x7FC0).unwrap();
    assert_eq!(h.title, "HELLO WORLD");
    assert_eq!(h.map_byte, 0x20);
    assert_eq!(h.chipset, 2);
    assert_eq!(h.rom_log2, 10);
    assert_eq!(h.ram_log2, 3);
    assert_eq!(h.country, 1);
    assert_eq!(h.dev_id, 0x33);
    assert_eq!(h.version, 2);
    assert_eq!(h.checksum, 0xEDCB);
    assert_eq!(h.complement, 0x1234);
}

#[test]
fn e03_01_parse_none() {
    let buf = vec![0u8; 0x8000];
    // Too small: pos + 0x40 > rom.len()
    assert!(header::parse_header(&buf[..0x7FFF], 0x7FC0).is_none());
    // Wrong offset: 0x7FC1 + 0x40 = 0x8001 > 0x8000
    assert!(header::parse_header(&buf, 0x7FC1).is_none());
}

#[test]
fn e03_01_title_nonascii() {
    let mut buf = vec![0u8; 0x8000];
    // Non-ASCII bytes become '?'
    buf[0x7FC0] = 0x00;
    buf[0x7FC1] = 0xFF;
    buf[0x7FC2] = 0x41; // 'A'

    let h = header::parse_header(&buf, 0x7FC0).unwrap();
    // Non-ASCII bytes become '?'; only trailing SPACES are trimmed, not '?'
    assert_eq!(h.title.len(), 21);
    assert_eq!(&h.title[0..3], "??A");

    // All zeros -> all '?' (no trailing space trim since they're '?')
    let buf2 = vec![0u8; 0x8000];
    let h2 = header::parse_header(&buf2, 0x7FC0).unwrap();
    assert_eq!(h2.title, "?????????????????????"); // 21 '?' chars
}

#[test]
fn e03_01_checksum_zero() {
    let buf = vec![0u8; 0x8000];
    // All zeros: sum of all bytes except the 4 header bytes (which are also 0)
    // plus forced complement 0xFFFF. Total = 0 + 0xFF + 0xFF = 0x1FE
    assert_eq!(header::compute_checksum(&buf, 0x7FC0), 0x01FE);
}

#[test]
fn e03_01_checksum_ignores_field() {
    let mut buf = vec![0u8; 0x8000];
    buf[0] = 1;
    buf[1] = 2;
    // Set complement/checksum fields to non-zero values - they should be ignored
    buf[0x7FDC] = 0x12;
    buf[0x7FDD] = 0x34;
    buf[0x7FDE] = 0x56;
    buf[0x7FDF] = 0x78;

    // Sum = 1 + 2 + forced complement (0xFF+0xFF) = 3 + 0x1FE = 0x201
    assert_eq!(header::compute_checksum(&buf, 0x7FC0), 0x0201);
}

#[test]
fn e03_01_region() {
    // NTSC codes
    assert_eq!(header::region_from_country(0x00), Region::Ntsc);
    assert_eq!(header::region_from_country(0x01), Region::Ntsc);
    assert_eq!(header::region_from_country(0x0D), Region::Ntsc);
    assert_eq!(header::region_from_country(0x0F), Region::Ntsc);
    assert_eq!(header::region_from_country(0x10), Region::Ntsc);

    // PAL codes
    assert_eq!(header::region_from_country(0x02), Region::Pal);
    assert_eq!(header::region_from_country(0x11), Region::Pal);
    assert_eq!(header::region_from_country(0xFF), Region::Pal);
}

#[test]
fn e03_01_sram() {
    assert_eq!(header::sram_bytes(0), 0);
    assert_eq!(header::sram_bytes(1), 2048);
    assert_eq!(header::sram_bytes(3), 8192);
    assert_eq!(header::sram_bytes(8), 262144);
    assert_eq!(header::sram_bytes(9), 0);
    assert_eq!(header::sram_bytes(255), 0);
}

#[test]
fn e03_01_chipset() {
    // No coprocessor, no battery
    let (bat, name) = header::chipset_info(0x00);
    assert_eq!((bat, name), (false, None));

    // ROM+RAM, no battery
    let (bat, name) = header::chipset_info(0x01);
    assert_eq!((bat, name), (false, None));

    // ROM+RAM+battery
    let (bat, name) = header::chipset_info(0x02);
    assert_eq!((bat, name), (true, None));

    // GSU coprocessor (high nibble 1)
    let (bat, name) = header::chipset_info(0x13);
    assert_eq!((bat, name), (false, Some("GSU")));

    // SA-1 coprocessor (high nibble 3)
    let (bat, name) = header::chipset_info(0x34);
    assert_eq!((bat, name), (false, Some("SA-1")));

    // OBC1 with battery (low=5 -> battery, high=2 -> OBC1)
    let (bat, name) = header::chipset_info(0x25);
    assert_eq!((bat, name), (true, Some("OBC1")));

    // DSP coprocessor (high nibble 0)
    let (bat, name) = header::chipset_info(0x03);
    assert_eq!((bat, name), (false, Some("DSP")));

    // Unknown high nibble with battery (low=6 -> battery, high=7 -> unknown)
    let (bat, name) = header::chipset_info(0x76);
    assert_eq!((bat, name), (true, Some("unknown")));

    // S-RTC coprocessor (high nibble 5) with battery (low=6)
    let (bat, name) = header::chipset_info(0x56);
    assert_eq!((bat, name), (true, Some("S-RTC")));
}
