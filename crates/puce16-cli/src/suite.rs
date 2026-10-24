#![allow(dead_code)]

mod baseline;
mod manifest;
mod report;
mod runner;

use std::path::Path;
use std::process::ExitCode;

/// Run the full test suite from a manifest file.
pub fn run(
    manifest: &Path,
    max_epic: Option<&str>,
    update_baseline: Option<&str>,
    dump_ppm: Option<&Path>,
) -> ExitCode {
    if !manifest.exists() {
        eprintln!("path not found: {}", manifest.display());
        return ExitCode::from(2);
    }
    let _ = (max_epic, update_baseline, dump_ppm);
    eprintln!("not implemented yet");
    ExitCode::from(2)
}
