from e06_a import CHAIN
from e03_e import HDR
from _dsl import T

CORE = "crates/puce16-core"
SRC = CORE + "/src"
TST = CORE + "/tests"

RULES = ("""Regle commune des sauvegardes d'etat (spec 08 section 9, spec 09 section 9): `impl State for X` sauve, DANS L'ORDRE DE DECLARATION, tous les champs de X qui (a) sont couverts par `hash_into`, ou (b) influencent le comportement futur. Ne sauve PAS les constantes derivees de la region ou de la ROM (reconstruites par le constructeur): les lister dans un commentaire `// not saved: <champ> (<raison>)`.
Les tableaux de taille fixe s'ecrivent sans prefixe de longueur (`bytes`, `slice_u16`), les `Vec` de taille variable avec prefixe (`vec_u8`, `vec_u16`). Un enum est sauve par un `u8` d'indice explicite; une valeur inconnue au chargement donne `StateError::Mismatch("<champ>")`. `load` ecrase les champs en place.
Les tests sont des tests unitaires DANS le fichier (`#[cfg(test)] mod tests`, noms `e08_NN_...`): ils peuvent donc ecrire les champs prives. Forme des tests: (1) `roundtrip_all_fields`: mettre CHAQUE champ sauve a une valeur non nulle et distincte, `save`, `load` dans une instance neuve (constructeur habituel du type), puis re-`save` de la copie = memes octets ET `hash_into` identique (si le type en a un); (2) `truncated`: pour n dans {0, len/2, len-1}, `load` sur `data[..n]` renvoie `Err(StateError::Truncated)`; (3) un test comportemental propre au type (indique ci-dessous).""")
EMB = ["08_INPUT_DESKTOP.md#9. Save states"]

# ------------------------------------------------------------------ E08 part A: save states du coeur (E08-01..14)

