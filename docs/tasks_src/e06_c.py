from e06_a import CHAIN
from _dsl import T

CORE = "crates/puce16-core"
APU = CORE + "/src/apu"
DSP = APU + "/dsp"
TST = CORE + "/tests"

DREADS = ["specs/07d_DSP_DESIGN.md (modules, structures, ordre d'un echantillon, integration)", "specs/07b_DSP.md (formules)", "specs/07_SPC700.md (registres $F0-$FF, timers, IPL)"]

# ------------------------------------------------------------------ E06 part C: squelette DSP, timers, IPL, ApuIo, Apu, integration (E06-31..37)

T("E06-31", "dsp/mod.rs: Dsp (fichier de registres, reset, hash) et Voice (structure)",
  size=130,
  context="""Squelette du DSP: les 128 registres avec leurs effets d'ecriture, l'etat de puissance, et la structure `Voice`. Le calcul d'un echantillon (`run_sample`) arrive plus tard: ici il ne fait RIEN.
Cette tache cree aussi les sous-modules vides `tables brr voice envelope echo` (voice.rs ne contient que la structure). """ + CHAIN,
  embed=["07d_DSP_DESIGN.md#2. Structures", "07b_DSP.md#1. Registres (acces via $F2/$F3, 128 octets, `regs[0x80]`)"],
  reads=DREADS,
  write=[DSP + "/mod.rs", DSP + "/voice.rs", DSP + "/tables.rs", DSP + "/brr.rs", DSP + "/envelope.rs", DSP + "/echo.rs", TST + "/e06_31_dsp_regs.rs"],
  spec="""1. `dsp/mod.rs`: `pub mod reg { ... }` avec, toutes de type `u8`: `MVOLL=0x0C MVOLR=0x1C EVOLL=0x2C EVOLR=0x3C KON=0x4C KOFF=0x5C FLG=0x6C ENDX=0x7C EFB=0x0D PMON=0x2D NON=0x3D EON=0x4D DIR=0x5D ESA=0x6D EDL=0x7D` et les decalages de voix `VOLL=0 VOLR=1 PITCHL=2 PITCHH=3 SRCN=4 ADSR1=5 ADSR2=6 GAIN=7 ENVX=8 OUTX=9`, plus deux fonctions `pub fn fir(i: usize) -> u8 { 0x0F + 0x10 * i as u8 }` et `pub fn vreg(x: usize, off: u8) -> usize { x * 16 + off as usize }` (indice dans `regs` du registre `off` de la voix `x`). Dans le code, on indexe `regs` par `reg::X as usize`. `pub mod tables; pub mod brr; pub mod voice; pub mod envelope; pub mod echo;`, la struct `Dsp` de 07d section 2 (champs prives) et:
```rust
impl Dsp {
    pub fn new() -> Dsp;                                     // power-on: regs tous 0 sauf FLG = 0xE0 ; counter 0 ; noise 0x4000 ; every_other true ; new_kon = kon = endx = 0 ; echo 0 ; hist 0 ; audio vide ; 8 Voice::new()
    pub fn reset(&mut self);                                 // FLG = 0xE0, les autres registres conserves ; voix, counter, noise, every_other, new_kon, kon, endx, echo, hist, audio remis a l'etat de power-on
    pub fn read(&mut self, addr: u8) -> u8;                  // regs[addr & 0x7F]
    pub fn write(&mut self, addr: u8, v: u8);                // a = addr & 0x7F ; regs[a] = v ; si a == 0x4C : new_kon = v ; si a == 0x7C : endx = 0 et regs[0x7C] = 0
    pub fn run_sample(&mut self, ram: &mut [u8; 65536]);     // E06-46 : ne fait rien pour l'instant
    pub fn take_audio(&mut self) -> Vec<i16>;                // vide et renvoie `audio` (L,R,L,R...)
    pub fn new_kon(&self) -> u8;  pub fn endx(&self) -> u8;  pub fn debug_set_endx(&mut self, v: u8);
    pub fn hash_into(&self, h: &mut crate::hash::Fnv64);     // regs (128 octets), counter, noise (u16), every_other (u8), new_kon, kon, endx (u8), echo_offset, echo_length (u16), hist_l, hist_r (chaque element en u32), puis chaque voix (Voice::hash_into)
}
impl Default for Dsp { ... }
```
2. `dsp/voice.rs`: `Mode`, `Voice` (07d section 2, champs `pub`), `Voice::new()` (kon_delay 0, mode Release, tous les nombres 0, buf [0;12]) et `Voice::hash_into(&self, h)` (kon_delay, mode en u8 (Attack 0 Decay 1 Sustain 2 Release 3), env, hidden_env, addr, block_header, group_in_block, buf (u32 chacun), buf_pos (u8), interp_pos, out, p1, p2).
3. `tables.rs`, `brr.rs`, `envelope.rs`, `echo.rs`: un commentaire `// filled by a later task`.""",
  tests=[("power_on", "`Dsp::new()`: `read(0x6C)==0xE0` ; `read(a)==0` pour a in 0..0x80 sauf 0x6C ; `new_kon()==0` ; `endx()==0` ; `take_audio()` vide."),
         ("write_read", "`write(0x00,0x55)`: `read(0x00)==0x55` ; `write(0x0F,0x7F)` (FIR0): `read(0x0F)==0x7F` ; registres sans effet stockes tels quels : `write(0x1D,0x9A)`: `read(0x1D)==0x9A`."),
         ("address_mask", "`write(0x81,0x07)`: `read(0x01)==0x07` et `read(0x81)==0x07` (adresse masquee a 7 bits)."),
         ("kon_write", "`write(0x4C,0x03)`: `read(0x4C)==0x03` et `new_kon()==0x03`."),
         ("endx_write_clears", "`debug_set_endx(0xFF)` : `endx()==0xFF` ; `write(0x7C,0x12)`: `endx()==0` et `read(0x7C)==0`."),
         ("reset_keeps_registers", "`write(0x00,1)`, `write(0x6C,0x20)`, `write(0x4C,1)` ; `reset()` : `read(0x6C)==0xE0`, `read(0x00)==1` (conserve), `new_kon()==0`."),
         ("reg_constants", "`reg::MVOLL==0x0C`, `reg::FLG==0x6C`, `reg::EDL==0x7D`, `reg::fir(0)==0x0F`, `reg::fir(7)==0x7F`, `reg::vreg(3, reg::GAIN)==0x37`, `reg::vreg(7, reg::OUTX)==0x79`."),
         ("voice_new", "`Voice::new()`: kon_delay==0, mode==Mode::Release, env==0, hidden_env==0, buf==[0;12], interp_pos==0, p1==0, p2==0."),
         ("hash_changes", "deux `Dsp::new()` ont le meme hash (Fnv64) ; apres `write(0x00,1)` il differe ; apres `debug_set_endx(1)` il differe aussi."),
         ("take_audio_drains", "`take_audio()` deux fois de suite : les deux vides (rien n'est produit encore).")],
  notes=CHAIN)

