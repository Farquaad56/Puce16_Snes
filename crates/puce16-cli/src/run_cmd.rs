//! Pure helpers for the `run` subcommand (E03-44).

use puce16_core::cart::CartError;
use puce16_core::snes::{DebugSnapshot, LoadError};

/// Human-readable description of a ROM load error.
pub fn describe_load_error(e: &LoadError) -> String {
    match e {
        LoadError::Cart(CartError::TooSmall) => "ROM too small".to_string(),
        LoadError::Cart(CartError::NoHeader) => "no valid ROM header found".to_string(),
        LoadError::Cart(CartError::UnsupportedCoprocessor(name)) => {
            format!("unsupported coprocessor: {}", name)
        }
    }
}

/// Format a debug snapshot for stdout.
pub fn format_debug(d: &DebugSnapshot) -> String {
    let e = if d.e { 1 } else { 0 };
    format!(
        "debug: pc={:02X}:{:04X} a={:04X} x={:04X} y={:04X} s={:04X} d={:04X} db={:02X} p={:02X} e={} v={} dot={} master={} frame={}",
        d.pb, d.pc, d.a, d.x, d.y, d.s, d.d, d.db, d.p, e, d.v, d.dot, d.master, d.frame
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e03_44_format_debug() {
        let d = DebugSnapshot {
            pc: 0x8000, pb: 0, a: 0x1234, x: 0xABCD, y: 0, s: 0x01FF,
            d: 0, db: 0x7E, p: 0x34, e: true, v: 225, dot: 3, master: 306905, frame: 1,
        };
        let s = format_debug(&d);
        assert_eq!(s, "debug: pc=00:8000 a=1234 x=ABCD y=0000 s=01FF d=0000 db=7E p=34 e=1 v=225 dot=3 master=306905 frame=1");

        let d2 = DebugSnapshot {
            pc: 0x0ABC, pb: 0x12, a: 0, x: 0, y: 0, s: 0, d: 0, db: 0, p: 0,
            e: false, v: 0, dot: 0, master: 0, frame: 0,
        };
        let s2 = format_debug(&d2);
        assert!(s2.starts_with("debug: pc=12:0ABC"));
        assert!(s2.contains(" e=0 "));
    }

    #[test]
    fn e03_44_describe_errors() {
        let too_small = LoadError::Cart(CartError::TooSmall);
        assert_eq!(describe_load_error(&too_small), "ROM too small");

        let no_header = LoadError::Cart(CartError::NoHeader);
        assert_eq!(describe_load_error(&no_header), "no valid ROM header found");

        let coproc = LoadError::Cart(CartError::UnsupportedCoprocessor("GSU"));
        assert_eq!(describe_load_error(&coproc), "unsupported coprocessor: GSU");
    }
}
