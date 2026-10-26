use std::fs;
use std::io::{Cursor, Write};
use puce16_desktop::files::*;

fn make_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut cursor = Cursor::new(Vec::new());
    let mut writer = zip::ZipWriter::new(&mut cursor);
    for (name, data) in entries {
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        writer.start_file(name, options).unwrap();
        writer.write_all(data).unwrap();
    }
    writer.finish().unwrap();
    cursor.into_inner()
}

#[test]
fn e03_45_is_rom_name() {
    assert!(is_rom_name("a.sfc"));
    assert!(is_rom_name("a.SMC"));
    assert!(is_rom_name("x.swc"));
    assert!(is_rom_name("x.Fig"));
    assert!(is_rom_name("dir/game.sfc"));

    assert!(!is_rom_name("a.txt"));
    assert!(!is_rom_name("a.zip"));
    assert!(!is_rom_name("sfc"));
    assert!(!is_rom_name("a.sfc.bak"));
    assert!(!is_rom_name(""));
}

#[test]
fn e03_45_zip_first_rom() {
    let bytes = make_zip(&[
        ("readme.txt", b"hi"),
        ("Game.SMC", &[1, 2, 3]),
        ("other.sfc", &[9]),
    ]);
    let (name, data) = rom_from_zip(&bytes).unwrap();
    assert_eq!(name, "Game.SMC");
    assert_eq!(data, vec![1, 2, 3]);
}

#[test]
fn e03_45_zip_no_rom() {
    let bytes = make_zip(&[("a.txt", b"hi")]);
    let err = rom_from_zip(&bytes).unwrap_err();
    assert!(err.contains("no ROM"));
}

#[test]
fn e03_45_zip_invalid() {
    let err = rom_from_zip(&[1, 2, 3]).unwrap_err();
    assert!(err.starts_with("invalid zip"));
}

#[test]
fn e03_45_load_plain() {
    let dir = std::env::temp_dir().join(format!("puce16_e03_45_plain_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("game.sfc");
    fs::write(&path, [1u8, 2, 3]).unwrap();

    let (name, data) = load_rom_file(&path).unwrap();
    assert_eq!(name, "game.sfc");
    assert_eq!(data, vec![1, 2, 3]);

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn e03_45_load_zip_file() {
    let dir = std::env::temp_dir().join(format!("puce16_e03_45_zipfile_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let zip_bytes = make_zip(&[("readme.txt", b"hi"), ("Game.SMC", &[1, 2, 3])]);
    let path = dir.join("pack.zip");
    fs::write(&path, &zip_bytes).unwrap();

    let (name, data) = load_rom_file(&path).unwrap();
    assert_eq!(name, "Game.SMC");
    assert_eq!(data, vec![1, 2, 3]);

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn e03_45_load_unsupported() {
    let dir = std::env::temp_dir().join(format!("puce16_e03_45_unsupp_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();

    let txt_path = dir.join("notes.txt");
    fs::write(&txt_path, b"hello").unwrap();
    let err = load_rom_file(&txt_path).unwrap_err();
    assert!(err.contains("unsupported file type: txt"));

    let noext_path = dir.join("gamefile");
    fs::write(&noext_path, b"data").unwrap();
    let err = load_rom_file(&noext_path).unwrap_err();
    assert!(err.contains("unsupported file type: "));

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn e03_45_load_missing() {
    let dir = std::env::temp_dir().join(format!("puce16_e03_45_{}", std::process::id()));
    let path = dir.join("x.sfc");
    let err = load_rom_file(&path).unwrap_err();
    assert!(err.contains("cannot read"));
}