T("E06-32", "apu/timer.rs: les trois timers du SMP",
  size=90,
  context="Timers 0/1 (diviseur 128) et 2 (diviseur 16) du SPC700, decrits en 07 section 3.1. Structure pure, sans lien avec le bus. " + CHAIN,
  embed=["07_SPC700.md#3.1 Timers"],
  reads=["specs/07d_DSP_DESIGN.md"],
  write=[APU + "/timer.rs", APU + "/mod.rs", TST + "/e06_32_timers.rs"],
  spec="""Cree `apu/timer.rs` et declare `pub mod timer;` dans `apu/mod.rs` (sans rien changer d'autre dans `mod.rs`).
```rust
#[derive(Clone, Debug)]
pub struct Timer { divider: u16, enable: bool, target: u8, stage0: u16, stage1: u8, out: u8 }
impl Timer {
    pub fn new(divider: u16) -> Timer;           // tout a 0, enable faux
    pub fn tick(&mut self);                      // algorithme de 07 section 3.1 (un appel = un cycle SMP)
    pub fn set_enable(&mut self, on: bool);      // transition faux -> vrai : stage1 = 0 et out = 0 (stage0 est conserve) ; vrai -> faux ou inchange : seulement enable = on
    pub fn set_target(&mut self, t: u8);
    pub fn read_out(&mut self) -> u8;            // renvoie out & 0x0F puis out = 0
    pub fn out(&self) -> u8;  pub fn stage1(&self) -> u8;  pub fn stage0(&self) -> u16;  pub fn enabled(&self) -> bool;
    pub fn hash_into(&self, h: &mut crate::hash::Fnv64);   // divider (u16), enable (u8), target (u8), stage0 (u16), stage1 (u8), out (u8)
}
```""",
  tests=[("period_timer2", "`Timer::new(16)`, `set_target(3)`, `set_enable(true)` : apres 47 `tick` `out()==0` ; apres 48 `out()==1` ; apres 96 `out()==2` ; `stage1()` revient a 0 a chaque sortie."),
         ("read_clears", "meme timer apres 48 ticks : `read_out()==1` puis `read_out()==0`."),
         ("target_zero_is_256", "`Timer::new(16)`, cible 0, enable : apres 16*255 ticks `out()==0` ; apres 16*256 ticks `out()==1`."),
         ("disabled_runs_stage0_only", "timer desactive, 20 ticks : `stage0()==4`, `stage1()==0`, `out()==0`."),
         ("enable_midway", "`Timer::new(16)`, cible 1 ; 8 ticks desactive ; `set_enable(true)` ; 8 ticks de plus (16 au total) : `out()==1` (stage0 n'a pas ete remis a zero)."),
         ("enable_transition_resets", "cible 1, enable, 16 ticks : out==1 ; `set_enable(false)` ; `set_enable(true)` : `out()==0` et `stage1()==0`. Sans transition (`set_enable(true)` deux fois de suite) : out conserve."),
         ("out_wraps_4_bits", "`Timer::new(1)`, cible 1, enable : apres 15 ticks `out()==15` ; apres 16 ticks `out()==0`."),
         ("divider_128", "`Timer::new(128)`, cible 1, enable : 127 ticks -> out 0 ; 128 ticks -> out 1."),
         ("hash", "deux timers neufs de meme diviseur ont le meme hash ; apres `tick()` il differe.")],
  notes=CHAIN)

