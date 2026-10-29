use std::fs;
use std::path::{Path, PathBuf};

use puce16_core::hash::Fnv64;
use puce16_desktop::files::*;

#[test]
fn e08_20_srm_path() {
    assert_eq!(srm_path(Path::new("/g/Super Mario World.sfc")), PathBuf::from("/g/Super Mario World.srm"));
    assert_eq!(srm_path(Path::new("rom.smc")), PathBuf::from("rom.srm"));
    assert_eq!(srm_path(Path::new("game")), PathBuf::from("game.srm"));
}

#[test]
fn e08_20_state_path() {
    assert_eq!(state_path(Path::new("/g/a.sfc"), 3), PathBuf::from("/g/a.3.p16s"));
    assert_eq!(state_path(Path::new("/g/a.sfc"), 12), PathBuf::from("/g/a.2.p16s"));
}

#[test]
fn e08_20_screenshot_path() {
    let rom = Path::new("/g/a.sfc");
    // No existing files -> a_1.ppm
    assert_eq!(screenshot_path(rom, &|_| false), PathBuf::from("/g/a_1.ppm"));
    // a_1 and a_2 exist -> a_3.ppm
    let exists = |p: &Path| {
        p == Path::new("/g/a_1.ppm") || p == Path::new("/g/a_2.ppm")
    };
    assert_eq!(screenshot_path(rom, &exists), PathBuf::from("/g/a_3.ppm"));
}

#[test]
fn e08_20_write_atomic() {
    let tmp = std::env::temp_dir().join("puce16_test_atomic");
    fs::create_dir_all(&tmp).unwrap();
    let path = tmp.join("data.bin");
    let tmp_path = tmp.join("data.bin.tmp");

    // Write and read back
    write_atomic(&path, &[1, 2, 3]).unwrap();
    assert_eq!(fs::read(&path).unwrap(), vec![1, 2, 3]);
    // .tmp file should not exist after rename
    assert!(!tmp_path.exists());

    // Second write replaces content
    write_atomic(&path, &[4, 5]).unwrap();
    assert_eq!(fs::read(&path).unwrap(), vec![4, 5]);

    // Non-existent directory -> Err containing "cannot write"
    let bad = tmp.join("no/such/dir/file.bin");
    let err = write_atomic(&bad, &[1]).unwrap_err();
    assert!(err.contains("cannot write"));

    fs::remove_dir_all(&tmp).ok();
}

#[test]
fn e08_20_rom_hash() {
    // Empty input -> FNV-1a offset basis
    assert_eq!(rom_hash(&[]), 0xcbf29ce484222325);

    // Compute expected hash for [1,2,3] using Fnv64 directly
    let mut h = Fnv64::new();
    h.write(&[1, 2, 3]);
    assert_eq!(rom_hash(&[1, 2, 3]), h.finish());

    // Different ROMs -> different hashes
    assert_ne!(rom_hash(b"ROM A"), rom_hash(b"ROM B"));
}

#[test]
fn e08_20_encode_header_layout() {
    let encoded = encode_state(0x0102030405060708, 1, &[9, 9]);
    assert_eq!(encoded.len(), 17);
    // magic + version LE + hash LE + region + blob
    let expected: Vec<u8> = b"P16S".to_vec()
        .into_iter().chain([1, 0])
        .chain([8, 7, 6, 5, 4, 3, 2, 1])
        .chain([1]).chain([9, 9])
        .collect();
    assert_eq!(encoded, expected);
}

#[test]
fn e08_20_decode_roundtrip() {
    let h = rom_hash(b"test rom");
    let blob = vec![42, 43, 44];
    let encoded = encode_state(h, 1, &blob);
    let (region, decoded_blob) = decode_state(&encoded, h).unwrap();
    assert_eq!(region, 1);
    assert_eq!(decoded_blob, blob.as_slice());
}

#[test]
fn e08_20_decode_errors() {
    let h = rom_hash(b"test rom");
    // Wrong magic
    let bad_magic: Vec<u8> = b"XXXX".to_vec().into_iter().chain(vec![0; 15]).collect();
    assert_eq!(decode_state(&bad_magic, h).unwrap_err(), "not a Puce16 state");

    // Too short (10 bytes)
    let too_short: Vec<u8> = b"P16S".to_vec().into_iter().chain(vec![0; 6]).collect();
    assert_eq!(decode_state(&too_short, h).unwrap_err(), "not a Puce16 state");

    // Wrong version (2)
    let bad_ver: Vec<u8> = b"P16S".to_vec()
        .into_iter().chain([2, 0])
        .chain(h.to_le_bytes())
        .chain([0]).collect();
    assert_eq!(decode_state(&bad_ver, h).unwrap_err(), "unsupported state container version 2");

    // Wrong ROM hash
    let wrong_hash = rom_hash(b"different rom");
    let bad_hash: Vec<u8> = b"P16S".to_vec()
        .into_iter().chain([1, 0])
        .chain(wrong_hash.to_le_bytes())
        .chain([0]).collect();
    assert_eq!(decode_state(&bad_hash, h).unwrap_err(), "state belongs to another ROM");
}
