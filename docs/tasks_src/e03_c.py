from _dsl import T

CORE = "crates/puce16-core"
SRC = CORE + "/src"
TST = CORE + "/tests"

CHAIN = ("Execution strictement sequentielle (D-020): cette tache suppose que toutes les precedentes sont terminees. "
         "Ne modifie QUE les fichiers de 'Write only in'.")

# ------------------------------------------------------------------ E03 part C: MMIO 5A22, DMA/HDMA engines, Timing (E03-14..25)

T("E03-14", "bus/mmio.rs: struct Mmio, power-on, ecritures $4200-$420D",
  size=130, deps=["E03-13"],
  context="""Registres du 5A22 cote ecriture. Les registres du multiplicateur/diviseur sont seulement MEMORISES ici (le calcul est E03-15).
Aucune dependance vers le bus: `Mmio` ne connait ni le DMA ni le PPU; il renvoie un `MmioEffect` que `SysBus` applique. """ + CHAIN,
  embed=["02_BUS_TIMING.md#6. Registres 5A22 ($4200-$421F)", "09_SYSTEM.md#3.5 Mmio (`bus/mmio.rs`)"],
  write=[SRC + "/bus/mmio.rs", TST + "/e03_14_mmio_write.rs"],
  spec="""Dans `bus/mmio.rs`:
```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MmioEffect { None, Mdmaen(u8), Hdmaen(u8), Memsel(bool), Wrio { old: u8, new: u8 } }
#[derive(Clone, Copy, Debug, Default)] pub struct HvState { pub vblank: bool, pub hblank: bool, pub autoread_busy: bool }
pub struct Mmio { pub nmitimen: u8, pub wrio: u8, pub htime: u16, pub vtime: u16, pub hdmaen: u8, pub memsel: bool,
                  pub rdnmi_flag: bool, pub timeup_flag: bool, pub irq_line: bool, pub joy: [u16; 4],
                  wrmpya: u8, wrdiv: u16, rddiv: u16, rdmpy: u16, math_pending: Option<MathPending> }
struct MathPending { done_at: u64, rddiv: u16, rdmpy: u16 }       // utilise par E03-15
impl Mmio {
    pub fn new() -> Mmio;     // nmitimen 0, wrio 0xFF, htime 0x1FF, vtime 0x1FF, hdmaen 0, memsel false, flags false, joy [0;4], wrmpya 0xFF, wrdiv 0xFFFF, rddiv 0, rdmpy 0, math_pending None
    pub fn reset(&mut self);  // nmitimen=0, hdmaen=0, memsel=false, rdnmi_flag=timeup_flag=irq_line=false, math_pending=None ; le reste est conserve
    pub fn write(&mut self, reg: u16, v: u8, master: u64) -> MmioEffect;   // reg 0x4200..=0x420D ; autre reg: None sans effet
    pub fn hash_into(&self, h: &mut crate::hash::Fnv64);     // tous les champs dans l'ordre de la struct (u8/u16 en LE, bool en u8 0/1, Option: 0 ou 1 + done_at u64 + rddiv u16 + rdmpy u16)
}
```
Ecritures (`master` ignore dans cette tache):
- `0x4200`: `nmitimen = v` ; si `(v >> 4) & 3 == 0` : `timeup_flag = false; irq_line = false`. Renvoie `None`.
- `0x4201`: `old = wrio; wrio = v;` renvoie `Wrio{old, new: v}`.
- `0x4202`: `wrmpya = v`. `0x4204`: `wrdiv = (wrdiv & 0xFF00) | v`. `0x4205`: `wrdiv = (wrdiv & 0x00FF) | (v << 8)`. `0x4203` et `0x4206`: ignores pour l'instant.
- `0x4207`: `htime = (htime & 0x100) | v` ; `0x4208`: `htime = (htime & 0x0FF) | ((v & 1) << 8)`. `0x4209`/`0x420A`: pareil pour `vtime`.
- `0x420B`: renvoie `Mdmaen(v)` (rien stocke). `0x420C`: `hdmaen = v`, renvoie `Hdmaen(v)`. `0x420D`: `memsel = v & 1 != 0`, renvoie `Memsel(memsel)`.""",
  tests=[("power_on", "`Mmio::new()`: nmitimen==0, wrio==0xFF, htime==0x1FF, vtime==0x1FF, hdmaen==0, memsel==false, les 3 drapeaux faux, joy==[0;4]."),
         ("nmitimen", "`write(0x4200,0x81,0)==MmioEffect::None` et nmitimen==0x81."),
         ("wrio_effect", "`write(0x4201,0x7F,0)==Wrio{old:0xFF,new:0x7F}` ; wrio==0x7F ; un 2eme `write(0x4201,0xFF,0)==Wrio{old:0x7F,new:0xFF}`."),
         ("htime", "`write(0x4207,0x34,0)` -> htime==0x134 (le bit 8 vient de la valeur 0x1FF initiale) ; `write(0x4208,0x00,0)` -> 0x034 ; `write(0x4208,0x03,0)` -> 0x134 (seul le bit 0 compte) ; `write(0x4207,0xFF,0)` -> 0x1FF."),
         ("vtime", "`write(0x4209,0xE0,0)` puis `write(0x420A,0x01,0)` -> vtime==0x1E0 ; `write(0x420A,0x00,0)` -> 0x0E0."),
         ("dma_effects", "`write(0x420B,0x05,0)==Mdmaen(5)` ; `write(0x420C,0x81,0)==Hdmaen(0x81)` et hdmaen==0x81 ; `write(0x420D,0x01,0)==Memsel(true)` (memsel true) ; `write(0x420D,0xFE,0)==Memsel(false)`."),
         ("clear_irq_on_off", "mets `timeup_flag=true; irq_line=true` ; `write(0x4200,0x80,0)` (V/H = 00) -> les deux faux. Remets-les a true ; `write(0x4200,0x10,0)` (mode 1) -> ils restent vrais ; `write(0x4200,0x31,0)` aussi."),
         ("other_regs_ignored", "`write(0x4210,0xFF,0)==None` et `write(0x4300,1,0)==None` ne changent pas le hash."),
         ("reset", "apres avoir mis nmitimen=0x81, hdmaen=0xFF, memsel=true, rdnmi_flag=true, timeup_flag=true, irq_line=true, htime=0x55, wrio=0x12: `reset()` -> nmitimen==0, hdmaen==0, memsel==false, flags faux ; htime==0x55 et wrio==0x12 conserves."),
         ("hash", "deux `Mmio::new()` ont le meme hash ; apres `write(0x4202,3,0)` le hash differe ; apres `write(0x4204,1,0)` il differe encore.")])

