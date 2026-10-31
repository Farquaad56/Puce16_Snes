use puce16_core::cheat::{parse_par, ParCode, ParseError};

#[test]
fn e10_01_valid_uppercase() {
    let code = parse_par("7E0DBF99").unwrap();
    assert_eq!(code.addr, 0x7E0DBF);
    assert_eq!(code.value, 0x99);
}

#[test]
fn e10_01_valid_lowercase_and_spaces() {
    let code = parse_par("  7e0dbf99 ").unwrap();
    assert_eq!(code.addr, 0x7E0DBF);
    assert_eq!(code.value, 0x99);

    let zero = parse_par("00000000").unwrap();
    assert_eq!(zero.addr, 0);
    assert_eq!(zero.value, 0);

    let max = parse_par("FFFFFFFF").unwrap();
    assert_eq!(max.addr, 0xFFFFFF);
    assert_eq!(max.value, 0xFF);
}

#[test]
fn e10_01_wrong_length() {
    assert_eq!(parse_par(""), Err(ParseError::Length(0)));
    assert_eq!(parse_par("7E0DBF9"), Err(ParseError::Length(7)));
    assert_eq!(parse_par("7E0DBF990"), Err(ParseError::Length(9)));
}

#[test]
fn e10_01_not_hex() {
    assert_eq!(parse_par("7E0DBG99"), Err(ParseError::NotHex('G')));
    assert_eq!(parse_par("ZZ0DBF99"), Err(ParseError::NotHex('Z')));
}

#[test]
fn e10_01_game_genie_shape_refused() {
    assert_eq!(parse_par("DF4B-7D6E"), Err(ParseError::LooksLikeGameGenie));
    assert_eq!(parse_par("7E0D-BF99"), Err(ParseError::LooksLikeGameGenie));
}

#[test]
fn e10_01_display_is_canonical() {
    let code = parse_par("7e0dbf99").unwrap();
    assert_eq!(format!("{}", code), "7E0DBF99");

    let manual = ParCode { addr: 0x12, value: 0x3 };
    assert_eq!(format!("{}", manual), "00001203");
}