T("E06-33", "apu/ipl.rs: la ROM IPL de 64 octets",
  size=50, kind="data",
  context="Les 64 octets de la ROM de demarrage du SMP, a recopier depuis le fichier de donnees (pas de memoire). " + CHAIN,
  reads=["specs/data/ipl_rom.hex (64 octets en hexa, 16 par ligne)"],
  write=[APU + "/ipl.rs", APU + "/mod.rs", TST + "/e06_33_ipl.rs"],
  spec="""Cree `apu/ipl.rs` avec `pub const IPL_ROM: [u8; 64] = [ ... ];` (les 64 octets du fichier `specs/data/ipl_rom.hex`, dans l'ordre, ecrits `0xCD, 0xEF, ...`) et declare `pub mod ipl;` dans `apu/mod.rs` (sans rien changer d'autre). Ajoute un commentaire d'en-tete: `// IPL ROM of the S-SMP, copied from docs/specs/data/ipl_rom.hex`.""",
  tests=[("length_and_sum", "`IPL_ROM.len()==64` ; la somme des 64 octets (en u32) vaut 10936."),
         ("known_bytes", "`IPL_ROM[0]==0xCD`, `IPL_ROM[1]==0xEF` (MOV X,#$EF) ; `IPL_ROM[9..12]==[0x8F,0xAA,0xF4]` (MOV $F4,#$AA) ; `IPL_ROM[47]==0xBA` (MOVW YA,$F6 a $FFEF) ; `IPL_ROM[59]==0x1F` (JMP [$0000+X])."),
         ("reset_vector", "`IPL_ROM[62]==0xC0` et `IPL_ROM[63]==0xFF` : le vecteur de reset ($FFFE/$FFFF) pointe sur $FFC0.")],
  notes=CHAIN)

