use serde::Deserialize;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RegionSel { Auto, Ntsc, Pal }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Check { Hash, Ram, Ports, None }

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RamExpect { pub addr: u32, pub value: u8 }

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RomEntry {
    pub name: String,
    pub path: String,
    pub region: RegionSel,
    pub frames: u32,
    pub optional: bool,
    pub epic: String,
    pub check: Check,
    pub expect_ram: Vec<RamExpect>,
    pub expect_ports: Option<[u8; 4]>,
}

#[derive(Deserialize)]
struct Raw { rom: Option<Vec<RawRom>> }

#[derive(Deserialize)]
struct RawRam { addr: u64, value: u64 }

#[derive(Deserialize)]
struct RawRom {
    name: String,
    path: String,
    region: Option<String>,
    frames: Option<u32>,
    optional: Option<bool>,
    epic: Option<String>,
    check: Option<String>,
    expect_ram: Option<Vec<RawRam>>,
    expect_ports: Option<Vec<u64>>,
}

pub fn parse(text: &str) -> Result<Vec<RomEntry>, String> {
    let raw: Raw = toml::from_str(text).map_err(|e| format!("invalid manifest: {}", e))?;
    let roms = raw.rom.unwrap_or_default();
    let mut out: Vec<RomEntry> = Vec::with_capacity(roms.len());
    for r in roms {
        let name = r.name.clone();
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
            return Err(format!("{}: invalid name", name));
        }
        if out.iter().any(|e| e.name == name) {
            return Err(format!("{}: duplicate", name));
        }
        let frames = r.frames.unwrap_or(120);
        if !(1..=100000).contains(&frames) {
            return Err(format!("{}: frames out of range", name));
        }
        let epic = r.epic.unwrap_or_default();
        if !epic_number(&epic).is_some() {
            return Err(format!("{}: invalid epic '{}'", name, epic));
        }
        let region = match r.region.as_deref().unwrap_or("auto") {
            "auto" => RegionSel::Auto,
            "ntsc" => RegionSel::Ntsc,
            "pal" => RegionSel::Pal,
            other => return Err(format!("{}: invalid region '{}'", name, other)),
        };
        let check = match r.check.as_deref().unwrap_or("hash") {
            "hash" => Check::Hash,
            "ram" => Check::Ram,
            "ports" => Check::Ports,
            "none" => Check::None,
            other => return Err(format!("{}: invalid check '{}'", name, other)),
        };
        let expect_ram = match r.expect_ram {
            Some(list) => list.into_iter().map(|re| {
                if re.addr > 0xFFFFFF || re.value > 255 {
                    Err(format!("{}: bad expect_ram entry", name))
                } else {
                    Ok(RamExpect { addr: re.addr as u32, value: re.value as u8 })
                }
            }).collect::<Result<Vec<_>, _>>()?,
            None => Vec::new(),
        };
        if check == Check::Ram && expect_ram.is_empty() {
            return Err(format!("{}: check=ram requires expect_ram", name));
        }
        let expect_ports = match r.expect_ports {
            Some(vals) if vals.len() != 4 => return Err(format!("{}: expect_ports must have 4 values", name)),
            Some(vals) => {
                for v in &vals {
                    if *v > 255 {
                        return Err(format!("{}: bad expect_ports value", name));
                    }
                }
                let mut arr = [0u8; 4];
                for (j, v) in vals.iter().enumerate() {
                    arr[j] = *v as u8;
                }
                Some(arr)
            },
            None => None,
        };
        if check == Check::Ports && expect_ports.is_none() {
            return Err(format!("{}: check=ports requires expect_ports", name));
        }
        out.push(RomEntry {
            name, path: r.path, region, frames, optional: r.optional.unwrap_or(false),
            epic, check, expect_ram, expect_ports,
        });
    }
    Ok(out)
}

