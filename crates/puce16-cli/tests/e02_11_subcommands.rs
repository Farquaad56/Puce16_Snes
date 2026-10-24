//! E02-11 tests for CLI subcommand registration (cpu-test, spc-test, suite).

use std::process::{Command, Output};

/// Launch the puce16-cli binary with the given arguments.
fn run_cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_puce16-cli"))
        .args(args)
        .output()
        .expect("failed to launch puce16-cli")
}

#[test]
fn e02_11_help_lists_commands() {
    let output = run_cli(&["--help"]);
    assert!(
        output.status.success(),
        "expected exit code 0, got {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    for cmd in ["cpu-test", "spc-test", "suite", "run"] {
        assert!(
            stdout.contains(cmd),
            "--help output missing subcommand '{}':\n{}",
            cmd,
            stdout
        );
    }
}

#[test]
fn e02_11_cpu_test_missing_dir() {
    let output = run_cli(&["cpu-test", "/nonexistent/dir"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "expected exit code 2, got {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("path not found"),
        "expected 'path not found' in stderr: {}",
        stderr
    );
}

#[test]
fn e02_11_spc_test_missing_dir() {
    let output = run_cli(&["spc-test", "/nonexistent/dir"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "expected exit code 2, got {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn e02_11_suite_missing_manifest() {
    let output = run_cli(&["suite", "/nonexistent/manifest.toml"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "expected exit code 2, got {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn e02_11_run_still_works() {
    let output = run_cli(&["run", "--test-pattern", "--frames", "1", "--hash"]);
    assert!(
        output.status.success(),
        "expected exit code 0, got {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("frame_hash=0x"),
        "expected 'frame_hash=0x' in stdout: {}",
        stdout
    );
}