T("E08-01", "savestate.rs: StateWriter, StateReader, StateError et trait State",
  size=140,
  context="Primitives binaires des sauvegardes d'etat: little-endian, aucune dependance, aucun serde. Toutes les taches suivantes les utilisent. " + CHAIN,
  embed=EMB,
  write=[SRC + "/savestate.rs", TST + "/e08_01_savestate.rs"],
  spec="""Dans `savestate.rs` (le module est deja declare dans lib.rs):
```rust
pub const STATE_VERSION: u32 = 1;
#[derive(Clone, Debug, PartialEq, Eq)] pub enum StateError { Truncated, Version(u32), Mismatch(&'static str) }
impl std::fmt::Display for StateError { ... }     // "state truncated" / "unsupported state version <n>" / "state mismatch: <what>"
impl std::error::Error for StateError {}
pub struct StateWriter { buf: Vec<u8> }
impl StateWriter {
    pub fn new() -> StateWriter;  pub fn len(&self) -> usize;  pub fn finish(self) -> Vec<u8>;
    pub fn u8(&mut self, v: u8);  pub fn u16(&mut self, v: u16);  pub fn u32(&mut self, v: u32);  pub fn u64(&mut self, v: u64);
    pub fn i16(&mut self, v: i16); pub fn i32(&mut self, v: i32); pub fn i64(&mut self, v: i64); pub fn bool(&mut self, v: bool);   // bool = 0 ou 1
    pub fn bytes(&mut self, v: &[u8]);            // brut, sans prefixe
    pub fn slice_u16(&mut self, v: &[u16]);       // brut, 2 octets LE chacun
    pub fn slice_u32(&mut self, v: &[u32]);       // brut, 4 octets LE chacun
    pub fn vec_u8(&mut self, v: &[u8]);           // prefixe u32 (longueur) puis octets
    pub fn vec_u16(&mut self, v: &[u16]);         // prefixe u32 (nombre d'elements) puis elements
}
pub struct StateReader<'a> { data: &'a [u8], pos: usize }
impl<'a> StateReader<'a> {
    pub fn new(data: &'a [u8]) -> StateReader<'a>;  pub fn remaining(&self) -> usize;
    pub fn u8(&mut self) -> Result<u8, StateError>;  /* u16 u32 u64 i16 i32 i64 */  pub fn bool(&mut self) -> Result<bool, StateError>;   // octet > 1 -> Mismatch("bool")
    pub fn bytes_into(&mut self, out: &mut [u8]) -> Result<(), StateError>;
    pub fn slice_u16_into(&mut self, out: &mut [u16]) -> Result<(), StateError>;
    pub fn slice_u32_into(&mut self, out: &mut [u32]) -> Result<(), StateError>;
    pub fn vec_u8(&mut self) -> Result<Vec<u8>, StateError>;      // longueur > restant -> Truncated (ne jamais allouer avant de verifier)
    pub fn vec_u16(&mut self) -> Result<Vec<u16>, StateError>;
    pub fn finish(self) -> Result<(), StateError>;                // octets restants -> Mismatch("trailing data")
}
pub trait State { fn save(&self, w: &mut StateWriter); fn load(&mut self, r: &mut StateReader) -> Result<(), StateError>; }
```
Toute lecture au-dela de la fin renvoie `StateError::Truncated` (jamais de panique).""",
  tests=[("little_endian_bytes", "`w.u16(0x1234); w.u32(0x01020304); w.u64(0x0102030405060708)` -> `finish()` == `[0x34,0x12, 0x04,0x03,0x02,0x01, 8,7,6,5,4,3,2,1]`."),
         ("signed_and_bool", "`w.i16(-2); w.i32(-3); w.i64(-4); w.bool(true); w.bool(false)` -> octets `[0xFE,0xFF, 0xFD,0xFF,0xFF,0xFF, 0xFC,0xFF*7, 1, 0]` et relecture identique."),
         ("roundtrip_all", "ecrire un u8, un u16, un u32, un u64, un i16, un i32, un i64, deux bool, `bytes([1,2,3])`, `slice_u16([0x1111,0x2222])`, `slice_u32([7,8])`, `vec_u8([9,9])`, `vec_u16([5])` ; relire dans le meme ordre : valeurs egales ; `finish()` Ok."),
         ("slice_layouts", "`slice_u16(&[0x0102])` -> `[2,1]` ; `slice_u32(&[1])` -> `[1,0,0,0]` ; `vec_u8(&[7,8])` -> `[2,0,0,0,7,8]` ; `vec_u16(&[0x0A0B])` -> `[1,0,0,0,0x0B,0x0A]`."),
         ("truncated_scalars", "lecteur sur `[1]` : `u16()` == Err(Truncated) ; sur `[]` : `u8()` == Err(Truncated) ; sur 7 octets : `u64()` == Err(Truncated)."),
         ("truncated_bytes_into", "`bytes_into` d'un tampon de 4 sur 3 octets -> Err(Truncated) ; idem `slice_u16_into` de 2 elements sur 3 octets."),
         ("vec_huge_length", "lecteur sur `[0xFF,0xFF,0xFF,0xFF, 1]` : `vec_u8()` == Err(Truncated) sans allocation enorme ni panique."),
         ("bool_invalid", "lecteur sur `[2]` : `bool()` == Err(Mismatch(\"bool\"))."),
         ("finish_trailing", "ecrire 2 octets, n'en lire qu'un : `finish()` == Err(Mismatch(\"trailing data\")) ; tout lire : Ok."),
         ("display", "`format!(\"{}\", StateError::Truncated)==\"state truncated\"` ; `Version(3)` -> `\"unsupported state version 3\"` ; `Mismatch(\"region\")` -> `\"state mismatch: region\"`."),
         ("trait_object_free_roundtrip", "un type local de test `struct P{a:u16,b:bool}` implemente `State` (ecrit `a` puis `b`) : save -> load dans un `P` neuf -> egal."),
         ("version_constant", "`STATE_VERSION==1`.")],
  notes=CHAIN)

