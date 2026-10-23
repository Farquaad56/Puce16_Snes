# SPEC 07d - S-DSP et integration APU: conception du code

Complete `07b_DSP.md` (formules) et `07_SPC700.md` (registres $F0-$FF). Pour que chaque tache ne touche qu'un ou deux fichiers.

## 1. Modules et privilege Rust
`puce16-core/src/apu/`: `mod.rs` (Apu), `io.rs` (ApuIo: RAM, registres $F0-$FF, ports, timers, DSP), `timer.rs`, `ipl.rs`, `spc700/*` (07c), `dsp/{mod,tables,brr,voice,envelope,echo}.rs`.
`dsp/voice.rs`, `dsp/envelope.rs` et `dsp/echo.rs` sont des **sous-modules de `dsp`**: leurs blocs `impl Dsp { ... }` accedent aux champs prives de `Dsp` (meme astuce que `ppu/bg.rs`). Les tests (dossier `tests/`) n'utilisent que l'API publique et les fonctions pures.

## 2. Structures
```rust
// dsp/voice.rs
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Mode { Attack, Decay, Sustain, Release }
#[derive(Clone, Debug)]
pub struct Voice { pub kon_delay: u8, pub mode: Mode, pub env: i32, pub hidden_env: i32, pub addr: u16, pub block_header: u8, pub group_in_block: u8,
                   pub buf: [i32; 12], pub buf_pos: usize, pub interp_pos: u32, pub out: i32, pub p1: i32, pub p2: i32 }
// dsp/mod.rs
pub struct Dsp { regs: [u8; 128], voices: [Voice; 8], counter: u16, noise: u16, every_other: bool, new_kon: u8, kon: u8, endx: u8,
                 echo_offset: u16, echo_length: u16, hist_l: [i32; 8], hist_r: [i32; 8], audio: Vec<i16> }
```
Constantes de registres (`pub mod reg` dans `dsp/mod.rs`, valeurs de 07b section 1): `MVOLL=0x0C MVOLR=0x1C EVOLL=0x2C EVOLR=0x3C KON=0x4C KOFF=0x5C FLG=0x6C ENDX=0x7C EFB=0x0D PMON=0x2D NON=0x3D EON=0x4D DIR=0x5D ESA=0x6D EDL=0x7D`; voix `x`: base `x*0x10`, `VOLL=0 VOLR=1 PITCHL=2 PITCHH=3 SRCN=4 ADSR1=5 ADSR2=6 GAIN=7 ENVX=8 OUTX=9`; FIR_i = `0x0F + 0x10*i`.

## 3. Ordre d'un echantillon (`Dsp::run_sample(&mut self, ram: &mut [u8; 65536])`)
1. `counter = if counter == 0 { 0x77FF } else { counter - 1 }`.
2. Bruit: `if poll(counter, FLG & 0x1F) { fb = (noise << 13) ^ (noise << 14); noise = (fb & 0x4000) ^ (noise >> 1) }`.
3. Scrutation KON/KOFF et soft reset (07b section 6).
4. Les 8 voix dans l'ordre 0..7 (07b section 4); `sum_l/sum_r/echo_l/echo_r` remis a 0 avant.
5. Echo, sortie, ecriture echo (07b section 7); `audio.push(L); audio.push(R)`.
6. Mise a jour de `regs[ENDX] = endx`.

## 4. APU (integration)
`ApuIo` (io.rs) possede `ram: Box<[u8; 65536]>`, `dsp: Dsp`, `timers: [Timer; 3]`, `port_in: [u8; 4]` (ecrits par le S-CPU, lus par le SMP en $F4-$F7), `port_out: [u8; 4]` (ecrits par le SMP, lus par le S-CPU), `control: u8` (power-on 0xB0), `dspaddr: u8`, `dsp_div: u8`. Il implemente `SmpBus`: chaque `read`/`write`/`idle` effectue l'acces PUIS `tick()` (3 timers `tick()`, `dsp_div += 1`, et a 32: `dsp_div = 0; dsp.run_sample(&mut ram)`). (`UNKNOWN - to confirm`: tick avant ou apres l'acces.)
`Apu { spc: Spc700, io: ApuIo, acc: i64, last_master: u64, master_hz: u64 }`; `step_to(master)`: `acc += (master - last_master) as i64 * APU_RESONATOR_HZ as i64; last_master = master; while acc > 0 { let c = spc.step(&mut io); acc -= c as i64 * 24 * master_hz as i64 }`.
Les ports vus par le S-CPU: `port_read(i) = io.port_out[i & 3]`, `port_write(i, v)`: `io.port_in[i & 3] = v`.
