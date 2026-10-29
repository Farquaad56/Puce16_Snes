// Debug helper: run the real emulation thread against a simulated 48 kHz audio callback (10 ms periods)
// and report underruns, ring level and output discontinuities.
use puce16_core::Region;
use puce16_desktop::audio::{new_shared_ring, Playback};
use puce16_desktop::emu_thread::{spawn_with_audio, Command};
use std::time::{Duration, Instant};

fn main() {
    let rom = std::fs::read(std::env::args().nth(1).expect("rom")).unwrap();
    let secs: u64 = std::env::args().nth(2).map(|s| s.parse().unwrap()).unwrap_or(10);
    let ring = new_shared_ring();
    let handle = spawn_with_audio(Region::Ntsc, false, Some(ring.clone()));
    handle.tx.send(Command::LoadRom(rom, "rom".into())).unwrap();
    let mut pb = Playback::new(48000);
    let mut out = vec![0f32; 480 * 2];
    let start = Instant::now();
    let mut next = start;
    let (mut under, mut calls, mut min_len, mut max_len) = (0u32, 0u32, usize::MAX, 0usize);
    let (mut jumps, mut prev) = (0u32, 0f32);
    while start.elapsed() < Duration::from_secs(secs) {
        next += Duration::from_millis(10);
        let now = Instant::now();
        if next > now { std::thread::sleep(next - now); }
        let mut g = ring.lock().unwrap();
        let len = g.len();
        if calls > 100 { min_len = min_len.min(len); max_len = max_len.max(len); }
        if len < 330 && calls > 100 { under += 1; }
        pb.render(&mut g, 1.0, false, 2, &mut out);
        drop(g);
        calls += 1;
        for f in out.chunks(2) {
            if (f[0] - prev).abs() > 0.15 { jumps += 1; }
            prev = f[0];
        }
    }
    println!("calls={calls} underruns(ring<330)={under} ring min={min_len} max={max_len} output jumps>0.15: {jumps}");
    handle.quit();
}