T("E08-02", "State pour Cpu (cpu/mod.rs)",
  size=110,
  context="Sauvegarde/restauration du coeur 65816 (registres, drapeaux caches, etat d'attente/arret, lignes d'interruption echantillonnees). " + RULES + " " + CHAIN,
  embed=EMB,
  reads=["cpu/mod.rs (struct Cpu et son hash_into: tous ses champs)"],
  write=[SRC + "/cpu/mod.rs"],
  spec="""Dans `cpu/mod.rs`, ajoute `impl crate::savestate::State for Cpu` (save + load). Les champs de `Cpu` (registres a x y s d db pb pc p e, WAI/STP, files d'interruption, etc.) sont lus dans la definition de la struct et dans `hash_into` : sauver TOUS ceux de `hash_into` plus tout champ d'etat non hash (decide dans le commentaire `// not saved:`). Test comportemental: construire deux `Cpu` identiques via save/load, executer les memes 10 instructions (programme simple a 0x008000 : `A9 12 18 69 01 EB 80 FE` etc. sur un `FlatBus` de `cpu::testbus`) sur chacun et verifier `hash_into` egal a la fin.""",
  tests=[("roundtrip_all_fields", "voir la regle commune (champs de `Cpu` mis a des valeurs distinctes non nulles)."),
         ("truncated", "voir la regle commune."),
         ("behaves_identically_after_load", "voir le test comportemental ci-dessus : `hash_into` des deux `Cpu` egal apres 10 `step`."),
         ("wai_stp_state_kept", "un `Cpu` en attente (`WAI` execute) puis sauve/restaure reste en attente : un `step` sans interruption ne fait pas avancer `pc`.")],
  accept=["cargo test -p puce16-core e08_02_"])

T("E08-03", "State pour Joypads, Cart (SRAM) et Wram",
  size=120,
  context="Trois petits composants dans trois fichiers. Cart ne sauve QUE la SRAM (la ROM n'est jamais sauvee). " + RULES + " " + CHAIN,
  embed=EMB,
  write=[SRC + "/joypad.rs", SRC + "/cart/mod.rs", SRC + "/bus/wram.rs"],
  spec="""1. `joypad.rs`: `impl State for Joypads` (`state`, `shift`, `out0`).
2. `cart/mod.rs`: `impl State for Cart`: sauve la SRAM avec `vec_u8(&sram)`; au `load`, la longueur lue DOIT egaler `self.sram.len()` sinon `Mismatch("sram size")` (sans modifier la SRAM) ; octets copies en place. Aucune autre donnee (rom, header, map, region: `// not saved`).
3. `bus/wram.rs`: `impl State for Wram`: `mem` (128 Kio, `bytes` sans prefixe) puis `addr` (u32).
Tests (dans chaque fichier): voir la regle commune ; en plus Cart: SRAM de 8 Kio remplie d'un motif, restauree dans un Cart neuf de MEME taille ; `load` sur un Cart a SRAM de 2 Kio -> `Err(Mismatch(\"sram size\"))` et SRAM inchangee ; cart sans SRAM: save ecrit un `vec_u8` vide, load Ok.""",
  tests=[("joypads_roundtrip", "dans `joypad.rs` : regle commune."),
         ("cart_roundtrip_and_size_mismatch", "dans `cart/mod.rs` : voir ci-dessus."),
         ("wram_roundtrip", "dans `bus/wram.rs` : motif `mem[i] = (i * 7) as u8`, `addr = 0x1ABCD` ; restauration a l'identique ; taille des octets sauves == 0x20000 + 4."),
         ("truncated_each", "pour chacun des trois types : troncature -> `Err(Truncated)`.")],
  accept=["cargo test -p puce16-core e08_03_"])

T("E08-04", "State pour Mmio et Timing",
  size=110,
  context="Registres $4200-$421F et division/multiplication en cours (Mmio), compteurs de ligne, rafraichissement et latchs d'interruption (Timing). " + RULES + " " + CHAIN,
  embed=EMB,
  write=[SRC + "/bus/mmio.rs", SRC + "/timing/mod.rs"],
  spec="""1. `bus/mmio.rs`: `impl State for Mmio` : tous les champs de la struct (registres ecrits, resultats de multiplication/division, drapeaux RDNMI/TIMEUP/HVBJOY, valeurs d'auto-read, etc.) dans l'ordre de declaration.
2. `timing/mod.rs`: `impl State for Timing` : `line_start`, `refreshed_this_line`, `nmi_edge`, `nmi_line_prev`, `autoread_end` (`Option<u64>` sauve par un `bool` de presence puis un `u64`) et tout autre champ.
Test comportemental pour Timing: deux `Timing` (original et restaure) soumis a la meme suite de `on_dot` (via le banc `Rig` de E03-23) donnent les memes `TimingOut` sur 2000 dots.""",
  tests=[("mmio_roundtrip", "dans `bus/mmio.rs` : regle commune."),
         ("timing_roundtrip", "dans `timing/mod.rs` : regle commune, avec `autoread_end = Some(12345)` puis `None` (deux cas)."),
         ("timing_behaves_identically", "voir ci-dessus."),
         ("truncated_each", "troncature pour les deux types.")],
  accept=["cargo test -p puce16-core e08_04_"])

