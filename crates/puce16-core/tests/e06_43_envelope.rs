use puce16_core::apu::dsp::{Dsp, reg};
use puce16_core::apu::dsp::voice::Mode;

fn setup_voice(dsp: &mut Dsp, adsr1: u8, adsr2: u8, gain: u8, mode: Mode, env: i32, hidden_env: i32) {
    dsp.write(reg::vreg(0, reg::ADSR1) as u8, adsr1);
    dsp.write(reg::vreg(0, reg::ADSR2) as u8, adsr2);
    dsp.write(reg::vreg(0, reg::GAIN) as u8, gain);
    let v = dsp.voice_mut(0);
    v.mode = mode;
    v.env = env;
    v.hidden_env = hidden_env;
}

#[test]
fn e06_43_attack_rate31() {
    let mut dsp = Dsp::new();
    setup_voice(&mut dsp, 0x8F, 0, 0, Mode::Attack, 0, 0);
    
    // First call: rate=31 (period=1), always polls true
    dsp.debug_run_envelope(0);
    let v = dsp.voice(0);
    assert_eq!(v.env, 1024, "first call env");
    assert_eq!(v.hidden_env, 1024, "first call hidden_env");
    assert_eq!(v.mode, Mode::Attack, "first call mode");
    
    // Second call: e=1024+1024=2048 > 0x7FF -> clamp to 0x7FF, transition to Decay
    dsp.debug_run_envelope(0);
    let v = dsp.voice(0);
    assert_eq!(v.env, 0x7FF, "second call env");
    assert_eq!(v.hidden_env, 2048, "second call hidden_env");
    assert_eq!(v.mode, Mode::Decay, "second call mode");
}

#[test]
fn e06_43_attack_slow_rate_needs_poll() {
    let mut dsp = Dsp::new();
    setup_voice(&mut dsp, 0x81, 0, 0, Mode::Attack, 0, 0);
    
    // rate=3 (period=1280, offset=536)
    // counter=0: (0+536)%1280 != 0 -> no update
    dsp.debug_set_counter(0);
    dsp.debug_run_envelope(0);
    let v = dsp.voice(0);
    assert_eq!(v.env, 0, "counter=0 env");
    assert_eq!(v.hidden_env, 32, "counter=0 hidden_env");
    
    // counter=744: (744+536)%1280 = 1280%1280 = 0 -> update
    setup_voice(&mut dsp, 0x81, 0, 0, Mode::Attack, 0, 0);
    dsp.debug_set_counter(744);
    dsp.debug_run_envelope(0);
    let v = dsp.voice(0);
    assert_eq!(v.env, 32, "counter=744 env");
}

#[test]
fn e06_43_attack_overflow_slow() {
    let mut dsp = Dsp::new();
    setup_voice(&mut dsp, 0x81, 0, 0, Mode::Attack, 0x7F0, 0x7F0);
    
    // rate=3, counter=744 -> polls true
    dsp.debug_set_counter(744);
    dsp.debug_run_envelope(0);
    let v = dsp.voice(0);
    assert_eq!(v.env, 0x7FF, "env clamped");
    assert_eq!(v.hidden_env, 2064, "hidden_env overflowed");
    assert_eq!(v.mode, Mode::Decay, "transitioned to Decay");
}

#[test]
fn e06_43_decay_to_sustain() {
    let mut dsp = Dsp::new();
    setup_voice(&mut dsp, 0xF0, 0x20, 0, Mode::Decay, 0x201, 0x201);
    
    // rate=31 (decay), counter=0 -> polls true
    dsp.debug_set_counter(0);
    dsp.debug_run_envelope(0);
    let v = dsp.voice(0);
    assert_eq!(v.env, 510, "env after decay");
    assert_eq!(v.mode, Mode::Sustain, "transitioned to Sustain");
}

#[test]
fn e06_43_decay_stays_decay() {
    let mut dsp = Dsp::new();
    setup_voice(&mut dsp, 0xF0, 0xE0, 0, Mode::Decay, 0x400, 0x400);
    
    // rate=31 (decay), counter=0 -> polls true
    dsp.debug_set_counter(0);
    dsp.debug_run_envelope(0);
    let v = dsp.voice(0);
    assert_eq!(v.env, 1020, "env after decay");
    assert_eq!(v.mode, Mode::Decay, "still in Decay");
}

#[test]
fn e06_43_sustain_exponential() {
    let mut dsp = Dsp::new();
    setup_voice(&mut dsp, 0x80, 0x1F, 0, Mode::Sustain, 0x400, 0x400);
    
    // rate=31 (sustain), counter=0 -> polls true
    dsp.debug_set_counter(0);
    dsp.debug_run_envelope(0);
    let v = dsp.voice(0);
    assert_eq!(v.env, 1020, "env after sustain");
}

#[test]
fn e06_43_sustain_rate0_holds() {
    let mut dsp = Dsp::new();
    setup_voice(&mut dsp, 0x80, 0x00, 0, Mode::Sustain, 0x400, 0x400);
    
    // rate=0 (sustain) -> never polls true
    dsp.debug_set_counter(0);
    dsp.debug_run_envelope(0);
    let v = dsp.voice(0);
    assert_eq!(v.env, 1024, "env unchanged");
    assert_eq!(v.hidden_env, 1020, "hidden_env updated");
}

