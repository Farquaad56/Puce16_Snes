use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct SpcState {
    pub pc: u16,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub psw: u8,
    pub ram: Vec<(u16, u8)>,
}

#[derive(Debug, Deserialize)]
pub struct CycleRec {
    pub addr: Option<u16>,
    pub val: Option<u8>,
    pub kind: String,
}

#[derive(Debug, Deserialize)]
pub struct Case {
    pub name: String,
    pub initial: SpcState,
    #[serde(rename = "final")]
    pub fin: SpcState,
    pub cycles: Vec<CycleRec>,
}

fn parse_cycle(raw: &[serde_json::Value]) -> Result<CycleRec, String> {
    if raw.len() != 3 {
        return Err(format!("cycle must have 3 elements, got {}", raw.len()));
    }
    let addr = match &raw[0] {
        serde_json::Value::Null => None,
        serde_json::Value::Number(n) => Some(n.as_u64().unwrap() as u16),
        _ => return Err("cycle addr must be number or null".to_string()),
    };
    let val = match &raw[1] {
        serde_json::Value::Null => None,
        serde_json::Value::Number(n) => Some(n.as_u64().unwrap() as u8),
        _ => return Err("cycle val must be number or null".to_string()),
    };
    let kind = match &raw[2] {
        serde_json::Value::String(s) => s.clone(),
        _ => return Err("cycle kind must be string".to_string()),
    };
    Ok(CycleRec { addr, val, kind })
}

fn parse_case(raw: serde_json::Value) -> Result<Case, String> {
    let obj = raw.as_object().ok_or("case must be object")?;
    let name = obj.get("name").and_then(|v| v.as_str()).ok_or("missing name")?.to_string();

    let initial_obj = obj.get("initial").and_then(|v| v.as_object()).ok_or("missing initial")?;
    let final_obj = obj.get("final").and_then(|v| v.as_object()).ok_or("missing final")?;

    fn parse_state(obj: &serde_json::Map<String, serde_json::Value>, label: &str) -> Result<SpcState, String> {
        let pc = obj.get("pc").and_then(|v| v.as_u64()).map(|n| n as u16).ok_or(format!("missing {}.pc", label))?;
        let a = obj.get("a").and_then(|v| v.as_u64()).map(|n| n as u8).ok_or(format!("missing {}.a", label))?;
        let x = obj.get("x").and_then(|v| v.as_u64()).map(|n| n as u8).ok_or(format!("missing {}.x", label))?;
        let y = obj.get("y").and_then(|v| v.as_u64()).map(|n| n as u8).ok_or(format!("missing {}.y", label))?;
        let sp = obj.get("sp").and_then(|v| v.as_u64()).map(|n| n as u8).ok_or(format!("missing {}.sp", label))?;
        let psw = obj.get("psw").and_then(|v| v.as_u64()).map(|n| n as u8).ok_or(format!("missing {}.psw", label))?;

        let ram_raw = obj.get("ram").and_then(|v| v.as_array()).ok_or(format!("missing {}.ram", label))?;
        let mut ram = Vec::with_capacity(ram_raw.len());
        for pair in ram_raw {
            let arr = pair.as_array().ok_or(format!("{}.ram entry must be array", label))?;
            if arr.len() != 2 {
                return Err(format!("{}.ram entry must have 2 elements", label));
            }
            let addr = arr[0].as_u64().map(|n| n as u16).ok_or(format!("{}.ram addr must be number", label))?;
            let val = arr[1].as_u64().map(|n| n as u8).ok_or(format!("{}.ram val must be number", label))?;
            ram.push((addr, val));
        }

        Ok(SpcState { pc, a, x, y, sp, psw, ram })
    }

    let initial = parse_state(initial_obj, "initial")?;
    let fin = parse_state(final_obj, "final")?;

    let cycles_raw = obj.get("cycles").and_then(|v| v.as_array()).ok_or("missing cycles")?;
    let mut cycles = Vec::with_capacity(cycles_raw.len());
    for (i, raw_cycle) in cycles_raw.iter().enumerate() {
        let arr = raw_cycle.as_array().ok_or(format!("cycle {} must be array", i))?;
        cycles.push(parse_cycle(arr)?);
    }

    Ok(Case { name, initial, fin, cycles })
}

pub fn load_file(path: &Path) -> Result<Vec<Case>, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("{}: {}", path.display(), e))?;
    let raw_cases: Vec<serde_json::Value> = serde_json::from_str(&content)
        .map_err(|e| format!("{}: {}", path.display(), e))?;

    let mut cases = Vec::with_capacity(raw_cases.len());
    for (i, raw) in raw_cases.into_iter().enumerate() {
        cases.push(parse_case(raw).map_err(|e| format!("{}:{}: {}", path.display(), i + 1, e))?);
    }
    Ok(cases)
}