T("E08-05", "State pour Dma (canaux et etat HDMA)",
  size=90,
  context="8 canaux avec registres $43x0-$43xA et l'etat interne HDMA (`a2a`, `line_counter`, `do_transfer`, `done`). " + RULES + " " + CHAIN,
  embed=EMB,
  write=[SRC + "/dma/mod.rs", SRC + "/dma/hdma.rs"],
  spec="""`impl State for Dma` (dans `dma/mod.rs`): pour chacun des 8 canaux, dans l'ordre de declaration de `Channel`: `dmap bbad a1t a1b das dasb a2a nltr unused` puis `do_transfer`, `done`, `line_counter` (et tout champ ajoute depuis). Si l'etat HDMA a un champ propre dans `dma/hdma.rs`, le sauver aussi (sinon ce fichier reste inchange). Test comportemental: un `Dma` restaure apres un `hdma_init` sur un canal donne le meme `hdma_state(n)` que l'original.""",
  tests=[("roundtrip_all_fields", "regle commune : 8 canaux, chaque champ de chaque canal distinct (ex. `dmap = 0x10 + n`)."),
         ("hdma_state_preserved", "apres `hdma_init` (FakeBus de E03-21, canal 0 direct 3 lignes) : `hdma_state(0)` identique avant/apres save-load."),
         ("fresh_dma_roundtrip", "`Dma::new()` -> save -> load dans un autre `Dma::new()` modifie : revient aux valeurs 0xFF / done=true."),
         ("truncated", "regle commune.")],
  accept=["cargo test -p puce16-core e08_05_"])

T("E08-06", "State pour Vram, Cgram et Oam",
  size=110,
  context="Les trois memoires du PPU avec leurs registres d'acces (adresse, latch, prefetch, rotation). " + RULES + " " + CHAIN,
  embed=EMB,
  write=[SRC + "/ppu/vram.rs", SRC + "/ppu/cgram.rs", SRC + "/ppu/oam.rs"],
  spec="""1. `ppu/vram.rs`: `impl State for Vram`: `mem` (32768 mots, `slice_u16`), `vmain`, `vmadd`, `latch` (et tout champ ajoute depuis, ex. prefetch).
2. `ppu/cgram.rs`: `impl State for Cgram`: `mem` (256 mots), `addr`, `toggle`, `latch`.
3. `ppu/oam.rs`: `impl State for Oam`: `mem` (544 octets, `bytes`), `oamadd_reg`, `internal`, `latch`, `rotation`.""",
  tests=[("vram_roundtrip", "dans `ppu/vram.rs` : motif `mem[i] = i as u16 ^ 0x5A5A`, registres distincts."),
         ("cgram_roundtrip", "dans `ppu/cgram.rs` : motif + `toggle = true`, `latch = 0x77`."),
         ("oam_roundtrip", "dans `ppu/oam.rs` : motif + `rotation = true`."),
         ("sizes", "taille des octets sauves : Vram 65536 + (nombre d'octets des registres) ; Cgram 512 + ... ; Oam 544 + ... (verifier simplement qu'elles sont >= 65536, >= 512, >= 544)."),
         ("truncated_each", "troncature pour les trois types.")],
  accept=["cargo test -p puce16-core e08_06_"])

