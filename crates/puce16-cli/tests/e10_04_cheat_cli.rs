//! E10-04: CLI --cheat and --cheat-code options.

use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static ROM_COUNTER: AtomicU64 = AtomicU64::new(0);

fn cli() -> Command {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "-p", "puce16-cli", "--quiet", "--", "run"]);
    cmd
}

/// Build a minimal LoROM that increments memory at 0x8000 then loops (spec: E6 00 80 FC).
fn cheat_rom() -> Vec<u8> {
    let mut r = vec![0u8; 0x8000];
    // INC $8000, BRA -2 (loop)
    r[0] = 0xE6;   // INC abs
    r[1] = 0x00;   // addr low
    r[2] = 0x80;   // addr high
    r[3] = 0xFC;   // BRA -2 (relative offset)
    // RESET vector -> 0x8000
    r[0x7FFC] = 0x00;
    r[0x7FFD] = 0x80;
    r
}

fn write_rom(rom: &[u8]) -> String {
    let tmp_dir = std::env::temp_dir();
    let n = ROM_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = tmp_dir.join(format!("puce16_e10_{}_{}.sfc", n, std::process::id()));
    fs::write(&path, rom).expect("failed to write test ROM");
    path.to_string_lossy().to_string()
}

fn cleanup(path: &str) {
    let _ = fs::remove_file(path);
}

#[test]
fn e10_04_binary_applies_cheat_code() {
    let rom_path = write_rom(&cheat_rom());

    // Run without cheat
    let mut cmd_no_cheat = cli();
    cmd_no_cheat.args([rom_path.as_str(), "--frames", "3", "--state-hash"]);
    let out_no_cheat = cmd_no_cheat.output().expect("failed to run CLI");
    assert_eq!(out_no_cheat.status.code(), Some(0), "no-cheat run should exit 0");

    // Run with cheat code that writes different value at same address
    let mut cmd_with_cheat = cli();
    cmd_with_cheat.args([rom_path.as_str(), "--frames", "3", "--cheat-code", "7E000042", "--state-hash"]);
    let out_with_cheat = cmd_with_cheat.output().expect("failed to run CLI");
    assert_eq!(out_with_cheat.status.code(), Some(0), "with-cheat run should exit 0");

    // State hashes should differ because cheat writes different value each frame
    let stdout_no = String::from_utf8_lossy(&out_no_cheat.stdout);
    let stdout_yes = String::from_utf8_lossy(&out_with_cheat.stdout);
    assert_ne!(stdout_no, stdout_yes, "cheat should change state hash");

    cleanup(&rom_path);
}

#[test]
fn e10_04_binary_rejects_bad_code() {
    let rom_path = write_rom(&cheat_rom());

    let mut cmd = cli();
    cmd.args([rom_path.as_str(), "--frames", "1", "--cheat-code", "ZZ"]);
    let out = cmd.output().expect("failed to run CLI");

    assert_ne!(out.status.code(), Some(0), "bad cheat code should exit non-zero");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("erreur cheat"), "should report 'erreur cheat': {}", stderr);

    cleanup(&rom_path);
}