T("E03-15", "bus/mmio.rs: unite math (multiplication 8x8, division 16/8) et lecture $4214-$4217",
  size=130,
  context="""Le resultat n'est visible qu'apres un delai (defaut v1: multiplication 48 clocks, division 96 clocks; `UNKNOWN - to confirm`). Avant, les registres gardent l'ancienne valeur.
Un nouveau calcul commence par figer le precedent s'il est termine, sinon l'ecrase. """ + CHAIN,
  embed=["02_BUS_TIMING.md#6.1 Multiplication / division (unite math du 5A22)"],
  write=[SRC + "/bus/mmio.rs", TST + "/e03_15_mmio_math.rs"],
  spec="""Dans `bus/mmio.rs`:
```rust
pub const MUL_CLOCKS: u64 = 48;     // UNKNOWN - to confirm
pub const DIV_CLOCKS: u64 = 96;     // UNKNOWN - to confirm
impl Mmio {
    fn settle(&mut self, master: u64);          // si math_pending = Some(p) et master >= p.done_at : rddiv = p.rddiv ; rdmpy = p.rdmpy ; math_pending = None
    pub fn read(&mut self, reg: u16, mdr: u8, master: u64, st: HvState) -> u8;   // d'abord settle(master)
}
```
- `write(0x4203, v, master)`: `settle(master)` ; puis `math_pending = Some(MathPending { done_at: master + MUL_CLOCKS, rddiv: <rddiv courant>, rdmpy: (wrmpya as u16) * (v as u16) })` ; renvoie `None`.
- `write(0x4206, v, master)`: `settle(master)` ; si `v != 0`: `q = wrdiv / v as u16`, `r = wrdiv % v as u16` ; si `v == 0`: `q = 0xFFFF`, `r = wrdiv` ; `math_pending = Some(MathPending { done_at: master + DIV_CLOCKS, rddiv: q, rdmpy: r })`.
  (Un nouveau `Some` ECRASE un calcul encore en cours: le resultat de l'ancien est perdu.)
- `read` pour `0x4214` = `rddiv` bas, `0x4215` = `rddiv` haut, `0x4216` = `rdmpy` bas, `0x4217` = `rdmpy` haut. Tout autre `reg`: renvoie `mdr` (les autres registres de lecture viennent en E03-16).""",
  tests=[("mul_timing", "master=100: `write(0x4202,7,100)`, `write(0x4203,9,100)` ; `read(0x4216,0,147,st)==0` (ancienne valeur) ; `read(0x4216,0,148,st)==0x3F` ; `read(0x4217,0,148,st)==0x00`."),
         ("mul_max", "`write(0x4202,0xFF,0)`, `write(0x4203,0xFF,0)` ; a master=48: `read(0x4216)==0x01` et `read(0x4217)==0xFE` (65025 = 0xFE01)."),
         ("mul_power_on_a", "sans ecrire 0x4202 (WRMPYA = 0xFF au power-on): `write(0x4203,2,0)` ; a master=48: `read(0x4216)==0xFE`, `read(0x4217)==0x01` (0x1FE)."),
         ("div_basic", "`write(0x4204,0x34,0)`, `write(0x4205,0x12,0)` (dividende 0x1234=4660), `write(0x4206,7,0)` ; `read(0x4214,0,95,st)==0` ; a master=96: `read(0x4214)==0x99`, `read(0x4215)==0x02` (quotient 665), `read(0x4216)==0x05`, `read(0x4217)==0x00` (reste 5)."),
         ("div_by_zero", "dividende 0x1234, `write(0x4206,0,0)` ; a master=96: quotient 0xFFFF (`read(0x4214)==0xFF`, `read(0x4215)==0xFF`) et reste 0x1234 (`read(0x4216)==0x34`, `read(0x4217)==0x12`)."),
         ("div_ffff_by_ff", "dividende 0xFFFF (power-on) ; `write(0x4206,0xFF,0)` ; a master=96: quotient 0x0101 (`0x4214`=0x01, `0x4215`=0x01), reste 0."),
         ("settle_before_new_op", "`write(0x4202,3,0)`, `write(0x4203,4,0)` (resultat 12 a t=48) ; dividende 100 (`write(0x4204,100,0)`, `write(0x4205,0,0)`) ; `write(0x4206,7,60)` : le calcul precedent est fige d'abord (rdmpy=12) ; `read(0x4216,0,100,st)==12` ; a 156: `read(0x4214)==14`, `read(0x4216)==2`."),
         ("overwrite_in_flight", "`write(0x4203,..)` a t=0 avec 3x4 ; avant t=48 (a t=10) `write(0x4206,7,10)` avec dividende 100 : le produit est PERDU ; `read(0x4216,0,60,st)==0` ; a 106: `read(0x4214)==14`, `read(0x4216)==2`."),
         ("other_reg_open_bus", "`read(0x4210,0xAB,0,st)==0xAB` (pas encore implemente) ; `read(0x4220,0xCD,0,st)==0xCD`.")])

T("E03-16", "bus/mmio.rs: lecture RDNMI, TIMEUP, HVBJOY, RDIO, JOYn",
  size=110,
  context="Registres de lecture du 5A22 avec open bus partiel (bits non definis = bits de `mdr`). " + CHAIN,
  embed=["02_BUS_TIMING.md#4. Open bus", "02_BUS_TIMING.md#6.2 NMI", "02_BUS_TIMING.md#6.4 Auto-read joypad"],
  write=[SRC + "/bus/mmio.rs", TST + "/e03_16_mmio_read.rs"],
  spec="""Complete `Mmio::read` (apres `settle`) pour:
- `0x4210` RDNMI: `v = (mdr & 0x70) | 0x02 | ((rdnmi_flag as u8) << 7)` ; puis `rdnmi_flag = false`.
- `0x4211` TIMEUP: `v = (mdr & 0x7F) | ((timeup_flag as u8) << 7)` ; puis `timeup_flag = false; irq_line = false`.
- `0x4212` HVBJOY: `v = (mdr & 0x3E) | ((st.vblank as u8) << 7) | ((st.hblank as u8) << 6) | (st.autoread_busy as u8)`.
- `0x4213` RDIO: `wrio`.
- `0x4218..=0x421F`: `i = (reg - 0x4218) / 2` ; octet bas de `joy[i]` si `reg` pair, haut si impair.
- `0x421?` hors liste et tout autre `reg` (`< 0x4210` ou `> 0x421F`): `mdr`.""",
  tests=[("rdnmi", "flag faux, mdr=0xFF: `read(0x4210,0xFF,0,st)==0x72` ; flag vrai, mdr=0xFF: 0xF2 ; flag vrai, mdr=0x00: 0x82 ; apres lecture, `rdnmi_flag==false` et la lecture suivante (mdr=0xFF) donne 0x72."),
         ("timeup", "flag vrai+irq_line vraie, mdr=0xFF: `read(0x4211,0xFF,0,st)==0xFF` ; apres: timeup_flag et irq_line faux ; flag faux, mdr=0xAB: `read(0x4211,0xAB,0,st)==0x2B` ; flag faux, mdr 0: 0x00."),
         ("hvbjoy", "st={vblank:true,hblank:false,autoread_busy:true}, mdr=0xFF: `read(0x4212)==0xBF` (0x3E|0x80|0x01) ; st tous faux, mdr=0: 0x00 ; st={vblank:true,hblank:true,busy:false}, mdr=0: 0xC0 ; st tous faux, mdr=0xFF: 0x3E."),
         ("rdio", "`write(0x4201,0x7F,0)` puis `read(0x4213,0xFF,0,st)==0x7F` ; au power-on 0xFF."),
         ("joy_regs", "`joy=[0x1234,0x5678,0x9ABC,0xDEF0]` (champ pub): 0x4218->0x34, 0x4219->0x12, 0x421A->0x78, 0x421B->0x56, 0x421C->0xBC, 0x421D->0x9A, 0x421E->0xF0, 0x421F->0xDE."),
         ("out_of_range", "`read(0x4220,0xEE,0,st)==0xEE` ; `read(0x420F,0x11,0,st)==0x11` ; `read(0x4200,0x22,0,st)==0x22`."),
         ("math_still_works", "multiplication 5x6 a t=0 puis `read(0x4216,0,48,st)==30` (non-regression de E03-15).")])

