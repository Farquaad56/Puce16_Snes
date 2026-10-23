use std::process::Command;

#[test]
fn e01_02_version_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_puce16-cli"))
        .arg("--version")
        .output()
        .expect("failed to launch puce16-cli");
    assert!(
        output.status.success(),
        "expected exit code 0, got {:?}",
        output.status.code()
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.contains("puce16-cli") || stderr.contains("puce16-cli"),
        "version output missing 'puce16-cli': stdout={stdout:?} stderr={stderr:?}"
    );
}
