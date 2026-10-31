// Debug helper (E09-12): run N frames, dump S-DSP registers + SPC RAM, write a WAV of the audio.
use puce16_core::Snes;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let rom = std::fs::read(&a[1]).unwrap();
    let frames: u64 = a[2].parse().unwrap();
    let out = &a[3];
    let mut snes = Snes::new(&rom, None).unwrap();
    let mut audio: Vec<[i16; 2]> = Vec::new();
    for _ in 0..frames {
        snes.run_frame();
        snes.take_audio(&mut audio);
    }
    let (regs, ram) = snes.debug_apu_dump().unwrap();
    std::fs::write(format!("{out}.spcram"), &ram).unwrap();
    std::fs::write(format!("{out}.dsp"), &regs[..]).unwrap();
    let mut wav = Vec::new();
    let data_len = (audio.len() * 4) as u32;
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&32000u32.to_le_bytes());
    wav.extend_from_slice(&(32000u32 * 4).to_le_bytes());
    wav.extend_from_slice(&4u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    for s in &audio {
        wav.extend_from_slice(&s[0].to_le_bytes());
        wav.extend_from_slice(&s[1].to_le_bytes());
    }
    std::fs::write(format!("{out}.wav"), wav).unwrap();
    println!("frames={frames} samples={}", audio.len());
}