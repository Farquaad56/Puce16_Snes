//! E05-19: PPU scene harness. Frame hashes are compared with the values
//! computed by the independent reference renderer (docs/tools/ppu_ref.py).

use puce16_core::hash::Fnv64;
use puce16_core::ppu::Ppu;
use puce16_core::region::Region;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

enum Load {
    Vram(u16, Vec<u16>),
    Cgram(u8, Vec<u16>),
    Oam(u16, Vec<u8>),
    Reg(u8, u8),
}

struct Scene {
    name: String,
    frames: u32,
    loads: Vec<Load>,
    // (line, reg, value)
    ats: Vec<(u32, u8, u8)>,
}

struct Expected {
    width: usize,
    height: usize,
    hash: String,
}

fn data_path(file: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("data").join(file)
}

fn hex(tok: &str, line: usize) -> u32 {
    u32::from_str_radix(tok, 16)
        .unwrap_or_else(|_| panic!("ppu_scenes.txt line {}: bad hex value '{}'", line, tok))
}

fn dec(tok: &str, line: usize) -> u32 {
    tok.parse::<u32>()
        .unwrap_or_else(|_| panic!("ppu_scenes.txt line {}: bad decimal value '{}'", line, tok))
}

fn parse_scenes(text: &str) -> Vec<Scene> {
    let mut scenes: Vec<Scene> = Vec::new();
    let mut cur: Option<Scene> = None;
    for (i, raw) in text.lines().enumerate() {
        let ln = i + 1;
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let t: Vec<&str> = line.split_whitespace().collect();
        match t[0] {
            "scene" => {
                if cur.is_some() {
                    panic!("ppu_scenes.txt line {}: 'scene' inside a scene", ln);
                }
                if t.len() != 3 {
                    panic!("ppu_scenes.txt line {}: expected 'scene <name> <frames>'", ln);
                }
                let frames = dec(t[2], ln);
                if frames != 1 && frames != 2 {
                    panic!("ppu_scenes.txt line {}: frames must be 1 or 2", ln);
                }
                cur = Some(Scene { name: t[1].to_string(), frames, loads: Vec::new(), ats: Vec::new() });
            }
            "end" => match cur.take() {
                Some(s) => scenes.push(s),
                None => panic!("ppu_scenes.txt line {}: 'end' outside a scene", ln),
            },
            "vram" | "cgram" | "oam" | "reg" | "at" => {
                let s = match cur.as_mut() {
                    Some(s) => s,
                    None => panic!("ppu_scenes.txt line {}: '{}' outside a scene", ln, t[0]),
                };
                match t[0] {
                    "vram" => {
                        if t.len() < 3 {
                            panic!("ppu_scenes.txt line {}: 'vram' needs an address and words", ln);
                        }
                        let addr = hex(t[1], ln) as u16;
                        let w: Vec<u16> = t[2..].iter().map(|x| hex(x, ln) as u16).collect();
                        s.loads.push(Load::Vram(addr, w));
                    }
                    "cgram" => {
                        if t.len() < 3 {
                            panic!("ppu_scenes.txt line {}: 'cgram' needs an index and words", ln);
                        }
                        let idx = dec(t[1], ln) as u8;
                        let w: Vec<u16> = t[2..].iter().map(|x| hex(x, ln) as u16).collect();
                        s.loads.push(Load::Cgram(idx, w));
                    }
                    "oam" => {
                        if t.len() < 3 {
                            panic!("ppu_scenes.txt line {}: 'oam' needs an address and bytes", ln);
                        }
                        let addr = dec(t[1], ln) as u16;
                        if addr % 2 != 0 {
                            panic!("ppu_scenes.txt line {}: oam address must be even", ln);
                        }
                        let b: Vec<u8> = t[2..].iter().map(|x| hex(x, ln) as u8).collect();
                        s.loads.push(Load::Oam(addr, b));
                    }
                    "reg" => {
                        if t.len() != 3 {
                            panic!("ppu_scenes.txt line {}: expected 'reg <reg> <val>'", ln);
                        }
                        s.loads.push(Load::Reg(hex(t[1], ln) as u8, hex(t[2], ln) as u8));
                    }
                    _ => {
                        if t.len() != 4 {
                            panic!("ppu_scenes.txt line {}: expected 'at <line> <reg> <val>'", ln);
                        }
                        s.ats.push((dec(t[1], ln), hex(t[2], ln) as u8, hex(t[3], ln) as u8));
                    }
                }
            }
            other => panic!("ppu_scenes.txt line {}: unknown directive '{}'", ln, other),
        }
    }
    if cur.is_some() {
        panic!("ppu_scenes.txt: last scene has no 'end'");
    }
    scenes
}

fn load_scenes() -> Vec<Scene> {
    let text = fs::read_to_string(data_path("ppu_scenes.txt")).expect("cannot read tests/data/ppu_scenes.txt");
    parse_scenes(&text)
}

