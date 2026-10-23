//! E01-07 tests for `puce16-cli run` (test pattern, --hash, --dump-ppm).

use std::process::{Command, Output};

/// Compile src/ppm.rs into this test crate so the encoder can be tested
/// directly: puce16-cli is a binary-only crate and E01-07 adds no lib target.
#[path = "../src/ppm.rs"]
mod ppm;

/// Launch the puce16-cli binary with the given arguments.
fn run_cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_puce16-cli"))
        .args(args)
        .output()
        .expect("failed to launch puce16-cli")
}

/// Extract the `frame_hash=0x...` line from stdout.
fn frame_hash_line(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .find(|line| line.starts_with("frame_hash=0x"))
        .unwrap_or_else(|| panic!("no frame_hash line in output:\n{stdout}"))
        .to_string()
}

/// Check that a hash line is `frame_hash=0x` followed by 16 lowercase hex.
fn assert_hash_format(line: &str) {
    let hex = line
        .strip_prefix("frame_hash=0x")
        .unwrap_or_else(|| panic!("bad prefix in {line:?}"));
    assert_eq!(hex.len(), 16, "expected 16 hex digits in {line:?}");
    assert!(
        hex.chars().all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
        "expected lowercase hex in {line:?}"
    );
}

#[test]
fn e01_07_ppm_2x1() {
    let bytes = ppm::encode_ppm(2, 1, &[0xFF10_2030, 0xFFFF_FFFF]);
    let mut expected: Vec<u8> = b"P6\n2 1\n255\n".to_vec();
    expected.extend_from_slice(&[0x10, 0x20, 0x30, 0xFF, 0xFF, 0xFF]);
    assert_eq!(bytes, expected);
}

#[test]
fn e01_07_run_hash_stable() {
    let first = run_cli(&["run", "--test-pattern", "--frames", "5", "--hash"]);
    assert!(
        first.status.success(),
        "expected exit code 0, got {:?}: {}",
        first.status.code(),
        String::from_utf8_lossy(&first.stderr)
    );
    let second = run_cli(&["run", "--test-pattern", "--frames", "5", "--hash"]);
    assert!(second.status.success());

    let h1 = frame_hash_line(&first);
    let h2 = frame_hash_line(&second);
    assert_hash_format(&h1);
    assert_hash_format(&h2);
    assert_eq!(h1, h2, "same command must produce the same hash");
}

#[test]
fn e01_07_run_hash_frames() {
    let five = run_cli(&["run", "--test-pattern", "--frames", "5", "--hash"]);
    assert!(five.status.success());
    let six = run_cli(&["run", "--test-pattern", "--frames", "6", "--hash"]);
    assert!(six.status.success());

    let h5 = frame_hash_line(&five);
    let h6 = frame_hash_line(&six);
    assert_ne!(h5, h6, "different frame counts must produce different hashes");
}

#[test]
fn e01_07_dump_ppm() {
    let path = std::env::temp_dir().join(format!("e01_07_dump_ppm_{}.ppm", std::process::id()));
    let _ = std::fs::remove_file(&path);

    let output = run_cli(&[
        "run",
        "--test-pattern",
        "--dump-ppm",
        path.to_str().expect("non-UTF-8 temp path"),
    ]);
    assert!(
        output.status.success(),
        "expected exit code 0, got {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let bytes = std::fs::read(&path).expect("PPM file was not created");
    assert_eq!(&bytes[..11], b"P6\n256 224\n", "bad PPM header");
    assert_eq!(bytes.len(), 15 + 256 * 224 * 3, "unexpected PPM size");

    let _ = std::fs::remove_file(&path);
}

#[test]
fn e01_07_no_pattern_usage() {
    let output = run_cli(&["run"]);
    assert_eq!(output.status.code(), Some(2), "expected exit code 2 without --test-pattern");
}