T("E03-17", "dma/mod.rs: Channel, Dma::new et registres $4300-$437F",
  size=120,
  context="Registres des 8 canaux (spec 03 section 1). Aucun transfert encore. Les champs d'etat HDMA sont `pub(crate)` pour que `dma/hdma.rs` puisse y acceder. " + CHAIN,
  embed=["03_DMA.md#1. Registres (8 canaux, n = 0..7, adresse $43n0-$43nF)", "09_SYSTEM.md#3.7 Dma (`dma/mod.rs`, `dma/hdma.rs`)"],
  write=[SRC + "/dma/mod.rs", TST + "/e03_17_dma_regs.rs"],
  spec="""Dans `dma/mod.rs` (garde `pub mod hdma;`):
```rust
#[derive(Default, Clone)]
pub struct Channel { pub dmap: u8, pub bbad: u8, pub a1t: u16, pub a1b: u8, pub das: u16, pub dasb: u8, pub a2a: u16, pub nltr: u8, pub unused: u8,
                     pub(crate) do_transfer: bool, pub(crate) done: bool, pub(crate) line_counter: u8 }
#[derive(Default, Clone)] pub struct Dma { pub ch: [Channel; 8] }
impl Dma {
    pub fn new() -> Dma;       // tous les registres a 0xFF (u16: 0xFFFF), done=true, do_transfer=false, line_counter=0
    pub fn read(&self, reg: u16) -> Option<u8>;        // reg 0x4300..=0x437F ; None = open bus
    pub fn write(&mut self, reg: u16, v: u8);
    pub fn hash_into(&self, h: &mut crate::hash::Fnv64);   // pour chaque canal: dmap bbad (u8), a1t (u16), a1b, das (u16), dasb, a2a (u16), nltr, unused, do_transfer done (u8 0/1), line_counter
}
```
Decodage: `n = ((reg >> 4) & 7) as usize`, `off = reg & 0xF`. Offsets: 0 dmap, 1 bbad, 2 a1t bas, 3 a1t haut, 4 a1b, 5 das bas, 6 das haut, 7 dasb, 8 a2a bas, 9 a2a haut, 0xA nltr, 0xB et 0xF `unused` (le MEME octet), 0xC-0xE: lecture `None`, ecriture ignoree. `reg` hors `0x4300..=0x437F`: lecture `None`, ecriture ignoree.""",
  tests=[("power_on", "`Dma::new()`: pour n in 0..8 et off in {0..=0xB, 0xF}: `read(0x4300|n<<4|off)==Some(0xFF)` ; `ch[3].a1t==0xFFFF`, `ch[3].das==0xFFFF`, `ch[3].a2a==0xFFFF`."),
         ("unmapped", "`read(0x430C)==None`, `read(0x430D)==None`, `read(0x437E)==None` ; `read(0x4380)==None` ; `read(0x42FF)==None`."),
         ("a1t_a1b", "`write(0x4312,0x34); write(0x4313,0x12); write(0x4314,0x7E)` -> `ch[1].a1t==0x1234`, `ch[1].a1b==0x7E`, `read(0x4312)==Some(0x34)`, `read(0x4313)==Some(0x12)`, `read(0x4314)==Some(0x7E)`."),
         ("das", "`write(0x4325,0x00); write(0x4326,0x01); write(0x4327,0x7F)` -> `ch[2].das==0x0100`, `ch[2].dasb==0x7F` ; relire donne les memes octets."),
         ("dmap_bbad", "`write(0x4300,0x01); write(0x4301,0x18)` -> `ch[0].dmap==1`, `ch[0].bbad==0x18`."),
         ("a2a_nltr", "`write(0x4348,0xCD); write(0x4349,0xAB); write(0x434A,0x85)` -> `ch[4].a2a==0xABCD`, `ch[4].nltr==0x85`."),
         ("unused_shared", "`write(0x435B,0x77)` -> `read(0x435F)==Some(0x77)` ; `write(0x435F,0x11)` -> `read(0x435B)==Some(0x11)` ; `write(0x436C,1)` ignore et `read(0x436C)==None`."),
         ("hash", "hash identique pour deux `Dma::new()` ; differe apres `write(0x4300,0)` ; differe aussi apres modification de `ch[7].das`.")])

T("E03-18", "dma/mod.rs: trait DmaBus et fonctions de pattern",
  size=90,
  context="Interface entre le moteur DMA et le bus (spec 09 section 3.7) et petites fonctions pures sur les patterns de transfert (spec 03 section 2). " + CHAIN,
  embed=["03_DMA.md#2. Patterns de transfert (octets envoyes a BBADn + offset)", "09_SYSTEM.md#3.7 Dma (`dma/mod.rs`, `dma/hdma.rs`)"],
  write=[SRC + "/dma/mod.rs", TST + "/e03_18_dma_patterns.rs"],
  spec="""Ajoute a `dma/mod.rs`:
```rust
pub trait DmaBus {
    fn a_read(&mut self, addr: u32) -> u8;      // lecture A-bus; le moteur DMA gere le temps (pas d'avance de master ici)
    fn a_write(&mut self, addr: u32, v: u8);
    fn b_read(&mut self, reg: u8) -> u8;        // registre $2100 + reg
    fn b_write(&mut self, reg: u8, v: u8);
    fn elapse(&mut self, clocks: u32);          // fait avancer master et appelle catch_up
    fn master(&self) -> u64;
}
pub fn pattern_offsets(p: u8) -> &'static [u8];    // p = dmap & 7 ; P0 [0] ; P1 [0,1] ; P2 [0,0] ; P3 [0,0,1,1] ; P4 [0,1,2,3] ; P5 [0,1,0,1] ; P6 = P2 ; P7 = P3
pub fn a_step(dmap: u8) -> i32;                    // bits 4-3: 0 -> +1 ; 2 -> -1 ; 1 ou 3 -> 0
pub fn is_wram_addr(addr: u32) -> bool;            // banques 7E-7F ; ou banque (b & 0x7F) <= 0x3F avec adresse < 0x2000
pub fn step_a(a1t: u16, step: i32) -> u16;         // wrapping dans 16 bits
```""",
  tests=[("offsets", "`pattern_offsets(0)==[0]` ; `(1)==[0,1]` ; `(2)==[0,0]` ; `(3)==[0,0,1,1]` ; `(4)==[0,1,2,3]` ; `(5)==[0,1,0,1]` ; `(6)==[0,0]` ; `(7)==[0,0,1,1]` ; `(0x0B)` (masque 7) == `(3)`."),
         ("a_step", "`a_step(0x00)==1` ; `a_step(0x08)==0` ; `a_step(0x10)==-1` ; `a_step(0x18)==0` ; `a_step(0x81)==1` (bits hors 4-3 ignores) ; `a_step(0x41)==1`."),
         ("wram_addr", "`is_wram_addr(0x7E0000)`, `0x7FFFFF`, `0x000000`, `0x001FFF`, `0x3F1234`, `0x800100`, `0xBF1FFF` vrais ; `0x002000`, `0x00FFFF`, `0x7D0000`, `0xC00000`, `0x400000`, `0x018000` faux."),
         ("step_a", "`step_a(0x1000,1)==0x1001` ; `step_a(0xFFFF,1)==0` ; `step_a(0,-1)==0xFFFF` ; `step_a(0x1234,0)==0x1234`."),
         ("trait_object_safe_enough", "une struct de test `Dummy` qui implemente `DmaBus` (toutes methodes triviales) compile et `Dummy.master()==0`.")])

