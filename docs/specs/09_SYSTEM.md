# SPEC 09 - ASSEMBLAGE DU SYSTEME (Snes, SysBus, temps, MMIO, DMA)

Complete 02_BUS_TIMING.md et 03_DMA.md (valeurs materielles) par la STRUCTURE de code et le flux de temps.
Les valeurs `UNKNOWN - to confirm` restent des constantes nommees (reglables par les taches de tuning).

## 1. Possession des composants

```rust
pub struct Snes { cpu: Cpu, sys: SysBus, region: Region, /* + champs du motif de test E01-06 */ }
pub struct SysBus {                       // bus/mod.rs ; implemente cpu::Bus
    pub master: u64,                      // date courante du CPU, en clocks maitres
    pub region: Region,
    pub cart: Cart, pub wram: Wram, pub mmio: Mmio, pub dma: Dma, pub ppu: Ppu, pub apu: Apu, pub joy: Joypads,
    pub timing: Timing,
    pub mdr: u8,                          // open bus CPU
    pub memsel: bool,                     // MEMSEL bit 0 (FastROM)
    ppu_t: u64,                           // date de DEBUT du dot courant du PPU
    pending_dma: u8, pending_hdma_init: bool, pending_hdma_run: bool,
}
```
`Snes::step_instruction` = `self.cpu.step(&mut self.sys)`. Le CPU ne possede pas le bus: le bus est passe a `step`.

## 2. Flux de temps

```rust
impl SysBus {
    fn advance(&mut self, clocks: u32) { self.master += clocks as u64; self.catch_up(); }
    fn catch_up(&mut self) {
        while self.ppu_t < self.master {                 // le dot s'execute quand master > debut du dot (strict)
            let c = self.ppu.dot_clocks() as u64;        // duree du dot courant
            let _ev = self.ppu.step_dot();
            self.ppu_t += c;
            // la position (ppu.v(), ppu.dot()) est maintenant celle du dot qui COMMENCE a ppu_t
            self.timing.on_dot(&mut self.mmio, &mut self.joy, &self.ppu, self.ppu_t);   // renvoie TimingOut
            ... stocker les drapeaux pending_hdma_init / pending_hdma_run
        }
        self.apu.step_to(self.master);                   // E06 (squelette avant)
    }
}
```
`Timing::on_dot` est une fonction position-dependante; elle ne fait aucun acces bus. `TimingOut { hdma_init: bool, hdma_run: bool, vblank_start: bool, frame_end: bool }`.

### 2.1 Cycle CPU (impl Bus pour SysBus)
```
fn read(addr):   begin_cycle(); advance(access_clocks(addr, memsel)); v = read_inner(addr); mdr = v (sauf registres a open bus partiel, voir read_inner); end_cycle(); v
fn write(addr,v): begin_cycle(); advance(access_clocks(..)); mdr = v; write_inner(addr, v); end_cycle()
fn idle():       begin_cycle(); advance(6); end_cycle()
begin_cycle: (1) rafraichissement DRAM (3.3 de 02): si !refreshed_this_line et (master - line_start) >= DRAM_REFRESH_HCLOCK: advance(40); refreshed_this_line = true
             (2) si pending_hdma_init ou pending_hdma_run: executer HDMA (voir section 5), puis effacer les drapeaux
end_cycle:   si pending_dma != 0: executer le DMA general (section 4) puis pending_dma = 0
```
(Le DMA part donc apres le cycle qui suit l'ecriture de MDMAEN: en pratique apres la lecture de l'opcode suivant, comme la spec 03 section 3.1.)
Les lignes d'interruption pour le CPU: `nmi_pending()` = front NMI detecte par `Timing` (consomme); `irq_asserted()` = `mmio.irq_line`.

### 2.2 Positions et constantes de `timing/mod.rs` (toutes `UNKNOWN - to confirm`, reglables)
```rust
pub const NMI_FLAG_DOT: u16 = 2;          // le flag RDNMI est leve quand la position devient (vblank_start, 2)
pub const IRQ_H_OFFSET_DOTS: u16 = 1;     // l'IRQ H/V se declenche quand dot == HTIME + 1 (mode V seul: dot == 1)
pub const AUTOREAD_START_DOT: u16 = 33;   // auto-read demarre a (vblank_start, 33) si NMITIMEN bit0
pub const AUTOREAD_CLOCKS: u64 = 4224;
pub const HDMA_INIT_DOT: u16 = 3;         // (v == 0, dot == 3)  = 12 clocks
pub const HDMA_RUN_DOT: u16 = 276;        // (v in 0..=vis_last, dot == 276) = 1104 clocks
pub const DRAM_REFRESH_HCLOCK: u32 = 538; // deja dans consts.rs (E01-03): ne pas redefinir
```
`vis_last = ppu.vblank_start() - 1` (224 ou 239).

