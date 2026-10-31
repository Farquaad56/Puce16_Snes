// Debug helper (E09-12): run N frames, dump S-DSP registers + SPC RAM, export an .spc snapshot
// (loadable by any SPC player, e.g. libgme) and write WAVs of the audio.
//   dump_apu <rom> <frames> <out-prefix> [frames-after]
// Outputs: <out>.dsp, <out>.spcram, <out>.spc, <out>.wav (all audio so far),
//          <out>_after.wav (audio of the `frames-after` frames that follow the snapshot).
use puce16_core::Snes;

fn wav(audio: &[[i16; 2]]) -> Vec<u8> {
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
    for s in audio {
        wav.extend_from_slice(&s[0].to_le_bytes());
        wav.extend_from_slice(&s[1].to_le_bytes());
    }
    wav
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let rom = std::fs::read(&a[1]).unwrap();
    let frames: u64 = a[2].parse().unwrap();
    let out = &a[3];
    let after: u64 = a.get(4).map(|s| s.parse().unwrap()).unwrap_or(0);
    let mut snes = Snes::new(&rom, None).unwrap();
    let mut audio: Vec<[i16; 2]> = Vec::new();
    for _ in 0..frames {
        snes.run_frame();
        snes.take_audio(&mut audio);
    }
    let (regs, ram) = snes.debug_apu_dump().unwrap();
    let (io, (ra, rx, ry, rsp, rpc, rpsw)) = snes.debug_apu_smp().unwrap();
    std::fs::write(format!("{out}.spcram"), &ram).unwrap();
    std::fs::write(format!("{out}.dsp"), &regs[..]).unwrap();

    // SPC file: 0x100 header, 64 KiB RAM, 128 DSP regs, 64 unused, 64 bytes IPL RAM.
    let mut spc = vec![0u8; 0x10200];
    spc[..33].copy_from_slice(b"SNES-SPC700 Sound File Data v0.30");
    spc[33] = 26;
    spc[34] = 26;
    spc[35] = 27; // no ID666 tag
    spc[36] = 30;
    spc[0x25] = rpc as u8;
    spc[0x26] = (rpc >> 8) as u8;
    spc[0x27] = ra;
    spc[0x28] = rx;
    spc[0x29] = ry;
    spc[0x2A] = rpsw;
    spc[0x2B] = rsp;
    let mut ram2 = ram.clone();
    for i in 0..16 {
        ram2[0xF0 + i] = io[i];
    }
    spc[0x100..0x10100].copy_from_slice(&ram2);
    spc[0x10100..0x10180].copy_from_slice(&regs);
    spc[0x101C0..0x10200].copy_from_slice(&ram[0xFFC0..0x10000]);
    std::fs::write(format!("{out}.spc"), &spc).unwrap();
    println!(
        "snapshot: pc={rpc:04X} a={ra:02X} x={rx:02X} y={ry:02X} sp={rsp:02X} psw={rpsw:02X} F1={:02X}",
        io[1]
    );

    std::fs::write(format!("{out}.wav"), wav(&audio)).unwrap();
    let mark = audio.len();
    snes.debug_apu_trace_set(true);
    let (t0, _) = snes.debug_apu_trace_take();
    for _ in 0..after {
        snes.run_frame();
        snes.take_audio(&mut audio);
    }
    // <out>.trace: u64 start cycle, u64 end cycle, then (u64 cycle, u8 reg, u8 value)*
    let (t1, trace) = snes.debug_apu_trace_take();
    let mut tr = Vec::new();
    tr.extend_from_slice(&t0.to_le_bytes());
    tr.extend_from_slice(&t1.to_le_bytes());
    for (t, r, v) in &trace {
        tr.extend_from_slice(&t.to_le_bytes());
        tr.push(*r);
        tr.push(*v);
    }
    std::fs::write(format!("{out}.trace"), tr).unwrap();
    // <out>.ramtrace: (u64 cycle, u16 address, u8 value)*  (SMP writes to RAM)
    let mut rt = Vec::new();
    for (t, a, v) in snes.debug_apu_ram_trace_take() {
        rt.extend_from_slice(&t.to_le_bytes());
        rt.extend_from_slice(&a.to_le_bytes());
        rt.push(v);
    }
    std::fs::write(format!("{out}.ramtrace"), rt).unwrap();
    // <out>.ports: (u64 cycle, u8 port, u8 value)*  (main CPU writes to $2140-$2143)
    let mut pt = Vec::new();
    for (t, p, v) in snes.debug_apu_port_trace_take() {
        pt.extend_from_slice(&t.to_le_bytes());
        pt.push(p);
        pt.push(v);
    }
    std::fs::write(format!("{out}.ports"), pt).unwrap();
    // <out>.instr: (u64 cycle, u16 pc, u8 a, u8 x, u8 y, u8 sp, u8 psw)*  (state before each SMP instruction)
    let mut it = Vec::new();
    for (t, pc, a, x, y, sp, psw) in snes.debug_apu_instr_trace_take() {
        it.extend_from_slice(&t.to_le_bytes());
        it.extend_from_slice(&pc.to_le_bytes());
        it.extend_from_slice(&[a, x, y, sp, psw]);
    }
    std::fs::write(format!("{out}.instr"), it).unwrap();
    std::fs::write(format!("{out}_after.wav"), wav(&audio[mark..])).unwrap();
    println!("frames={frames}+{after} samples={} after={}", audio.len(), audio.len() - mark);
}