T("E03-19", "dma/mod.rs: run_general (A -> B), patterns, ajustement A, temps",
  size=140,
  context="""Moteur du DMA general: copie octet par octet, avec le temps decrit en spec 09 section 4. Le sens B->A et le cas WRAM/$2180 sont E03-20.
Cette tache cree aussi `tests/common/fake_dma_bus.rs` (bus factice reutilise par E03-20..22). """ + CHAIN,
  embed=["03_DMA.md#3. DMA general (MDMAEN $420B)", "09_SYSTEM.md#4. DMA general: temps (decisions provisoires, spec 03 section 3.1)"],
  write=[SRC + "/dma/mod.rs", TST + "/common/fake_dma_bus.rs", TST + "/e03_19_dma_general.rs"],
  spec="""1. `tests/common/fake_dma_bus.rs` (declare dans les tests par `#[path = "common/fake_dma_bus.rs"] mod fake_dma_bus;`, NE modifie PAS `common/mod.rs`):
```rust
use std::collections::HashMap;  use puce16_core::dma::DmaBus;
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Ev { Elapse(u32), ARead(u32), AWrite(u32, u8), BRead(u8), BWrite(u8, u8) }
pub struct FakeBus { pub mem: HashMap<u32, u8>, pub ev: Vec<Ev>, pub master: u64, pub b_counter: u8 }
impl FakeBus { pub fn new(master: u64) -> FakeBus;                       // mem vide, ev vide, b_counter 0
               pub fn poke(&mut self, addr: u32, v: u8);
               pub fn b_writes(&self) -> Vec<(u8, u8)>;  pub fn a_writes(&self) -> Vec<(u32, u8)>;   // dans l'ordre
               pub fn count_b_reads(&self) -> usize; }
```
`a_read(addr)`: ajoute `ARead(addr)`, renvoie `mem[addr]` ou, si absent, `addr as u8`. `a_write`: `AWrite`, insere dans `mem`. `b_read(reg)`: `BRead(reg)`, `b_counter += 1`, renvoie `b_counter`. `b_write`: `BWrite(reg, v)`. `elapse(c)`: `Elapse(c)`, `master += c`. `master()` renvoie `master`.
2. Dans `dma/mod.rs`: `impl Dma { pub fn run_general<B: DmaBus>(&mut self, bus: &mut B, mask: u8); }`.
   a. `align = ((8 - bus.master() % 8) % 8) as u32` ; si `align > 0`: `bus.elapse(align)`. Puis `bus.elapse(8)` (depart).
   b. Pour n de 0 a 7 avec le bit n de `mask`: `bus.elapse(8)` ; `count = if das == 0 {65536} else {das as u32}` ; `offs = pattern_offsets(dmap & 7)` ; `step = a_step(dmap)` ; `i = 0` ; repeter `count` fois: `bus.elapse(8)` PUIS: adresse A = `(a1b << 16) | a1t` ; reg B = `bbad.wrapping_add(offs[i % offs.len()])` ; si `dmap & 0x80 == 0` (A->B) : `v = bus.a_read(A)` ; `bus.b_write(regB, v)` ; ensuite `a1t = step_a(a1t, step)` ; `i += 1`.
   c. A la fin du canal: `das = 0`. (Le sens B->A est ajoute en E03-20: ici le bit 7 de dmap n'est pas traite.)""",
  tests=[("p0_basic", "FakeBus::new(0). ch0: dmap=0x00, bbad=0x18, a1b=0x7E, a1t=0x1000, das=3. `run_general(&mut bus, 0x01)`: `b_writes()==[(0x18,0x00),(0x18,0x01),(0x18,0x02)]` (la source par defaut vaut `addr as u8`) ; `ch[0].das==0` ; `ch[0].a1t==0x1003` ; `bus.master==40` (8 + 8 + 3*8)."),
         ("event_order", "1 octet (das=1): `bus.ev==[Elapse(8), Elapse(8), Elapse(8), ARead(0x7E1000), BWrite(0x18,0x00)]`."),
         ("p1", "dmap=0x01, bbad=0x18, a1t=0x2000, das=4: regs B ecrits dans l'ordre 0x18,0x19,0x18,0x19 avec les valeurs 0x00,0x01,0x02,0x03."),
         ("p3", "dmap=0x03, bbad=0x04, das=4: regs 0x04,0x04,0x05,0x05."),
         ("p4_partial", "dmap=0x04, bbad=0x18, das=6: regs 0x18,0x19,0x1A,0x1B,0x18,0x19 (le transfert s'arrete au milieu du pattern)."),
         ("p5_p2", "dmap=0x05, bbad=0x22, das=4: regs 0x22,0x23,0x22,0x23 ; dmap=0x02, bbad=0x22, das=2: 0x22,0x22."),
         ("reg_wrap", "dmap=0x01, bbad=0xFF, das=2: regs 0xFF puis 0x00."),
         ("fixed_mode", "dmap=0x08 (A fixe), a1t=0x1000, das=3: les 3 lectures `ARead` sont a 0x7E1000 ; valeurs 0x00,0x00,0x00 ; `ch[0].a1t==0x1000` apres."),
         ("decrement_mode", "dmap=0x10, a1t=0x1002, das=3: lectures a 0x7E1002, 0x7E1001, 0x7E1000 (valeurs 0x02,0x01,0x00) ; `a1t==0x0FFF`."),
         ("a1t_wrap_in_bank", "dmap=0x00, a1b=0x7E, a1t=0xFFFE, das=3: lectures a 0x7EFFFE, 0x7EFFFF, 0x7E0000 (la banque ne change pas) ; `a1t==0x0001`."),
         ("das_zero_65536", "das=0, a1t=0, dmap=0: `b_writes().len()==65536` ; `bus.master==524_304` (8+8+65536*8) ; `a1t==0` ; `das==0`."),
         ("two_channels", "mask=0b101, ch0 das=1, ch2 das=2: ordre des `BWrite`: ch0 d'abord, puis ch2 ; `bus.master==8 + (8+8) + (8+16)==48` ; ch1 inchange (das reste 0xFFFF)."),
         ("alignment", "FakeBus::new(3): ch0 das=1: premier evenement `Elapse(5)`, puis Elapse(8) depart, Elapse(8) canal, Elapse(8) octet ; `bus.master==3+5+24==32`."),
         ("empty_mask", "`run_general(&mut bus, 0)`: aucun BWrite ; le temps de depart (alignement + 8) est quand meme consomme: `bus.master==8` pour FakeBus::new(0).")])

