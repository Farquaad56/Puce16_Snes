use std::fs;
use std::panic::{catch_unwind, AssertUnwindSafe};

use puce16_core::hash::Fnv64;
use puce16_core::snes::Snes;
use puce16_core::Region;

use super::baseline::Entry;
use super::manifest::{Check, RomEntry, RegionSel};
use super::report::Verdict;
use crate::run_cmd::{describe_load_error, format_debug};

#[derive(Debug)]
pub struct Run {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
    pub frame_hash: u64,
    pub state_hash: u64,
    pub audio_hash: u64,
    pub ram: Vec<(u32, u8)>,
    pub ports: [u8; 4],
}

pub fn frame_hash_of(width: usize, height: usize, pixels: &[u32]) -> u64 {
    let mut h = Fnv64::new();
    h.write_u32(width as u32);
    h.write_u32(height as u32);
    for &p in pixels {
        h.write_u32(p);
    }
    h.finish()
}

pub fn audio_hash_of(samples: &[[i16; 2]]) -> u64 {
    let mut h = Fnv64::new();
    for s in samples {
        h.write_u16(s[0] as u16);
        h.write_u16(s[1] as u16);
    }
    h.finish()
}

fn region_from_sel(sel: RegionSel) -> Option<Region> {
    match sel {
        RegionSel::Auto => None,
        RegionSel::Ntsc => Some(Region::Ntsc),
        RegionSel::Pal => Some(Region::Pal),
    }
}

pub fn run_rom(e: &RomEntry) -> Result<Run, String> {
    let bytes = fs::read(&e.path).map_err(|err| format!("cannot read {}: {}", e.path, err))?;
    let region = region_from_sel(e.region);
    let mut snes = Snes::new(&bytes, region)
        .map_err(|err| format!("cannot load ROM: {}", describe_load_error(&err)))?;

    let frames = e.frames as u64;
    let mut audio_all: Vec<[i16; 2]> = Vec::new();
    for _ in 0..frames {
        catch_unwind(AssertUnwindSafe(|| snes.run_frame()))
            .map_err(|_| format!("emulator panic: {}", format_debug(&snes.debug())))?;
        let mut audio_buf: Vec<[i16; 2]> = Vec::new();
        snes.take_audio(&mut audio_buf);
        audio_all.extend(audio_buf);
    }

    let frame = snes.frame();
    let width = frame.width;
    let height = frame.height;
    let pixels: Vec<u32> = frame.pixels.to_vec();
    drop(frame);

    let frame_hash = frame_hash_of(width, height, &pixels);
    let state_hash = snes.state_hash();
    let audio_hash = audio_hash_of(&audio_all);

    let mut ram = Vec::new();
    for re in &e.expect_ram {
        if (0x7E0000..=0x7FFFFF).contains(&re.addr) {
            let val = snes.peek_wram(re.addr - 0x7E0000);
            ram.push((re.addr, val));
        } else {
            return Err(format!("{}: ram address 0x{:06X} not in WRAM", e.name, re.addr));
        }
    }

    let ports = [
        snes.apu_port(0),
        snes.apu_port(1),
        snes.apu_port(2),
        snes.apu_port(3),
    ];

    Ok(Run {
        width,
        height,
        pixels,
        frame_hash,
        state_hash,
        audio_hash,
        ram,
        ports,
    })
}

