//! E03-44: CLI run command with ROM loading, --state-hash, --mapper, --debug.

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
fn e03_44_run_rom_state_hash() {
    let rom_path = write_rom(&tiny_rom());
    let mut cmd = cli();
    cmd.args([rom_path.as_str(), "--frames", "3", "--state-hash"]);
    let out = cmd.output().expect("failed to run CLI");
    cleanup(&rom_path);

    assert_eq!(out.status.code(), Some(0), "exit code should be 0, got {:?}", out.status.code());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Should contain state_hash=0x followed by 16 lowercase hex digits
    assert!(stdout.contains("state_hash=0x"), "missing state_hash line: {}", stdout);
    // Verify format with regex-like check
    for line in stdout.lines() {
        if let Some(rest) = line.strip_prefix("state_hash=0x") {
            assert_eq!(rest.len(), 16, "state_hash should have 16 hex digits");
            assert!(rest.chars().all(|c| c.is_ascii_hexdigit()), "non-hex in state_hash");
        }
    }
}

#[test]
fn e03_44_run_rom_deterministic() {
    let rom_path = write_rom(&tiny_rom());

    // Two identical runs should produce identical output
    let mut cmd1 = cli();
    cmd1.args([rom_path.as_str(), "--frames", "3", "--state-hash"]);
    let out1 = cmd1.output().expect("failed to run CLI");

    let mut cmd2 = cli();
    cmd2.args([rom_path.as_str(), "--frames", "3", "--state-hash"]);
    let out2 = cmd2.output().expect("failed to run CLI");

    assert_eq!(out1.stdout, out2.stdout, "deterministic runs should match");

    // Different frame counts should produce different state hashes
    let mut cmd3 = cli();
    cmd3.args([rom_path.as_str(), "--frames", "4", "--state-hash"]);
    let out3 = cmd3.output().expect("failed to run CLI");

    assert_ne!(out1.stdout, out3.stdout, "different frame counts should differ");
    cleanup(&rom_path);
}

#[test]
fn e03_44_run_rom_hash_line() {
    let rom_path = write_rom(&tiny_rom());
    let mut cmd = cli();
    cmd.args([rom_path.as_str(), "--frames", "1", "--hash"]);
    let out = cmd.output().expect("failed to run CLI");
    cleanup(&rom_path);

    assert_eq!(out.status.code(), Some(0), "exit code should be 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("frame_hash=0x"), "missing frame_hash line: {}", stdout);
}

#[test]
fn e03_44_run_rom_debug() {
    let rom_path = write_rom(&tiny_rom());
    let mut cmd = cli();
    cmd.args([rom_path.as_str(), "--frames", "1", "--debug"]);
    let out = cmd.output().expect("failed to run CLI");
    cleanup(&rom_path);

    assert_eq!(out.status.code(), Some(0), "exit code should be 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    // ROM loops at 0x8000, so PC should be there; e=1 (emulation mode)
    assert!(stdout.contains("debug: pc=00:8000 "), "missing debug line with correct PC");
    assert!(stdout.contains(" e=1 "), "should be in emulation mode");
    // After 1 frame, v should be 225 (end of visible scanlines) and frame=1
    assert!(stdout.contains(" v=225 "), "v should be 225 after 1 frame");
    assert!(stdout.contains(" frame=1"), "frame should be 1");
}

#[test]
fn e03_44_run_rom_region() {
    let rom_path = write_rom(&tiny_rom());

    // NTSC and PAL should produce different state hashes
    let mut cmd_ntsc = cli();
    cmd_ntsc.args([rom_path.as_str(), "--frames", "3", "--state-hash", "--region", "ntsc"]);
    let out_ntsc = cmd_ntsc.output().expect("failed to run CLI");

    let mut cmd_pal = cli();
    cmd_pal.args([rom_path.as_str(), "--frames", "3", "--state-hash", "--region", "pal"]);
    let out_pal = cmd_pal.output().expect("failed to run CLI");

    assert_ne!(out_ntsc.stdout, out_pal.stdout, "NTSC and PAL should differ");

    // Invalid region should fail with exit code 2 (clap error)
    let mut cmd_bad = cli();
    cmd_bad.args([rom_path.as_str(), "--frames", "1", "--region", "xyz"]);
    let out_bad = cmd_bad.output().expect("failed to run CLI");
    assert_eq!(out_bad.status.code(), Some(2), "invalid region should exit 2");

    cleanup(&rom_path);
}

#[test]
fn e03_44_run_missing_file() {
    let mut cmd = cli();
    cmd.args(["/nonexistent/x.sfc", "--frames", "1"]);
    let out = cmd.output().expect("failed to run CLI");

    assert_eq!(out.status.code(), Some(2), "missing file should exit 2");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("cannot read"), "should report cannot read: {}", stderr);
}

#[test]
fn e03_44_run_too_small() {
    // Write a file that's too small to be a valid ROM (< 128 bytes)
    let tmp_dir = std::env::temp_dir();
    let path = tmp_dir.join(format!("puce16_tiny_{}.sfc", std::process::id()));
    fs::write(&path, vec![0u8; 100]).expect("failed to write small file");

    let mut cmd = cli();
    cmd.args([path.to_string_lossy().as_ref(), "--frames", "1"]);
    let out = cmd.output().expect("failed to run CLI");
    fs::remove_file(&path).ok();

    assert_eq!(out.status.code(), Some(2), "too small ROM should exit 2");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("too small"), "should report too small: {}", stderr);
}

