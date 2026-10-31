//! E09-10: S-DD1 end-to-end test with synthetic ROM and DMA to WRAM.

use puce16_core::snes::Snes;

mod common;
#[path = "common/prog.rs"]
mod prog;
use prog::build_rom_with;

/// LCG-based test stream generator (matches docs/tools/sdd1_ref.py).
fn lcg_bytes(seed: u32, n: usize) -> Vec<u8> {
    let mut x = seed;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        x = x.wrapping_mul(1664525).wrapping_add(1013904223);
        out.push((x >> 24) as u8);
    }
    out
}

/// Build the S-DD1 test program that triggers DMA from $C0:0100 to WRAM.
fn sdd1_dma_program() -> Vec<u8> {
    vec![
        0x78, // SEI
        0x18, // CLC
        0xFB, // XCE (enter native mode)
        0xA9, 0x01, 0x8D, 0x00, 0x48, // LDA #$01; STA $4800 (S-DD1 hard enable ch0)
        0x8D, 0x01, 0x48,             // STA $4801 (S-DD1 soft enable ch0)
        0xA9, 0x08, 0x8D, 0x00, 0x43, // LDA #$08; STA $4300 (DMAP: A->B, fixed addr, mode 0)
        0xA9, 0x80, 0x8D, 0x01, 0x43, // LDA #$80; STA $4301 (dest $2180 = WRAM data port)
        0xA9, 0x00, 0x8D, 0x02, 0x43, // LDA #$00; STA $4302 (src addr low: $C0:0100)
        0xA9, 0x01, 0x8D, 0x03, 0x43, // LDA #$01; STA $4303 (src addr mid)
        0xA9, 0xC0, 0x8D, 0x04, 0x43, // LDA #$C0; STA $4304 (src addr high = bank C0)
        0xA9, 0x40, 0x8D, 0x05, 0x43, // LDA #$40; STA $4305 (size low: 64 bytes)
        0xA9, 0x00, 0x8D, 0x06, 0x43, // LDA #$00; STA $4306 (size high)
        0x8D, 0x81, 0x21,             // STA $2181 (WRAM addr low: $7E:0000)
        0x8D, 0x82, 0x21,             // STA $2182 (WRAM addr mid)
        0x8D, 0x83, 0x21,             // STA $2183 (WRAM addr high)
        0xA9, 0x01, 0x8D, 0x0B, 0x42, // LDA #$01; STA $420B (MDMAEN: enable ch0)
        0x80, 0xFE,                   // BRA -2 (loop forever)
    ]
}

/// Run the S-DD1 DMA test and return WRAM contents at offset 0..64.
fn run_sdd1_dma_test(header_byte: u8) -> Vec<u8> {
    let k = header_byte >> 4;
    let seed = 0x1000 + k as u32;

    // Generate compressed stream with S-DD1 header byte
    let mut data = lcg_bytes(seed, 4096);
    data[0] = (data[0] & 0x0F) | header_byte;

    let prog = sdd1_dma_program();
    let rom = build_rom_with(
        &[(0, &prog[..]), (0x100, &data[..])],
        &[("reset", 0x8000)],
        0,
        0x43, // S-DD1 chipset
    );

    let mut snes = Snes::new(&rom, None).expect("ROM should load");
    snes.run_frame();
    snes.run_frame();

    let mut wram = Vec::with_capacity(64);
    for i in 0..64 {
        wram.push(snes.peek_wram(i));
    }
    wram
}

/// Parse hex string into bytes.
fn hex_to_bytes(hex: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(hex.len() / 2);
    for i in (0..hex.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex[i..i + 2], 16).expect("valid hex");
        out.push(byte);
    }
    out
}

#[test]
fn e09_10_wram_gets_decompressed_header_00() {
    let wram = run_sdd1_dma_test(0x00);
    // Expected from sdd1_vectors.tsv header=00: 751224913c890eac...
    let expected = hex_to_bytes("751224913c890eac349e3941f151e11bc9418165105451c737ef1948f07ce4cbc0a0526960c34c510d104bb8cbb4fdb48e92cf490e5c44136519460ddc059850");
    assert_eq!(wram, expected);
}

#[test]
fn e09_10_wram_gets_decompressed_header_30() {
    let wram = run_sdd1_dma_test(0x30);
    // Expected from sdd1_vectors.tsv header=30: 455611a4ffbb37ed...
    let expected = hex_to_bytes("455611a4ffbb37ed27b3ffed1d7e48a89ca5a2f114a93c2713d7301d864e6b6f03f0c6bd73e9512237ad732c6059e54269ee3c2f57a17c564fe561d419ff128f");
    assert_eq!(wram, expected);
}

#[test]
fn e09_10_wram_gets_decompressed_mode7_c0() {
    let wram = run_sdd1_dma_test(0xC0);
    // Expected from sdd1_vectors.tsv header=c0: 3a66c648329260c9...
    let expected = hex_to_bytes("3a66c648329260c91fa4781c297bf3683d7479af4b7bc5e52fbd4fdfc95fc66828947bd64c1747b92a5c05457ddb75f95e7d9225e11347d01099133183ff5787");
    assert_eq!(wram, expected);
}

#[test]
fn e09_10_without_sdd1_same_byte_repeated() {
    // Same program but chipset 0x00 (no S-DD1). Fixed-address DMA copies same byte 64 times.
    let k = 0;
    let seed = 0x1000 + k as u32;
    let mut data = lcg_bytes(seed, 4096);
    data[0] = (data[0] & 0x0F) | 0x00;

    let prog = sdd1_dma_program();
    let rom = build_rom_with(
        &[(0, &prog[..]), (0x100, &data[..])],
        &[("reset", 0x8000)],
        0,
        0x00, // No S-DD1 chipset
    );

    let mut snes = Snes::new(&rom, None).expect("ROM should load");
    snes.run_frame();
    snes.run_frame();

    // All 64 bytes in WRAM should be the same (the first byte of the stream)
    let first_byte = data[0];
    for i in 0..64 {
        assert_eq!(snes.peek_wram(i), first_byte, "WRAM byte {} should equal rom[0x100]", i);
    }
}