pub fn judge(e: &RomEntry, run: &Run, base: Option<&Entry>) -> Verdict {
    match e.check {
        Check::None => Verdict::Pass,
        Check::Ram => {
            for (i, re) in e.expect_ram.iter().enumerate() {
                let got = run.ram[i].1;
                if got != re.value {
                    return Verdict::Fail(format!(
                        "ram 0x{:06X}: expected 0x{:02X} got 0x{:02X}",
                        re.addr, re.value, got
                    ));
                }
            }
            Verdict::Pass
        }
        Check::Ports => {
            let expect = e.expect_ports.unwrap_or([0; 4]);
            if run.ports == expect {
                Verdict::Pass
            } else {
                let exp_str: Vec<String> = expect.iter().map(|b| format!("{:02X}", b)).collect();
                let got_str: Vec<String> = run.ports.iter().map(|b| format!("{:02X}", b)).collect();
                Verdict::Fail(format!(
                    "ports: expected [{}] got [{}]",
                    exp_str.join(", "),
                    got_str.join(", ")
                ))
            }
        }
        Check::Hash => {
            match base {
                None => Verdict::New {
                    frame_hash: run.frame_hash,
                    state_hash: run.state_hash,
                    audio_hash: run.audio_hash,
                },
                Some(b) => {
                    if b.frames != e.frames {
                        return Verdict::Fail(format!(
                            "baseline frames {} != manifest {}",
                            b.frames, e.frames
                        ));
                    }
                    let mut diffs = Vec::new();
                    if run.frame_hash != b.frame_hash {
                        diffs.push(format!(
                            "frame_hash 0x{:016x} != 0x{:016x}",
                            run.frame_hash, b.frame_hash
                        ));
                    }
                    if run.state_hash != b.state_hash {
                        diffs.push(format!(
                            "state_hash 0x{:016x} != 0x{:016x}",
                            run.state_hash, b.state_hash
                        ));
                    }
                    if run.audio_hash != b.audio_hash {
                        diffs.push(format!(
                            "audio_hash 0x{:016x} != 0x{:016x}",
                            run.audio_hash, b.audio_hash
                        ));
                    }
                    if diffs.is_empty() {
                        Verdict::Pass
                    } else {
                        Verdict::Fail(diffs.join(", "))
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use crate::suite::manifest::RamExpect;

    fn temp_dir() -> String {
        let base = std::env::temp_dir();
        let name = format!("puce16_test_{}", std::process::id());
        let path = base.join(&name);
        let _ = fs::create_dir_all(&path);
        path.to_string_lossy().to_string()
    }

    fn write_rom(dir: &str, name: &str, data: &[u8]) -> String {
        let path = format!("{}/{}", dir, name);
        let mut f = fs::File::create(&path).unwrap();
        f.write_all(data).unwrap();
        path
    }

    fn tiny_rom() -> Vec<u8> {
        let mut r = vec![0u8; 0x8000];
        r[0] = 0x80;
        r[1] = 0xFE;
        r[0x7FFC] = 0x00;
        r[0x7FFD] = 0x80;
        r
    }

    fn wram_rom() -> Vec<u8> {
        let mut r = tiny_rom();
        r[..10].copy_from_slice(&[0x78, 0x18, 0xFB, 0xA9, 0x42, 0x8D, 0x10, 0x00, 0x80, 0xFE]);
        r
    }

    fn make_entry(path: &str, frames: u32) -> RomEntry {
        RomEntry {
            name: "test".into(),
            path: path.into(),
            region: RegionSel::Auto,
            frames,
            optional: false,
            epic: "E04".into(),
            check: Check::Hash,
            expect_ram: Vec::new(),
            expect_ports: None,
        }
    }

    #[test]
    fn e07_08_frame_hash_of() {
        let h = frame_hash_of(2, 1, &[0xFF000000, 0xFFFFFFFF]);
        let mut manual = Fnv64::new();
        manual.write_u32(2);
        manual.write_u32(1);
        manual.write_u32(0xFF000000);
        manual.write_u32(0xFFFFFFFF);
        assert_eq!(h, manual.finish());

        let h2 = frame_hash_of(2, 1, &[0xFF000000, 0xFFFFFFFF]);
        assert_eq!(h, h2);

        let h3 = frame_hash_of(2, 1, &[0xFF000000, 0x00000000]);
        assert_ne!(h, h3);
    }

    #[test]
    fn e07_08_audio_hash_of() {
        let empty = audio_hash_of(&[]);
        let manual_empty = Fnv64::new().finish();
        assert_eq!(empty, manual_empty);

        let h = audio_hash_of(&[[1, -1]]);
        let mut manual = Fnv64::new();
        manual.write_u16(1u16);
        manual.write_u16((-1i16) as u16);
        assert_eq!(h, manual.finish());
    }

    #[test]
    fn e07_08_run_tiny_rom() {
        let dir = temp_dir();
        let path = write_rom(&dir, "tiny.sfc", &tiny_rom());
        let entry = make_entry(&path, 3);
        let run = run_rom(&entry).unwrap();
        assert_eq!(run.width, 256);
        assert_eq!(run.height, 224);
        assert_eq!(run.ports, [0xAA, 0xBB, 0, 0]);
        assert_eq!(run.pixels.len(), 57344);
    }

    #[test]
    fn e07_08_run_is_deterministic() {
        let dir = temp_dir();
        let path = write_rom(&dir, "tiny.sfc", &tiny_rom());
        let entry = make_entry(&path, 3);
        let run1 = run_rom(&entry).unwrap();
        let run2 = run_rom(&entry).unwrap();
        assert_eq!(run1.frame_hash, run2.frame_hash);
        assert_eq!(run1.state_hash, run2.state_hash);
        assert_eq!(run1.audio_hash, run2.audio_hash);
    }

    #[test]
    fn e07_08_run_ram_values() {
        let dir = temp_dir();
        let path = write_rom(&dir, "wram.sfc", &wram_rom());
        let mut entry = make_entry(&path, 3);
        entry.expect_ram.push(RamExpect { addr: 0x7E0010, value: 0x42 });
        let run = run_rom(&entry).unwrap();
        assert_eq!(run.ram, vec![(0x7E0010, 0x42)]);
    }

    #[test]
    fn e07_08_run_ram_bad_address() {
        let dir = temp_dir();
        let path = write_rom(&dir, "tiny.sfc", &tiny_rom());
        let mut entry = make_entry(&path, 3);
        entry.expect_ram.push(RamExpect { addr: 0x008000, value: 1 });
        let err = run_rom(&entry).unwrap_err();
        assert!(err.contains("not in WRAM"));
    }

    #[test]
    fn e07_08_run_missing_file() {
        let entry = make_entry("/nonexistent/rom.sfc", 3);
        let err = run_rom(&entry).unwrap_err();
        assert!(err.starts_with("cannot read"));
    }

    #[test]
    fn e07_08_run_bad_rom() {
        let dir = temp_dir();
        let path = write_rom(&dir, "bad.sfc", &[0u8; 100]);
        let entry = make_entry(&path, 3);
        let err = run_rom(&entry).unwrap_err();
        assert!(err.starts_with("cannot load ROM"));
    }

    #[test]
    fn e07_08_judge_none() {
        let mut entry = make_entry("", 3);
        entry.check = Check::None;
        let run = Run {
            width: 256, height: 224, pixels: vec![], frame_hash: 0, state_hash: 0, audio_hash: 0,
            ram: vec![], ports: [0; 4],
        };
        assert_eq!(judge(&entry, &run, None), Verdict::Pass);
    }

    #[test]
    fn e07_08_judge_ram() {
        let mut entry = make_entry("", 3);
        entry.check = Check::Ram;
        entry.expect_ram.push(RamExpect { addr: 0x7E0010, value: 0x42 });
        let run_ok = Run {
            width: 0, height: 0, pixels: vec![], frame_hash: 0, state_hash: 0, audio_hash: 0,
            ram: vec![(0x7E0010, 0x42)], ports: [0; 4],
        };
        assert_eq!(judge(&entry, &run_ok, None), Verdict::Pass);

        let run_fail = Run {
            width: 0, height: 0, pixels: vec![], frame_hash: 0, state_hash: 0, audio_hash: 0,
            ram: vec![(0x7E0010, 0x42)], ports: [0; 4],
        };
        let mut entry_fail = make_entry("", 3);
        entry_fail.check = Check::Ram;
        entry_fail.expect_ram.push(RamExpect { addr: 0x7E0010, value: 0x43 });
        let v = judge(&entry_fail, &run_fail, None);
        assert_eq!(v, Verdict::Fail("ram 0x7E0010: expected 0x43 got 0x42".into()));
    }

    #[test]
    fn e07_08_judge_ports() {
        let mut entry = make_entry("", 3);
        entry.check = Check::Ports;
        entry.expect_ports = Some([0xAA, 0xBB, 0, 0]);
        let run_ok = Run {
            width: 0, height: 0, pixels: vec![], frame_hash: 0, state_hash: 0, audio_hash: 0,
            ram: vec![], ports: [0xAA, 0xBB, 0, 0],
        };
        assert_eq!(judge(&entry, &run_ok, None), Verdict::Pass);

        let mut entry_fail = make_entry("", 3);
        entry_fail.check = Check::Ports;
        entry_fail.expect_ports = Some([0xAA, 0xBC, 0, 0]);
        let v = judge(&entry_fail, &run_ok, None);
        if let Verdict::Fail(msg) = v {
            assert!(msg.contains("expected [AA, BC, 00, 00] got [AA, BB, 00, 00]"));
        } else {
            panic!("expected Fail");
        }
    }

    #[test]
    fn e07_08_judge_hash_new() {
        let entry = make_entry("", 3);
        let run = Run {
            width: 0, height: 0, pixels: vec![], frame_hash: 1, state_hash: 2, audio_hash: 3,
            ram: vec![], ports: [0; 4],
        };
        match judge(&entry, &run, None) {
            Verdict::New { frame_hash, state_hash, audio_hash } => {
                assert_eq!(frame_hash, 1);
                assert_eq!(state_hash, 2);
                assert_eq!(audio_hash, 3);
            }
            other => panic!("expected New, got {:?}", other),
        }
    }

    #[test]
    fn e07_08_judge_hash_pass_and_fail() {
        let entry = make_entry("", 3);
        let run = Run {
            width: 0, height: 0, pixels: vec![], frame_hash: 1, state_hash: 2, audio_hash: 3,
            ram: vec![], ports: [0; 4],
        };
        let base_ok = Entry {
            frames: 3, frame_hash: 1, state_hash: 2, audio_hash: 3,
        };
        assert_eq!(judge(&entry, &run, Some(&base_ok)), Verdict::Pass);

        let base_bad_state = Entry {
            frames: 3, frame_hash: 1, state_hash: 99, audio_hash: 3,
        };
        match judge(&entry, &run, Some(&base_bad_state)) {
            Verdict::Fail(msg) => {
                assert!(msg.contains("state_hash"));
                assert!(!msg.contains("frame_hash"));
            }
            other => panic!("expected Fail, got {:?}", other),
        }

        let base_bad_frames = Entry {
            frames: 5, frame_hash: 1, state_hash: 2, audio_hash: 3,
        };
        match judge(&entry, &run, Some(&base_bad_frames)) {
            Verdict::Fail(msg) => {
                assert!(msg.contains("baseline frames"));
            }
            other => panic!("expected Fail, got {:?}", other),
        }
    }
}

