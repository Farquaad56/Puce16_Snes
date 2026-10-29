use puce16_core::joypad::{
    BTN_A, BTN_B, BTN_DOWN, BTN_L, BTN_LEFT, BTN_R, BTN_RIGHT, BTN_SELECT, BTN_START, BTN_UP,
    BTN_X, BTN_Y,
};
use puce16_desktop::input::*;

#[test]
fn e08_29_pad_defaults() {
    assert_eq!(pad_buttons(&[PadButton::South]), BTN_B);
    assert_eq!(pad_buttons(&[PadButton::East]), BTN_A);
    assert_eq!(pad_buttons(&[PadButton::West]), BTN_Y);
    assert_eq!(pad_buttons(&[PadButton::North]), BTN_X);
    assert_eq!(pad_buttons(&[PadButton::LeftTrigger]), BTN_L);
    assert_eq!(pad_buttons(&[PadButton::RightTrigger]), BTN_R);
    assert_eq!(pad_buttons(&[PadButton::Start]), BTN_START);
    assert_eq!(pad_buttons(&[PadButton::Select]), BTN_SELECT);
    assert_eq!(
        pad_buttons(&[PadButton::DPadUp, PadButton::DPadLeft]),
        BTN_UP | BTN_LEFT
    );
    assert_eq!(pad_buttons(&[PadButton::Other]), 0);
    assert_eq!(pad_buttons(&[]), 0);
}

#[test]
fn e08_29_stick_threshold() {
    // Threshold is strict: abs(axis) > 0.5
    assert_eq!(stick_buttons(0.5, 0.0), 0);
    assert_eq!(stick_buttons(0.51, 0.0), BTN_RIGHT);
    assert_eq!(stick_buttons(-0.9, 0.0), BTN_LEFT);
    assert_eq!(stick_buttons(0.0, 0.9), BTN_UP);
    assert_eq!(stick_buttons(0.0, -0.9), BTN_DOWN);
    assert_eq!(stick_buttons(0.7, 0.7), BTN_RIGHT | BTN_UP);
    assert_eq!(stick_buttons(0.1, -0.2), 0);
}

#[test]
fn e08_29_snes_names() {
    assert_eq!(snes_button_bit("B"), Some(BTN_B));
    assert_eq!(snes_button_bit("Select"), Some(BTN_SELECT));
    assert_eq!(snes_button_bit("Up"), Some(BTN_UP));
    assert_eq!(snes_button_bit("b"), None);
    assert_eq!(snes_button_bit(""), None);
}

#[test]
fn e08_29_pad_names() {
    assert_eq!(pad_button_name(PadButton::South), "South");
    assert_eq!(pad_button_name(PadButton::DPadRight), "DPadRight");
    assert_eq!(pad_button_name(PadButton::Other), "Other");
}

#[test]
fn e08_29_pad_mapped_empty_is_default() {
    let map = std::collections::BTreeMap::new();
    let pressed = [PadButton::South, PadButton::Start];
    assert_eq!(pad_buttons_mapped(&pressed, &map), pad_buttons(&pressed));
}

#[test]
fn e08_29_pad_mapped_override() {
    // Map B -> West (instead of South)
    let mut map = std::collections::BTreeMap::new();
    map.insert("B".to_string(), "West".to_string());

    // Pressing West: B is active via map, Y is still active via default (Y not in map)
    assert_eq!(pad_buttons_mapped(&[PadButton::West], &map), BTN_B | BTN_Y);

    // Pressing South: B no longer follows default South -> 0
    assert_eq!(pad_buttons_mapped(&[PadButton::South], &map), 0);
}

#[test]
fn e08_29_pad_mapped_unknown_ignored() {
    let mut map = std::collections::BTreeMap::new();
    map.insert("B".to_string(), "Nope".to_string());
    map.insert("Zed".to_string(), "South".to_string());

    // B is in the map so it no longer follows South, but "Nope" doesn't exist -> 0
    assert_eq!(pad_buttons_mapped(&[PadButton::South], &map), 0);
}

#[test]
fn e08_29_key_names() {
    assert_eq!(key_name(Key::Up), "ArrowUp");
    assert_eq!(key_name(Key::Enter), "Enter");
    assert_eq!(key_name(Key::ShiftRight), "ShiftRight");
    assert_eq!(key_name(Key::Z), "Z");
}

#[test]
fn e08_29_keyboard_mapped_empty() {
    let map = std::collections::BTreeMap::new();
    let pressed = [Key::X, Key::Enter];
    assert_eq!(keyboard_buttons_mapped(&pressed, &map), default_buttons(&pressed));
}

#[test]
fn e08_29_keyboard_mapped_override() {
    // Map A -> [Q, Enter] (instead of C/L)
    let mut map = std::collections::BTreeMap::new();
    map.insert("A".to_string(), vec!["Q".to_string(), "Enter".to_string()]);

    // Pressing Q: Left via default + A via custom map
    assert_eq!(keyboard_buttons_mapped(&[Key::Q], &map), BTN_A | BTN_LEFT);

    // Pressing C: A no longer follows default C -> 0
    assert_eq!(keyboard_buttons_mapped(&[Key::C], &map), 0);

    // Pressing Enter: Start via default + A via custom map
    assert_eq!(keyboard_buttons_mapped(&[Key::Enter], &map), BTN_START | BTN_A);
}