fn load_expected() -> BTreeMap<String, Expected> {
    let text = fs::read_to_string(data_path("ppu_scenes.expected")).expect("cannot read tests/data/ppu_scenes.expected");
    let mut m = BTreeMap::new();
    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let t: Vec<&str> = line.split_whitespace().collect();
        if t.len() != 4 {
            panic!("ppu_scenes.expected line {}: expected 'name width height hash'", i + 1);
        }
        let w: usize = t[1].parse().unwrap_or_else(|_| panic!("ppu_scenes.expected line {}: bad width", i + 1));
        let h: usize = t[2].parse().unwrap_or_else(|_| panic!("ppu_scenes.expected line {}: bad height", i + 1));
        m.insert(t[0].to_string(), Expected { width: w, height: h, hash: t[3].to_lowercase() });
    }
    m
}

fn apply_load(p: &mut Ppu, l: &Load) {
    match l {
        Load::Vram(addr, words) => {
            p.write(0x15, 0x80);
            p.write(0x16, (*addr & 0xFF) as u8);
            p.write(0x17, (*addr >> 8) as u8);
            for w in words {
                p.write(0x18, (*w & 0xFF) as u8);
                p.write(0x19, (*w >> 8) as u8);
            }
        }
        Load::Cgram(idx, words) => {
            p.write(0x21, *idx);
            for w in words {
                p.write(0x22, (*w & 0xFF) as u8);
                p.write(0x22, (*w >> 8) as u8);
            }
        }
        Load::Oam(addr, bytes) => {
            let a = *addr / 2;
            p.write(0x02, (a & 0xFF) as u8);
            p.write(0x03, ((a >> 8) & 1) as u8);
            for b in bytes {
                p.write(0x04, *b);
            }
        }
        Load::Reg(r, v) => p.write(*r, *v),
    }
}

/// Plays one scene; returns (width, height, hash as 16 hex digits).
fn run_scene(s: &Scene) -> (usize, usize, String) {
    let mut p = Ppu::new(Region::Ntsc);
    for l in &s.loads {
        apply_load(&mut p, l);
    }
    let n: u64 = 225 * 340 + 1 + (s.frames as u64 - 1) * 262 * 340;
    for step in 0..n {
        let line = ((step / 340) % 262) as u32;
        let dot = step % 340;
        if dot == 0 {
            for (l, r, v) in &s.ats {
                if *l == line {
                    p.write(*r, *v);
                }
            }
        }
        p.step_dot();
    }
    let f = p.frame();
    let mut h = Fnv64::new();
    h.write_u32(f.width as u32);
    h.write_u32(f.height as u32);
    h.write_u32_slice(f.pixels);
    write_ppm(&s.name, f.width, f.height, f.pixels);
    (f.width, f.height, format!("{:016x}", h.finish()))
}

fn ppm_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("out")
        .join("ppu_scenes")
        .join(format!("{}.ppm", name))
}

fn write_ppm(name: &str, w: usize, h: usize, pixels: &[u32]) {
    let path = ppm_path(name);
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).expect("cannot create tests/out/ppu_scenes");
    }
    let mut data: Vec<u8> = format!("P6\n{} {}\n255\n", w, h).into_bytes();
    for &px in pixels {
        data.push(((px >> 16) & 0xFF) as u8);
        data.push(((px >> 8) & 0xFF) as u8);
        data.push((px & 0xFF) as u8);
    }
    fs::write(&path, data).expect("cannot write ppm");
}

#[test]
fn e05_19_scene_file_parses() {
    let scenes = load_scenes();
    let expected = load_expected();
    assert!(scenes.len() >= 10, "only {} scenes", scenes.len());
    for s in &scenes {
        assert!(expected.contains_key(&s.name), "scene '{}' has no expected line", s.name);
    }
    for name in expected.keys() {
        assert!(scenes.iter().any(|s| &s.name == name), "expected line '{}' has no scene", name);
    }
}

#[test]
fn e05_19_all_scenes_match() {
    let scenes = load_scenes();
    let expected = load_expected();
    let mut errors: Vec<String> = Vec::new();
    for s in &scenes {
        let e = match expected.get(&s.name) {
            Some(e) => e,
            None => {
                errors.push(format!("{}: no expected line", s.name));
                continue;
            }
        };
        let (w, h, hash) = run_scene(s);
        if w != e.width || h != e.height || hash != e.hash {
            errors.push(format!(
                "{}: expected {}x{} {}, got {}x{} {}",
                s.name, e.width, e.height, e.hash, w, h, hash
            ));
        }
    }
    assert!(errors.is_empty(), "{} scene(s) differ:\n{}", errors.len(), errors.join("\n"));
}

#[test]
fn e05_19_scenes_deterministic() {
    let scenes = load_scenes();
    let a = run_scene(&scenes[0]);
    let b = run_scene(&scenes[0]);
    assert_eq!(a, b);
}

#[test]
fn e05_19_ppm_written() {
    let scenes = load_scenes();
    let first = &scenes[0];
    let (w, h, _) = run_scene(first);
    let data = fs::read(ppm_path(&first.name)).expect("ppm not written");
    assert!(data.starts_with(b"P6\n"));
    let header = format!("P6\n{} {}\n255\n", w, h).len();
    assert_eq!(data.len(), header + w * h * 3);
}
