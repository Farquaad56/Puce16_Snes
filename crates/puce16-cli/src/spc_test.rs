#![allow(dead_code)]

mod compare;
mod oracle;

use std::path::Path;
use std::process::ExitCode;

/// Run SPC700 audio tests from a directory.
pub fn run(dir: &Path, filter: Option<&str>) -> ExitCode {
    if !dir.exists() {
        eprintln!("path not found: {}", dir.display());
        return ExitCode::from(2);
    }
    let _ = filter;
    eprintln!("not implemented yet");
    ExitCode::from(2)
}