T("E06-34", "apu/io.rs: ApuIo (RAM 64 Kio, registres $F0-$FF, ports, timers, IPL)",
  size=150,
  context="""Le bus du SPC700 dans la console: RAM, IPL superposee en $FFC0-$FFFF, registres $F0-$FF, ports, timers et DSP. Chaque acces (`read`, `write`, `idle`) fait l'acces PUIS un `tick()` (timers + diviseur DSP). """ + CHAIN,
  embed=["07_SPC700.md#3. Registres $F0-$FF", "07_SPC700.md#2. Memoire 64 Kio", "07d_DSP_DESIGN.md#4. APU (integration)"],
  reads=DREADS,
  write=[APU + "/io.rs", APU + "/mod.rs", TST + "/e06_34_io.rs"],
  spec="""Cree `apu/io.rs` et declare `pub mod io;` dans `apu/mod.rs` (sans rien changer d'autre).
```rust
pub struct ApuIo { pub ram: Box<[u8; 65536]>, pub dsp: Dsp, pub timers: [Timer; 3], pub port_in: [u8; 4], pub port_out: [u8; 4], pub control: u8, pub dspaddr: u8, dsp_div: u8 }
impl ApuIo { pub fn new() -> ApuIo; pub fn reset(&mut self); pub fn hash_into(&self, h: &mut crate::hash::Fnv64); }
impl SmpBus for ApuIo { fn read(&mut self, addr: u16) -> u8; fn write(&mut self, addr: u16, v: u8); fn idle(&mut self); }
```
`new`: RAM a 0 (`UNKNOWN - to confirm`), `control = 0xB0`, timers `Timer::new(128)`, `Timer::new(128)`, `Timer::new(16)`, ports 0, `dspaddr = 0`, `dsp_div = 0`, `Dsp::new()`. `reset`: ports 0, `control = 0xB0`, `dspaddr = 0`, `dsp_div = 0`, timers neufs, `dsp.reset()`; la RAM est CONSERVEE.
`tick()` (prive, appele apres chaque acces): `for t in timers { t.tick() }` ; `dsp_div += 1` ; si `dsp_div == 32` : `dsp_div = 0` ; `dsp.run_sample(&mut *ram)`.
`read(addr)`: `$F0`->0 ; `$F1`->0 ; `$F2`-> `dspaddr & 0x7F` ; `$F3` -> `dsp.read(dspaddr & 0x7F)` ; `$F4..=$F7` -> `port_in[addr-0xF4]` ; `$F8`,`$F9` -> RAM ; `$FA..=$FC` -> 0 ; `$FD..=$FF` -> `timers[addr-0xFD].read_out()` ; `$FFC0..=$FFFF` -> `IPL_ROM[addr-0xFFC0]` si `control & 0x80 != 0` sinon RAM ; sinon RAM.
`write(addr, v)`: `$F0` ignore ; `$F1` : `control = v` ; `timers[i].set_enable(v >> i & 1 != 0)` pour i in 0..3 ; si `v & 0x10` : `port_in[0] = port_in[1] = 0` ; si `v & 0x20` : `port_in[2] = port_in[3] = 0` ; `$F2` : `dspaddr = v` ; `$F3` : si `dspaddr & 0x80 == 0` : `dsp.write(dspaddr, v)` ; `$F4..=$F7` -> `port_out[addr-0xF4] = v` ; `$F8`,`$F9` -> RAM ; `$FA..=$FC` -> `timers[addr-0xFA].set_target(v)` ; `$FD..=$FF` ignore ; toute autre adresse (y compris `$FFC0..=$FFFF`, meme IPL visible) -> RAM. (Les ecritures dans `$F0-$F7` et `$FA-$FF` ne modifient PAS la RAM.)
`idle()` ne fait que le `tick()`.""",
  tests=[("ipl_visible_default", "`ApuIo::new()`: `read(0xFFC0)==0xCD` ; `write(0xFFC0,0x11)` ; `read(0xFFC0)==0xCD` (IPL visible, l'ecriture est allee dans la RAM) ; `write(0xF1,0x00)` : `read(0xFFC0)==0x11` ; `write(0xF1,0x80)`: `read(0xFFC0)==0xCD`."),
         ("ram_plain", "`write(0x0200,0x5A)`: `read(0x0200)==0x5A` ; `write(0xF8,0x12)`: `read(0xF8)==0x12` ; `write(0xF9,0x34)`: `read(0xF9)==0x34`."),
         ("test_and_control_read_zero", "`read(0xF0)==0` ; apres `write(0xF1,0x80)`: `read(0xF1)==0`."),
         ("ports_directions", "`port_in[0]=0x11` (champ public) : `read(0xF4)==0x11` ; `write(0xF4,0x22)`: `port_out[0]==0x22` et `port_in[0]==0x11` ; idem pour $F7 / index 3."),
         ("control_clears_inputs", "`port_in=[1,2,3,4]` ; `write(0xF1,0x10)`: `port_in==[0,0,3,4]` ; `port_in=[1,2,3,4]`, `write(0xF1,0x20)`: `[1,2,0,0]` ; `write(0xF1,0x30)`: tout a 0. `port_out` jamais touche."),
         ("dsp_access", "`write(0xF2,0x0C)`, `write(0xF3,0x55)`: `read(0xF3)==0x55` et `dsp.read(0x0C)==0x55` ; `write(0xF2,0x8C)`, `write(0xF3,0x66)` : ignore (`dsp.read(0x0C)==0x55`) ; `read(0xF2)==0x0C` (bit 7 lu a 0)."),
         ("timer_targets_read_zero", "`write(0xFA,3)`, `write(0xFB,4)`, `write(0xFC,5)` : `read(0xFA)==0`, `read(0xFB)==0`, `read(0xFC)==0` (les cibles sont en ecriture seule) ; `read(0xFD)==0`, `read(0xFE)==0`, `read(0xFF)==0`."),
         ("timer2_counts", "sequence : `write(0xFC,1)` (cible du timer 2, acces 1) puis `write(0xF1,0x04)` (active le timer 2, acces 2) puis 13 `idle()` (15 acces au total) : `read(0xFF)==0` ; le meme scenario avec 14 `idle()` (16 acces au total avant le read) : `read(0xFF)==1` puis `read(0xFF)==0`. (stage0 tourne depuis la creation: la sortie arrive au 16eme tick.)"),
         ("timer_enable_transition", "meme scenario avec 14 idle (out==1 non lu), puis `write(0xF1,0x00)` et `write(0xF1,0x04)` : `read(0xFF)==0` (out remis a 0 par la transition)."),
         ("timer0_divider_128", "`write(0xFA,1)`, `write(0xF1,0x01)` puis 126 `idle()` (2 ecritures + 126 idle = 128 ticks) : `timers[0].out()==1` ; avec 125 `idle()` seulement : `timers[0].out()==0`."),
         ("reset", "ports non nuls, `control=0`, `write(0x0200,7)` ; `reset()` : ports 0, `control==0xB0`, `read(0x0200)==7` (RAM conservee)."),
         ("hash", "deux `ApuIo::new()` ont le meme hash ; `write(0x0200,1)` le change.")],
  notes=CHAIN)