T("E08-07", "State pour Scroll, M7Regs et Regs (registres PPU)",
  size=110,
  context="Les registres de configuration du PPU qui vivent dans leurs propres structures. " + RULES + " " + CHAIN,
  embed=EMB,
  write=[SRC + "/ppu/regs.rs", SRC + "/ppu/mode7.rs"],
  spec="""1. `ppu/regs.rs`: `impl State for Scroll` (`hofs[4]`, `vofs[4]`, latchs) et `impl State for Regs` (TOUS les champs de la struct `Regs` : INIDISP, OBSEL, BGMODE, MOSAIC, BGnSC, BGnNBA, fenetres, TM/TS, TMW/TSW, CGWSEL/CGADSUB, COLDATA/fixed, SETINI, VMAIN... dans l'ordre de declaration).
2. `ppu/mode7.rs`: `impl State for M7Regs` (M7SEL: flip_x, flip_y, fill, outside..., M7A-D, M7X/Y, scroll, latch).""",
  tests=[("scroll_roundtrip", "dans `ppu/regs.rs` : valeurs distinctes."),
         ("regs_roundtrip", "dans `ppu/regs.rs` : chaque champ de `Regs` a une valeur non nulle distincte (mettre le champ a `(i as u8) + 1` selon l'ordre)."),
         ("m7_roundtrip", "dans `ppu/mode7.rs` : drapeaux a true, matrices distinctes (valeurs negatives incluses)."),
         ("truncated_each", "troncature pour les trois types.")],
  accept=["cargo test -p puce16-core e08_07_"])

T("E08-08", "State pour Ppu (position, tampons de ligne, framebuffer)",
  size=140,
  context="""Assemble l'etat du PPU: appelle les `State` de Vram/Cgram/Oam/Scroll/M7Regs/Regs puis sauve les champs propres de `Ppu` (position `v`/`dot`/`field`, drapeaux d'entrelacement et de surbalayage, latchs H/V, tampons de ligne des fonds et des sprites, `line_obj`, tampons de fenetre, framebuffer 512x478 complet, `row_hires`...). """ + RULES + " " + CHAIN,
  embed=EMB,
  reads=["ppu/mod.rs (struct Ppu et hash_into complet, E04-12)"],
  write=[SRC + "/ppu/mod.rs"],
  spec="""`impl State for Ppu` dans `ppu/mod.rs`. Ordre: `region` (u8: Ntsc 0, Pal 1; au load, une region differente de celle de l'instance donne `Mismatch("region")`), puis les sous-structures (vram, cgram, oam, scroll, m7, regs) par leur `save`, puis les champs restants de `Ppu` dans l'ordre de declaration. Le framebuffer `fb` (512*478 `u32`) s'ecrit avec `slice_u32`. Les tampons de ligne (rendus precedemment) sont sauves aussi: un save state au milieu d'une ligne doit redonner la meme image.
Test comportemental: deux `Ppu` (original et restaure en milieu de frame, par exemple `v = 100`, `dot = 77`, apres 120 000 `step_dot` d'une scene de VRAM/CGRAM ecrite par les registres) executent 50 000 `step_dot` de plus : `hash_into` egaux et `frame()` identiques pixel a pixel.""",
  tests=[("roundtrip_all_fields", "regle commune (position et tampons de ligne non nuls obtenus en faisant tourner le PPU avec une scene)."),
         ("region_mismatch", "sauver un `Ppu::new(Ntsc)` et charger dans `Ppu::new(Pal)` -> `Err(Mismatch(\"region\"))`."),
         ("behaves_identically_midframe", "voir ci-dessus."),
         ("framebuffer_roundtrip", "pixel (10,20) du framebuffer a 0xFF123456 avant save : identique apres load."),
         ("truncated", "regle commune (n = 0, len/2, len-1).")],
  accept=["cargo test -p puce16-core e08_08_"])

T("E08-09", "State pour Spc700 et Timer",
  size=80,
  context="Coeur SMP et les trois timers du SPC700. " + RULES + " " + CHAIN,
  embed=EMB,
  write=[SRC + "/apu/spc700/mod.rs", SRC + "/apu/timer.rs"],
  spec="""1. `apu/spc700/mod.rs`: `impl State for Spc700`: champs publics `a x y sp pc psw halted` puis les champs prives `cyc`, `taken`.
2. `apu/timer.rs`: `impl State for Timer`: `divider`, `enable`, `target`, `stage0`, `stage1`, `out` (le `divider` est saved et verifie au load: valeur differente -> `Mismatch(\"timer divider\")`).""",
  tests=[("spc_roundtrip", "dans `spc700/mod.rs` : regle commune."),
         ("spc_behaves_identically", "meme programme court (`E8 10 BC 2F FE` : MOV A,#$10 ; INC A ; BRA -2) sur `FlatSmpBus` pour l'original et le restaure : 20 `step` -> registres identiques."),
         ("timer_roundtrip", "dans `timer.rs` : regle commune."),
         ("timer_divider_mismatch", "sauver un `Timer::new(16)`, charger dans un `Timer::new(128)` -> `Err(Mismatch(\"timer divider\"))`."),
         ("timer_behaves_identically", "apres restauration en plein comptage, 100 `tick` donnent le meme `out()` et `stage1()`.")],
  accept=["cargo test -p puce16-core e08_09_"])

