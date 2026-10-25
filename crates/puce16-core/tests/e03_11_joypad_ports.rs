use puce16_core::hash::Fnv64;
use puce16_core::joypad::{self, Joypads};

#[test]
fn e03_11_initial() {
    let mut j = Joypads::new();
    assert_eq!(j.read_port(0), 1); // shift register full of 1s
    assert_eq!(j.buttons(0), 0);
}

#[test]
fn e03_11_latch_sequence() {
    let mut j = Joypads::new();
    j.set_buttons(0, joypad::BTN_B | joypad::BTN_A); // 0x8080
    j.write_out(true);
    j.write_out(false);
    let expected = [1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0];
    for i in 0..16 {
        assert_eq!(j.read_port(0), expected[i], "bit {}", i);
    }
    // After 16 bits, shift register is full of 1s again
    assert_eq!(j.read_port(0), 1);
    assert_eq!(j.read_port(0), 1);
}

#[test]
fn e03_11_port1_independent() {
    let mut j = Joypads::new();
    j.set_buttons(1, joypad::BTN_START); // 0x1000
    j.write_out(true);
    j.write_out(false);
    assert_eq!(j.read_port(1), 0);
    assert_eq!(j.read_port(1), 0);
    assert_eq!(j.read_port(1), 0);
    assert_eq!(j.read_port(1), 1); // bit 12 = Start
    // Port 0 has no buttons pressed
    assert_eq!(j.read_port(0), 0);
    assert_eq!(j.read_port(0), 0);
}

#[test]
fn e03_11_hold_out0() {
    let mut j = Joypads::new();
    j.set_buttons(0, joypad::BTN_B); // 0x8000
    j.write_out(true);
    assert_eq!(j.read_port(0), 1);
    assert_eq!(j.read_port(0), 1);
    assert_eq!(j.read_port(0), 1);
    j.set_buttons(0, 0);
    assert_eq!(j.read_port(0), 0); // immediate change when OUT0 held high
    j.write_out(false);
    assert_eq!(j.read_port(0), 0); // bit 15 of state (now 0)
}

#[test]
fn e03_11_no_reload_when_idle() {
    let mut j = Joypads::new();
    j.set_buttons(0, joypad::BTN_B | joypad::BTN_Y); // 0xC000
    j.write_out(true);
    j.write_out(false);
    assert_eq!(j.read_port(0), 1);
    assert_eq!(j.read_port(0), 1);
    // write_out(false) when out0 is already false does NOT reload
    j.write_out(false);
    assert_eq!(j.read_port(0), 0); // bit 13 (Select), not reloaded to B
}

#[test]
fn e03_11_reload_on_write_true() {
    let mut j = Joypads::new();
    j.set_buttons(0, joypad::BTN_B); // 0x8000
    j.write_out(true);
    j.write_out(false);
    assert_eq!(j.read_port(0), 1);
    assert_eq!(j.read_port(0), 0);
    assert_eq!(j.read_port(0), 0);
    // write_out(true) reloads the shift register
    j.write_out(true);
    assert_eq!(j.read_port(0), 1); // bit 15 again (B pressed)
}

#[test]
fn e03_11_autoread() {
    let mut j = Joypads::new();
    j.set_buttons(0, joypad::BTN_B | joypad::BTN_R); // 0x8010
    j.set_buttons(1, joypad::BTN_A | joypad::BTN_X | joypad::BTN_L | joypad::BTN_R); // 0x00F0
    let vals = j.autoread_values();
    assert_eq!(vals, [0x8010, 0x00F0, 0, 0]);
    // After autoread, shift registers are reset to 0xFFFF
    assert_eq!(j.read_port(0), 1);
    assert_eq!(j.read_port(1), 1);
}

#[test]
fn e03_11_bad_port() {
    let mut j = Joypads::new();
    j.set_buttons(2, 0xFFFF); // should not panic
    assert_eq!(j.buttons(2), 0);
    assert_eq!(j.read_port(2), 1);
}

#[test]
fn e03_11_hash() {
    let mut j = Joypads::new();
    let mut h1 = Fnv64::new();
    j.hash_into(&mut h1);
    let hash_before = h1.finish();

    j.set_buttons(0, 1);
    let mut h2 = Fnv64::new();
    j.hash_into(&mut h2);
    assert_ne!(h2.finish(), hash_before); // state changed

    j.write_out(true);
    j.write_out(false);
    j.read_port(0);
    let mut h3 = Fnv64::new();
    j.hash_into(&mut h3);
    assert_ne!(h3.finish(), h2.finish()); // shift register changed
}
