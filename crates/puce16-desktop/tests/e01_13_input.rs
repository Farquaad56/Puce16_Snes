//! E01-13 tests: keyboard mapping (AZERTY) and emulator hotkeys.

use eframe::egui;
use puce16_core::joypad::{BTN_A, BTN_B, BTN_L, BTN_R, BTN_RIGHT, BTN_SELECT, BTN_START, BTN_UP, BTN_X, BTN_Y};
use puce16_desktop::input::{default_buttons, hotkey_for, Hotkey, Key};

#[test]
fn e01_13_buttons_dpad() {
    assert_eq!(default_buttons(&[Key::Up, Key::Right]), BTN_UP | BTN_RIGHT);
    // Z = up, D = right (AZERTY).
    assert_eq!(default_buttons(&[Key::Z, Key::D]), BTN_UP | BTN_RIGHT);
}

#[test]
fn e01_13_buttons_face() {
    assert_eq!(default_buttons(&[Key::X]), BTN_B);
    assert_eq!(default_buttons(&[Key::C]), BTN_A);
    assert_eq!(default_buttons(&[Key::W]), BTN_Y);
    assert_eq!(default_buttons(&[Key::V]), BTN_X);
    assert_eq!(default_buttons(&[Key::A]), BTN_L);
    assert_eq!(default_buttons(&[Key::E]), BTN_R);
    assert_eq!(default_buttons(&[Key::K]), BTN_B);
    assert_eq!(default_buttons(&[Key::L]), BTN_A);
}

#[test]
fn e01_13_buttons_start_select() {
    assert_eq!(default_buttons(&[Key::Enter]), BTN_START);
    assert_eq!(default_buttons(&[Key::Backspace]), BTN_SELECT);
    assert_eq!(default_buttons(&[Key::ShiftRight]), BTN_SELECT);
}

#[test]
fn e01_13_buttons_combo() {
    assert_eq!(default_buttons(&[Key::X, Key::C, Key::Enter]), BTN_B | BTN_A | BTN_START);
    assert_eq!(default_buttons(&[Key::Other]), 0);
    assert_eq!(default_buttons(&[]), 0);
}

#[test]
fn e01_13_hotkeys() {
    assert_eq!(hotkey_for(egui::Key::P, false), Some(Hotkey::Pause));
    assert_eq!(hotkey_for(egui::Key::F12, false), Some(Hotkey::Reset));
    assert_eq!(hotkey_for(egui::Key::O, true), Some(Hotkey::OpenRom));
    assert_eq!(hotkey_for(egui::Key::O, false), None);
    assert_eq!(hotkey_for(egui::Key::PageUp, false), Some(Hotkey::SlotPrev));
}