T("E08-10", "State pour Dsp et Voice",
  size=110,
  context="Registres, 8 voix, compteurs, echo et sortie audio non consommee du DSP. " + RULES + " " + CHAIN,
  embed=EMB,
  write=[SRC + "/apu/dsp/mod.rs", SRC + "/apu/dsp/voice.rs"],
  spec="""1. `dsp/voice.rs`: `impl State for Voice`: `kon_delay`, `mode` (indice u8 Attack 0 Decay 1 Sustain 2 Release 3; autre -> `Mismatch(\"voice mode\")`), `env`, `hidden_env`, `addr`, `block_header`, `group_in_block`, `buf` (12 `i32`), `buf_pos` (u8), `interp_pos`, `out`, `p1`, `p2`.
2. `dsp/mod.rs`: `impl State for Dsp`: `regs` (128 octets), 8 voix, `counter`, `noise`, `every_other`, `new_kon`, `kon`, `endx`, `echo_offset`, `echo_length`, `hist_l[8]`, `hist_r[8]` (i32), et `audio` (`vec_i16` : ecrire avec `vec_u16` apres conversion `as u16`, relire avec `as i16`).
Test comportemental: un `Dsp` configure avec une note BRR (montage de E06-49: DIR, BRR constant, KON) et avance de 100 echantillons, sauve/restaure, puis avance de 200 echantillons : audio et hash egaux a ceux de l'original avance de 200 aussi.""",
  tests=[("voice_roundtrip", "dans `voice.rs` : regle commune (mode Sustain, buf distincts)."),
         ("voice_bad_mode", "un octet de mode = 9 dans les donnees -> `Err(Mismatch(\"voice mode\"))`."),
         ("dsp_roundtrip", "dans `dsp/mod.rs` : regle commune, y compris `audio` non vide (3 paires)."),
         ("dsp_behaves_identically", "voir ci-dessus."),
         ("truncated_each", "troncature pour Voice et Dsp.")],
  accept=["cargo test -p puce16-core e08_10_"])

T("E08-11", "State pour ApuIo et Apu",
  size=100,
  context="RAM 64 Kio du SMP, registres $F0-$FF, ports, timers, DSP, SMP et l'accumulateur de rattrapage. " + RULES + " " + CHAIN,
  embed=EMB,
  write=[SRC + "/apu/io.rs", SRC + "/apu/mod.rs"],
  spec="""1. `apu/io.rs`: `impl State for ApuIo`: `ram` (65536 octets), `control`, `dspaddr`, `dsp_div`, `port_in[4]`, `port_out[4]`, 3 timers (`Timer::save`), `dsp` (`Dsp::save`).
2. `apu/mod.rs`: `impl State for Apu`: `spc`, `io`, `acc` (i64), `last_master` (u64). `master_hz` n'est pas sauve (derive de la region: `// not saved`).
Test comportemental: un `Apu` fait tourner jusqu'a `step_to(100_000)` (handshake IPL), sauve/restaure, puis les deux avancent a `step_to(300_000)` : hash egal et `port_read(0..4)` egaux.""",
  tests=[("io_roundtrip", "dans `io.rs` : regle commune (RAM motif, ports distincts)."),
         ("apu_roundtrip", "dans `apu/mod.rs` : regle commune."),
         ("apu_behaves_identically", "voir ci-dessus."),
         ("ipl_handshake_survives", "`Apu` apres handshake : sauve/restaure -> `port_read(0)==0xAA`, `port_read(1)==0xBB`."),
         ("truncated_each", "troncature pour ApuIo et Apu.")],
  accept=["cargo test -p puce16-core e08_11_"])

