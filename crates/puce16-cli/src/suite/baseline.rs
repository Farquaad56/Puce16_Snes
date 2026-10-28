use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Entry {
    pub frames: u32,
    pub frame_hash: u64,
    pub state_hash: u64,
    pub audio_hash: u64,
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Baseline {
    pub roms: BTreeMap<String, Entry>,
}

pub fn hex64(v: u64) -> String {
    format!("0x{:016x}", v)
}

pub fn parse_hex64(s: &str) -> Option<u64> {
    if s.len() != 18 || !s.starts_with("0x") && !s.starts_with("0X") {
        return None;
    }
    let digits = &s[2..];
    u64::from_str_radix(digits, 16).ok()
}

impl Baseline {
    pub fn parse(text: &str) -> Result<Baseline, String> {
        let val: serde_json::Value =
            serde_json::from_str(text.trim()).map_err(|e| format!("invalid baseline: {}", e))?;
        if !val.is_object() {
            return Err("invalid baseline: not an object".to_string());
        }
        let obj = val.as_object().unwrap();
        let version = obj.get("version")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| "invalid baseline: missing version".to_string())?;
        if version != 1 {
            return Err(format!("unsupported baseline version {}", version));
        }
        let roms_val = obj.get("roms")
            .and_then(|v| v.as_object())
            .ok_or_else(|| "invalid baseline: missing roms".to_string())?;
        let mut roms = BTreeMap::new();
        for (name, entry_val) in roms_val {
            if !entry_val.is_object() {
                return Err(format!("{}: not an object", name));
            }
            let eobj = entry_val.as_object().unwrap();
            let frames = eobj.get("frames")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| format!("{}: missing frames", name))? as u32;
            let frame_hash_s = eobj.get("frame_hash")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("{}: missing frame_hash", name))?;
            let state_hash_s = eobj.get("state_hash")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("{}: missing state_hash", name))?;
            let audio_hash_s = eobj.get("audio_hash")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("{}: missing audio_hash", name))?;
            let frame_hash = parse_hex64(frame_hash_s)
                .ok_or_else(|| format!("{}: bad frame_hash '{}'", name, frame_hash_s))?;
            let state_hash = parse_hex64(state_hash_s)
                .ok_or_else(|| format!("{}: bad state_hash '{}'", name, state_hash_s))?;
            let audio_hash = parse_hex64(audio_hash_s)
                .ok_or_else(|| format!("{}: bad audio_hash '{}'", name, audio_hash_s))?;
            roms.insert(name.clone(), Entry { frames, frame_hash, state_hash, audio_hash });
        }
        Ok(Baseline { roms })
    }

    pub fn to_json(&self) -> String {
        let mut parts = Vec::new();
        parts.push("{\n  \"version\": 1".to_string());
        if self.roms.is_empty() {
            parts.push(",\n  \"roms\": {}\n}".to_string());
        } else {
            parts.push(",\n  \"roms\": {\n".to_string());
            let mut first = true;
            for (name, entry) in &self.roms {
                if !first {
                    parts.push(",\n".to_string());
                }
                first = false;
                parts.push(format!("    \"{}\": {{\n", name));
                parts.push(format!("      \"frames\": {},\n", entry.frames));
                parts.push(format!("      \"frame_hash\": \"{}\",\n", hex64(entry.frame_hash)));
                parts.push(format!("      \"state_hash\": \"{}\",\n", hex64(entry.state_hash)));
                parts.push(format!("      \"audio_hash\": \"{}\"\n", hex64(entry.audio_hash)));
                parts.push("    }".to_string());
            }
            parts.push("\n  }\n}".to_string());
        }
        let s = parts.join("");
        format!("{}\n", s)
    }

    pub fn load(path: &Path) -> Result<Baseline, String> {
        if !path.exists() {
            return Ok(Baseline::default());
        }
        let text = fs::read_to_string(path)
            .map_err(|e| format!("invalid baseline: cannot read {}: {}", path.display(), e))?;
        Baseline::parse(&text)
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, self.to_json())
            .map_err(|e| format!("cannot write {}: {}", tmp.display(), e))?;
        fs::rename(&tmp, path)
            .map_err(|e| format!("cannot rename {} to {}: {}", tmp.display(), path.display(), e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn e07_05_hex64() {
        assert_eq!(hex64(0), "0x0000000000000000");
        assert_eq!(hex64(0xDEADBEEF), "0x00000000deadbeef");
        assert_eq!(parse_hex64("0x00000000DEADBEEF"), Some(0xDEADBEEF));
        assert_eq!(parse_hex64("0xDEADBEEF"), None);
        assert_eq!(parse_hex64("deadbeef"), None);
    }

    #[test]
    fn e07_05_roundtrip() {
        let mut b = Baseline::default();
        b.roms.insert("a".to_string(), Entry { frames: 120, frame_hash: 0x11, state_hash: 0x22, audio_hash: 0x33 });
        b.roms.insert("b".to_string(), Entry { frames: 240, frame_hash: 0x44, state_hash: 0x55, audio_hash: 0x66 });
        let json = b.to_json();
        let parsed = Baseline::parse(&json).unwrap();
        assert_eq!(parsed, b);
    }

    #[test]
    fn e07_05_json_exact() {
        let b = Baseline::default();
        assert_eq!(b.to_json(), "{\n  \"version\": 1,\n  \"roms\": {}\n}\n");
    }

    #[test]
    fn e07_05_keys_sorted() {
        let mut b = Baseline::default();
        b.roms.insert("b".to_string(), Entry { frames: 120, frame_hash: 0, state_hash: 0, audio_hash: 0 });
        b.roms.insert("a".to_string(), Entry { frames: 120, frame_hash: 0, state_hash: 0, audio_hash: 0 });
        let json = b.to_json();
        let a_pos = json.find("\"a\"").unwrap();
        let b_pos = json.find("\"b\"").unwrap();
        assert!(a_pos < b_pos);
    }

    #[test]
    fn e07_05_entry_field_order() {
        let mut b = Baseline::default();
        b.roms.insert("x".to_string(), Entry { frames: 120, frame_hash: 0, state_hash: 0, audio_hash: 0 });
        let json = b.to_json();
        let frames_pos = json.find("\"frames\"").unwrap();
        let frame_hash_pos = json.find("\"frame_hash\"").unwrap();
        let state_hash_pos = json.find("\"state_hash\"").unwrap();
        let audio_hash_pos = json.find("\"audio_hash\"").unwrap();
        assert!(frames_pos < frame_hash_pos);
        assert!(frame_hash_pos < state_hash_pos);
        assert!(state_hash_pos < audio_hash_pos);
    }

    #[test]
    fn e07_05_bad_version() {
        let err = Baseline::parse("{\"version\":2,\"roms\":{}}").unwrap_err();
        assert!(err.contains("unsupported baseline version"));
    }

    #[test]
    fn e07_05_bad_hash() {
        let json = "{\"version\":1,\"roms\":{\"x\":{\"frames\":120,\"frame_hash\":\"0x12\",\"state_hash\":\"0x0000000000000000\",\"audio_hash\":\"0x0000000000000000\"}}}";
        let err = Baseline::parse(json).unwrap_err();
        assert!(err.contains("x"));
    }

    #[test]
    fn e07_05_bad_json() {
        let err = Baseline::parse("not json").unwrap_err();
        assert!(err.contains("invalid baseline"));
    }

    #[test]
    fn e07_05_load_missing_is_empty() {
        let dir = std::env::temp_dir().join(format!("baseline_test_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("missing.json");
        let b = Baseline::load(&path).unwrap();
        assert_eq!(b, Baseline::default());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn e07_05_save_load_atomic() {
        let dir = std::env::temp_dir().join(format!("baseline_test2_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("baseline.json");
        let mut b = Baseline::default();
        b.roms.insert("test".to_string(), Entry { frames: 120, frame_hash: 0xABCD, state_hash: 0x1234, audio_hash: 0x5678 });
        b.save(&path).unwrap();
        let tmp = path.with_extension("tmp");
        assert!(!tmp.exists());
        let loaded = Baseline::load(&path).unwrap();
        assert_eq!(loaded, b);
        fs::remove_dir_all(&dir).ok();
    }
}

