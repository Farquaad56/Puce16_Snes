mod compare;
mod oracle;

use std::path::Path;
use std::process::ExitCode;

use crate::spc_test::compare::{run_case, Level};
use puce16_core::apu::spc700::testbus::FlatSmpBus;
use puce16_core::apu::spc700::Spc700;

/// Run SPC700 instruction tests from a directory.
pub fn run(dir: &Path, filter: Option<&str>) -> ExitCode {
    if !dir.exists() {
        eprintln!("path not found: {}", dir.display());
        return ExitCode::from(2);
    }

    let files = match oracle::list_files(dir, filter) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("error: {}", e);
            return ExitCode::from(2);
        }
    };

    if files.is_empty() {
        eprintln!("no test files found");
        return ExitCode::from(2);
    }

    let mut cpu = Spc700::new();
    let mut bus = FlatSmpBus::new();

    let total = files.len();
    let mut ok_count = 0;
    let mut fail_count = 0;

    for file in &files {
        let name = file.file_name().unwrap().to_string_lossy().to_string();

        match oracle::load_file(file) {
            Ok(cases) => {
                let n = cases.len();
                let mut failed_cases = Vec::new();

                for case in &cases {
                    let diffs = run_case(case, &mut cpu, &mut bus, Level::Count);
                    if !diffs.is_empty() {
                        failed_cases.push((case.name.clone(), diffs));
                    }
                }

                if failed_cases.is_empty() {
                    println!("OK {} ({} tests)", name, n);
                    ok_count += 1;
                } else {
                    let k = failed_cases.len();
                    println!("FAIL {} {}/{}", name, k, n);

                    // Print up to 5 detailed failures
                    for (i, (test_name, diffs)) in failed_cases.iter().enumerate() {
                        if i >= 5 {
                            break;
                        }
                        println!("  {}", test_name);
                        for diff in diffs {
                            println!("    {}", diff);
                        }
                    }

                    fail_count += 1;
                }
            }
            Err(e) => {
                // File read error counts as file failure
                println!("FAIL {} (load error)", name);
                eprintln!("  {}", e);
                fail_count += 1;
            }
        }
    }

    println!("files: {} ok: {} failed: {}", total, ok_count, fail_count);

    if fail_count > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
