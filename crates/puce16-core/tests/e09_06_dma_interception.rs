//! E09-06: S-DD1 DMA interception and on-the-fly decompression tests.

use puce16_core::coprocessor::sdd1::regs::Sdd1;

/// LCG-based test stream generator (matches docs/tools/sdd1_ref.py).
fn lcg_bytes(seed: u32, n: usize) -> Vec<u8> {
    let mut x = seed;
    (0..n)
        .map(|_| {
            x = x.wrapping_mul(1664525).wrapping_add(1013904223);
            (x >> 24) as u8
        })
        .collect()
}

/// Build a test ROM with compressed data at offset 0x100.
fn build_rom(k: u8) -> Vec<u8> {
    let mut rom = vec![0u8; 1 << 20];
    let mut data = lcg_bytes(0x1000 + k as u32, 4096);
    // Embed header byte in high nibble of first byte
    data[0] = (data[0] & 0x0F) | (k << 4);
    rom[0x100..0x100 + 4096].copy_from_slice(&data);
    rom
}

#[test]
fn e09_06_stream_matches_golden_header_00() {
    let mut s = Sdd1::new();
    let rom = build_rom(0);

    // Enable channel 0, set DMA address to 0xC00100, size 4
    s.io_write(0x4800, 1);
    s.io_write(0x4801, 1);
    s.dma_write(0x4302, 0x00);
    s.dma_write(0x4303, 0x01);
    s.dma_write(0x4304, 0xC0);
    s.dma_write(0x4305, 4);
    s.dma_write(0x4306, 0);

    // Read 4 decompressed bytes - expect golden values for header 0x00
    let b0 = s.mcu_read(0xC00100, &rom);
    let b1 = s.mcu_read(0xC00100, &rom);
    let b2 = s.mcu_read(0xC00100, &rom);
    let b3 = s.mcu_read(0xC00100, &rom);

    assert_eq!(b0, 0x75, "byte 0 mismatch");
    assert_eq!(b1, 0x12, "byte 1 mismatch");
    assert_eq!(b2, 0x24, "byte 2 mismatch");
    assert_eq!(b3, 0x91, "byte 3 mismatch");

    // Channel should be done: r4801 bit 0 cleared
    let status = s.io_read(0x4801).unwrap();
    assert_eq!(status, 0, "channel 0 not finished");
}

#[test]
fn e09_06_after_finish_reads_plain_rom() {
    let mut s = Sdd1::new();
    let rom = build_rom(0);

    // Same setup as previous test
    s.io_write(0x4800, 1);
    s.io_write(0x4801, 1);
    s.dma_write(0x4302, 0x00);
    s.dma_write(0x4303, 0x01);
    s.dma_write(0x4304, 0xC0);
    s.dma_write(0x4305, 4);
    s.dma_write(0x4306, 0);

    // Consume all 4 bytes
    for _ in 0..4 {
        s.mcu_read(0xC00100, &rom);
    }

    // After finish, reads should return plain ROM (compressed header byte)
    let b = s.mcu_read(0xC00100, &rom);
    assert_eq!(b, rom[0x100], "should read plain ROM after DMA completes");
}

#[test]
fn e09_06_stream_matches_golden_header_30() {
    let mut s = Sdd1::new();
    let rom = build_rom(3); // header 0x30

    // Enable channel 2, address 0xC00100, size 8
    s.io_write(0x4800, 0x04);
    s.io_write(0x4801, 0x04);
    s.dma_write(0x4322, 0x00);
    s.dma_write(0x4323, 0x01);
    s.dma_write(0x4324, 0xC0);
    s.dma_write(0x4325, 8);
    s.dma_write(0x4326, 0);

    // Read 8 decompressed bytes - expect golden values for header 0x30
    let expected = [0x45, 0x56, 0x11, 0xA4, 0xFF, 0xBB, 0x37, 0xED];
    for i in 0..8 {
        let b = s.mcu_read(0xC00100, &rom);
        assert_eq!(b, expected[i], "byte {} mismatch (header 0x30)", i);
    }

    // Channel should be done
    let status = s.io_read(0x4801).unwrap();
    assert_eq!(status, 0, "channel 2 not finished");
}

#[test]
fn e09_06_inactive_cases_return_rom() {
    let mut s = Sdd1::new();
    let rom = build_rom(0);

    // Case 1: r4800=0 (no channels enabled), r4801=1
    s.io_write(0x4800, 0);
    s.io_write(0x4801, 1);
    let b = s.mcu_read(0xC00100, &rom);
    assert_eq!(b, rom[0x100], "r4800=0 should read plain ROM");

    // Case 2: address mismatch
    s.io_write(0x4800, 1);
    s.dma_write(0x4302, 0x00);
    s.dma_write(0x4303, 0x01);
    s.dma_write(0x4304, 0xC0);
    let b = s.mcu_read(0xC00101, &rom); // different address
    assert_eq!(b, rom[0x101], "address mismatch should read plain ROM");

    // Case 3: channel not active in r4801
    s.io_write(0x4801, 2); // only channel 1 active
    let b = s.mcu_read(0xC00100, &rom);
    assert_eq!(b, rom[0x100], "inactive channel should read plain ROM");
}

#[test]
fn e09_06_size_zero_wraps() {
    let mut s = Sdd1::new();
    let rom = build_rom(0);

    // Set size to 0 (means 65536)
    s.io_write(0x4800, 1);
    s.io_write(0x4801, 1);
    s.dma_write(0x4302, 0x00);
    s.dma_write(0x4303, 0x01);
    s.dma_write(0x4304, 0xC0);
    // size = 0 (both bytes zero)

    // Read one byte - should decompress and wrap size to 0xFFFF
    let _b = s.mcu_read(0xC00100, &rom);

    let (_, size) = s.dma_channel(0);
    assert_eq!(size, 0xFFFF, "size should wrap from 0 to 0xFFFF");

    // r4801 bit should still be set (not finished)
    let status = s.io_read(0x4801).unwrap();
    assert_eq!(status & 1, 1, "channel should still be active after wrap");
}

#[test]
fn e09_06_dma_write_assembles_fields() {
    let mut s = Sdd1::new();

    // Write address bytes for channel 3: 0x12, 0x34, 0xC5 -> addr 0xC53412
    s.dma_write(0x4332, 0x12);
    s.dma_write(0x4333, 0x34);
    s.dma_write(0x4334, 0xC5);

    // Write size bytes for channel 3: 0x56, 0x78 -> size 0x7856
    s.dma_write(0x4335, 0x56);
    s.dma_write(0x4336, 0x78);

    let (addr, size) = s.dma_channel(3);
    assert_eq!(addr, 0xC53412, "address assembly failed");
    assert_eq!(size, 0x7856, "size assembly failed");
}