T("E06-35", "apu/mod.rs: Apu complet (SMP + ApuIo), rattrapage step_to, region",
  size=150,
  context="""L'`Apu` reel remplace le squelette de E03-12 en gardant son API (`port_read`, `port_write`, `step_to`, `cpu_to_apu`, `debug_set_from_apu`, `hash_into`). `step_to(master)` fait tourner le SMP pour rattraper l'horloge maitre (07 section 8.1).
Attention: l'APU repond maintenant `$AA`/`$BB` sur `$2140`/`$2141` apres ~50 000 clocks maitres. Si un test E03/E04 echoue uniquement a cause de cela, corrige LE TEST (attente documentee dans son commentaire) et signale-le dans le rapport. """ + CHAIN,
  embed=["07d_DSP_DESIGN.md#4. APU (integration)", "07_SPC700.md#8. Interface Rust", "09_SYSTEM.md#3.3 Apu (squelette, `apu/mod.rs`; sera complete en E06)"],
  reads=DREADS,
  write=[APU + "/mod.rs", CORE + "/src/bus/mod.rs", TST + "/e03_12_apu_skeleton.rs", TST + "/e06_35_apu.rs"],
  spec="""Dans `apu/mod.rs` (garde `pub mod spc700; pub mod dsp; pub mod timer; pub mod ipl; pub mod io;`):
```rust
pub struct Apu { spc: Spc700, io: ApuIo, acc: i64, last_master: u64, master_hz: u64 }
impl Apu {
    pub fn new() -> Apu;                       // master_hz = MASTER_HZ_NTSC ; spc = Spc700::new() avec pc = 0xFFC0 (vecteur de l'IPL : (IPL_ROM[63] << 8) | IPL_ROM[62]) ; acc = 0 ; last_master = 0
    pub fn reset(&mut self);                   // io.reset() ; spc = Spc700::new() + pc comme ci-dessus ; acc = 0 ; last_master = 0 (master_hz conserve)
    pub fn set_region(&mut self, r: Region);   // master_hz = r.master_hz()
    pub fn port_read(&self, i: usize) -> u8;   // io.port_out[i & 3]
    pub fn port_write(&mut self, i: usize, v: u8);   // io.port_in[i & 3] = v
    pub fn step_to(&mut self, master: u64);    // 07d section 4 (si master <= last_master : ne fait rien)
    pub fn cpu_to_apu(&self, i: usize) -> u8;  // io.port_in[i & 3]
    pub fn debug_set_from_apu(&mut self, i: usize, v: u8);   // io.port_out[i & 3] = v
    pub fn take_audio(&mut self) -> Vec<i16>;  // io.dsp.take_audio()
    pub fn debug_poke(&mut self, addr: u16, bytes: &[u8]);   // ecrit dans io.ram sans tick (wrap 16 bits)
    pub fn debug_peek(&self, addr: u16) -> u8;               // io.ram[addr]
    pub fn debug_set_pc(&mut self, pc: u16);   pub fn debug_pc(&self) -> u16;
    pub fn debug_dsp_write(&mut self, reg: u8, v: u8);       // io.dsp.write(reg, v)
    pub fn hash_into(&self, h: &mut crate::hash::Fnv64);     // spc : a x y sp (u8) pc (u16) psw halted (u8) ; acc (u64 as i64) ; last_master (u64) ; puis io.hash_into
}
```
`step_to`: `acc += (master - last_master) as i64 * APU_RESONATOR_HZ as i64 ; last_master = master ; while acc > 0 { let c = spc.step(&mut io) ; acc -= c as i64 * 24 * master_hz as i64 }`.
Dans `bus/mod.rs`: `SysBus::new(cart, region)` appelle `apu.set_region(region)` juste apres la creation de l'Apu ; `SysBus::reset` conserve la region de l'Apu (rien d'autre a changer).
Dans `tests/e03_12_apu_skeleton.rs`: SUPPRIME le test `e03_12_step_to_noop` (obsolete); les autres tests de ce fichier doivent rester verts sans changement.""",
  tests=[("new_defaults", "`Apu::new()`: `port_read(i)==0` pour i in 0..4 ; `cpu_to_apu(i)==0` ; `debug_pc()==0xFFC0`."),
         ("handshake_aa_bb", "`step_to(100_000)`: `port_read(0)==0xAA` et `port_read(1)==0xBB` (miroirs : `port_read(4)==0xAA`)."),
         ("not_yet_before_clear_loop", "`step_to(10_000)`: `port_read(0)==0` (l'IPL efface encore la RAM)."),
         ("step_to_is_incremental", "A: `step_to(100_000)` ; B: `step_to(50_000)` puis `step_to(100_000)` : les deux ont le meme hash (la decoupe n'a pas d'effet)."),
         ("step_to_backwards_noop", "apres `step_to(100_000)`, `step_to(50_000)` ne change pas le hash."),
         ("ipl_clears_low_ram", "`debug_poke(0x0010,&[0x99])` puis `step_to(100_000)` : `debug_peek(0x0010)==0`."),
         ("port_write_visible_to_smp", "apres le handshake, `port_write(0,0xCC)`: `cpu_to_apu(0)==0xCC` ; `port_read(0)` reste `0xAA` tant que le SMP n'a pas repondu."),
         ("set_region_pal", "`set_region(Region::Pal)` puis `step_to(100_000)` : handshake AA/BB aussi (pas de panique)."),
         ("reset_restarts_ipl", "apres le handshake, `reset()`: `port_read(0)==0`, `debug_pc()==0xFFC0`, puis `step_to(100_000)` redonne 0xAA."),
         ("debug_dsp_write", "`debug_dsp_write(0x0C,0x40)` puis `hash` different de celui d'un Apu neuf."),
         ("take_audio_empty", "`take_audio()` est vide juste apres `new()`."),
         ("determinism", "deux `Apu::new()` avances de la meme facon donnent le meme hash.")],
  notes=CHAIN)

