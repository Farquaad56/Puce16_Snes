//! E07-10: Integration tests for `puce16-cli suite` with synthetic ROMs.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

/// Build a minimal LoROM (32 KiB) that loops forever at 0x8000.
fn tiny_rom() -> Vec<u8> {
    let mut r = vec![0u8; 0x8000];
    // BRA -2 at 0x8000 (infinite loop)
    r[0] = 0x80;
    r[1] = 0xFE;
    // RESET vector -> 0x8000
    r[0x7FFC] = 0x00;
    r[0x7FFD] = 0x80;
    r
}

/// ROM that writes 0x42 to WRAM at $7E0010, then loops.
fn wram_rom() -> Vec<u8> {
    let mut r = tiny_rom();
    // SEI ; CLC ; XCE ; LDA #$42 ; STA $0010 ; BRA -2
    r[..10].copy_from_slice(&[0x78, 0x18, 0xFB, 0xA9, 0x42, 0x8D, 0x10, 0x00, 0x80, 0xFE]);
    r
}

/// Write ROM bytes to a file and return the path.
fn write_rom(dir: &Path, name: &str, data: &[u8]) -> String {
    let path = dir.join(name);
    fs::write(&path, data).expect("failed to write test ROM");
    path.to_string_lossy().to_string()
}

/// Write a manifest TOML file.
fn write_manifest(dir: &Path, content: &str) -> String {
    let path = dir.join("manifest.toml");
    fs::write(&path, content).expect("failed to write manifest");
    path.to_string_lossy().to_string()
}

/// Run the puce16-cli suite command with given arguments.
fn run_suite(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_puce16-cli"))
        .arg("suite")
        .args(args)
        .output()
        .expect("failed to launch puce16-cli suite")
}

/// Convert a path to use forward slashes (for TOML compatibility on Windows).
fn to_forward_slashes(path: &Path) -> String {
    path.to_string_lossy().to_string().replace('\\', "/")
}

/// Setup a test directory with ROMs and manifest. Returns manifest_path.
fn setup(dir: &Path) -> String {
    write_rom(dir, "tiny.sfc", &tiny_rom());
    write_rom(dir, "wram.sfc", &wram_rom());

    // Build manifest with absolute paths using forward slashes (TOML-safe on Windows)
    let d = to_forward_slashes(dir);
    let manifest = format!(
        "[[rom]]\nname = \"tiny_hash\"\npath = \"{}/tiny.sfc\"\nframes = 3\nepic = \"E04\"\n\n\
         [[rom]]\nname = \"wram_ram\"\npath = \"{}/wram.sfc\"\nframes = 2\nepic = \"E03\"\ncheck = \"ram\"\nexpect_ram = [ {{ addr = 0x7E0010, value = 0x42 }} ]\n\n\
         [[rom]]\nname = \"tiny_ports\"\npath = \"{}/tiny.sfc\"\nframes = 3\nepic = \"E06\"\ncheck = \"ports\"\nexpect_ports = [0xAA, 0xBB, 0, 0]\n\n\
         [[rom]]\nname = \"absent_game\"\npath = \"{}/nothere.sfc\"\nframes = 1\nepic = \"E07\"\noptional = true",
        d, d, d, d
    );

    write_manifest(dir, &manifest)
}

