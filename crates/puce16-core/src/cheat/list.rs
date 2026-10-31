use std::fmt;

use super::par::{parse_par, ParCode, ParseError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheatEntry {
    pub name: String,
    pub enabled: bool,
    pub codes: Vec<ParCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheatFileErrorKind {
    MissingSign,
    MissingEquals,
    EmptyName,
    NoCode,
    BadCode(ParseError),
}

impl fmt::Display for CheatFileErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheatFileErrorKind::MissingSign => write!(f, "missing + or - sign"),
            CheatFileErrorKind::MissingEquals => write!(f, "missing = separator"),
            CheatFileErrorKind::EmptyName => write!(f, "empty cheat name"),
            CheatFileErrorKind::NoCode => write!(f, "no PAR code provided"),
            CheatFileErrorKind::BadCode(e) => write!(f, "bad PAR code: {:?}", e),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheatFileError {
    pub line: usize,
    pub kind: CheatFileErrorKind,
}

impl fmt::Display for CheatFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.kind)
    }
}

fn parse_line(line: &str, line_num: usize) -> Result<Option<CheatEntry>, CheatFileError> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return Ok(None);
    }

    let first_char = trimmed.chars().next().unwrap();
    let enabled = match first_char {
        '+' => true,
        '-' => false,
        _ => return Err(CheatFileError { line: line_num, kind: CheatFileErrorKind::MissingSign }),
    };

    let rest = &trimmed[1..];
    let eq_pos = rest.find('=').ok_or_else(|| CheatFileError {
        line: line_num,
        kind: CheatFileErrorKind::MissingEquals,
    })?;

    let name = rest[..eq_pos].trim().to_string();
    if name.is_empty() {
        return Err(CheatFileError {
            line: line_num,
            kind: CheatFileErrorKind::EmptyName,
        });
    }

    let codes_str = &rest[eq_pos + 1..];
    let segments: Vec<&str> = codes_str.split(|c: char| c == ' ' || c == ',').collect();
    let mut codes = Vec::new();
    for seg in segments {
        if seg.is_empty() {
            continue;
        }
        match parse_par(seg) {
            Ok(code) => codes.push(code),
            Err(e) => return Err(CheatFileError { line: line_num, kind: CheatFileErrorKind::BadCode(e) }),
        }
    }

    if codes.is_empty() {
        return Err(CheatFileError {
            line: line_num,
            kind: CheatFileErrorKind::NoCode,
        });
    }

    Ok(Some(CheatEntry { name, enabled, codes }))
}

pub fn parse_cheat_file(text: &str) -> Result<Vec<CheatEntry>, CheatFileError> {
    let mut entries = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line_num = i + 1;
        match parse_line(line, line_num)? {
            Some(entry) => entries.push(entry),
            None => {}
        }
    }
    Ok(entries)
}

pub fn format_cheat_file(entries: &[CheatEntry]) -> String {
    let mut result = String::from("# Puce16 cheats\n");
    for entry in entries {
        let sign = if entry.enabled { '+' } else { '-' };
        let codes_str: Vec<String> = entry.codes.iter().map(|c| c.to_string()).collect();
        result.push_str(&format!("{} {} = {}\n", sign, entry.name, codes_str.join(" ")));
    }
    result
}
