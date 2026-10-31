use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParCode {
    pub addr: u32,
    pub value: u8,
}

impl fmt::Display for ParCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:06X}{:02X}", self.addr, self.value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseError {
    Length(usize),
    NotHex(char),
    LooksLikeGameGenie,
}

fn hex_digit(c: char) -> Option<u8> {
    match c {
        '0'..='9' => Some(c as u8 - b'0'),
        'a'..='f' => Some(c as u8 - b'a' + 10),
        'A'..='F' => Some(c as u8 - b'A' + 10),
        _ => None,
    }
}

pub fn parse_par(s: &str) -> Result<ParCode, ParseError> {
    let s = s.trim();
    if s.contains('-') {
        return Err(ParseError::LooksLikeGameGenie);
    }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() != 8 {
        return Err(ParseError::Length(chars.len()));
    }
    for &c in &chars {
        if hex_digit(c).is_none() {
            return Err(ParseError::NotHex(c));
        }
    }
    let addr = (hex_digit(chars[0]).unwrap() as u32) << 20
        | (hex_digit(chars[1]).unwrap() as u32) << 16
        | (hex_digit(chars[2]).unwrap() as u32) << 12
        | (hex_digit(chars[3]).unwrap() as u32) << 8
        | (hex_digit(chars[4]).unwrap() as u32) << 4
        | hex_digit(chars[5]).unwrap() as u32;
    let value = (hex_digit(chars[6]).unwrap() as u8) << 4
        | hex_digit(chars[7]).unwrap();
    Ok(ParCode { addr, value })
}