T("E06-36", "Tests d'integration IPL: handshake et envoi d'un programme par les ports",
  size=110,
  context="""Preuve que l'Apu complet execute la vraie ROM IPL: le S-CPU (ici simule par des appels `port_write`/`port_read`) envoie un petit programme par le protocole IPL et le lance.
Protocole (reference: l'IPL desassemblee) : (1) attendre `$AA` en port 0 et `$BB` en port 1 ; (2) ecrire l'adresse destination en ports 2 (bas) et 3 (haut), un octet NON NUL en port 1, puis `$CC` en port 0 ; attendre l'ecoute `$CC` en port 0 ; (3) pour chaque octet i (i = 0, 1, 2...) : ecrire l'octet en port 1, puis i (u8) en port 0, attendre que port 0 relu vaille i ; (4) pour lancer : ecrire l'adresse d'entree en ports 2/3, `0` en port 1, puis `(n + 1) as u8` en port 0 (n = nombre d'octets envoyes).""" + " " + CHAIN,
  write=[TST + "/e06_36_ipl_upload.rs"],
  spec="""Ecris `tests/e06_36_ipl_upload.rs` (API publique seulement: `Apu::new`, `port_write`, `port_read`, `step_to`, `debug_peek`).
Aides: `fn run(apu: &mut Apu, master: &mut u64, clocks: u64) { *master += clocks; apu.step_to(*master); }` et `fn wait_port(apu: &mut Apu, master: &mut u64, port: usize, val: u8) -> bool` qui avance par pas de 2000 clocks, au plus 200 fois, jusqu'a `port_read(port) == val`.
Programme de test (6 octets) a l'adresse 0x0200: `E8 42  C4 F4  2F FE` = `MOV A,#$42 ; MOV $F4,A ; BRA -2` (met `$42` dans le port 0 cote CPU puis boucle).""",
  tests=[("handshake", "`Apu::new()` ; `run(100_000)` : `port_read(0)==0xAA` et `port_read(1)==0xBB`."),
         ("upload_and_run", "handshake, puis les etapes (2) a (4) du protocole avec le programme de 6 octets a 0x0200 et l'entree 0x0200 : `wait_port(0, 0xCC)` vrai a l'etape 2 ; chaque echo d'octet attendu vrai ; apres le lancement `wait_port(0, 0x42)` vrai ; `debug_peek(0x0200)==0xE8`, `debug_peek(0x0205)==0xFE`."),
         ("upload_two_blocks", "(a) handshake puis premier transfert: le programme de 6 octets a 0x0200 (etapes 2 et 3, SANS lancer); (b) commande du second transfert: ports 2/3 = 0x0300, port 1 = 1, port 0 = 7 (= n + 1, n = 6) ; attendre `port_read(0)==7` ; (c) envoyer les 3 octets `[0xAA,0xBB,0xCC]` avec les compteurs 0, 1, 2 (attendre l'echo de chaque compteur) ; (d) lancer: ports 2/3 = 0x0200, port 1 = 0, port 0 = 4 (= 3 + 1) ; attendre `port_read(0)==0x42` ; verifier `debug_peek(0x0300)==0xAA`, `debug_peek(0x0301)==0xBB`, `debug_peek(0x0302)==0xCC` et `debug_peek(0x0200)==0xE8`."),
         ("ipl_does_not_start_before_cc", "apres le handshake, sans ecrire `$CC` : `step_to` de 100 000 clocks supplementaires : `port_read(0)` vaut toujours 0xAA et `debug_peek(0x0200)==0`.")],
  notes=CHAIN + "\nSi le programme n'est pas lance, ne change PAS l'Apu au hasard: le protocole ci-dessus est la reference; ecris une question `spec` avec les valeurs de ports observees.")

