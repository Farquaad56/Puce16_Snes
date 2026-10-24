use puce16_core::joypad::{
    filter_opposites, BTN_A, BTN_B, BTN_DOWN, BTN_LEFT, BTN_L, BTN_R, BTN_RIGHT, BTN_SELECT,
    BTN_START, BTN_UP, BTN_X, BTN_Y,
};

#[test]
fn e01_12_values() {
    assert_eq!(BTN_B, 0x8000);
    assert_eq!(BTN_Y, 0x4000);
    assert_eq!(BTN_SELECT, 0x2000);
    assert_eq!(BTN_START, 0x1000);
    assert_eq!(BTN_UP, 0x0800);
    assert_eq!(BTN_DOWN, 0x0400);
    assert_eq!(BTN_LEFT, 0x0200);
    assert_eq!(BTN_RIGHT, 0x0100);
    assert_eq!(BTN_A, 0x0080);
    assert_eq!(BTN_X, 0x0040);
    assert_eq!(BTN_L, 0x0020);
    assert_eq!(BTN_R, 0x0010);
}

#[test]
fn e01_12_no_overlap() {
    let all = [
        BTN_B, BTN_Y, BTN_SELECT, BTN_START, BTN_UP, BTN_DOWN, BTN_LEFT, BTN_RIGHT, BTN_A, BTN_X,
        BTN_L, BTN_R,
    ];
    for i in 0..all.len() {
        for j in (i + 1)..all.len() {
            assert_eq!(all[i] & all[j], 0);
        }
    }
    let mut ou = 0u16;
    for c in all {
        ou |= c;
    }
    assert_eq!(ou, 0xFFF0);
}

#[test]
fn e01_12_filter_lr() {
    assert_eq!(filter_opposites(BTN_LEFT | BTN_RIGHT | BTN_A), BTN_A);
}

#[test]
fn e01_12_filter_ud_keep() {
    assert_eq!(filter_opposites(BTN_UP | BTN_LEFT), BTN_UP | BTN_LEFT);
    assert_eq!(filter_opposites(BTN_UP | BTN_DOWN | BTN_LEFT | BTN_RIGHT), 0);
}
