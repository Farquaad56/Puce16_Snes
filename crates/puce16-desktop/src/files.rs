use std::io::{Cursor, Read};
use std::path::Path;

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
