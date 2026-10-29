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

// === Pad button mapping (spec 08 section 3) ===

/// Physical pad buttons (gilrs naming convention).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PadButton {
    South,
    East,
    West,
    North,
    LeftTrigger,
    RightTrigger,
    Start,
    Select,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    Other,
}

/// SNES button bits for one pad button (0 if unmapped).
fn pad_button_bit(b: PadButton) -> u16 {
    match b {
        PadButton::South => BTN_B,
        PadButton::East => BTN_A,
        PadButton::West => BTN_Y,
        PadButton::North => BTN_X,
        PadButton::LeftTrigger => BTN_L,
        PadButton::RightTrigger => BTN_R,
        PadButton::Start => BTN_START,
        PadButton::Select => BTN_SELECT,
        PadButton::DPadUp => BTN_UP,
        PadButton::DPadDown => BTN_DOWN,
        PadButton::DPadLeft => BTN_LEFT,
        PadButton::DPadRight => BTN_RIGHT,
        PadButton::Other => 0,
    }
}

/// OR of SNES button bits for every pressed pad button.
pub fn pad_buttons(pressed: &[PadButton]) -> u16 {
    let mut bits = 0u16;
    for &b in pressed {
        bits |= pad_button_bit(b);
    }
    bits
}

/// Left stick axes to D-pad buttons (threshold abs > 0.5).
pub fn stick_buttons(x: f32, y: f32) -> u16 {
    let mut bits = 0u16;
    if x > 0.5 {
        bits |= BTN_RIGHT;
    } else if x < -0.5 {
        bits |= BTN_LEFT;
    }
    if y > 0.5 {
        bits |= BTN_UP;
    } else if y < -0.5 {
        bits |= BTN_DOWN;
    }
    bits
}

/// SNES button bit for a canonical name (exact case).
pub fn snes_button_bit(name: &str) -> Option<u16> {
    match name {
        "B" => Some(BTN_B),
        "A" => Some(BTN_A),
        "Y" => Some(BTN_Y),
        "X" => Some(BTN_X),
        "L" => Some(BTN_L),
        "R" => Some(BTN_R),
        "Start" => Some(BTN_START),
        "Select" => Some(BTN_SELECT),
        "Up" => Some(BTN_UP),
        "Down" => Some(BTN_DOWN),
        "Left" => Some(BTN_LEFT),
        "Right" => Some(BTN_RIGHT),
        _ => None,
    }
}

/// Canonical name for a pad button.
pub fn pad_button_name(b: PadButton) -> &'static str {
    match b {
        PadButton::South => "South",
        PadButton::East => "East",
        PadButton::West => "West",
        PadButton::North => "North",
        PadButton::LeftTrigger => "LeftTrigger",
        PadButton::RightTrigger => "RightTrigger",
        PadButton::Start => "Start",
        PadButton::Select => "Select",
        PadButton::DPadUp => "DPadUp",
        PadButton::DPadDown => "DPadDown",
        PadButton::DPadLeft => "DPadLeft",
        PadButton::DPadRight => "DPadRight",
        PadButton::Other => "Other",
    }
}

/// Custom pad mapping: map SNES button name -> pad button name.
pub fn pad_buttons_mapped(
    pressed: &[PadButton],
    map: &std::collections::BTreeMap<String, String>,
) -> u16 {
    if map.is_empty() {
        return pad_buttons(pressed);
    }

    // Compute default bits and mask out SNES buttons that are in the custom map.
    let mut bits = 0u16;
    for &b in pressed {
        bits |= pad_button_bit(b);
    }
    let mut override_mask = 0u16;
    for (snes_name, _) in map {
        if let Some(bit) = snes_button_bit(snes_name) {
            override_mask |= bit;
        }
    }
    bits &= !override_mask;

    // Apply custom mappings.
    for (snes_name, pad_name) in map {
        if let Some(bit) = snes_button_bit(snes_name) {
            for &b in pressed {
                if pad_button_name(b) == pad_name.as_str() {
                    bits |= bit;
                    break;
                }
            }
        }
    }

    bits
}

// === Keyboard name mapping (spec 08 section 2) ===

/// Canonical name for a logical key.
pub fn key_name(k: Key) -> &'static str {
    match k {
        Key::Up => "ArrowUp",
        Key::Down => "ArrowDown",
        Key::Left => "ArrowLeft",
        Key::Right => "ArrowRight",
        Key::Z => "Z",
        Key::S => "S",
        Key::Q => "Q",
        Key::D => "D",
        Key::X => "X",
        Key::K => "K",
        Key::C => "C",
        Key::L => "L",
        Key::W => "W",
        Key::J => "J",
        Key::V => "V",
        Key::I => "I",
        Key::A => "A",
        Key::U => "U",
        Key::E => "E",
        Key::O => "O",
        Key::Enter => "Enter",
        Key::Backspace => "Backspace",
        Key::ShiftRight => "ShiftRight",
        Key::Other => "Other",
    }
}

/// Custom keyboard mapping: map SNES button name -> list of key names.
pub fn keyboard_buttons_mapped(
    pressed: &[Key],
    map: &std::collections::BTreeMap<String, Vec<String>>,
) -> u16 {
    if map.is_empty() {
        return default_buttons(pressed);
    }

    // Compute default bits and mask out SNES buttons that are in the custom map.
    let mut bits = 0u16;
    for &k in pressed {
        bits |= key_button(k);
    }
    let mut override_mask = 0u16;
    for (snes_name, _) in map {
        if let Some(bit) = snes_button_bit(snes_name) {
            override_mask |= bit;
        }
    }
    bits &= !override_mask;

    // Apply custom mappings.
    for (snes_name, key_names) in map {
        if let Some(bit) = snes_button_bit(snes_name) {
            for &k in pressed {
                if key_names.contains(&key_name(k).to_string()) {
                    bits |= bit;
                    break;
                }
            }
        }
    }

    bits
}

