// Debug helper: time Playback::render for 1 s of audio at 48 kHz in 480-frame callbacks.
use puce16_desktop::audio::{AudioRing, Playback};
use std::time::Instant;
fn main() {
    let mut pb = Playback::new(48000);
    let mut ring = AudioRing::new();
    let mut out = vec![0f32; 480 * 2];
    let mut worst = 0u128;
    let t0 = Instant::now();
    for i in 0..100 {
        let frames: Vec<[i16; 2]> = (0..330).map(|k| { let v = (((i * 330 + k) as f32 * 0.05).sin() * 8000.0) as i16; [v, v] }).collect();
        ring.push(&frames);
        let t = Instant::now();
        pb.render(&mut ring, 1.0, false, 2, &mut out);
        worst = worst.max(t.elapsed().as_micros());
    }
    println!("total {:?} for 1 s of audio; worst callback {} us (budget 10000 us)", t0.elapsed(), worst);
}