T("E03-20", "dma/mod.rs: run_general sens B -> A et cas WRAM <-> $2180",
  size=100,
  context="""Ajoute le sens B->A (bit 7 de DMAP) et la regle "WRAM <-> $2180 est ignore": quand l'adresse A est en WRAM et que le registre B vise $2180 (reg 0x80),
le transfert n'a aucun effet (rien n'est lu/ecrit cote B ou A, WMADD n'avance pas) mais le temps passe et les compteurs avancent. Detail `UNKNOWN - to confirm` (spec 03 section 3). """ + CHAIN,
  embed=["03_DMA.md#3. DMA general (MDMAEN $420B)"],
  write=[SRC + "/dma/mod.rs", TST + "/e03_20_dma_reverse.rs"],
  spec="""Modifie la boucle d'octet de `run_general` (dans l'ordre, pour chaque octet, APRES `bus.elapse(8)`):
```
regB = bbad.wrapping_add(offs[i % offs.len()])
wram_skip = regB == 0x80 && is_wram_addr(A)
if dmap & 0x80 == 0 {                       // A -> B
    if !wram_skip { v = bus.a_read(A); bus.b_write(regB, v); }
} else {                                    // B -> A
    if !wram_skip { v = bus.b_read(regB); bus.a_write(A, v); }
}
a1t = step_a(a1t, step); i += 1
```
(Dans le cas `wram_skip` en A->B on ne lit meme pas l'A-bus.) Les tests utilisent `FakeBus` (voir E03-19).""",
  tests=[("b_to_a", "dmap=0x80, bbad=0x39, a1b=0x7E, a1t=0x3000, das=2: `a_writes()==[(0x7E3000,1),(0x7E3001,2)]` ; `count_b_reads()==2` ; `a1t==0x3002` ; `das==0`."),
         ("b_to_a_p1", "dmap=0x81, bbad=0x39, das=4: les `BRead` visent les regs 0x39,0x3A,0x39,0x3A ; valeurs ecrites en A: 1,2,3,4."),
         ("b_to_a_decrement", "dmap=0x90, a1t=0x3001, das=2: ecritures A a 0x7E3001 puis 0x7E3000."),
         ("wram_skip_a_to_b", "dmap=0x00, bbad=0x80, a1b=0x7E, a1t=0x1000, das=3: AUCUN `BWrite` et AUCUN `ARead` ; `a1t==0x1003` ; `bus.master==40` (le temps passe quand meme)."),
         ("wram_skip_mirror", "meme test avec a1b=0x00, a1t=0x0100 (miroir WRAM) et avec a1b=0x01, a1t=0x1000 (das=3, reste < 0x2000): aucun BWrite."),
         ("rom_not_skipped", "bbad=0x80, a1b=0xC0, a1t=0x8000 (ROM): les `BWrite(0x80, ..)` ont lieu (3 pour das=3)."),
         ("wram_skip_b_to_a", "dmap=0x80, bbad=0x80, a1b=0x7E, das=2: aucun `BRead`, aucun `AWrite` ; temps consomme ; `a1t` avance de 2."),
         ("skip_per_transfer_reg", "dmap=0x01 (P1), bbad=0x7F, a1b=0x7E, das=4: les transferts vers reg 0x80 (offsets 1) sont ignores, ceux vers 0x7F ont lieu: `b_writes()` contient exactement 2 ecritures, toutes a reg 0x7F ; `a1t` avance de 4."),
         ("a_to_b_regression", "dmap=0x00, bbad=0x18, a1b=0x7E, a1t=0x1000, das=3 : meme resultat que E03-19 `p0_basic`.")])

T("E03-21", "dma/hdma.rs: hdma_init (debut de frame)",
  size=130,
  context="""Initialisation HDMA a chaque debut de frame (spec 03 section 4.1). Le PREMIER transfert n'a pas lieu ici: il est execute par `hdma_run_line` a la ligne 0 (E03-22).
Temps: `elapse(18)` une fois (si `mask != 0`), puis `elapse(8)` par canal actif. La lecture des octets de table ne coute rien (`UNKNOWN - to confirm`). """ + CHAIN,
  embed=["03_DMA.md#4.1 Initialisation (debut de chaque frame)", "09_SYSTEM.md#5. HDMA: declenchement"],
  write=[SRC + "/dma/hdma.rs", TST + "/e03_21_hdma_init.rs"],
  spec="""Dans `dma/hdma.rs`: `impl Dma { pub fn hdma_init<B: DmaBus>(&mut self, bus: &mut B, mask: u8); pub fn hdma_state(&self, n: usize) -> (u16, u8, bool, bool); }`
`hdma_state(n)` renvoie `(a2a, line_counter, do_transfer, done)` du canal `n`.
Algorithme de `hdma_init`: si `mask == 0` retourner sans rien faire. `bus.elapse(18)`. Pour chaque canal `n` dont le bit est dans `mask` (ordre 0..7): `bus.elapse(8)` ; `a2a = a1t` ; lire `b = bus.a_read((a1b << 16) | a2a)` ; `a2a = a2a.wrapping_add(1)` ; `nltr = b` ;
- si `b == 0`: `done = true; do_transfer = false; line_counter = 0` ; fin pour ce canal ;
- sinon: `done = false; do_transfer = true; line_counter = if b & 0x7F == 0 { 128 } else { b & 0x7F }` ; si le mode est indirect (`dmap & 0x40 != 0`): lire `lo` puis `hi` aux adresses `(a1b << 16) | a2a` (a2a +1 apres chaque lecture, wrap 16 bits), `das = (hi << 8) | lo`.
Les canaux hors `mask` ne sont pas modifies. Une lecture A-bus passe par `bus.a_read` (aucun `elapse`).""",
  tests=[("direct_basic", "FakeBus; ch0: dmap=0x00, bbad=0x26, a1b=0x7E, a1t=0x2000 ; `poke(0x7E2000, 0x03)`. `hdma_init(&mut bus, 1)`: `hdma_state(0)==(0x2001, 3, true, false)` ; `ch[0].nltr==3` ; `bus.master==26` (18+8) ; `b_writes()` vide."),
         ("event_order", "meme test: `bus.ev==[Elapse(18), Elapse(8), ARead(0x7E2000)]`."),
         ("zero_first_byte", "table `0x00`: `hdma_state(0)==(0x2001, 0, false, true)`."),
         ("repeat_flag_count", "octet 0x85: `line_counter==5`, `ch.nltr==0x85`. Octet 0x80: `line_counter==128`. Octet 0x01: 1. Octet 0x7F: 127. Octet 0xFF: 127."),
         ("indirect", "dmap=0x40, a1t=0x3000, table `[0x02, 0x00, 0x40]`: `hdma_state(0)==(0x3003, 2, true, false)` et `ch[0].das==0x4000`."),
         ("indirect_zero_line", "dmap=0x40, table `[0x00, 0x11, 0x22]`: done=true, a2a==0x3001 (pas de lecture des 2 octets indirects)."),
         ("mask_selects", "ch0 et ch2 actifs, ch1 pas: ch1 inchange (`hdma_state(1)==(0xFFFF, 0, false, true)` au power-on) ; `bus.master==18+8+8==34` ; les ARead arrivent dans l'ordre ch0 puis ch2."),
         ("empty_mask", "`hdma_init(&mut bus, 0)`: aucun evenement, `bus.master==0`."),
         ("a2a_wrap", "a1t=0xFFFF, table octet a 7E:FFFF = 0x02 : apres init `a2a==0x0000` (wrap dans la banque)."),
         ("reinit_resets_done", "un canal `done=true` (octet 0) re-initialise avec un octet non nul repasse a `done=false, do_transfer=true`.")])