// === gilrs gamepad support (spec 08 section 3) ===

/// Convert a gilrs button to our PadButton enum.
pub fn pad_button_from_gilrs(b: gilrs::Button) -> PadButton {
    match b {
        gilrs::Button::South => PadButton::South,
        gilrs::Button::East => PadButton::East,
        gilrs::Button::West => PadButton::West,
        gilrs::Button::North => PadButton::North,
        gilrs::Button::LeftTrigger => PadButton::LeftTrigger,
        gilrs::Button::RightTrigger => PadButton::RightTrigger,
        gilrs::Button::Start => PadButton::Start,
        gilrs::Button::Select => PadButton::Select,
        gilrs::Button::DPadUp => PadButton::DPadUp,
        gilrs::Button::DPadDown => PadButton::DPadDown,
        gilrs::Button::DPadLeft => PadButton::DPadLeft,
        gilrs::Button::DPadRight => PadButton::DPadRight,
        _ => PadButton::Other,
    }
}

/// Polls gamepads via gilrs and returns SNES button bits for each port.
pub struct PadPoller {
    gilrs: Option<gilrs::Gilrs>,
    ports: [Option<gilrs::GamepadId>; 2],
    pressed: [[PadButton; 16]; 2],
    pressed_count: [usize; 2],
    stick: [(f32, f32); 2],
}

impl PadPoller {
    /// Create a new poller. If gilrs fails to initialize, returns an empty poller (no gamepads).
    pub fn new() -> Self {
        let gilrs = match gilrs::Gilrs::new() {
            Ok(g) => Some(g),
            Err(_) => None,
        };

        // Assign ports for already-connected gamepads.
        let mut poller = Self {
            gilrs: None,
            ports: [None, None],
            pressed: [[PadButton::Other; 16]; 2],
            pressed_count: [0, 0],
            stick: [(0.0, 0.0); 2],
        };

        if let Some(g) = gilrs {
            // Collect gamepad IDs first to avoid borrow conflicts.
            let ids: Vec<gilrs::GamepadId> = g.gamepads().map(|(id, _)| id).collect();
            poller.gilrs = Some(g);
            for id in ids {
                poller.assign_port(id);
            }
        }

        poller
    }

    /// Assign a gamepad to the lowest free port. Returns true if assigned.
    fn assign_port(&mut self, id: gilrs::GamepadId) -> bool {
        for i in 0..2 {
            if self.ports[i].is_none() {
                self.ports[i] = Some(id);
                return true;
            }
        }
        false
    }

    /// Find which port a gamepad is on, or None if not connected.
    fn port_for(&self, id: gilrs::GamepadId) -> Option<usize> {
        for i in 0..2 {
            if self.ports[i] == Some(id) {
                return Some(i);
            }
        }
        None
    }

    /// Poll all pending events and return SNES button bits for each port.
    pub fn poll(&mut self) -> [u16; 2] {
        let mut result = [0u16; 2];

        // Collect all pending events first (to release gilrs borrow before processing).
        let mut events = Vec::new();
        if let Some(ref mut g) = self.gilrs {
            while let Some(event) = g.next_event() {
                events.push(event);
            }
        }

        // Process collected events.
        for event in events {
            match event.event {
                gilrs::EventType::ButtonPressed(button, _) => {
                    if let Some(port) = self.port_for(event.id) {
                        let pb = pad_button_from_gilrs(button);
                        if !self.pressed[port].iter().take(self.pressed_count[port]).any(|&b| b == pb) {
                            if self.pressed_count[port] < 16 {
                                self.pressed[port][self.pressed_count[port]] = pb;
                                self.pressed_count[port] += 1;
                            }
                        }
                    }
                }
                gilrs::EventType::ButtonReleased(button, _) => {
                    if let Some(port) = self.port_for(event.id) {
                        let pb = pad_button_from_gilrs(button);
                        for i in 0..self.pressed_count[port] {
                            if self.pressed[port][i] == pb {
                                let j = self.pressed_count[port] - 1;
                                if i < j {
                                    self.pressed[port][i] = self.pressed[port][j];
                                }
                                self.pressed_count[port] -= 1;
                                break;
                            }
                        }
                    }
                }
                gilrs::EventType::AxisChanged(axis, value, _) => {
                    if let Some(port) = self.port_for(event.id) {
                        match axis {
                            gilrs::Axis::LeftStickX => {
                                self.stick[port].0 = value;
                            }
                            gilrs::Axis::LeftStickY => {
                                // gilrs Y is inverted (up is negative), SNES expects up positive.
                                self.stick[port].1 = -value;
                            }
                            _ => {}
                        }
                    }
                }
                gilrs::EventType::Connected => {
                    self.assign_port(event.id);
                }
                gilrs::EventType::Disconnected => {
                    if let Some(port) = self.port_for(event.id) {
                        self.ports[port] = None;
                        self.pressed_count[port] = 0;
                        self.stick[port] = (0.0, 0.0);
                    }
                }
                _ => {}
            }
        }

        // Compute button bits for each port.
        for i in 0..2 {
            let mut bits = pad_buttons(&self.pressed[i][..self.pressed_count[i]]);
            bits |= stick_buttons(self.stick[i].0, self.stick[i].1);
            result[i] = bits;
        }

        result
    }

    /// Return which ports have a gamepad connected.
    pub fn connected(&self) -> [bool; 2] {
        let mut result = [false; 2];
        for i in 0..2 {
            result[i] = self.ports[i].is_some();
        }
        result
    }
}

