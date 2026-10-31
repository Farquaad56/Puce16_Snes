use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

use puce16_core::cheat::{parse_cheat_file, format_cheat_file, CheatEntry};
use puce16_core::hash::Fnv64;

/// Check if a filename has a ROM extension (case-insensitive).
pub fn is_rom_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".sfc") || lower.ends_with(".smc") || lower.ends_with(".swc") || lower.ends_with(".fig")
}

/// Extract the first ROM entry from zip bytes.
pub fn rom_from_zip(zip_bytes: &[u8]) -> Result<(String, Vec<u8>), String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(zip_bytes))
        .map_err(|e| format!("invalid zip: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("invalid zip: {}", e))?;

        let fname = file.name()
            .map_err(|e| format!("invalid zip: {}", e))?.into_owned();
        if is_rom_name(&fname) {
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)
                .map_err(|e| format!("invalid zip: {}", e))?;
            return Ok((fname, contents));
        }
    }

    Err("no ROM file in zip".to_string())
}

/// Load a ROM file (plain or from zip).
pub fn load_rom_file(path: &Path) -> Result<(String, Vec<u8>), String> {
    let bytes = std::fs::read(path)
        .map_err(|e| format!("cannot read {}: {}", path.display(), e))?;

    let name = path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    let ext = path.extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "zip" => rom_from_zip(&bytes),
        _ if is_rom_name(&name) => Ok((name, bytes)),
        _ => Err(format!("unsupported file type: {}", ext)),
    }
}

/// SRAM save path: same directory as ROM, stem + ".srm".
pub fn srm_path(rom: &Path) -> PathBuf {
    let mut p = rom.to_path_buf();
    if let Some(stem) = p.file_stem() {
        p.set_file_name(format!("{}.srm", stem.to_string_lossy()));
    } else {
        p.set_file_name("game.srm");
    }
    p
}

/// Cheat file path: same directory as ROM, stem + ".cht".
pub fn cheat_path(rom: &Path) -> PathBuf {
    let mut p = rom.to_path_buf();
    if let Some(stem) = p.file_stem() {
        p.set_file_name(format!("{}.cht", stem.to_string_lossy()));
    } else {
        p.set_file_name("game.cht");
    }
    p
}

/// Load cheat entries from a .cht file. Missing file returns Ok(vec![]).
pub fn load_cheat_entries(path: &Path) -> Result<Vec<CheatEntry>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
    parse_cheat_file(&text).map_err(|e| format!("{}", e))
}

/// Save cheat entries to a .cht file (atomic write).
pub fn save_cheat_entries(path: &Path, entries: &[CheatEntry]) -> Result<(), String> {
    let text = format_cheat_file(entries);
    write_atomic(path, text.as_bytes())
}

/// Save state path: "<stem>.<slot>.p16s" (slot 0..=9, otherwise slot % 10).
pub fn state_path(rom: &Path, slot: u8) -> PathBuf {
    let mut p = rom.to_path_buf();
    if let Some(stem) = p.file_stem() {
        let s = slot % 10;
        p.set_file_name(format!("{}.{}.p16s", stem.to_string_lossy(), s));
    } else {
        p.set_file_name("game.0.p16s");
    }
    p
}

/// Screenshot path: "<stem>_<n>.ppm" where n is the first integer >= 1 such that !exists(path).
pub fn screenshot_path(rom: &Path, exists: &dyn Fn(&Path) -> bool) -> PathBuf {
    let stem = rom.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "game".to_string());
    let mut n = 1;
    loop {
        let p = rom.with_file_name(format!("{}_{}.ppm", stem, n));
        if !exists(&p) {
            return p;
        }
        n += 1;
    }
}

/// Atomic write: write to "<path>.tmp" then rename.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let tmp = path.with_extension(format!("{}.tmp", path.extension().map(|e| e.to_string_lossy()).unwrap_or_default()));
    fs::write(&tmp, bytes)
        .and_then(|_| fs::rename(&tmp, path))
        .map_err(|e| format!("cannot write {}: {}", path.display(), e))
}

/// FNV-1a 64 hash of ROM bytes.
pub fn rom_hash(bytes: &[u8]) -> u64 {
    let mut h = Fnv64::new();
    h.write(bytes);
    h.finish()
}

/// Save state container magic and version.
pub const STATE_MAGIC: [u8; 4] = *b"P16S";
pub const STATE_CONTAINER_VERSION: u16 = 1;

/// Encode a save state blob into the .p16s container format.
/// Layout: magic(4) + version(u16 LE) + rom_hash(u64 LE) + region(u8) + blob
pub fn encode_state(rom_hash: u64, region: u8, blob: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(15 + blob.len());
    out.extend_from_slice(&STATE_MAGIC);
    out.extend_from_slice(&STATE_CONTAINER_VERSION.to_le_bytes());
    out.extend_from_slice(&rom_hash.to_le_bytes());
    out.push(region);
    out.extend_from_slice(blob);
    out
}

/// Decode a .p16s container, verifying magic, version, and ROM hash.
/// Returns (region, blob) or an error string.
pub fn decode_state(data: &[u8], expect_rom_hash: u64) -> Result<(u8, &[u8]), String> {
    if data.len() < 15 || &data[0..4] != STATE_MAGIC.as_slice() {
        return Err("not a Puce16 state".to_string());
    }
    let version = u16::from_le_bytes([data[4], data[5]]);
    if version != STATE_CONTAINER_VERSION {
        return Err(format!("unsupported state container version {}", version));
    }
    let stored_hash = u64::from_le_bytes([data[6], data[7], data[8], data[9], data[10], data[11], data[12], data[13]]);
    if stored_hash != expect_rom_hash {
        return Err("state belongs to another ROM".to_string());
    }
    let region = data[14];
    Ok((region, &data[15..]))
}