T("E03-22", "dma/hdma.rs: hdma_run_line (un transfert par scanline)",
  size=140,
  context="""Execution HDMA d'une scanline (spec 03 sections 4.2, 4.3). Pour chaque canal actif (bit dans `mask` ET `done == false`): surcout, transfert eventuel, puis mise a jour du compteur de lignes.
Temps: `elapse(18)` une fois (seulement s'il y a au moins un canal actif), `elapse(8)` par canal actif, `elapse(8)` par octet transfere (avant la copie). """ + CHAIN,
  embed=["03_DMA.md#4.2 Table et transfert (chaque scanline, V = 0..derniere ligne visible-1)", "03_DMA.md#4.3 Timing HDMA (defauts)"],
  write=[SRC + "/dma/hdma.rs", TST + "/e03_22_hdma_run.rs"],
  spec="""Ajoute a `impl Dma` (dans `dma/hdma.rs`): `pub fn hdma_run_line<B: DmaBus>(&mut self, bus: &mut B, mask: u8);`
1. `active = mask & bits des canaux dont done == false`. Si `active == 0`: retourner. `bus.elapse(18)`.
2. Pour chaque canal actif n (0..7): `bus.elapse(8)`.
   a. Si `do_transfer`: `offs = pattern_offsets(dmap & 7)`; pour chaque `k` de `offs`: `bus.elapse(8)` ; source = si `dmap & 0x40 == 0` (direct): `(a1b << 16) | a2a` puis `a2a += 1` (wrap 16 bits) ; si indirect: `(dasb << 16) | das` puis `das += 1` ; `v = bus.a_read(src)` ; `bus.b_write(bbad.wrapping_add(k), v)`. (Le bit de direction est ignore.)
   b. `line_counter -= 1`.
   c. Si `line_counter == 0`: lire `b = bus.a_read((a1b << 16) | a2a)`, `a2a += 1`, `nltr = b` ; si `b == 0`: `done = true; do_transfer = false` ; sinon `line_counter = (b & 0x7F) ou 128 si 0 ; do_transfer = true` et, en mode indirect, lire `lo`,`hi` (comme en init) dans `das`.
      Sinon (compteur > 0): `do_transfer = (nltr & 0x80) != 0`.
Pas de gestion de `forced blank` (HDMA ecrit meme en forced blank).""",
  tests=[("repeat_free_entry", "FakeBus. ch0: dmap=0x00, bbad=0x26, a1b=0x7E, a1t=0x2000 ; table 7E:2000 = [0x03, 0xAA, 0x82, 0xBB, 0xCC, 0x00]. `hdma_init(1)` puis 6 appels `hdma_run_line(1)`: `b_writes()==[(0x26,0xAA),(0x26,0xBB),(0x26,0xCC)]` ; apres le 5eme appel `hdma_state(0).3==true` (done) ; le 6eme appel ne produit AUCUN evenement."),
         ("states_walk", "meme table, etat apres chaque `hdma_run_line`: apres le 1er: `(0x2002, 2, false, false)` ; 2eme: `(0x2002, 1, false, false)` ; 3eme: `(0x2003, 2, true, false)` (lu 0x82) ; 4eme: `(0x2004, 1, true, false)` ; 5eme: `(0x2005 + 1, 0, false, true)`."),
         ("timing_first_call", "apres `hdma_init` (master 26): le 1er `hdma_run_line` ajoute 18+8+8==34 -> `bus.master==60` ; le 2eme (pas de transfert) ajoute 26 -> 86."),
         ("event_order_run", "le 1er run_line (apres l'init): evenements `[Elapse(18), Elapse(8), Elapse(8), ARead(0x7E2001), BWrite(0x26,0xAA)]` (ignorer les evenements de l'init en les effacant avec `bus.ev.clear()` avant)."),
         ("pattern_p1", "dmap=0x01, bbad=0x18, table [0x01, 0x11, 0x22, 0x00]: 1er run: `b_writes()==[(0x18,0x11),(0x19,0x22)]` ; temps 18+8+16 apres l'init ; apres ce 1er run: `hdma_state(0).3==true` (done, l'octet suivant est 0x00) ; un 2eme run ne produit aucun evenement."),
         ("indirect_repeat", "dmap=0x40, dasb=0x7F, bbad=0x26, table 7E:3000 = [0x82, 0x00, 0x40, 0x00], donnees 7F:4000 = [0x55, 0x66]: apres init `das==0x4000` ; 1er run ecrit 0x55 (das->0x4001, do_transfer reste vrai) ; 2eme run ecrit 0x66 (das->0x4002) puis lit le 0x00 -> done."),
         ("indirect_non_repeat", "meme avec octet 0x02 (non repete): seul 0x55 est ecrit (1er run) ; le 2eme run ne transfere pas mais lit l'octet suivant (0x00) -> done."),
         ("line_count_128", "octet 0x80: `line_counter==128` apres init ; apres 127 runs `line_counter==1` ; au 128eme run la table est relue."),
         ("mask_excludes", "avec `mask` ne contenant pas le canal: aucun evenement. Canal `done`: aucun evenement."),
         ("two_channels", "ch0 et ch1 avec chacun une entree d'1 ligne P0 (1 octet): 1er run: `Elapse(18)` une seule fois, puis ch0 (8 + 8 + octet) puis ch1 (8 + 8 + octet) ; temps total apres l'init: 18 + 2*(8+8)==50."),
         ("a2a_wrap_in_run", "a2a==0xFFFF en direct: l'octet lu est 7E:FFFF et a2a repasse a 0.")])

