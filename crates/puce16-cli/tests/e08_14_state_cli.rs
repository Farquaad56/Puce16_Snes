//! E08-14: CLI --save-state and --load-state options.

use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static ROM_COUNTER: AtomicU64 = AtomicU64::new(0);

fn cli() -> Command {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "-p", "puce16-cli", "--quiet", "--", "run"]);
    cmd
}

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

fn write_rom(rom: &[u8]) -> String {
    let tmp_dir = std::env::temp_dir();
    let n = ROM_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = tmp_dir.join(format!("puce16_test_{}_{}.sfc", std::process::id(), n));
    fs::write(&path, rom).expect("failed to write test ROM");
    path.to_string_lossy().to_string()
}

fn cleanup(path: &str) {
    let _ = fs::remove_file(path);
}

#[test]
fn e08_14_save_then_load_matches() {
    let rom_path = write_rom(&tiny_rom());
    let tmp_dir = std::env::temp_dir();
    let save_path = tmp_dir.join(format!("puce16_state_{}.sav", std::process::id()));

    // Save state after 3 frames
    let mut cmd_save = cli();
    cmd_save.args([rom_path.as_str(), "--frames", "3", "--save-state", save_path.to_string_lossy().as_ref()]);
    let out_save = cmd_save.output().expect("failed to run CLI");
    assert_eq!(out_save.status.code(), Some(0), "save should exit 0, got {:?}", out_save.status.code());

    // Verify state file was written and is > 1 MB
    let save_data = fs::read(&save_path).expect("state file not written");
    assert!(save_data.len() > 1_000_000, "state file too small: {} bytes", save_data.len());

    // Load state and run 2 more frames (total 5)
    let mut cmd_load = cli();
    cmd_load.args([rom_path.as_str(), "--frames", "2", "--load-state", save_path.to_string_lossy().as_ref(), "--state-hash"]);
    let out_load = cmd_load.output().expect("failed to run CLI");
    assert_eq!(out_load.status.code(), Some(0), "load should exit 0, got {:?}", out_load.status.code());

    // Run 5 frames directly for comparison
    let mut cmd_direct = cli();
    cmd_direct.args([rom_path.as_str(), "--frames", "5", "--state-hash"]);
    let out_direct = cmd_direct.output().expect("failed to run CLI");
    assert_eq!(out_direct.status.code(), Some(0), "direct should exit 0, got {:?}", out_direct.status.code());

    // Both should produce the same state hash (3+2 == 5 frames)
    let load_stdout = String::from_utf8_lossy(&out_load.stdout);
    let direct_stdout = String::from_utf8_lossy(&out_direct.stdout);
    assert_eq!(load_stdout, direct_stdout, "state hashes should match: {} vs {}", load_stdout, direct_stdout);

    cleanup(&rom_path);
    fs::remove_file(&save_path).ok();
}

#[test]
fn e08_14_missing_state_file() {
    let rom_path = write_rom(&tiny_rom());

    let mut cmd = cli();
    cmd.args([rom_path.as_str(), "--frames", "1", "--load-state", "/nonexistent/state.sav"]);
    let out = cmd.output().expect("failed to run CLI");

    assert_eq!(out.status.code(), Some(2), "missing state file should exit 2, got {:?}", out.status.code());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("cannot read"), "should report cannot read: {}", stderr);

    cleanup(&rom_path);
}

#[test]
fn e08_14_bad_state_file() {
    let rom_path = write_rom(&tiny_rom());
    let tmp_dir = std::env::temp_dir();
    let bad_path = tmp_dir.join(format!("puce16_bad_{}.sav", std::process::id()));

    // Write garbage data (too small to be a valid state)
    fs::write(&bad_path, vec![0u8; 10]).expect("failed to write bad state file");

    let mut cmd = cli();
    cmd.args([rom_path.as_str(), "--frames", "1", "--load-state", bad_path.to_string_lossy().as_ref()]);
    let out = cmd.output().expect("failed to run CLI");

    assert_eq!(out.status.code(), Some(2), "bad state file should exit 2, got {:?}", out.status.code());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("cannot load state"), "should report cannot load state: {}", stderr);

    cleanup(&rom_path);
    fs::remove_file(&bad_path).ok();
}

#[test]
fn e08_14_unwritable_save() {
    let rom_path = write_rom(&tiny_rom());

    let mut cmd = cli();
    cmd.args([rom_path.as_str(), "--frames", "1", "--save-state", "/nonexistent/dir/state.sav"]);
    let out = cmd.output().expect("failed to run CLI");

    assert_eq!(out.status.code(), Some(2), "unwritable save should exit 2, got {:?}", out.status.code());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("cannot write"), "should report cannot write: {}", stderr);

    cleanup(&rom_path);
}

#[test]
fn e08_14_pattern_refused() {
    let tmp_dir = std::env::temp_dir();
    let save_path = tmp_dir.join(format!("puce16_pat_{}.sav", std::process::id()));

    // --save-state with --test-pattern should be rejected
    let mut cmd_save = cli();
    cmd_save.args(["--test-pattern", "--frames", "1", "--save-state", save_path.to_string_lossy().as_ref()]);
    let out_save = cmd_save.output().expect("failed to run CLI");

    assert_eq!(out_save.status.code(), Some(2), "pattern with --save-state should exit 2, got {:?}", out_save.status.code());
    let stderr_save = String::from_utf8_lossy(&out_save.stderr);
    assert!(stderr_save.contains("needs a ROM"), "should report needs a ROM: {}", stderr_save);

    // --load-state with --test-pattern should also be rejected
    fs::write(&save_path, vec![0u8; 10]).ok();
    let mut cmd_load = cli();
    cmd_load.args(["--test-pattern", "--frames", "1", "--load-state", save_path.to_string_lossy().as_ref()]);
    let out_load = cmd_load.output().expect("failed to run CLI");

    assert_eq!(out_load.status.code(), Some(2), "pattern with --load-state should exit 2, got {:?}", out_load.status.code());
    let stderr_load = String::from_utf8_lossy(&out_load.stderr);
    assert!(stderr_load.contains("needs a ROM"), "should report needs a ROM: {}", stderr_load);

    fs::remove_file(&save_path).ok();
}