pub fn epic_number(s: &str) -> Option<u32> {
    if s.len() != 3 || !s.starts_with('E') { return None; }
    let digits = &s[1..];
    if !digits.chars().all(|c| c.is_ascii_digit()) { return None; }
    let n: u32 = digits.parse().ok()?;
    if n < 1 || n > 99 { return None; }
    Some(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, path: &str) -> String {
        format!("[[rom]]\nname = \"{}\"\npath = \"{}\"\nframes = 120\nepic = \"E04\"", name, path)
    }

    #[test]
    fn e07_04_parse_full() {
        let text = format!(
            "{}\ncheck = \"ram\"\nexpect_ram = [ {{ addr = 0x7E0010, value = 0x42 }} ]\n\n{}\ncheck = \"ports\"\nexpect_ports = [0xAA, 0xBB, 0, 0]\n\n{}\noptional = true\nregion = \"pal\"",
            entry("a", "p/a.sfc"), entry("b", "p/b.sfc"), entry("c", "p/c.sfc")
        );
        let entries = parse(&text).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].name, "a");
        assert_eq!(entries[0].check, Check::Ram);
        assert_eq!(entries[0].expect_ram.len(), 1);
        assert_eq!(entries[0].expect_ram[0].addr, 0x7E0010);
        assert_eq!(entries[0].expect_ram[0].value, 0x42);
        assert_eq!(entries[1].name, "b");
        assert_eq!(entries[1].check, Check::Ports);
        assert_eq!(entries[1].expect_ports, Some([0xAA, 0xBB, 0, 0]));
        assert_eq!(entries[2].name, "c");
        assert!(entries[2].optional);
        assert_eq!(entries[2].region, RegionSel::Pal);
    }

    #[test]
    fn e07_04_defaults() {
        let text = entry("a", "p/a.sfc");
        let entries = parse(&text).unwrap();
        let e = &entries[0];
        assert_eq!(e.region, RegionSel::Auto);
        assert!(!e.optional);
        assert_eq!(e.check, Check::Hash);
        assert!(e.expect_ram.is_empty());
        assert_eq!(e.expect_ports, None);
    }

    #[test]
    fn e07_04_empty_manifest() {
        let entries = parse("").unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn e07_04_bad_toml() {
        let err = parse("[[rom").unwrap_err();
        assert!(err.contains("invalid manifest"));
    }

    #[test]
    fn e07_04_duplicate_name() {
        let text = format!("{}\n\n{}", entry("a", "p/a.sfc"), entry("a", "p/b.sfc"));
        let err = parse(&text).unwrap_err();
        assert!(err.contains("a: duplicate"));
    }

    #[test]
    fn e07_04_bad_name() {
        let text = format!("[[rom]]\nname = \"Bad-Name\"\npath = \"p/a.sfc\"\nframes = 120\nepic = \"E04\"");
        let err = parse(&text).unwrap_err();
        assert!(err.contains("Bad-Name"));
    }

    #[test]
    fn e07_04_bad_frames() {
        for frames in [0, 100001] {
            let text = format!("[[rom]]\nname = \"a\"\npath = \"p/a.sfc\"\nframes = {}\nepic = \"E04\"", frames);
            parse(&text).unwrap_err();
        }
    }

    #[test]
    fn e07_04_bad_epic() {
        for epic in ["E4", "e04", "E100"] {
            let text = format!("[[rom]]\nname = \"a\"\npath = \"p/a.sfc\"\nframes = 120\nepic = \"{}\"", epic);
            parse(&text).unwrap_err();
        }
        let text = entry("a", "p/a.sfc");
        parse(&text).unwrap();
    }

    #[test]
    fn e07_04_bad_enums() {
        let text = format!("[[rom]]\nname = \"a\"\npath = \"p/a.sfc\"\nframes = 120\nepic = \"E04\"\nregion = \"jp\"");
        parse(&text).unwrap_err();
        let text = format!("[[rom]]\nname = \"a\"\npath = \"p/a.sfc\"\nframes = 120\nepic = \"E04\"\ncheck = \"crc\"");
        parse(&text).unwrap_err();
    }

    #[test]
    fn e07_04_ram_needs_expect() {
        let text = format!("[[rom]]\nname = \"a\"\npath = \"p/a.sfc\"\nframes = 120\nepic = \"E04\"\ncheck = \"ram\"");
        let err = parse(&text).unwrap_err();
        assert!(err.contains("expect_ram"));
        let text = format!("[[rom]]\nname = \"a\"\npath = \"p/a.sfc\"\nframes = 120\nepic = \"E04\"\ncheck = \"ram\"\nexpect_ram = [ {{ addr = 0x1000000, value = 0 }} ]");
        parse(&text).unwrap_err();
    }

    #[test]
    fn e07_04_ports_needs_four() {
        let text = format!("[[rom]]\nname = \"a\"\npath = \"p/a.sfc\"\nframes = 120\nepic = \"E04\"\ncheck = \"ports\"\nexpect_ports = [0, 0, 0]");
        let err = parse(&text).unwrap_err();
        assert!(err.contains("expect_ports"));
    }

    #[test]
    fn e07_04_epic_number() {
        assert_eq!(epic_number("E04"), Some(4));
        assert_eq!(epic_number("E10"), Some(10));
        assert_eq!(epic_number("E4"), None);
        assert_eq!(epic_number("e04"), None);
        assert_eq!(epic_number("E100"), None);
    }
}
