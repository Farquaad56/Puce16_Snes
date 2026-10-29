//! E08-30: gilrs gamepad input tests.

use puce16_desktop::input::{pad_button_from_gilrs, PadButton, PadPoller};

#[test]
fn e08_30_button_conversion() {
    // Face buttons (Xbox layout).
    assert_eq!(pad_button_from_gilrs(gilrs::Button::South), PadButton::South);
    assert_eq!(pad_button_from_gilrs(gilrs::Button::East), PadButton::East);
    assert_eq!(pad_button_from_gilrs(gilrs::Button::West), PadButton::West);
    assert_eq!(pad_button_from_gilrs(gilrs::Button::North), PadButton::North);

    // Triggers.
    assert_eq!(pad_button_from_gilrs(gilrs::Button::LeftTrigger), PadButton::LeftTrigger);
    assert_eq!(pad_button_from_gilrs(gilrs::Button::RightTrigger), PadButton::RightTrigger);

    // Menu buttons.
    assert_eq!(pad_button_from_gilrs(gilrs::Button::Start), PadButton::Start);
    assert_eq!(pad_button_from_gilrs(gilrs::Button::Select), PadButton::Select);

    // D-pad directions.
    assert_eq!(pad_button_from_gilrs(gilrs::Button::DPadUp), PadButton::DPadUp);
    assert_eq!(pad_button_from_gilrs(gilrs::Button::DPadDown), PadButton::DPadDown);
    assert_eq!(pad_button_from_gilrs(gilrs::Button::DPadLeft), PadButton::DPadLeft);
    assert_eq!(pad_button_from_gilrs(gilrs::Button::DPadRight), PadButton::DPadRight);
}

#[test]
fn e08_30_button_other() {
    // Buttons not mapped to SNES should become Other.
    assert_eq!(pad_button_from_gilrs(gilrs::Button::Mode), PadButton::Other);
    assert_eq!(pad_button_from_gilrs(gilrs::Button::LeftTrigger2), PadButton::Other);
    assert_eq!(pad_button_from_gilrs(gilrs::Button::C), PadButton::Other);
    assert_eq!(pad_button_from_gilrs(gilrs::Button::Unknown), PadButton::Other);
}

#[test]
fn e08_30_poller_without_device_is_safe() {
    // Creating a poller and polling it should not panic even with no gamepads connected.
    let mut poller = PadPoller::new();
    let result = poller.poll();
    assert_eq!(result.len(), 2);

    // If no gamepad is connected, both ports should be zero.
    // (If a gamepad IS connected on the test machine, we just verify no panic.)
    if !poller.connected()[0] && !poller.connected()[1] {
        assert_eq!(result[0], 0);
        assert_eq!(result[1], 0);
    }
}

#[test]
fn e08_30_poller_poll_twice() {
    // Two successive poll calls should not panic.
    let mut poller = PadPoller::new();
    let _r1 = poller.poll();
    let _r2 = poller.poll();
}