T("E08-12", "Snes::save_state et load_state (assemblage, en-tete du blob)",
  size=130,
  context="Assemble le blob complet dans l'ordre de la spec 09 section 9 et implemente l'API publique `save_state` / `load_state` de la spec 01 section 5. " + CHAIN,
  embed=EMB + ["01_ARCHITECTURE.md#5. API publique de `puce16-core` (stable, ne pas changer sans question)"],
  write=[SRC + "/bus/mod.rs", SRC + "/snes.rs", TST + "/e08_12_snes_state.rs"],
  spec="""1. `bus/mod.rs`: `impl State for SysBus` : `master` (u64), `ppu_t` (u64), `mdr`, `memsel`, `pending_dma`, `pending_hdma_init`, `pending_hdma_run`, ET tout compteur de VBlank/frames utilise par `Snes::frame_count()` et `run_frame` (ex. `vblank_count`, drapeau de VBlank pris), puis dans l'ordre: `wram`, `mmio`, `dma`, `timing`, `ppu`, `apu`, `joy`, `cart` (SRAM seulement). `region` n'est pas sauvee ici (`// not saved`).
2. `snes.rs` : `Snes::save_state(&self) -> Vec<u8>` et `Snes::load_state(&mut self, data: &[u8]) -> Result<(), StateError>` (variant reel ; variant motif de test: `save_state` renvoie un blob d'au moins l'en-tete et `load_state` l'accepte, sans plus). Format: `u32 STATE_VERSION`, `u8 region` (Ntsc 0, Pal 1), `u64 frame`, `u64 cpu_master` (= `sys.master`), puis `Cpu::save`, puis `SysBus::save`. `load_state`: version differente -> `Err(Version(v))` ; region differente de la region de la console -> `Err(Mismatch(\"region\"))` ; octets restants -> `Mismatch(\"trailing data\")` ; le Snes n'est modifie qu'apres lecture reussie de l'en-tete (si le corps est corrompu, l'etat peut etre incoherent: documente-le par un commentaire; le desktop recharge alors la ROM).
3. `Snes::state_hash` reste inchange.""",
  tests=[("roundtrip_hash", "ROM `common::make_rom(&[0x78,0x18,0xFB,0x80,0xFE], Map::LoRom)` ; `Snes::new` ; 5 `run_frame` ; `blob = save_state()` ; `Snes` neuf ; `load_state(&blob)` Ok ; `state_hash()` egal."),
         ("blob_size", "`blob.len()` entre 1_000_000 et 1_600_000 (VRAM+WRAM+RAM APU+framebuffer)."),
         ("resave_identical", "`load_state(&blob)` puis `save_state()` == `blob` (octet a octet)."),
         ("continue_equals_uninterrupted", "A: 5 frames, save, 5 frames, hash. B: neuf, `load_state` de la sauvegarde de A, 5 frames, hash : egaux."),
         ("bad_version", "blob dont les 4 premiers octets sont remplaces par `[2,0,0,0]` -> `Err(Version(2))`."),
         ("region_mismatch", "blob d'une console NTSC charge dans une console PAL -> `Err(Mismatch(\"region\"))`."),
         ("truncated", "blob coupe a la moitie -> `Err(Truncated)` ; blob + 1 octet -> `Err(Mismatch(\"trailing data\"))`."),
         ("frame_count_restored", "apres `load_state` dans une console neuve, `frame_count()` egale celui de la console sauvee, et un `run_frame()` de plus donne `frame_count()` + 1 (pas de repart a zero)."),
         ("sram_roundtrip", "ROM avec SRAM (chipset 0x02, ram_log2 3, via `make_rom_with`) : `load_sram` d'un motif avant `save_state`; apres `load_state` dans une console neuve, `sram()` == motif.")],
  notes=CHAIN)