#[test]
fn e07_10_first_run_reports_new() {
    let dir = std::env::temp_dir().join(format!("puce16_e07_10_first_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let manifest = setup(&dir);

    let out = run_suite(&[&manifest]);
    assert_eq!(out.status.code(), Some(0), "exit code should be 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("NEW tiny_hash frame_hash=0x"), "missing NEW line: {}", stdout);
    assert!(stdout.contains("PASS wram_ram"), "missing PASS wram_ram: {}", stdout);
    assert!(stdout.contains("PASS tiny_ports"), "missing PASS tiny_ports: {}", stdout);
    assert!(stdout.contains("SKIP absent_game: file missing"), "missing SKIP line: {}", stdout);
    assert!(stdout.contains("suite: 2 pass, 0 fail, 1 skip, 1 new"), "wrong summary: {}", stdout);

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn e07_10_update_then_pass() {
    let dir = std::env::temp_dir().join(format!("puce16_e07_10_update_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let manifest = setup(&dir);

    // First update the baseline for tiny_hash
    let out = run_suite(&[&manifest, "--update-baseline", "tiny_hash"]);
    assert_eq!(out.status.code(), Some(0), "update should exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("baseline updated: tiny_hash"), "missing update message: {}", stdout);

    // Check baseline.json exists and contains expected data
    let baseline_path = dir.join("baseline.json");
    assert!(baseline_path.exists(), "baseline.json should exist");
    let baseline_text = fs::read_to_string(&baseline_path).unwrap();
    assert!(baseline_text.contains("\"tiny_hash\""), "baseline missing tiny_hash entry");
    assert!(baseline_text.contains("\"frames\": 3"), "baseline missing frames: 3");

    // Now run the suite again - tiny_hash should PASS
    let out2 = run_suite(&[&manifest]);
    assert_eq!(out2.status.code(), Some(0), "second run should exit 0");
    let stdout2 = String::from_utf8_lossy(&out2.stdout);
    assert!(stdout2.contains("PASS tiny_hash"), "tiny_hash should PASS after update: {}", stdout2);

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn e07_10_hash_mismatch_fails() {
    let dir = std::env::temp_dir().join(format!("puce16_e07_10_mismatch_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let manifest = setup(&dir);

    // Update baseline first
    run_suite(&[&manifest, "--update-baseline", "tiny_hash"]);

    // Corrupt the state_hash in baseline.json
    let baseline_path = dir.join("baseline.json");
    let mut text = fs::read_to_string(&baseline_path).unwrap();
    // Replace any 16-hex-digit value after "state_hash": with zeros
    if let Some(pos) = text.find("\"state_hash\": \"0x") {
        let start = pos + "\"state_hash\": \"0x".len();
        let end = start + 16;
        let before = &text[..start];
        let after = &text[end..];
        text = format!("{}{:0>16}{}", before, "0", after);
    }
    fs::write(&baseline_path, &text).unwrap();

    // Run suite - should fail on tiny_hash due to state_hash mismatch
    let out = run_suite(&[&manifest]);
    assert_eq!(out.status.code(), Some(1), "should exit 1 on hash mismatch");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("FAIL tiny_hash:"), "missing FAIL line: {}", stdout);
    assert!(stdout.contains("state_hash"), "missing state_hash in error: {}", stdout);

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn e07_10_ram_mismatch_fails() {
    let dir = std::env::temp_dir().join(format!("puce16_e07_10_ram_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();

    write_rom(&dir, "tiny.sfc", &tiny_rom());
    write_rom(&dir, "wram.sfc", &wram_rom());

    let d = to_forward_slashes(&dir);
    // Expect 0x43 instead of 0x42 - should fail
    let manifest = format!(
        "[[rom]]\nname = \"wram_ram\"\npath = \"{}/wram.sfc\"\nframes = 2\nepic = \"E03\"\ncheck = \"ram\"\nexpect_ram = [ {{ addr = 0x7E0010, value = 0x43 }} ]",
        d
    );
    let manifest_path = write_manifest(&dir, &manifest);

    let out = run_suite(&[&manifest_path]);
    assert_eq!(out.status.code(), Some(1), "should exit 1 on RAM mismatch");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("FAIL wram_ram: ram 0x7E0010: expected 0x43 got 0x42"), "wrong error: {}", stdout);

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn e07_10_max_epic_filters() {
    let dir = std::env::temp_dir().join(format!("puce16_e07_10_filter_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let manifest = setup(&dir);

    // Update baseline for all ROMs first so they PASS
    run_suite(&[&manifest, "--update-baseline", "tiny_hash"]);

    // Run with --max-epic E03 - only wram_ram (E03) should execute
    let out = run_suite(&[&manifest, "--max-epic", "E03"]);
    assert_eq!(out.status.code(), Some(0), "should exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("PASS wram_ram"), "wram_ram should run: {}", stdout);
    assert!(!stdout.contains("tiny_hash"), "tiny_hash (E04) should be filtered: {}", stdout);
    assert!(!stdout.contains("tiny_ports"), "tiny_ports (E06) should be filtered: {}", stdout);

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn e07_10_bad_max_epic() {
    let dir = std::env::temp_dir().join(format!("puce16_e07_10_badepic_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let manifest = setup(&dir);

    let out = run_suite(&[&manifest, "--max-epic", "X1"]);
    assert_eq!(out.status.code(), Some(2), "should exit 2 on bad epic");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("bad --max-epic"), "missing error message: {}", stderr);

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn e07_10_missing_required_rom_fails() {
    let dir = std::env::temp_dir().join(format!("puce16_e07_10_reqrom_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();

    write_rom(&dir, "tiny.sfc", &tiny_rom());
    write_rom(&dir, "wram.sfc", &wram_rom());

    let d = to_forward_slashes(&dir);
    // absent_game is NOT optional here
    let manifest = format!(
        "[[rom]]\nname = \"absent_game\"\npath = \"{}/nothere.sfc\"\nframes = 1\nepic = \"E07\"",
        d
    );
    let manifest_path = write_manifest(&dir, &manifest);

    let out = run_suite(&[&manifest_path]);
    assert_eq!(out.status.code(), Some(1), "should exit 1 on missing required ROM");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("FAIL absent_game: file missing"), "wrong error: {}", stdout);

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn e07_10_unknown_update_target() {
    let dir = std::env::temp_dir().join(format!("puce16_e07_10_updtgt_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let manifest = setup(&dir);

    let out = run_suite(&[&manifest, "--update-baseline", "nope"]);
    assert_eq!(out.status.code(), Some(2), "should exit 2 on unknown target");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("unknown rom: nope"), "missing error message: {}", stderr);

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn e07_10_dump_ppm() {
    let dir = std::env::temp_dir().join(format!("puce16_e07_10_ppm_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let manifest = setup(&dir);

    // Update baseline first so tiny_hash passes
    run_suite(&[&manifest, "--update-baseline", "tiny_hash"]);

    let ppm_dir = dir.join("out");
    let out = run_suite(&[&manifest, "--dump-ppm", &ppm_dir.to_string_lossy()]);
    assert_eq!(out.status.code(), Some(0), "should exit 0");

    // Check PPM file exists and has correct format
    let ppm_path = ppm_dir.join("tiny_hash.ppm");
    assert!(ppm_path.exists(), "PPM file should exist");
    let data = fs::read(&ppm_path).unwrap();
    assert!(data.starts_with(b"P6\n256 224\n255\n"), "invalid PPM header");
    let expected_size = 15 + 256 * 224 * 3;
    assert_eq!(data.len(), expected_size, "PPM size mismatch: got {}, expected {}", data.len(), expected_size);

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn e07_10_invalid_manifest() {
    let dir = std::env::temp_dir().join(format!("puce16_e07_10_badman_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();

    // Write invalid TOML
    let manifest_path = write_manifest(&dir, "[[rom\n");

    let out = run_suite(&[&manifest_path]);
    assert_eq!(out.status.code(), Some(2), "should exit 2 on invalid manifest");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("invalid manifest"), "missing error message: {}", stderr);

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn e07_10_deterministic_output() {
    let dir = std::env::temp_dir().join(format!("puce16_e07_10_det_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let manifest = setup(&dir);

    // Update baseline so all ROMs pass
    run_suite(&[&manifest, "--update-baseline", "tiny_hash"]);

    // Run twice and compare output
    let out1 = run_suite(&[&manifest]);
    let out2 = run_suite(&[&manifest]);

    assert_eq!(out1.status.code(), Some(0), "first run should exit 0");
    assert_eq!(out2.status.code(), Some(0), "second run should exit 0");
    assert_eq!(out1.stdout, out2.stdout, "output should be deterministic");

    fs::remove_dir_all(&dir).ok();
}