T("E06-37", "Snes: le S-CPU voit le handshake de l'IPL (jalon audio A1)",
  size=100,
  context="""Premier test de bout en bout du SPC700 dans la console: un vrai programme 65816 attend `$AA` en `$2140` et `$BB` en `$2141`, puis allume l'ecran en rouge. L'image est rouge SI ET SEULEMENT SI l'APU a repondu. Programme (LoRom, comme E04-15). """ + CHAIN,
  write=[TST + "/e06_37_snes_handshake.rs"],
  spec="""Programme `PROG_APU` (a 0x8000, 41 octets) :
`78 18 FB` SEI ; CLC ; XCE
`AD 40 21` LDA $2140 ; `C9 AA` CMP #$AA ; `D0 F9` BNE (retour au LDA $2140)
`AD 41 21` LDA $2141 ; `C9 BB` CMP #$BB ; `D0 F2` BNE (retour au LDA $2140)
`A9 00 8D 21 21` ; `A9 1F 8D 22 21` ; `A9 00 8D 22 21` (couleur 0 = rouge) ; `A9 0F 8D 00 21` (ecran allume) ; `80 FE` (boucle).
Soit: `78 18 FB AD 40 21 C9 AA D0 F9 AD 41 21 C9 BB D0 F2 A9 00 8D 21 21 A9 1F 8D 22 21 A9 00 8D 22 21 A9 0F 8D 00 21 80 FE`.
Utilise le meme montage que `tests/e04_15_snes_video.rs` (module `common`, `build_rom(&[(0, PROG)], &[])`, `Snes::new(&rom, None)`).""",
  tests=[("handshake_turns_screen_red", "`Snes::new` sur `PROG_APU` ; `run_frame()` 3 fois : `frame().width==256`, `frame().height==224`, tous les 57344 pixels == 0xFFFF0000 (rouge : l'APU a repondu)."),
         ("first_frame_still_black", "apres UN seul `run_frame()` (~357 000 clocks maitres mais le SMP n'a pas encore fini d'effacer sa RAM selon le timing) : si l'image est deja rouge le test reste valide ; ce test verifie seulement que `frame()` est soit tout noir (0xFF000000) soit tout rouge (aucun pixel d'une autre couleur)."),
         ("determinism", "deux `Snes` identiques apres 3 frames ont le meme `state_hash()`."),
         ("reset_restarts", "apres 3 frames rouges, `reset()` puis 3 frames : image rouge de nouveau (le handshake est refait).")],
  notes=CHAIN + "\nSi l'image reste noire: tracer avec `Snes::state_hash` ne suffit pas; poser une question `spec` (le S-CPU lit-il bien `$2140` via `apu.port_read` et l'APU avance-t-il pendant les acces du CPU ?).")