T("E03-23", "timing/mod.rs: Timing (ligne, refresh, VBlank/NMI) + banc de test Rig",
  size=140,
  context="""Debut de `Timing::on_dot` (spec 09 section 8, points 1 a 3) et petit banc de test `Rig` qui reproduit la boucle `catch_up` du bus:
`c = ppu.dot_clocks(); ppu.step_dot(); t += c; timing.on_dot(&mut mmio, &mut joy, &ppu, t)`. """ + CHAIN,
  embed=["09_SYSTEM.md#2.2 Positions et constantes de `timing/mod.rs` (toutes `UNKNOWN - to confirm`, reglables)", "09_SYSTEM.md#3.6 Timing (`timing/mod.rs`)", "09_SYSTEM.md#8. Algorithme de `Timing::on_dot`"],
  write=[SRC + "/timing/mod.rs", TST + "/common/rig.rs", TST + "/e03_23_timing_nmi.rs"],
  spec="""1. `timing/mod.rs`:
```rust
pub const NMI_FLAG_DOT: u16 = 2;  pub const IRQ_H_OFFSET_DOTS: u16 = 1;  pub const AUTOREAD_START_DOT: u16 = 33;
pub const AUTOREAD_CLOCKS: u64 = 4224;  pub const HDMA_INIT_DOT: u16 = 3;  pub const HDMA_RUN_DOT: u16 = 276;
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)] pub struct TimingOut { pub hdma_init: bool, pub hdma_run: bool, pub vblank_start: bool, pub frame_end: bool }
pub struct Timing { line_start: u64, refreshed_this_line: bool, nmi_edge: bool, nmi_line_prev: bool, autoread_end: Option<u64> }
impl Timing {
    pub fn new() -> Timing;        // tout a 0/false/None
    pub fn on_dot(&mut self, mmio: &mut Mmio, joy: &mut Joypads, ppu: &Ppu, t: u64) -> TimingOut;
    pub fn take_nmi_edge(&mut self) -> bool;                   // renvoie nmi_edge puis le met a false
    pub fn hclock(&self, master: u64) -> u32;                  // (master - line_start) as u32
    pub fn refresh_due(&self, master: u64) -> bool;            // !refreshed_this_line && hclock(master) >= consts::DRAM_REFRESH_HCLOCK
    pub fn mark_refreshed(&mut self);
    pub fn autoread_busy(&self, master: u64) -> bool;          // E03-25 ; ici: false
    pub fn hash_into(&self, h: &mut crate::hash::Fnv64);       // line_start u64, refreshed u8, nmi_edge u8, nmi_line_prev u8, autoread_end (0 ou 1 + u64)
}
```
`on_dot` (cette tache: points 1 a 3 seulement; `v = ppu.v()`, `dot = ppu.dot()` APRES le step):
1. `dot == 0`: `line_start = t ; refreshed_this_line = false` ; si `v == 0`: `out.frame_end = true ; mmio.rdnmi_flag = false`.
2. `v == ppu.vblank_start() && dot == NMI_FLAG_DOT`: `mmio.rdnmi_flag = true ; out.vblank_start = true`.
3. `line = mmio.rdnmi_flag && (mmio.nmitimen & 0x80 != 0)` ; si `line && !nmi_line_prev`: `nmi_edge = true` ; `nmi_line_prev = line`.
Les points 4 (IRQ), 5 (auto-read), 6 (HDMA) sont E03-24 et E03-25.
2. `tests/common/rig.rs` (inclus par `#[path = "common/rig.rs"] mod rig;`):
```rust
pub struct Rig { pub ppu: Ppu, pub timing: Timing, pub mmio: Mmio, pub joy: Joypads, pub t: u64 }
impl Rig { pub fn new(region: Region) -> Rig;                 // t = 0, position (0,0) sans avoir appele on_dot
           pub fn step(&mut self) -> TimingOut;               // la boucle catch_up ci-dessus pour UN dot
           pub fn run_until(&mut self, v: u16, dot: u16);     // step tant que (ppu.v(), ppu.dot()) != (v, dot) ; panic si plus de 200_000 steps
           pub fn pos(&self) -> (u16, u16); }
```""",
  tests=[("hclock_line_start", "`Rig::new(Ntsc)`; `run_until(1,0)`: `timing.hclock(rig.t)==0` ; `run_until(1,10)`: `hclock(rig.t)==40` ; `run_until(1,324)`: hclock==1297."),
         ("refresh_due", "`run_until(1,134)` (hclock 536): `refresh_due(t)==false` ; `run_until(1,135)` (hclock 540): true ; apres `mark_refreshed()`: false ; `run_until(2,0)` puis `run_until(2,135)`: de nouveau true."),
         ("rdnmi_flag_timing", "`run_until(225,1)`: `mmio.rdnmi_flag==false` ; un `step()` de plus (position (225,2)): `rdnmi_flag==true` et le `TimingOut` de CE step a `vblank_start==true` ; le step suivant a `vblank_start==false`."),
         ("one_vblank_per_frame", "sur 262*340 steps NTSC: exactement 1 `vblank_start==true` et exactement 1 `frame_end==true` (le step qui arrive en (0,0))."),
         ("frame_end_clears_flag", "apres le vblank, `rdnmi_flag==true` ; en arrivant en (0,0) de la frame suivante: `rdnmi_flag==false` et `frame_end==true`."),
         ("nmi_edge_enabled", "`mmio.nmitimen=0x80` avant de commencer ; `run_until(225,2)`: `take_nmi_edge()==true` ; un 2eme appel: false ; apres 100 steps de plus: `take_nmi_edge()==false`."),
         ("nmi_edge_disabled", "nmitimen=0: `run_until(230,0)`: `take_nmi_edge()==false`. Puis `mmio.nmitimen=0x80` et UN `step()`: `take_nmi_edge()==true` (le flag RDNMI est encore vrai)."),
         ("nmi_flag_cleared_by_read", "nmitimen=0x80 ; `run_until(225,2)` ; `take_nmi_edge()` consomme ; `mmio.rdnmi_flag=false` (comme une lecture de $4210) ; 5 steps ; `take_nmi_edge()==false` ; ensuite sur la frame suivante, a (225,2): un nouveau front (`true`)."),
         ("overscan_pal", "NTSC `rig.ppu.write(0x33,0x04)`: le front `vblank_start` arrive en (240,2) ; PAL (sans overscan): en (225,2)."),
         ("hash", "deux `Timing::new()` ont le meme hash ; apres `mark_refreshed()` il differe.")])

