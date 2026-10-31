use puce16_core::cheat::{parse_cheat_file, format_cheat_file, CheatEntry, CheatFileErrorKind};
use puce16_core::cheat::ParCode;

fn make_entry(name: &str, enabled: bool, codes: Vec<(u32, u8)>) -> CheatEntry {
    let parsed_codes = codes.into_iter().map(|(addr, value)| ParCode { addr, value }).collect();
    CheatEntry { name: name.to_string(), enabled, codes: parsed_codes }
}

#[test]
fn e10_02_parse_two_entries() {
    let text = "# note\n+ Vies = 7E0DBE05\n\n- Invincible = 7E1497FF, 7E1490FF\n";
    let entries = parse_cheat_file(text).unwrap();
    assert_eq!(entries.len(), 2);

    assert_eq!(entries[0].name, "Vies");
    assert!(entries[0].enabled);
    assert_eq!(entries[0].codes.len(), 1);
    assert_eq!(entries[0].codes[0], ParCode { addr: 0x7E0DBE, value: 0x05 });

    assert_eq!(entries[1].name, "Invincible");
    assert!(!entries[1].enabled);
    assert_eq!(entries[1].codes.len(), 2);
    assert_eq!(entries[1].codes[0], ParCode { addr: 0x7E1497, value: 0xFF });
    assert_eq!(entries[1].codes[1], ParCode { addr: 0x7E1490, value: 0xFF });
}

#[test]
fn e10_02_parse_crlf_and_blanks() {
    let text = "# note\r\n+ Vies = 7E0DBE05\r\n\r\n   \r\n- Invincible = 7E1497FF, 7E1490FF\r\n";
    let entries = parse_cheat_file(text).unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].name, "Vies");
    assert!(entries[0].enabled);
    assert_eq!(entries[1].name, "Invincible");
    assert!(!entries[1].enabled);

    let empty = parse_cheat_file("").unwrap();
    assert!(empty.is_empty());

    let comments_only = parse_cheat_file("# a\n# b\n").unwrap();
    assert!(comments_only.is_empty());
}

#[test]
fn e10_02_error_missing_sign() {
    let err = parse_cheat_file("Vies = 7E0DBE05").unwrap_err();
    assert_eq!(err.line, 1);
    assert_eq!(err.kind, CheatFileErrorKind::MissingSign);

    let err = parse_cheat_file("# a\n\n= 7E0DBE05").unwrap_err();
    assert_eq!(err.line, 3);
    assert_eq!(err.kind, CheatFileErrorKind::MissingSign);
}

#[test]
fn e10_02_error_missing_equals_name_code() {
    let err = parse_cheat_file("+ Vies 7E0DBE05").unwrap_err();
    assert_eq!(err.kind, CheatFileErrorKind::MissingEquals);

    let err = parse_cheat_file("+  = 7E0DBE05").unwrap_err();
    assert_eq!(err.kind, CheatFileErrorKind::EmptyName);

    let err = parse_cheat_file("+ Vies=").unwrap_err();
    assert_eq!(err.kind, CheatFileErrorKind::NoCode);

    let err = parse_cheat_file("+ Vies = , ").unwrap_err();
    assert_eq!(err.kind, CheatFileErrorKind::NoCode);
}

#[test]
fn e10_02_error_bad_code_reports_line() {
    let err = parse_cheat_file("+ A = 7E0DBE05\n+ B = 7E0DBE0").unwrap_err();
    assert_eq!(err.line, 2);
    match err.kind {
        CheatFileErrorKind::BadCode(puce16_core::cheat::ParseError::Length(7)) => {}
        _ => panic!("expected BadCode(Length(7)), got {:?}", err.kind),
    }

    let err = parse_cheat_file("+ B = 7E0DBE0G").unwrap_err();
    match err.kind {
        CheatFileErrorKind::BadCode(puce16_core::cheat::ParseError::NotHex('G')) => {}
        _ => panic!("expected BadCode(NotHex('G')), got {:?}", err.kind),
    }
}

#[test]
fn e10_02_format_exact_text() {
    let entries = vec![
        make_entry("Vies", true, vec![(0x7E0DBE, 0x05)]),
        make_entry("Inv", false, vec![(0x7E1497, 0xFF), (0x7E1490, 0xFF)]),
    ];
    let formatted = format_cheat_file(&entries);
    assert_eq!(formatted, "# Puce16 cheats\n+ Vies = 7E0DBE05\n- Inv = 7E1497FF 7E1490FF\n");
}

#[test]
fn e10_02_roundtrip() {
    let empty: Vec<CheatEntry> = vec![];
    assert_eq!(parse_cheat_file(&format_cheat_file(&empty)).unwrap(), empty);

    let one = vec![make_entry("Test", true, vec![(0x7E0DBE, 0x05)])];
    assert_eq!(parse_cheat_file(&format_cheat_file(&one)).unwrap(), one);

    let three = vec![
        make_entry("A", true, vec![(0x7E0DBE, 0x05)]),
        make_entry("B", false, vec![(0x7E1497, 0xFF), (0x7E1490, 0xFF)]),
        make_entry("C", true, vec![(0x7E2000, 0xAA), (0x7E2001, 0xBB), (0x7E2002, 0xCC)]),
    ];
    assert_eq!(parse_cheat_file(&format_cheat_file(&three)).unwrap(), three);
}