T("E08-13", "Tests d'integration des save states sur des scenarios (NMI, DMA, HDMA, PPU, APU)",
  size=140,
  context="Prouve que sauver puis recharger au milieu d'une execution ne change RIEN au futur: pour chaque scenario, `A` (sans sauvegarde) et `B` (sauve a la frame K, recharge dans une console neuve, continue) donnent le meme `state_hash` et les memes pixels/audio a chaque frame. " + CHAIN,
  write=[TST + "/e08_13_state_scenarios.rs"],
  spec="""Fichier de test : """ + HDR + """ Fonction `fn compare(rom: &[u8], k: u64, n: u64)`: A = `Snes::new` + `k + n` frames (enregistrer le `state_hash`, les pixels de `frame()` et l'audio de `take_audio` a chaque frame) ; B = `Snes::new`, `k` frames, `blob = save_state()` ; C = `Snes::new` puis `C.load_state(&blob)` puis `n` frames. A chaque frame apres K: `state_hash`, pixels de `frame()` et audio de C egaux a ceux de A.
Scenarios (les programmes sont COPIES des tests d'integration existants du meme depot, fichiers `tests/e03_37_nmi.rs`, `e03_39_*.rs`, `e03_40_*.rs`, `e04_15_snes_video.rs`, `e05_16_*.rs`, `e06_37_snes_handshake.rs`; recopier les octets, ne pas les reecrire de memoire):
1. NMI: `PROG_A` + `HANDLER_INC0100` a l'offset 0x100 + vecteur `nmi` 0x8100 (E03-37).
2. DMA general ROM -> WRAM par `$2180`: le programme du premier test de E03-39.
3. HDMA vers `$2180`, table directe: le programme du premier test de E03-40.
4. PPU: la scene du premier test de E05-16 (mode 1, BG1, couleurs) ou, a defaut, le programme rouge de E04-15.
5. APU: `PROG_APU` de E06-37.
6. DMA long: le programme de E03-39 dans lequel la taille du transfert (`DAS`, deux octets) est mise a `0x0000` (= 65536 octets).
7. PAL: scenario 4 avec `Snes::new(&rom, Some(Region::Pal))`.""",
  tests=[("nmi_counter", "scenario 1, K=3, N=5."),
         ("dma_general", "scenario 2, K=2, N=3."),
         ("hdma_table", "scenario 3, K=2, N=3."),
         ("ppu_scene_mode1", "scenario 4, K=2, N=3, comparer aussi la couleur du pixel (0,0) a la fin."),
         ("apu_handshake_red", "scenario 5, K=1 puis K=3 (avant et apres l'apparition du rouge)."),
         ("long_dma", "scenario 6, K=1, N=3 puis K=2, N=3 (tester au moins un K qui tombe pendant le DMA si le temps du DMA depasse une frame)."),
         ("pal_console", "scenario 4 en PAL, K=2, N=3.")],
  notes=CHAIN)

T("E08-14", "CLI: puce16-cli run --save-state / --load-state",
  size=90, crate="puce16-cli",
  context="Options prevues par la spec 01 section 6 pour tester les save states depuis la ligne de commande. " + CHAIN,
  embed=["01_ARCHITECTURE.md#6. CLI"],
  write=["crates/puce16-cli/src/main.rs", "crates/puce16-cli/src/run_cmd.rs", "crates/puce16-cli/tests/e08_14_state_cli.rs"],
  spec="""Ajoute a `run <ROM>` : `--save-state <FICHIER>` (apres l'execution des `--frames` frames, ecrit `Snes::save_state()` dans le fichier ; erreur d'ecriture -> stderr `cannot write <chemin>: <err>`, code 2) et `--load-state <FICHIER>` (lu AVANT d'executer les frames, juste apres `Snes::new_with` : fichier illisible -> stderr `cannot read <chemin>: <err>`, code 2 ; `load_state` en erreur -> stderr `cannot load state: <StateError>`, code 2). Les deux options sont refusees (code 2, stderr `--save-state needs a ROM`) avec `--test-pattern`. Les sorties (`--hash`, `--state-hash`, `--debug`) viennent apres.""",
  tests=[("save_then_load_matches", "ROM `tiny_rom()` (voir E03-44) dans un dossier temporaire : `run ROM --frames 3 --save-state S` (code 0, fichier S de plus de 1 Mo) ; `run ROM --frames 2 --load-state S --state-hash` donne le meme `state_hash` que `run ROM --frames 5 --state-hash` (3+2 frames)."),
         ("missing_state_file", "`--load-state /nonexistent/s` -> code 2, stderr contient `cannot read`."),
         ("bad_state_file", "fichier de 10 octets aleatoires en `--load-state` -> code 2, stderr contient `cannot load state`."),
         ("unwritable_save", "`--save-state /nonexistent/dir/s` -> code 2, stderr contient `cannot write`."),
         ("pattern_refused", "`run --test-pattern --save-state S` -> code 2, stderr contient `needs a ROM`.")],
  accept=["cargo test -p puce16-cli e08_14_", "cargo test -p puce16-cli e03_44_"])
