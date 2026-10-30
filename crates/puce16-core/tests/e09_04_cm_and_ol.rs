//! E09-04 tests: CM (context model), OL (output logic), and golden vectors.

use puce16_core::coprocessor::sdd1::Decompressor;

/// LCG random number generator matching docs/tools/sdd1_ref.py
fn lcg_bytes(seed: u32, n: usize) -> Vec<u8> {
    let mut x = seed;
    (0..n).map(|_| {
        x = x.wrapping_mul(1664525).wrapping_add(1013904223);
        (x >> 24) as u8
    }).collect()
}

/// FNV-1a 64-bit hash
fn fnv64(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// Format u64 as 16-digit lowercase hex
fn hex_u64(v: u64) -> String {
    format!("{:016x}", v)
}

/// Decode a hex string to bytes
fn hex_decode(s: &str) -> Vec<u8> {
    let mut result = Vec::with_capacity(s.len() / 2);
    for i in (0..s.len()).step_by(2) {
        let byte = u8::from_str_radix(&s[i..i+2], 16).unwrap();
        result.push(byte);
    }
    result
}

/// Format bytes as lowercase hex string
fn hex_encode(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02x}", b)).collect()
}

#[test]
fn e09_04_cm_bit_first16_headers() {
    // seed 0x6666, data = lcg_bytes(0x6666, 64) with data[0] = (data[0] & 0x0F) | h
    let base_data = lcg_bytes(0x6666, 64);

    // Expected bit sequences for each header type (16 bits each)
    let expected: [(u8, &[u8]); 5] = [
        (0x00, &[0,1,0,1,1,0,1,0,0,1,0,0,1,1,1,0]),
        (0x30, &[0,1,0,1,1,1,1,0,1,0,1,0,1,1,0,1]),
        (0x40, &[0,1,0,1,1,0,1,0,0,1,0,0,1,1,1,0]),
        (0x80, &[0,1,0,1,1,0,1,0,0,1,0,0,1,1,1,0]),
        (0xC0, &[0,1,0,0,1,0,1,1,1,0,1,0,1,0,1,0]),
    ];

    for &(header, bits) in &expected {
        let mut data = base_data.clone();
        data[0] = (data[0] & 0x0F) | header;
        let rom = |a: u32| data[(a as usize) % data.len()];

        let mut dec = Decompressor::new();
        dec.init(0, &rom);

        for i in 0..16 {
            let bit = dec.cm_bit(&rom);
            assert_eq!(bit, bits[i], "header {:02x} bit {}", header, i);
        }
    }
}

#[test]
fn e09_04_read_golden_vectors_all_headers() {
    // Read the golden vectors TSV file
    let tsv_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/specs/data/sdd1_vectors.tsv");
    let content = std::fs::read_to_string(tsv_path).expect("Failed to read sdd1_vectors.tsv");

    for line in content.lines() {
        if line.is_empty() || line.starts_with("#") {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 7 {
            continue; // header or malformed
        }

        let header_hex = fields[0].trim();
        let seed_str = fields[1].trim();
        let input_hex_first32 = fields[2].trim();
        let output_hex_first64 = fields[3].trim();
        let fnv64_first256_expected = fields[4].trim();
        let fnv64_first8192_expected = fields[5].trim();

        // Parse header and seed
        let header: u8 = u8::from_str_radix(header_hex, 16).expect("header hex");
        let seed: u32 = seed_str.parse().expect("seed decimal");

        // Generate test stream
        let mut data = lcg_bytes(seed, 4096);
        data[0] = (data[0] & 0x0F) | header;

        // Verify input matches expected first 32 bytes
        assert_eq!(hex_encode(&data[..32]), input_hex_first32, "input mismatch for seed {}", seed);

        let rom = |a: u32| data[(a as usize) % data.len()];

        // Decompress 8192 bytes
        let mut dec = Decompressor::new();
        dec.init(0, &rom);

        let mut output = Vec::with_capacity(8192);
        for _ in 0..8192 {
            output.push(dec.read(&rom));
        }

        // Verify first 64 bytes match expected
        assert_eq!(hex_encode(&output[..64]), output_hex_first64, "output mismatch for seed {}", seed);

        // Verify FNV-64 of first 256 bytes
        let fnv_256 = hex_u64(fnv64(&output[..256]));
        assert_eq!(fnv_256, fnv64_first256_expected, "FNV-64(256) mismatch for seed {}", seed);

        // Verify FNV-64 of all 8192 bytes
        let fnv_8192 = hex_u64(fnv64(&output));
        assert_eq!(fnv_8192, fnv64_first8192_expected, "FNV-64(8192) mismatch for seed {}", seed);
    }
}

#[test]
fn e09_04_reinit_restarts_stream() {
    let data = lcg_bytes(0x7777, 4096);
    let rom = |a: u32| data[(a as usize) % data.len()];

    // First instance: read 100 bytes then reinit and read 32 more
    let mut dec1 = Decompressor::new();
    dec1.init(0, &rom);
    for _ in 0..100 {
        dec1.read(&rom);
    }
    dec1.init(0, &rom); // reinit
    let mut after_reinit = Vec::with_capacity(32);
    for _ in 0..32 {
        after_reinit.push(dec1.read(&rom));
    }

    // Second instance: fresh start, read first 32 bytes
    let mut dec2 = Decompressor::new();
    dec2.init(0, &rom);
    let mut fresh_first_32 = Vec::with_capacity(32);
    for _ in 0..32 {
        fresh_first_32.push(dec2.read(&rom));
    }

    assert_eq!(after_reinit, fresh_first_32, "reinit should restart stream");
}

#[test]
fn e09_04_mode7_header_reads_eight_bit_values() {
    // Header 0xC0 (mode7), same stream as cm_bit test (seed 0x6666)
    let base_data = lcg_bytes(0x6666, 4096);
    let header: u8 = 0xC0;
    let mut data = base_data.clone();
    data[0] = (data[0] & 0x0F) | header;

    let rom = |a: u32| data[(a as usize) % data.len()];

    // Instance A: use read() to get bytes
    let mut dec_a = Decompressor::new();
    dec_a.init(0, &rom);

    // Instance B: use cm_bit() 8 times per byte, assemble LSB first
    let mut dec_b = Decompressor::new();
    dec_b.init(0, &rom);

    for i in 0..16 {
        let byte_a = dec_a.read(&rom);

        // Assemble byte from 8 cm_bit calls (LSB first)
        let mut byte_b: u8 = 0;
        for bit_pos in 0..8 {
            let bit = dec_b.cm_bit(&rom);
            if bit != 0 {
                byte_b |= 1 << bit_pos;
            }
        }

        assert_eq!(byte_a, byte_b, "mode7 read vs cm_bit mismatch at byte {}", i);
    }
}