#[test]
fn e06_43_release() {
    let mut dsp = Dsp::new();
    setup_voice(&mut dsp, 0, 0, 0, Mode::Release, 20, 20);
    
    // Release: env -= 8 each call, no rate/poll
    dsp.debug_run_envelope(0);
    assert_eq!(dsp.voice(0).env, 12, "after 1st release");
    
    dsp.debug_run_envelope(0);
    assert_eq!(dsp.voice(0).env, 4, "after 2nd release");
    
    dsp.debug_run_envelope(0);
    let v = dsp.voice(0);
    assert_eq!(v.env, 0, "after 3rd release");
    assert_eq!(v.hidden_env, 0, "hidden_env matches env");
    
    // 4th call stays at 0 (max(env-8, 0))
    dsp.debug_run_envelope(0);
    assert_eq!(dsp.voice(0).env, 0, "after 4th release");
}

#[test]
fn e06_43_gain_direct() {
    let mut dsp = Dsp::new();
    
    // GAIN direct mode (m<4): rate=31, always applies
    setup_voice(&mut dsp, 0x00, 0, 0x40, Mode::Sustain, 0, 0);
    dsp.debug_set_counter(0);
    dsp.debug_run_envelope(0);
    assert_eq!(dsp.voice(0).env, 0x400, "GAIN=0x40");
    
    setup_voice(&mut dsp, 0x00, 0, 0x7F, Mode::Sustain, 0, 0);
    dsp.debug_set_counter(0);
    dsp.debug_run_envelope(0);
    assert_eq!(dsp.voice(0).env, 2032, "GAIN=0x7F");
}

#[test]
fn e06_43_gain_linear_decrease() {
    let mut dsp = Dsp::new();
    
    // GAIN linear decrease (m==4): rate=31
    setup_voice(&mut dsp, 0x00, 0, 0x9F, Mode::Sustain, 100, 100);
    dsp.debug_set_counter(0);
    dsp.debug_run_envelope(0);
    assert_eq!(dsp.voice(0).env, 68, "linear decrease from 100");
    
    setup_voice(&mut dsp, 0x00, 0, 0x9F, Mode::Sustain, 10, 10);
    dsp.debug_set_counter(0);
    dsp.debug_run_envelope(0);
    let v = dsp.voice(0);
    assert_eq!(v.env, 0, "clamped to 0");
    assert_eq!(v.hidden_env, -22, "hidden_env negative");
}

#[test]
fn e06_43_gain_exponential_decrease() {
    let mut dsp = Dsp::new();
    
    // GAIN exponential decrease (m==5): rate=31
    setup_voice(&mut dsp, 0x00, 0, 0xBF, Mode::Sustain, 0x400, 0x400);
    dsp.debug_set_counter(0);
    dsp.debug_run_envelope(0);
    assert_eq!(dsp.voice(0).env, 1020, "exponential decrease");
}

#[test]
fn e06_43_gain_linear_increase() {
    let mut dsp = Dsp::new();
    
    // GAIN linear increase (m==6): rate=31
    setup_voice(&mut dsp, 0x00, 0, 0xDF, Mode::Sustain, 100, 100);
    dsp.debug_set_counter(0);
    dsp.debug_run_envelope(0);
    assert_eq!(dsp.voice(0).env, 132, "linear increase");
}

#[test]
fn e06_43_gain_bent_line() {
    let mut dsp = Dsp::new();
    
    // GAIN bent line (m==7): rate=31
    setup_voice(&mut dsp, 0x00, 0, 0xFF, Mode::Sustain, 0x500, 0x500);
    dsp.debug_set_counter(0);
    dsp.debug_run_envelope(0);
    assert_eq!(dsp.voice(0).env, 0x520, "bent line below 0x600");
    
    setup_voice(&mut dsp, 0x00, 0, 0xFF, Mode::Sustain, 0x600, 0x600);
    dsp.debug_set_counter(0);
    dsp.debug_run_envelope(0);
    assert_eq!(dsp.voice(0).env, 0x608, "bent line at 0x600");
    
    setup_voice(&mut dsp, 0x00, 0, 0xFF, Mode::Sustain, 0x500, 0x600);
    dsp.debug_set_counter(0);
    dsp.debug_run_envelope(0);
    assert_eq!(dsp.voice(0).env, 1288, "bent line with hidden_env >= 0x600");
    
    setup_voice(&mut dsp, 0x00, 0, 0xFF, Mode::Sustain, 2032, 2032);
    dsp.debug_set_counter(0);
    dsp.debug_run_envelope(0);
    assert_eq!(dsp.voice(0).env, 2040, "bent line near max");
}

#[test]
fn e06_43_gain_slow_rate() {
    let mut dsp = Dsp::new();
    
    // GAIN with slow rate (rate=30: period=2, offset=536)
    setup_voice(&mut dsp, 0x00, 0, 0xDE, Mode::Sustain, 100, 100);
    
    // counter=0: (0+536)%2 = 0 -> polls true
    dsp.debug_set_counter(0);
    dsp.debug_run_envelope(0);
    assert_eq!(dsp.voice(0).env, 132, "counter=0 env");
    
    // counter=1: (1+536)%2 = 1 -> no update
    setup_voice(&mut dsp, 0x00, 0, 0xDE, Mode::Sustain, 100, 100);
    dsp.debug_set_counter(1);
    dsp.debug_run_envelope(0);
    let v = dsp.voice(0);
    assert_eq!(v.env, 100, "counter=1 env unchanged");
    assert_eq!(v.hidden_env, 132, "counter=1 hidden updated");
}

#[test]
fn e06_43_counter_accessors() {
    let mut dsp = Dsp::new();
    assert_eq!(dsp.counter(), 0, "initial counter");
    
    dsp.debug_set_counter(0x1234);
    assert_eq!(dsp.counter(), 0x1234, "after set");
}
