//! Puce16 SNES emulator command line interface (E01-07).
//!
//! `run` executes frames of the built-in test pattern and can print a frame
//! hash or dump the last frame as a PPM image. ROM loading arrives in E03.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{CommandFactory, Parser, Subcommand};
use puce16_core::hash::Fnv64;
use puce16_core::{Region, Snes};

mod ppm;

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
    /// Run frames of the test pattern (ROM loading is not supported yet).
    Run(RunArgs),
}

/// Arguments for `run` (E01-07: test pattern only).
#[derive(Debug, Parser)]
struct RunArgs {
    /// Use the built-in test pattern instead of a ROM.
    #[arg(long)]
    test_pattern: bool,

    /// Number of frames to run.
    #[arg(long, default_value_t = 1)]
    frames: u64,

    /// Print `frame_hash=0x<16 hex>` for the last frame.
    #[arg(long)]
    hash: bool,

    /// Video region.
    #[arg(long, value_enum, default_value_t = RegionOpt::Ntsc)]
    region: RegionOpt,

    /// Write the PPM (P6) of the last frame to this file.
    #[arg(long)]
    dump_ppm: Option<PathBuf>,
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
        None => {
            // No subcommand given: display the help.
            println!("{}", Cli::command().render_help());
            ExitCode::SUCCESS
        }
    }
}

/// Execute `run`; returns the process exit code (0 OK, 2 usage/file error).
fn run_command(args: &RunArgs) -> ExitCode {
    if !args.test_pattern {
        // ROM loading is not implemented yet (E03).
        eprintln!("error: rom path not supported yet");
        return ExitCode::from(2);
    }

    let mut snes = Snes::new_test_pattern(args.region.into());
    for _ in 0..args.frames {
        snes.run_frame();
    }

    if args.hash {
        // FNV-1a 64 of width, height (u32 little-endian) then the pixels of
        // frame() as u32 little-endian, row by row.
        let view = snes.frame();
        let mut h = Fnv64::new();
        h.write_u32(view.width as u32);
        h.write_u32(view.height as u32);
        for &pixel in view.pixels {
            h.write_u32(pixel);
        }
        println!("frame_hash=0x{:016x}", h.finish());
    }

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
