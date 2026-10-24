//! Keyboard mapping (spec 08 section 2): AZERTY logical keys -> SNES buttons,
//! plus emulator hotkeys. Pure functions so they can be tested without a window.

use eframe::egui;
use puce16_core::joypad::{
    BTN_A, BTN_B, BTN_DOWN, BTN_L, BTN_LEFT, BTN_R, BTN_RIGHT, BTN_SELECT, BTN_START, BTN_UP,
    BTN_X, BTN_Y,
};

/// Logical keyboard keys used by the emulator (spec 08 section 2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Z,
    S,
    Q,
    D,
    X,
    K,
    C,
    L,
    W,
    J,
    V,
    I,
    A,
    U,
    E,
    O,
    Enter,
    Backspace,
    ShiftRight,
    Other,
}

/// The egui keys that map to a logical [`Key`] other than [`Key::Other`].
pub const MAPPED_KEYS: [egui::Key; 23] = [
    egui::Key::ArrowUp,
    egui::Key::ArrowDown,
    egui::Key::ArrowLeft,
    egui::Key::ArrowRight,
    egui::Key::Z,
    egui::Key::S,
    egui::Key::Q,
    egui::Key::D,
    egui::Key::X,
    egui::Key::K,
    egui::Key::C,
    egui::Key::L,
    egui::Key::W,
    egui::Key::J,
    egui::Key::V,
    egui::Key::I,
    egui::Key::A,
    egui::Key::U,
    egui::Key::E,
    egui::Key::O,
    egui::Key::Enter,
    egui::Key::Backspace,
    egui::Key::ShiftRight,
];

/// Map an egui key to a logical [`Key`] (`Other` if not mapped).
pub fn from_egui_key(k: egui::Key) -> Key {
    match k {
        egui::Key::ArrowUp => Key::Up,
        egui::Key::ArrowDown => Key::Down,
        egui::Key::ArrowLeft => Key::Left,
        egui::Key::ArrowRight => Key::Right,
        egui::Key::Z => Key::Z,
        egui::Key::S => Key::S,
        egui::Key::Q => Key::Q,
        egui::Key::D => Key::D,
        egui::Key::X => Key::X,
        egui::Key::K => Key::K,
        egui::Key::C => Key::C,
        egui::Key::L => Key::L,
        egui::Key::W => Key::W,
        egui::Key::J => Key::J,
        egui::Key::V => Key::V,
        egui::Key::I => Key::I,
        egui::Key::A => Key::A,
        egui::Key::U => Key::U,
        egui::Key::E => Key::E,
        egui::Key::O => Key::O,
        egui::Key::Enter => Key::Enter,
        egui::Key::Backspace => Key::Backspace,
        egui::Key::ShiftRight => Key::ShiftRight,
        _ => Key::Other,
    }
}

/// SNES button bits for one logical key (0 if the key has no button).
fn key_button(k: Key) -> u16 {
    match k {
        Key::Up | Key::Z => BTN_UP,
        Key::Down | Key::S => BTN_DOWN,
        Key::Left | Key::Q => BTN_LEFT,
        Key::Right | Key::D => BTN_RIGHT,
        Key::X | Key::K => BTN_B,
        Key::C | Key::L => BTN_A,
        Key::W | Key::J => BTN_Y,
        Key::V | Key::I => BTN_X,
        Key::A | Key::U => BTN_L,
        Key::E | Key::O => BTN_R,
        Key::Enter => BTN_START,
        Key::Backspace | Key::ShiftRight => BTN_SELECT,
        Key::Other => 0,
    }
}

/// OR of the SNES button bits for every pressed key (no filtering).
pub fn default_buttons(pressed: &[Key]) -> u16 {
    let mut bits = 0u16;
    for &k in pressed {
        bits |= key_button(k);
    }
    bits
}

/// Emulator hotkeys (spec 08 section 2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Hotkey {
    Pause,
    FrameAdvance,
    Reset,
    SaveState,
    LoadState,
    SlotPrev,
    SlotNext,
    Screenshot,
    Fullscreen,
    Debugger,
    Mute,
    FastForward,
    OpenRom,
    Quit,
}

/// Map a key press (with the Ctrl modifier state) to an emulator hotkey.
pub fn hotkey_for(k: egui::Key, ctrl: bool) -> Option<Hotkey> {
    match k {
        egui::Key::P => Some(Hotkey::Pause),
        egui::Key::N => Some(Hotkey::FrameAdvance),
        egui::Key::F12 => Some(Hotkey::Reset),
        egui::Key::F5 => Some(Hotkey::SaveState),
        egui::Key::F7 => Some(Hotkey::LoadState),
        egui::Key::PageUp => Some(Hotkey::SlotPrev),
        egui::Key::PageDown => Some(Hotkey::SlotNext),
        egui::Key::F2 => Some(Hotkey::Screenshot),
        egui::Key::F11 => Some(Hotkey::Fullscreen),
        egui::Key::F10 => Some(Hotkey::Debugger),
        egui::Key::M => Some(Hotkey::Mute),
        egui::Key::Tab => Some(Hotkey::FastForward),
        egui::Key::O if ctrl => Some(Hotkey::OpenRom),
        egui::Key::Escape => Some(Hotkey::Quit),
        _ => None,
    }
}

