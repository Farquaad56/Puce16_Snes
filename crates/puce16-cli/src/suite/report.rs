
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Verdict {
    Pass,
    Fail(String),
    Skip(String),
    New { frame_hash: u64, state_hash: u64, audio_hash: u64 },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Counts {
    pub pass: u32,
    pub fail: u32,
    pub skip: u32,
    pub new: u32,
}

pub fn format_line(name: &str, v: &Verdict) -> String {
    match v {
        Verdict::Pass => format!("PASS {}", name),
        Verdict::Fail(reason) => format!("FAIL {}: {}", name, reason),
        Verdict::Skip(reason) => format!("SKIP {}: {}", name, reason),
        Verdict::New { frame_hash, state_hash, audio_hash } => {
            format!(
                "NEW {} frame_hash=0x{:016x} state_hash=0x{:016x} audio_hash=0x{:016x}",
                name, frame_hash, state_hash, audio_hash
            )
        }
    }
}

pub fn tally(vs: &[Verdict]) -> Counts {
    let mut c = Counts::default();
    for v in vs {
        match v {
            Verdict::Pass => c.pass += 1,
            Verdict::Fail(_) => c.fail += 1,
            Verdict::Skip(_) => c.skip += 1,
            Verdict::New { .. } => c.new += 1,
        }
    }
    c
}

pub fn summary(c: &Counts) -> String {
    format!("suite: {} pass, {} fail, {} skip, {} new", c.pass, c.fail, c.skip, c.new)
}

pub fn exit_code(c: &Counts) -> i32 {
    if c.fail > 0 { 1 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e07_06_lines() {
        assert_eq!(format_line("a", &Verdict::Pass), "PASS a");
        assert_eq!(
            format_line("a", &Verdict::Fail("ram 0x7E0010: expected 0x42 got 0x00".into())),
            "FAIL a: ram 0x7E0010: expected 0x42 got 0x00"
        );
        assert_eq!(format_line("a", &Verdict::Skip("file missing".into())), "SKIP a: file missing");
    }

    #[test]
    fn e07_06_new_line() {
        let v = Verdict::New { frame_hash: 1, state_hash: 2, audio_hash: 0xABC };
        assert_eq!(
            format_line("a", &v),
            "NEW a frame_hash=0x0000000000000001 state_hash=0x0000000000000002 audio_hash=0x0000000000000abc"
        );
    }

    #[test]
    fn e07_06_tally() {
        let vs = vec![
            Verdict::Pass,
            Verdict::Pass,
            Verdict::Fail("x".into()),
            Verdict::Skip("y".into()),
            Verdict::New { frame_hash: 0, state_hash: 0, audio_hash: 0 },
        ];
        let c = tally(&vs);
        assert_eq!(c.pass, 2);
        assert_eq!(c.fail, 1);
        assert_eq!(c.skip, 1);
        assert_eq!(c.new, 1);
    }

    #[test]
    fn e07_06_summary() {
        let c = Counts { pass: 3, fail: 0, skip: 1, new: 2 };
        assert_eq!(summary(&c), "suite: 3 pass, 0 fail, 1 skip, 2 new");
    }

    #[test]
    fn e07_06_exit_codes() {
        let c_fail = Counts { pass: 1, fail: 1, skip: 0, new: 0 };
        assert_eq!(exit_code(&c_fail), 1);
        let c_ok = Counts { pass: 2, fail: 0, skip: 1, new: 1 };
        assert_eq!(exit_code(&c_ok), 0);
        assert_eq!(exit_code(&Counts::default()), 0);
    }
}
