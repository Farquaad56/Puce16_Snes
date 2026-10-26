//! Puce16 SNES emulator command line interface (E03-44).
//!
//! `run` executes frames of a ROM or the built-in test pattern and can print
//! frame/state hashes, debug snapshots, or dump the last frame as a PPM image.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{CommandFactory, Parser, Subcommand};
use puce16_core::hash::Fnv64;
use puce16_core::cart::MapMode;
use puce16_core::{Region, Snes};

mod cpu_test;
mod ppm;
mod run_cmd;
mod spc_test;
mod suite;

/// Puce16 SNES emulator command line interface.
#[derive(Debug, Parser)]
#[command(name = "puce16-cli", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

/// Subcommands.
#[derive(Debug, Subcommand)]
enum Command {
    /// Run frames of a ROM or the built-in test pattern.
    Run(RunArgs),
    /// Run CPU instruction tests from a directory.
    CpuTest(CpuTestArgs),
    /// Run SPC700 audio tests from a directory.
    SpcTest(SpcTestArgs),
    /// Run the full test suite from a manifest file.
    Suite(SuiteArgs),
}

/// Arguments for `cpu-test`.
#[derive(Debug, Parser)]
struct CpuTestArgs {
    /// Directory containing CPU test ROMs.
    dir: PathBuf,
    /// Comma-separated list of tests to run (optional).
    #[arg(long)]
    filter: Option<String>,
}

/// Arguments for `spc-test`.
#[derive(Debug, Parser)]
struct SpcTestArgs {
    /// Directory containing SPC700 test ROMs.
    dir: PathBuf,
    /// Comma-separated list of tests to run (optional).
    #[arg(long)]
    filter: Option<String>,
}

/// Arguments for `suite`.
#[derive(Debug, Parser)]
struct SuiteArgs {
    /// Path to the test suite manifest file.
    manifest: PathBuf,
    /// Maximum epic number to include (e.g., "E02").
    #[arg(long)]
    max_epic: Option<String>,
    /// Update baseline for a specific test by name.
    #[arg(long)]
    update_baseline: Option<String>,
    /// Dump PPM frames to this directory.
    #[arg(long)]
    dump_ppm: Option<PathBuf>,
}

/// Arguments for `run` (E03-44: ROM loading with debug options).
#[derive(Debug, Parser)]
struct RunArgs {
    /// Path to a ROM file (optional; use --test-pattern instead).
    rom: Option<PathBuf>,

    /// Use the built-in test pattern instead of a ROM.
    #[arg(long)]
    test_pattern: bool,

    /// Number of frames to run.
    #[arg(long, default_value_t = 1)]
    frames: u64,

    /// Print `frame_hash=0x<16 hex>` for the last frame.
    #[arg(long)]
    hash: bool,

    /// Print `state_hash=0x<16 hex>` after running.
    #[arg(long)]
    state_hash: bool,

    /// Force mapper mode (lorom, hirom, exhirom).
    #[arg(long, value_enum)]
    mapper: Option<MapModeOpt>,

    /// Print a debug snapshot line after running.
    #[arg(long)]
    debug: bool,

    /// Video region (forces region; otherwise inferred from ROM header or NTSC for pattern).
    #[arg(long, value_enum)]
    region: Option<RegionOpt>,

    /// Write the PPM (P6) of the last frame to this file.
    #[arg(long)]
    dump_ppm: Option<PathBuf>,
}

/// CLI spelling of mapper mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum MapModeOpt {
    Lorom,
    Hirom,
    Exhirom,
}

impl From<MapModeOpt> for MapMode {
    fn from(opt: MapModeOpt) -> Self {
        match opt {
            MapModeOpt::Lorom => MapMode::LoRom,
            MapModeOpt::Hirom => MapMode::HiRom,
            MapModeOpt::Exhirom => MapMode::ExHiRom,
        }
    }
}

/// CLI spelling of `puce16_core::Region`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum RegionOpt {
    Ntsc,
    Pal,
}

impl From<RegionOpt> for Region {
    fn from(opt: RegionOpt) -> Self {
        match opt {
            RegionOpt::Ntsc => Region::Ntsc,
            RegionOpt::Pal => Region::Pal,
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match &cli.command {
        Some(Command::Run(args)) => run_command(args),
        Some(Command::CpuTest(args)) => cpu_test::run(&args.dir, args.filter.as_deref()),
        Some(Command::SpcTest(args)) => spc_test::run(&args.dir, args.filter.as_deref()),
        Some(Command::Suite(args)) => suite::run(
            &args.manifest,
            args.max_epic.as_deref(),
            args.update_baseline.as_deref(),
            args.dump_ppm.as_ref().map(|p| p as &std::path::Path),
        ),
        None => {
            // No subcommand given: display the help.
            println!("{}", Cli::command().render_help());
            ExitCode::SUCCESS
        }
    }
}

/// Execute `run`; returns the process exit code (0 OK, 1 emulation failure, 2 usage/file error).
fn run_command(args: &RunArgs) -> ExitCode {
    // Determine region: explicit > ROM header > NTSC default for pattern.
    let region_opt = args.region.map(|r| r.into());

    // Build the Snes instance (ROM or test pattern).
    let mut snes = match (&args.rom, args.test_pattern) {
        (Some(path), false) => {
            // Load ROM from file.
            let bytes = match std::fs::read(path) {
                Ok(b) => b,
                Err(err) => {
                    eprintln!("error: cannot read {}: {}", path.display(), err);
                    return ExitCode::from(2);
                }
            };
            let map_opt = args.mapper.map(|m| m.into());
            match Snes::new_with(&bytes, region_opt, map_opt) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: cannot load ROM: {}", run_cmd::describe_load_error(&e));
                    return ExitCode::from(2);
                }
            }
        }
        (None, true) => Snes::new_test_pattern(region_opt.unwrap_or(Region::Ntsc)),
        _ => {
            eprintln!("error: rom path or --test-pattern required");
            return ExitCode::from(2);
        }
    };

    // Run frames with panic catching.
    for _ in 0..args.frames {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| snes.run_frame()));
        if let Err(_) = result {
            eprintln!("emulator panic");
            println!("{}", run_cmd::format_debug(&snes.debug()));
            return ExitCode::from(1);
        }
    }

    // Output in order: frame_hash, state_hash, debug.
    if args.hash {
        let view = snes.frame();
        let mut h = Fnv64::new();
        h.write_u32(view.width as u32);
        h.write_u32(view.height as u32);
        for &pixel in view.pixels {
            h.write_u32(pixel);
        }
        println!("frame_hash=0x{:016x}", h.finish());
    }

    if args.state_hash {
        println!("state_hash=0x{:016x}", snes.state_hash());
    }

    if args.debug {
        println!("{}", run_cmd::format_debug(&snes.debug()));
    }

    // Dump PPM if requested.
    if let Some(path) = &args.dump_ppm {
        let view = snes.frame();
        let bytes = ppm::encode_ppm(view.width, view.height, view.pixels);
        if let Err(err) = std::fs::write(path, &bytes) {
            eprintln!("error: cannot write {}: {}", path.display(), err);
            return ExitCode::from(2);
        }
    }

    ExitCode::SUCCESS
}
