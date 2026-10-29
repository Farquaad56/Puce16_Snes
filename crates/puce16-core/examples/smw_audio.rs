// Debug helper: analyse the DSP output of N frames (clipping, jumps, samples per frame) and dump a WAV.
use puce16_core::Snes;
fn main() {
    let rom = std::fs::read(std::env::args().nth(1).expect("rom")).unwrap();
    let frames: u64 = std::env::args().nth(2).unwrap().parse().unwrap();
    let wav = std::env::args().nth(3);
    let mut snes = Snes::new(&rom, None).unwrap();
    let mut all: Vec<[i16; 2]> = Vec::new();
    let mut per_frame = Vec::new();
    for _ in 0..frames {
        snes.run_frame();
        let mut v = Vec::new();
        snes.take_audio(&mut v);
        per_frame.push(v.len());
        all.extend(v);
    }
    for (i, c) in per_frame.chunks(100).enumerate() { println!("frames {}..: avg {:.1}", i * 100, c.iter().sum::<usize>() as f64 / c.len() as f64); }
    let mn = per_frame.iter().min().unwrap();
    let mx = per_frame.iter().max().unwrap();
    println!("frames={frames} samples={} per-frame min={mn} max={mx}", all.len());
    let clip = all.iter().filter(|s| s[0].abs() >= 32760 || s[1].abs() >= 32760).count();
    println!("clipped samples: {clip}");
    let mut hist = [0usize; 6];
    let mut nonzero = 0;
    for w in all.windows(2) {
        if w[1][0] != 0 { nonzero += 1; }
        let d = (w[1][0] as i32 - w[0][0] as i32).abs();
        let b = match d { 0..=999 => 0, 1000..=4999 => 1, 5000..=9999 => 2, 10000..=19999 => 3, 20000..=39999 => 4, _ => 5 };
        hist[b] += 1;
    }
    println!("nonzero L samples: {nonzero}");
    println!("L jump histogram <1k,<5k,<10k,<20k,<40k,>=40k: {:?}", hist);
    if let Some(p) = wav {
        let data_len = (all.len() * 4) as u32;
        let mut b = Vec::new();
        b.extend_from_slice(b"RIFF"); b.extend_from_slice(&(36 + data_len).to_le_bytes());
        b.extend_from_slice(b"WAVEfmt "); b.extend_from_slice(&16u32.to_le_bytes());
        b.extend_from_slice(&1u16.to_le_bytes()); b.extend_from_slice(&2u16.to_le_bytes());
        b.extend_from_slice(&32040u32.to_le_bytes()); b.extend_from_slice(&(32040u32 * 4).to_le_bytes());
        b.extend_from_slice(&4u16.to_le_bytes()); b.extend_from_slice(&16u16.to_le_bytes());
        b.extend_from_slice(b"data"); b.extend_from_slice(&data_len.to_le_bytes());
        for s in &all { b.extend_from_slice(&s[0].to_le_bytes()); b.extend_from_slice(&s[1].to_le_bytes()); }
        std::fs::write(p, b).unwrap();
    }
}
