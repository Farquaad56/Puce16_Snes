use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn bin_path() -> PathBuf {
    let path = std::env::var("CARGO_BIN_EXE_puce16-cli").expect("CARGO_BIN_EXE_puce16-cli not set");
    PathBuf::from(path)
}

fn run_spc_test(dir: &Path, filter: Option<&str>) -> std::process::Output {
    let mut cmd = Command::new(bin_path());
    cmd.args(["spc-test", dir.to_string_lossy().as_ref()]);
    if let Some(f) = filter {
        cmd.arg("--filter").arg(f);
    }
    cmd.output().expect("failed to run puce16-cli")
}

fn make_temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("e06_09_{}", name));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn e06_09_wrong_expectation_fails() {
    // NOP (opcode 0x00) with wrong expected pc after execution.
    // Initial pc=30256, opcode at that address is 0x00 (NOP).
    // After NOP, pc should be 30257 but we expect 4660 (wrong).
    let json = r#"[{
        "name": "nop_wrong_pc",
        "initial": {
            "pc": 30256, "a": 56, "x": 78, "y": 127, "sp": 236, "psw": 145,
            "ram": [[30256, 0]]
        },
        "final": {
            "pc": 4660, "a": 56, "x": 78, "y": 127, "sp": 236, "psw": 145,
            "ram": [[30256, 0]]
        },
        "cycles": [
            [30256, 0, "read"],
            [30257, null, "read"]
        ]
    }]"#;

    let dir = make_temp_dir("wrong_expectation");
    fs::write(dir.join("00.json"), json).expect("failed to write test file");

    let output = run_spc_test(&dir, None);
    cleanup(&dir);

    assert_eq!(output.status.code(), Some(1), "expected exit code 1");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("FAIL"), "expected FAIL in output: {}", stdout);
}

#[test]
fn e06_09_empty_dir_usage() {
    let dir = make_temp_dir("empty");

    let output = run_spc_test(&dir, None);
    cleanup(&dir);

    assert_eq!(output.status.code(), Some(2), "expected exit code 2 for empty dir");
}

#[test]
fn e06_09_filter_no_match() {
    // Create a file with opcode 00 but filter for E4 (no match)
    let json = r#"[{
        "name": "nop",
        "initial": {
            "pc": 30256, "a": 56, "x": 78, "y": 127, "sp": 236, "psw": 145,
            "ram": [[30256, 0]]
        },
        "final": {
            "pc": 30257, "a": 56, "x": 78, "y": 127, "sp": 236, "psw": 145,
            "ram": [[30256, 0]]
        },
        "cycles": [
            [30256, 0, "read"],
            [30257, null, "read"]
        ]
    }]"#;

    let dir = make_temp_dir("filter_no_match");
    fs::write(dir.join("00.json"), json).expect("failed to write test file");

    let output = run_spc_test(&dir, Some("E4"));
    cleanup(&dir);

    assert_eq!(output.status.code(), Some(2), "expected exit code 2 for no matching files");
}

#[test]
fn e06_09_summary_line() {
    // Same setup as wrong_expectation_fails - should produce summary line
    let json = r#"[{
        "name": "nop_wrong_pc",
        "initial": {
            "pc": 30256, "a": 56, "x": 78, "y": 127, "sp": 236, "psw": 145,
            "ram": [[30256, 0]]
        },
        "final": {
            "pc": 4660, "a": 56, "x": 78, "y": 127, "sp": 236, "psw": 145,
            "ram": [[30256, 0]]
        },
        "cycles": [
            [30256, 0, "read"],
            [30257, null, "read"]
        ]
    }]"#;

    let dir = make_temp_dir("summary");
    fs::write(dir.join("00.json"), json).expect("failed to write test file");

    let output = run_spc_test(&dir, None);
    cleanup(&dir);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.lines().any(|l| l.starts_with("files: 1 ok: 0 failed: 1")),
        "expected summary line 'files: 1 ok: 0 failed: 1', got:\n{}",
        stdout
    );
}

#[test]
fn e06_09_bad_json_is_failure() {
    let dir = make_temp_dir("bad_json");
    fs::write(dir.join("ea.json"), "not valid json").expect("failed to write bad file");

    let output = run_spc_test(&dir, None);
    cleanup(&dir);

    assert_eq!(output.status.code(), Some(1), "expected exit code 1 for bad JSON");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("FAIL"), "expected FAIL in output: {}", stdout);
}