#[test]
fn e03_44_run_bad_mapper() {
    // tiny_rom is LoROM (32 KiB). Forcing HiRom should fail because the header
    // at 0x7FD0 doesn't match HiRom expectations.
    let rom_path = write_rom(&tiny_rom());

    let mut cmd = cli();
    cmd.args([rom_path.as_str(), "--frames", "1", "--mapper", "hirom"]);
    let out = cmd.output().expect("failed to run CLI");
    cleanup(&rom_path);

    assert_eq!(out.status.code(), Some(2), "bad mapper should exit 2");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("no valid ROM header"), "should report no valid header: {}", stderr);
}

#[test]
fn e03_44_run_dump_ppm() {
    let rom_path = write_rom(&tiny_rom());
    let tmp_dir = std::env::temp_dir();
    let ppm_path = tmp_dir.join(format!("puce16_frame_{}.ppm", std::process::id()));

    let mut cmd = cli();
    cmd.args([rom_path.as_str(), "--frames", "1", "--dump-ppm", ppm_path.to_string_lossy().as_ref()]);
    let out = cmd.output().expect("failed to run CLI");
    cleanup(&rom_path);

    assert_eq!(out.status.code(), Some(0), "exit code should be 0");
    let data = fs::read(&ppm_path).expect("PPM file not written");
    fs::remove_file(&ppm_path).ok();

    // P6 header: "P6\n256 224\n255\n" = 15 bytes, then 256*224*3 pixel data
    let expected_size = 15 + 256 * 224 * 3;
    assert_eq!(data.len(), expected_size, "PPM size mismatch");
    // Check PPM magic header
    assert!(data.starts_with(b"P6\n256 224\n255\n"), "invalid PPM header");
}

#[test]
fn e03_44_legacy_pattern_still_works() {
    // --test-pattern should still work as before
    let mut cmd = cli();
    cmd.args(["--test-pattern", "--frames", "5", "--hash"]);
    let out = cmd.output().expect("failed to run CLI");

    assert_eq!(out.status.code(), Some(0), "test pattern should exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("frame_hash=0x"), "missing frame_hash: {}", stdout);

    // No ROM and no --test-pattern should fail with usage error
    let mut cmd2 = cli();
    cmd2.args(vec![] as Vec<&str>);
    let out2 = cmd2.output().expect("failed to run CLI");
    assert_eq!(out2.status.code(), Some(2), "no args should exit 2");
    let stderr = String::from_utf8_lossy(&out2.stderr);
    assert!(stderr.contains("rom path or --test-pattern required"), "wrong error: {}", stderr);
}