## 3. Interface des composants (signatures obligatoires)

### 3.1 Wram (`bus/wram.rs`)
```rust
pub struct Wram { mem: Vec<u8> /* 0x20000, rempli de 0x55 */, pub addr: u32 /* WMADD, 17 bits */ }
impl Wram { pub fn new() -> Wram; pub fn reset_addr(&mut self);
    pub fn read(&self, idx: u32) -> u8; pub fn write(&mut self, idx: u32, v: u8);      // idx & 0x1FFFF
    pub fn data_read(&mut self) -> u8;  pub fn data_write(&mut self, v: u8);           // $2180 (lit/ecrit a addr puis addr = (addr+1) & 0x1FFFF)
    pub fn set_addr_l(&mut self, v: u8); pub fn set_addr_m(&mut self, v: u8); pub fn set_addr_h(&mut self, v: u8);  // $2181/2/3 (h: seul le bit 0)
    pub fn hash_into(&self, h: &mut Fnv64); }
```
### 3.2 Joypads (`joypad.rs`, ajouter a E01-12)
```rust
pub struct Joypads { state: [u16; 2], shift: [u16; 2], out0: bool }
impl Joypads { pub fn new() -> Joypads;
    pub fn set_buttons(&mut self, port: usize, v: u16);          // format de set_joypad (spec 02 6.5)
    pub fn write_out(&mut self, bit0: bool);                      // $4016 ecriture: `if bit0 || self.out0 { shift = state }` (rechargement) puis `out0 = bit0`
    pub fn read_port(&mut self, port: usize) -> u8;               // bit0 = D0 (poids 1), bit1 = D1 = 0 ; voir 3.2.1
    pub fn autoread_values(&mut self) -> [u16; 4];                // [JOY1, JOY2, 0, 0] = state ; apres l'appel shift[p] = 0xFFFF
    pub fn hash_into(&self, h: &mut Fnv64); }
```
3.2.1 `read_port`: si `out0` : renvoie `(state[p] >> 15) & 1` sans decaler; sinon `bit = (shift[p] >> 15) & 1; shift[p] = (shift[p] << 1) | 1; bit`.
`write_out(bit0)`: `if bit0 || self.out0 { shift = state }` (sur 2 ports); puis `self.out0 = bit0`.

### 3.3 Apu (squelette, `apu/mod.rs`; sera complete en E06)
```rust
pub struct Apu { to_apu: [u8; 4], from_apu: [u8; 4] /* valeurs vues par le CPU */ }
impl Apu { pub fn new() -> Apu; pub fn reset(&mut self);
    pub fn port_read(&self, i: usize) -> u8 { self.from_apu[i & 3] }     // $2140-$2143 (miroir tous les 4)
    pub fn port_write(&mut self, i: usize, v: u8) { self.to_apu[i & 3] = v }
    pub fn step_to(&mut self, master: u64) {}                              // E06
    pub fn hash_into(&self, h: &mut Fnv64); }
```
### 3.4 Ppu (squelette de timing en E03, rendu en E04)
Interface complete = 05_PPU.md section 8. E03 fournit: compteurs v/dot/field, tables de dots, `step_dot` (evenements, SANS rendu), `dot_clocks`, `line_dots`, `vblank_start()` (225 / 240 selon SETINI bit 2), `in_vblank`, `in_hblank`, `write` ne traitant que 0x00 (INIDISP: seulement le bit 7 forced_blank stocke) et 0x33 (SETINI: interlace bit0, overscan bit2), `read` renvoyant `cpu_mdr`. E04 complete.