pub fn list_files(dir: &Path, filter: Option<&str>) -> Result<Vec<PathBuf>, String> {
    let mut results = Vec::new();

    let opcode_set: Option<std::collections::HashSet<u8>> = filter.map(|f| {
        f.split(',')
            .filter_map(|s| u8::from_str_radix(s.trim(), 16).ok())
            .collect()
    });

    fn walk(dir: &Path, results: &mut Vec<PathBuf>, opcode_set: &Option<std::collections::HashSet<u8>>) -> Result<(), String> {
        let entries = fs::read_dir(dir).map_err(|e| format!("{}: {}", dir.display(), e))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("{}", e))?;
            let path = entry.path();
            if path.is_dir() {
                walk(&path, results, opcode_set)?;
            } else if path.extension().map_or(false, |ext| ext == "json") {
                if let Some(ref set) = *opcode_set {
                    if let Some(opcode) = opcode_of_file(path.file_name().unwrap().to_str().unwrap()) {
                        if !set.contains(&opcode) {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
                results.push(path);
            }
        }
        Ok(())
    }

    walk(dir, &mut results, &opcode_set)?;
    results.sort();
    Ok(results)
}

pub fn opcode_of_file(name: &str) -> Option<u8> {
    let stem = name.rsplit('/').next().unwrap_or(name);
    if !stem.ends_with(".json") {
        return None;
    }
    let hex_part = &stem[..stem.len() - 5];
    if hex_part.len() != 2 {
        return None;
    }
    u8::from_str_radix(hex_part, 16).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Simple defer macro for cleanup in tests
    macro_rules! defer {
        ($expr:expr) => {
            let _guard = ::std::panic::AssertUnwindSafe({
                struct Guard<F: FnOnce()> { f: Option<F> }
                impl<F: FnOnce()> Drop for Guard<F> {
                    fn drop(&mut self) {
                        if let Some(f) = self.f.take() { f(); }
                    }
                }
                Guard { f: Some(|| $expr) }
            });
        };
    }

    const SAMPLE_JSON: &str = include_str!("../../tests/data/e06_07_sample.json");

    fn write_temp_file(name: &str, content: &str) -> PathBuf {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("e06_07_test_{}", name));
        fs::write(&path, content).unwrap();
        path
    }

    fn cleanup(path: &Path) {
        let _ = fs::remove_file(path);
    }

    #[test]
    fn e06_07_load_sample() {
        let path = write_temp_file("sample.json", SAMPLE_JSON);
        defer!(cleanup(&path));

        let cases = load_file(&path).unwrap();
        assert_eq!(cases.len(), 1, "expected 1 case");

        let c = &cases[0];
        // Verify initial state (from real 00.json test 0)
        assert_eq!(c.initial.pc, 30256);
        assert_eq!(c.initial.a, 56);
        assert_eq!(c.initial.x, 78);

        // Verify final RAM value
        let found = c.fin.ram.iter().any(|(addr, val)| *addr == 30256 && *val == 0);
        assert!(found, "expected opcode byte in final ram");

        // Verify cycle count (NOP = 2 cycles)
        assert_eq!(c.cycles.len(), 2, "expected 2 cycles");
    }

    #[test]
    fn e06_07_load_bad_json() {
        let path = write_temp_file("bad.json", "not json");
        defer!(cleanup(&path));

        let result = load_file(&path);
        assert!(result.is_err(), "expected error for bad JSON");
        let err = result.unwrap_err();
        assert!(err.contains("bad.json"), "error should contain filename: {}", err);
    }

    #[test]
    fn e06_07_opcode_of_file() {
        // Real file names from the oracle (2 hex digits + .json)
        assert_eq!(opcode_of_file("e4.json"), Some(0xE4));
        assert_eq!(opcode_of_file("f4.json"), Some(0xF4));
        assert_eq!(opcode_of_file("00.json"), Some(0x00));
        assert_eq!(opcode_of_file("91.json"), Some(0x91));

        // Non-matching names
        assert_eq!(opcode_of_file("readme.md"), None);
        assert_eq!(opcode_of_file("LICENSE"), None);
        assert_eq!(opcode_of_file("100.json"), None);
    }

    #[test]
    fn e06_07_list_files_filter() {
        let dir = std::env::temp_dir().join("e06_07_list_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        // Create fake files with different opcodes
        fs::write(dir.join("e4.json"), "[]").unwrap();
        fs::write(dir.join("f4.json"), "[]").unwrap();
        fs::write(dir.join("00.json"), "[]").unwrap();

        // Filter for E4 and 00
        let filtered = list_files(&dir, Some("E4,00")).unwrap();
        assert_eq!(filtered.len(), 2, "expected 2 files with filter");
        // Should be sorted by name
        assert!(filtered[0].file_name().unwrap().to_str().unwrap().starts_with("00"));
        assert!(filtered[1].file_name().unwrap().to_str().unwrap().starts_with("e4"));

        // No filter - all files
        let all = list_files(&dir, None).unwrap();
        assert_eq!(all.len(), 3, "expected 3 files without filter");

        // Non-existent directory
        let missing = dir.join("missing");
        let result = list_files(&missing, None);
        assert!(result.is_err(), "expected error for missing directory");

        // Cleanup
        fs::remove_dir_all(&dir).unwrap();
    }
}