T("E03-24", "timing/mod.rs: IRQ H/V",
  size=100,
  context="Point 4 de l'algorithme `on_dot`: IRQ par comparaison de H et/ou V avec HTIME/VTIME (spec 02 section 6.3 et spec 09 section 8). " + CHAIN,
  embed=["02_BUS_TIMING.md#6.3 IRQ timer (H/V)", "09_SYSTEM.md#8. Algorithme de `Timing::on_dot`"],
  write=[SRC + "/timing/mod.rs", TST + "/e03_24_timing_irq.rs"],
  spec="""Ajoute a `on_dot`, APRES le point 3: `mode = (mmio.nmitimen >> 4) & 3`. Si `mode != 0`:
- `htime_ok = mmio.htime <= 339` ; `vtime_ok = mmio.vtime <= ppu.last_line()` ;
- mode 1: `hit = htime_ok && dot == htime + IRQ_H_OFFSET_DOTS` ;
- mode 2: `hit = vtime_ok && v == vtime && dot == IRQ_H_OFFSET_DOTS` ;
- mode 3: `hit = htime_ok && vtime_ok && v == vtime && dot == htime + IRQ_H_OFFSET_DOTS`.
Si `hit`: `mmio.timeup_flag = true; mmio.irq_line = true`. (Les types de `htime`/`vtime`: `u16`; `dot`, `v`: `u16`.)""",
  tests=[("mode1", "`Rig::new(Ntsc)`; `mmio.nmitimen=0x10; mmio.htime=100`: `run_until(0,101)`: avant ce step `timeup_flag==false`, apres: true et `irq_line==true`. Position exacte: le step qui ARRIVE en (0,101)."),
         ("mode1_every_line", "apres avoir efface (`mmio.timeup_flag=false; mmio.irq_line=false`), `run_until(1,100)`: toujours faux ; `run_until(1,101)`: vrai (a chaque ligne)."),
         ("mode2", "`nmitimen=0x20; vtime=10`: aucun drapeau avant (10,1) (verifier en parcourant tous les steps jusqu'a (10,0)) ; `run_until(10,1)`: vrai ; pas de nouveau declenchement a (11,1) apres effacement."),
         ("mode3", "`nmitimen=0x30; htime=50; vtime=10`: faux jusqu'a (10,50) ; `run_until(10,51)`: vrai. Ligne 11 (11,51): rien."),
         ("mode0_never", "nmitimen=0x00 avec htime=100: sur 2 lignes entieres jamais de drapeau."),
         ("htime_too_big", "mode 1, `htime=340`: sur une frame entiere (262*340 steps): jamais de drapeau ; `htime=339` (dot 340 n'existe pas sur ligne normale) : jamais non plus ; `htime=338`: declenche en dot 339."),
         ("vtime_too_big", "NTSC mode 2, `vtime=262` (> last_line 261): jamais sur une frame. PAL mode 2, `vtime=262`: declenche en (262,1)."),
         ("clear_by_4211", "apres declenchement, `mmio.read(0x4211,0,rig.t,Default::default())` renvoie un octet avec bit 7 a 1 puis `timeup_flag==false`, `irq_line==false`."),
         ("clear_by_nmitimen", "apres declenchement, `mmio.write(0x4200,0x00,t)` efface les deux ; `mmio.write(0x4200,0x10,t)` ne les efface pas."),
         ("vtime_zero_mode3", "mode 3, `vtime=0`, `htime=0`: declenche en (0,1) de la 1re frame.")])

T("E03-25", "timing/mod.rs: auto-read joypad et declencheurs HDMA",
  size=110,
  context="Points 5 et 6 de `on_dot`: l'auto-read dure 4224 clocks (HVBJOY.0 vrai), les registres JOYn sont mis a jour a la fin; HDMA init a (0,3) et run a (v<=vis_last,276). " + CHAIN,
  embed=["02_BUS_TIMING.md#6.4 Auto-read joypad", "09_SYSTEM.md#5. HDMA: declenchement", "09_SYSTEM.md#8. Algorithme de `Timing::on_dot`"],
  write=[SRC + "/timing/mod.rs", TST + "/e03_25_timing_autoread_hdma.rs"],
  spec="""Ajoute a `on_dot`, apres le point 4:
5. `if mmio.nmitimen & 1 != 0 && v == ppu.vblank_start() && dot == AUTOREAD_START_DOT { autoread_end = Some(t + AUTOREAD_CLOCKS) }` ; puis `if let Some(e) = autoread_end { if t >= e { let a = joy.autoread_values(); mmio.joy = a; autoread_end = None } }`.
6. `if mmio.hdmaen != 0 { if v == 0 && dot == HDMA_INIT_DOT { out.hdma_init = true } ; if v <= ppu.vblank_start() - 1 && dot == HDMA_RUN_DOT { out.hdma_run = true } }`.
`autoread_busy(master)`: `matches!(autoread_end, Some(e) if master < e)` ; cependant le debut doit etre connu: ajoute le champ `autoread_start: u64` pose en meme temps que `autoread_end`, et `busy = master >= autoread_start && master < e`. (Champ ajoute a `hash_into`.)""",
  tests=[("autoread_updates_at_end", "`Rig::new(Ntsc)`; `mmio.nmitimen=1`; `joy.set_buttons(0,0x8001)`, `joy.set_buttons(1,0x0F00)`. `run_until(225,33)`: `mmio.joy==[0;4]` et `timing.autoread_busy(rig.t)==true`. Noter `t0=rig.t`. Continuer pas a pas jusqu'au 1er step apres lequel `mmio.joy[0]!=0` : `dt = rig.t - t0` verifie `4224 <= dt && dt < 4229` ; alors `mmio.joy==[0x8001,0x0F00,0,0]`, `autoread_busy(rig.t)==false`."),
         ("autoread_busy_window", "avec nmitimen=1: `autoread_busy(t0 - 1)==false` (t0 = debut en (225,33)), `autoread_busy(t0)==true`, `autoread_busy(t0+4223)==true` ; apres le step qui met `mmio.joy` a jour: `autoread_busy(rig.t)==false`."),
         ("shift_reset_after_autoread", "apres la mise a jour, `rig.joy.read_port(0)==1` (registre a 0xFFFF)."),
         ("autoread_disabled", "nmitimen=0: `run_until(230,0)` puis 2000 steps : `mmio.joy==[0;4]` et `autoread_busy` toujours faux."),
         ("autoread_once_per_frame", "nmitimen=1, apres la mise a jour, changer `joy.set_buttons(0,0x0001)`: `mmio.joy` ne change plus avant la frame suivante ; sur la frame suivante, il prend 0x0001."),
         ("hdma_off", "hdmaen=0: sur une frame, aucun `hdma_init`/`hdma_run`."),
         ("hdma_init_once", "hdmaen=1: sur une frame (a partir de la position (0,0)): exactement un `hdma_init`, et c'est le step qui arrive en (0,3)."),
         ("hdma_run_count", "hdmaen=1, NTSC: 225 `hdma_run` par frame (v=0..=224, dot 276) ; avec `ppu.write(0x33,0x04)` (overscan): 240."),
         ("hdma_run_position", "le 1er `hdma_run` arrive en (0,276) ; il n'y en a pas pour v>=225."),
         ("hash_includes_autoread", "le hash de `Timing` differe pendant l'auto-read (apres (225,33)) et apres.")])
