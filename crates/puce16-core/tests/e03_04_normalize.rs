use puce16_core::cart::{strip_copier_header, normalize};

#[test]
fn e03_04_strip_512() {
    let len = 0x8000 + 512;
    let data: Vec<u8> = (0..len).map(|i| (i % 251) as u8).collect();
    let stripped = strip_copier_header(&data);
    assert_eq!(stripped.len(), 0x8000);
    assert_eq!(stripped[0], data[512]);

    // No header case: exactly 0x8000 bytes returns same slice
    let no_header = strip_copier_header(&data[..0x8000]);
    assert_eq!(no_header.len(), 0x8000);
}

#[test]
fn e03_04_strip_edge() {
    // Exactly 512 bytes: returns empty slice
    let small = vec![0u8; 512];
    let stripped = strip_copier_header(&small);
    assert_eq!(stripped.len(), 0);

    // 1000 bytes (not a multiple of 1024 + 512): unchanged
    let mid = vec![1u8; 1000];
    let stripped = strip_copier_header(&mid);
    assert_eq!(stripped.len(), 1000);
}

#[test]
fn e03_04_pow2_noop() {
    let data64k: Vec<u8> = (0..0x8000).map(|i| (i % 251) as u8).collect();
    let result = normalize(&data64k);
    assert_eq!(result, data64k);

    let data128k: Vec<u8> = (0..0x10000).map(|i| (i % 251) as u8).collect();
    let result = normalize(&data128k);
    assert_eq!(result, data128k);
}

#[test]
fn e03_04_pow2_96k() {
    // len 0x18000 (96 KiB): big=0x10000, rest=0x8000, small=0x8000
    let data: Vec<u8> = (0..0x18000).map(|i| (i % 251) as u8).collect();
    let r = normalize(&data);
    assert_eq!(r.len(), 0x20000);
    assert_eq!(&r[..0x18000], &data[..]);
    // Second half is copy of data[0x10000..0x18000]
    assert_eq!(&r[0x18000..0x20000], &data[0x10000..0x18000]);
}

#[test]
fn e03_04_pow2_80k() {
    // len 0x14000 (80 KiB): big=0x10000, rest=0x4000, small=0x4000
    let data: Vec<u8> = (0..0x14000).map(|i| (i % 251) as u8).collect();
    let r = normalize(&data);
    assert_eq!(r.len(), 0x20000);
    // First half: original data up to big, then rest copied once
    assert_eq!(&r[0x10000..0x14000], &data[0x10000..]);
    // Rest repeated 3 more times (big/small = 4 total)
    let chunk = &data[0x10000..0x14000];
    assert_eq!(&r[0x14000..0x18000], chunk);
    assert_eq!(&r[0x18000..0x1C000], chunk);
    assert_eq!(&r[0x1C000..0x20000], chunk);
}

#[test]
fn e03_04_pow2_non_pow_rest() {
    // len 0x16000: big=0x10000, rest=0x6000, small=0x8000 (next pow of 2)
    let data: Vec<u8> = (0..0x16000).map(|i| (i % 251) as u8).collect();
    let r = normalize(&data);
    assert_eq!(r.len(), 0x20000);
    // Original data preserved in first half
    assert_eq!(&r[0x10000..0x16000], &data[0x10000..]);
    // Zero padding from 0x16000 to 0x18000 (end of small block)
    assert!(r[0x16000..0x18000].iter().all(|&b| b == 0));
    // Second copy of the padded block: data again then zeros
    assert_eq!(&r[0x18000..0x1E000], &data[0x10000..]);
    assert!(r[0x1E000..0x20000].iter().all(|&b| b == 0));
}

#[test]
fn e03_04_pow2_3mib() {
    // len 3 MiB: big=2 MiB, rest=1 MiB, small=1 MiB
    let size = 3 * 1024 * 1024;
    let data: Vec<u8> = (0..size).map(|i| (i % 251) as u8).collect();
    let r = normalize(&data);
    assert_eq!(r.len(), 4 * 1024 * 1024);
    // Second half is copy of data[2MiB..3MiB]
    let two_mib = 2 * 1024 * 1024;
    let three_mib = 3 * 1024 * 1024;
    assert_eq!(&r[three_mib..], &data[two_mib..three_mib]);
}