### 3.5 Mmio (`bus/mmio.rs`)
```rust
pub enum MmioEffect { None, Mdmaen(u8), Hdmaen(u8), Memsel(bool), Wrio { old: u8, new: u8 } }
pub struct Mmio { pub nmitimen: u8, pub wrio: u8, pub htime: u16, pub vtime: u16, pub hdmaen: u8, pub memsel: bool,
                  pub rdnmi_flag: bool, pub timeup_flag: bool, pub irq_line: bool, pub joy: [u16; 4],
                  /* unite math */ ... }
impl Mmio { pub fn new() -> Mmio; pub fn reset(&mut self);
    pub fn write(&mut self, reg: u16 /* 0x4200..=0x420D */, v: u8, master: u64) -> MmioEffect;
    pub fn read(&mut self, reg: u16 /* 0x4210..=0x421F */, mdr: u8, master: u64, st: HvState) -> u8;
    pub fn hash_into(&self, h: &mut Fnv64); }
pub struct HvState { pub vblank: bool, pub hblank: bool, pub autoread_busy: bool }
```
### 3.6 Timing (`timing/mod.rs`)
```rust
pub struct Timing { line_start: u64, refreshed_this_line: bool, nmi_edge: bool, nmi_line_prev: bool, autoread_end: Option<u64>, ... }
pub struct TimingOut { pub hdma_init: bool, pub hdma_run: bool, pub vblank_start: bool, pub frame_end: bool }
impl Timing { pub fn new() -> Timing;
    pub fn on_dot(&mut self, mmio: &mut Mmio, joy: &mut Joypads, ppu: &Ppu, t: u64) -> TimingOut;   // t = date de debut du dot (nouvelle position)
    pub fn take_nmi_edge(&mut self) -> bool;               // consomme le front NMI
    pub fn hclock(&self, master: u64) -> u32;               // master - line_start
    pub fn autoread_busy(&self, master: u64) -> bool;
    pub fn hash_into(&self, h: &mut Fnv64); }
```
(`joy` est `&mut` car la fin de l'auto-read appelle `autoread_values`.)

### 3.7 Dma (`dma/mod.rs`, `dma/hdma.rs`)
```rust
pub trait DmaBus {
    fn a_read(&mut self, addr: u32) -> u8;      // lecture A-bus SANS avancer le temps (le moteur DMA gere le temps)
    fn a_write(&mut self, addr: u32, v: u8);
    fn b_read(&mut self, reg: u8) -> u8;        // $2100 + reg
    fn b_write(&mut self, reg: u8, v: u8);
    fn elapse(&mut self, clocks: u32);          // avance master et appelle catch_up
    fn master(&self) -> u64;
}
#[derive(Default)] pub struct Channel { pub dmap: u8, pub bbad: u8, pub a1t: u16, pub a1b: u8, pub das: u16, pub dasb: u8, pub a2a: u16, pub nltr: u8, pub unused: u8,
                                         /* etat HDMA prive */ do_transfer: bool, done: bool, line_counter: u8 }
#[derive(Default)] pub struct Dma { pub ch: [Channel; 8] }
impl Dma { pub fn new() -> Dma;                                   // valeurs de power-on de la spec 03
    pub fn read(&self, reg: u16) -> Option<u8>; pub fn write(&mut self, reg: u16, v: u8);    // $4300-$437F ; None = open bus
    pub fn run_general<B: DmaBus>(&mut self, bus: &mut B, mask: u8);
    pub fn hdma_init<B: DmaBus>(&mut self, bus: &mut B, mask: u8);
    pub fn hdma_run_line<B: DmaBus>(&mut self, bus: &mut B, mask: u8);
    pub fn hash_into(&self, h: &mut Fnv64); }
```
`SysBus` appelle `let mut d = std::mem::take(&mut self.dma); d.run_general(self, mask); self.dma = d;` (evite l'emprunt double). `SysBus` implemente `DmaBus`.

## 4. DMA general: temps (decisions provisoires, spec 03 section 3.1)
`run_general(bus, mask)`: `bus.elapse(align8)` ou `align8` amene `master` au multiple de 8 clocks suivant (`(8 - master % 8) % 8`); `bus.elapse(8)` surcout de depart; par canal actif (bit de `mask`, ordre 0..7): `bus.elapse(8)`; par octet transfere: `bus.elapse(8)` PUIS la copie.
Un canal dont `das == 0` au depart transfere 65536 octets. Apres le canal `das = 0`, `a1t` avance selon le mode.

## 5. HDMA: declenchement
- `pending_hdma_init`: `Timing` renvoie `hdma_init` a (v == 0, dot == HDMA_INIT_DOT) si `mmio.hdmaen != 0` (valeur au moment du declenchement).
- `pending_hdma_run`: `hdma_run` a (v in 0..=vis_last, dot == HDMA_RUN_DOT) si `mmio.hdmaen != 0`.
- Execution dans `begin_cycle`: init: `dma.hdma_init(self, mmio.hdmaen)`; run: `dma.hdma_run_line(self, mmio.hdmaen)`. Surcout: `bus.elapse(18)` (une fois par appel) + 8 par canal actif + 8 par octet (spec 03 4.3), appliques par le moteur HDMA.
- Un canal HDMA actif est un canal dont le bit est dans `hdmaen` et dont `done == false`.

## 6. Open bus et routage (`read_inner` / `write_inner`)
Banque `b = addr >> 16`, `a = addr & 0xFFFF`. Zones systeme pour `(b & 0x7F) <= 0x3F` (banques 00-3F et 80-BF):
| a | read | write |
|---|---|---|
| 0000-1FFF | `wram.read(a)` | `wram.write(a, v)` |
| 2100-213F | `ppu.read(a-0x2100, mdr, wrio bit7)` (E03: renvoie mdr) ; met a jour `mdr` | `ppu.write(a-0x2100, v)` |
| 2140-217F | `apu.port_read(a & 3)` | `apu.port_write(a & 3, v)` |
| 2180 | `wram.data_read()` | `wram.data_write(v)` |
| 2181-2183 | open bus (mdr) | `wram.set_addr_l/m/h(v)` |
| 4016 | `(mdr & 0xFC) \| joy.read_port(0)` | `joy.write_out(v & 1 != 0)` |
| 4017 | `(mdr & 0xE0) \| 0x1C \| joy.read_port(1)` | ignoree |
| 4200-420D | open bus | `mmio.write(..)` puis appliquer l'effet (Mdmaen -> `pending_dma = v`; Hdmaen -> `mmio.hdmaen = v` ; Memsel -> `self.memsel`; Wrio -> `ppu.wrio_write(old,new)`) |
| 4210-421F | `mmio.read(..)` | ignoree |
| 4300-437F | `dma.read(a)` (None -> mdr) | `dma.write(a, v)` |
| 6000-FFFF | `cart.read(b, a)` (None -> mdr) | `cart.write(b, a, v)` |
| autre | open bus (mdr) | ignoree |
Banques 40-7D / C0-FF: `7E-7F` = `wram.read(((b & 1) << 16) | a)` ; autres: `cart.read(b, a)` (None -> mdr). Les ecritures cartouche passent par `cart.write`.
Les lectures ne mettent `mdr` a jour que quand la valeur renvoyee provient du bus de donnees externe: en v1, `mdr = valeur lue` pour TOUTE lecture CPU.

## 7. Boucle d'une frame
`Snes::run_frame`: `loop { cpu.step(&mut sys); if sys.take_vblank_flag() { break } }` (le drapeau est pose quand `TimingOut.vblank_start` est vu). Garde-fou: si plus de 2 000 000 instructions sans VBlank -> `panic!("run_frame: no vblank")` (CPU arrete par STP, par exemple).
`Snes::run_master(n)`: `let end = sys.master + n; while sys.master < end { cpu.step(..) }`.
`frame_count` = nombre de `vblank_start` vus.

## 8. Algorithme de `Timing::on_dot` (position = (ppu.v(), ppu.dot()) APRES le step, `t` = debut de ce dot)
Dans cet ordre:
1. `dot == 0`: `line_start = t`; `refreshed_this_line = false`. Si de plus `v == 0`: `out.frame_end = true`; `mmio.rdnmi_flag = false` (fin du VBlank).
2. `v == ppu.vblank_start() && dot == NMI_FLAG_DOT`: `mmio.rdnmi_flag = true`; `out.vblank_start = true`.
3. Ligne NMI: `line = mmio.rdnmi_flag && (mmio.nmitimen & 0x80 != 0)`; si `line && !nmi_line_prev`: `nmi_edge = true`; `nmi_line_prev = line`.
4. IRQ H/V: `mode = (mmio.nmitimen >> 4) & 3`; si `mode != 0`: mode 1: `dot == htime + IRQ_H_OFFSET_DOTS`; mode 2: `v == vtime && dot == IRQ_H_OFFSET_DOTS`; mode 3: `v == vtime && dot == htime + IRQ_H_OFFSET_DOTS` ; `htime > 339` ou `vtime > derniere ligne` : jamais. Si vrai: `mmio.timeup_flag = true; mmio.irq_line = true`.
5. Auto-read: si `nmitimen & 1 != 0 && v == vblank_start && dot == AUTOREAD_START_DOT`: `autoread_end = Some(t + AUTOREAD_CLOCKS)`. Puis, si `autoread_end == Some(e)` et `t >= e`: `mmio.joy = joy.autoread_values()`; `autoread_end = None`.
6. HDMA: `mmio.hdmaen != 0`: `v == 0 && dot == HDMA_INIT_DOT` -> `out.hdma_init`; `v <= vis_last && dot == HDMA_RUN_DOT` -> `out.hdma_run`.
`mmio.read` de `$4210` efface `rdnmi_flag` ; de `$4211` efface `timeup_flag` ET `irq_line` ; `mmio.write(NMITIMEN)` avec bits V/H = 00 efface `timeup_flag` et `irq_line`.
