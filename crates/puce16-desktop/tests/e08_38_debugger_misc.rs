// E08-38: parse_pc and sprite_line tests.

use puce16_desktop::debugger::{parse_pc, sprite_line, SpriteInfo};

#[test]
fn e08_38_parse_pc_forms() {
    assert_eq!(parse_pc("8008"), Some(0x008008));
    assert_eq!(parse_pc("008008"), Some(0x008008));
    assert_eq!(parse_pc("7E1234"), Some(0x7E1234));
    assert_eq!(parse_pc("0x8000"), Some(0x8000));
    assert_eq!(parse_pc("$00FF00"), Some(0x00FF00));
    assert_eq!(parse_pc("  12  "), Some(0x12));
}

#[test]
fn e08_38_parse_pc_invalid() {
    assert_eq!(parse_pc(""), None);
    assert_eq!(parse_pc("  "), None);
    assert_eq!(parse_pc("xyz"), None);
    assert_eq!(parse_pc("12345678"), None);
    assert_eq!(parse_pc("80 08"), None);
    assert_eq!(parse_pc("-1"), None);
}

#[test]
fn e08_38_sprite_line_format() {
    let s = SpriteInfo {
        index: 3,
        x: -16,
        y: 32,
        tile: 0x105,
        palette: 2,
        priority: 1,
        hflip: true,
        vflip: false,
        large: true,
    };
    assert_eq!(sprite_line(&s, 0), "003 x= -16 y= 32 t=105 p=2 pr=1 H- 16x16");
}

#[test]
fn e08_38_sprite_line_small() {
    let s = SpriteInfo {
        index: 127,
        x: 255,
        y: 0,
        tile: 0,
        palette: 7,
        priority: 3,
        hflip: false,
        vflip: true,
        large: false,
    };
    assert_eq!(sprite_line(&s, 6), "127 x= 255 y=  0 t=000 p=7 pr=3 -V 16x32");
}
