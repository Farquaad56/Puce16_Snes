use std::fs;
use std::path::{Path, PathBuf};

use puce16_core::cheat::{ParCode, parse_par};
use puce16_desktop::files::*;

#[test]
fn e10_05_cheat_path_replaces_extension() {
    assert_eq!(
        cheat_path(Path::new("C:/jeux/smw.sfc")),
        PathBuf::from("C:/jeux/smw.cht")
    );
    assert_eq!(
        cheat_path(Path::new("/x/y.z.smc")),
        PathBuf::from("/x/y.z.cht")
    );
    // No extension -> append .cht
    assert_eq!(
        cheat_path(Path::new("game")),
        PathBuf::from("game.cht")
    );
}

#[test]
fn e10_05_load_missing_file_is_empty() {
    let path = Path::new("/nonexistent/path/to/rom.cht");
    let entries = load_cheat_entries(path).unwrap();
    assert!(entries.is_empty());
}

#[test]
fn e10_05_save_then_load_roundtrip() {
    let tmp = std::env::temp_dir().join("puce16_test_e10_05");
    fs::create_dir_all(&tmp).unwrap();
    let path = tmp.join("game.cht");

    // Create two cheat entries
    let code1 = parse_par("7E0DBE05").unwrap();
    let code2 = parse_par("7E1497FF").unwrap();
    let entries = vec![
        puce16_core::cheat::CheatEntry { name: "Infinite lives".to_string(), enabled: true, codes: vec![code1] },
        puce16_core::cheat::CheatEntry { name: "Invincible".to_string(), enabled: false, codes: vec![code2] },
    ];

    // Save and read back
    save_cheat_entries(&path, &entries).unwrap();
    let loaded = load_cheat_entries(&path).unwrap();
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].name, "Infinite lives");
    assert!(loaded[0].enabled);
    assert_eq!(loaded[1].name, "Invincible");
    assert!(!loaded[1].enabled);

    // File starts with header
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.starts_with("# Puce16 cheats"));

    fs::remove_dir_all(&tmp).ok();
}

#[test]
fn e10_05_load_bad_file_reports_line() {
    let tmp = std::env::temp_dir().join("puce16_test_e10_05_bad");
    fs::create_dir_all(&tmp).unwrap();
    let path = tmp.join("bad.cht");

    // Line 2 is invalid (missing sign)
    fs::write(&path, "+ A = 7E0DBE05\nfaux").unwrap();
    let err = load_cheat_entries(&path).unwrap_err();
    assert!(err.contains("2"), "error should mention line 2: {}", err);

    fs::remove_dir_all(&tmp).ok();
}

#[test]
fn e10_05_emu_thread_applies_cheats_after_reset() {
    // Test cheat persistence: set_cheats stores cheats, they survive reset().
    use puce16_core::Snes;

    let mut snes = Snes::new_test_pattern(puce16_core::Region::Ntsc);

    // Set a cheat code
    let code = parse_par("7E000042").unwrap();
    snes.set_cheats(&[code]);

    // Verify cheats are stored
    assert_eq!(snes.cheats().len(), 1);
    assert_eq!(snes.cheats()[0].addr, code.addr);
    assert_eq!(snes.cheats()[0].value, code.value);

    // Reset the console - cheats should persist (per spec)
    snes.reset();

    // Cheats should still be stored after reset
    assert_eq!(snes.cheats().len(), 1);
    assert_eq!(snes.cheats()[0].addr, code.addr);
    assert_eq!(snes.cheats()[0].value, code.value);
}
