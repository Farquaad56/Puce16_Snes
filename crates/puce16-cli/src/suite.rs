mod baseline;
mod manifest;
mod report;
mod runner;

use std::fs;
use std::path::Path;
use std::process::ExitCode;

use crate::ppm;

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

    let text = match fs::read_to_string(manifest) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("cannot read {}: {}", manifest.display(), e);
            return ExitCode::from(2);
        }
    };

    let entries = match manifest::parse(&text) {
        Ok(e) => e,
        Err(msg) => {
            eprintln!("{}", msg);
            return ExitCode::from(2);
        }
    };

    // Parse max_epic filter
    let max_epic_num: Option<u32> = match max_epic {
        Some(s) => match manifest::epic_number(s) {
            Some(n) => Some(n),
            None => {
                eprintln!("bad --max-epic: {}", s);
                return ExitCode::from(2);
            }
        },
        None => None,
    };

    // Load baseline from manifest directory
    let baseline_path = match manifest.parent() {
        Some(dir) => dir.join("baseline.json"),
        None => Path::new(".").join("baseline.json"),
    };
    let mut baseline = match baseline::Baseline::load(&baseline_path) {
        Ok(b) => b,
        Err(msg) => {
            eprintln!("{}", msg);
            return ExitCode::from(2);
        }
    };

    // Update-baseline mode: run single ROM and update its entry
    if let Some(name) = update_baseline {
        let entry = match entries.iter().find(|e| e.name == *name) {
            Some(e) => e,
            None => {
                eprintln!("unknown rom: {}", name);
                return ExitCode::from(2);
            }
        };

        let run_result = match runner::run_rom(entry) {
            Ok(r) => r,
            Err(msg) => {
                eprintln!("{}", msg);
                return ExitCode::from(1);
            }
        };

        baseline.roms.insert(name.to_string(), baseline::Entry {
            frames: entry.frames,
            frame_hash: run_result.frame_hash,
            state_hash: run_result.state_hash,
            audio_hash: run_result.audio_hash,
        });

        if let Err(msg) = baseline.save(&baseline_path) {
            eprintln!("{}", msg);
            return ExitCode::from(1);
        }

        println!("baseline updated: {}", name);
        return ExitCode::from(0);
    }

    // Normal suite run: execute all ROMs filtered by epic number
    let mut verdicts = Vec::new();

    for entry in &entries {
        // Filter by epic number
        if let Some(max) = max_epic_num {
            if let Some(entry_epic) = manifest::epic_number(&entry.epic) {
                if entry_epic > max {
                    continue;
                }
            }
        }

        let name = &entry.name;

        // Check file existence
        if !Path::new(&entry.path).exists() {
            if entry.optional {
                verdicts.push(report::Verdict::Skip("file missing".to_string()));
            } else {
                verdicts.push(report::Verdict::Fail(format!("file missing: {}", entry.path)));
            }
            println!("{}", report::format_line(name, &verdicts.last().unwrap()));
            continue;
        }

        // Run the ROM
        let run_result = match runner::run_rom(entry) {
            Ok(r) => r,
            Err(msg) => {
                verdicts.push(report::Verdict::Fail(msg));
                println!("{}", report::format_line(name, &verdicts.last().unwrap()));
                continue;
            }
        };

        // Dump PPM if requested
        if let Some(dir) = dump_ppm {
            if fs::create_dir_all(dir).is_ok() {
                let ppm_path = dir.join(format!("{}.ppm", name));
                let bytes = ppm::encode_ppm(run_result.width, run_result.height, &run_result.pixels);
                let _ = fs::write(&ppm_path, bytes);
            }
        }

        // Judge against baseline
        let base_entry = baseline.roms.get(name);
        let verdict = runner::judge(entry, &run_result, base_entry);
        verdicts.push(verdict.clone());
        println!("{}", report::format_line(name, &verdict));
    }

    // Print summary and return exit code
    let counts = report::tally(&verdicts);
    println!("{}", report::summary(&counts));
    ExitCode::from(report::exit_code(&counts) as u8)
}
