use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn bin_path() -> PathBuf {
    // CARGO_BIN_EXE_<name> is set by cargo when running integration tests
    let path = std::env::var("CARGO_BIN_EXE_puce16-cli").expect("CARGO_BIN_EXE_puce16-cli not set");
    PathBuf::from(path)
}

fn run_cpu_test(dir: &Path, filter: Option<&str>) -> std::process::Output {
    let mut cmd = Command::new(bin_path());
    cmd.args(["cpu-test", dir.to_string_lossy().as_ref()]);
    if let Some(f) = filter {
        cmd.arg("--filter").arg(f);
    }
    cmd.output().expect("failed to run puce16-cli")
}

fn make_temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("e02_14_{}", name));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn e02_14_wrong_expectation_fails() {
    // Use LDA #imm (0xA9) - may or may not be implemented yet.
    // Either way, the command should report FAIL with some diff info.
    // If unimplemented: "panic: unimplemented opcode A9"
    // If implemented but wrong expectation: register/PC diff
    let json = r#"[{
        "name": "a9 e 1",
        "initial": {
            "pc": 8192, "s": 255, "p": 64, "a": 0, "x": 0, "y": 0,
            "dbr": 0, "d": 0, "pbr": 0, "e": 1,
            "ram": [[8192, 169], [8193, 42]]
        },
        "final": {
            "pc": 4660, "s": 255, "p": 64, "a": 42, "x": 0, "y": 0,
            "dbr": 0, "d": 0, "pbr": 0, "e": 1,
            "ram": [[8192, 169], [8193, 42]]
        },
        "cycles": [
            [8192, 169, "dp-r-m--"],
            [8193, 42, "dp-r-m--"]
        ]
    }]"#;

    let dir = make_temp_dir("wrong_expectation");
    fs::write(dir.join("a9.e.json"), json).expect("failed to write test file");

    let output = run_cpu_test(&dir, None);
    cleanup(&dir);

    assert_eq!(output.status.code(), Some(1), "expected exit code 1");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("FAIL"), "expected FAIL in output: {}", stdout);
    // Either a panic (unimplemented opcode) or a register/PC diff
    let has_diff = stdout.contains("panic:") || stdout.contains("pc: expected") || stdout.contains("a: expected");
    assert!(has_diff, "expected some diff info in output: {}", stdout);
}

#[test]
fn e02_14_empty_dir_usage() {
    let dir = make_temp_dir("empty");

    let output = run_cpu_test(&dir, None);
    cleanup(&dir);

    assert_eq!(output.status.code(), Some(2), "expected exit code 2 for empty dir");
}

#[test]
fn e02_14_filter_no_match() {
    // Create a file with opcode A9 (LDA #imm) but filter for 61 (ADC)
    let json = r#"[{
        "name": "a9 e 1",
        "initial": {
            "pc": 8192, "s": 255, "p": 64, "a": 0, "x": 0, "y": 0,
            "dbr": 0, "d": 0, "pbr": 0, "e": 1,
            "ram": [[8192, 169], [8193, 42]]
        },
        "final": {
            "pc": 8194, "s": 255, "p": 64, "a": 42, "x": 0, "y": 0,
            "dbr": 0, "d": 0, "pbr": 0, "e": 1,
            "ram": [[8192, 169], [8193, 42]]
        },
        "cycles": [
            [8192, 169, "dp-r-m--"],
            [8193, 42, "dp-r-m--"]
        ]
    }]"#;

    let dir = make_temp_dir("filter_no_match");
    fs::write(dir.join("a9.e.json"), json).expect("failed to write test file");

    let output = run_cpu_test(&dir, Some("61"));
    cleanup(&dir);

    assert_eq!(output.status.code(), Some(2), "expected exit code 2 for no matching files");
}

#[test]
fn e02_14_summary_line() {
    // Same setup as wrong_expectation_fails - should produce summary line
    let json = r#"[{
        "name": "a9 e 1",
        "initial": {
            "pc": 8192, "s": 255, "p": 64, "a": 0, "x": 0, "y": 0,
            "dbr": 0, "d": 0, "pbr": 0, "e": 1,
            "ram": [[8192, 169], [8193, 42]]
        },
        "final": {
            "pc": 4660, "s": 255, "p": 64, "a": 42, "x": 0, "y": 0,
            "dbr": 0, "d": 0, "pbr": 0, "e": 1,
            "ram": [[8192, 169], [8193, 42]]
        },
        "cycles": [
            [8192, 169, "dp-r-m--"],
            [8193, 42, "dp-r-m--"]
        ]
    }]"#;

    let dir = make_temp_dir("summary");
    fs::write(dir.join("a9.e.json"), json).expect("failed to write test file");

    let output = run_cpu_test(&dir, None);
    cleanup(&dir);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.lines().any(|l| l.starts_with("files: 1 ok: 0 failed: 1")),
        "expected summary line 'files: 1 ok: 0 failed: 1', got:\n{}",
        stdout
    );
}

#[test]
fn e02_14_bad_json_is_failure() {
    let dir = make_temp_dir("bad_json");
    fs::write(dir.join("ea.e.json"), "not valid json").expect("failed to write bad file");

    let output = run_cpu_test(&dir, None);
    cleanup(&dir);

    assert_eq!(output.status.code(), Some(1), "expected exit code 1 for bad JSON");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("FAIL"), "expected FAIL in output: {}", stdout);
}
