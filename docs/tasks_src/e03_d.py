from _dsl import T

CORE = "crates/puce16-core"
SRC = CORE + "/src"
TST = CORE + "/tests"

CHAIN = ("Execution strictement sequentielle (D-020): cette tache suppose que toutes les precedentes sont terminees. "
         "Ne modifie QUE les fichiers de 'Write only in'.")

# Test helper recurrent: un SysBus de test se construit ainsi (a recopier dans chaque fichier de test)
MK = ("Dans les tests: `mod common; use common::*;` puis "
      "`fn mk(code: &[u8]) -> SysBus { SysBus::new(Cart::from_bytes(&make_rom(code, Map::LoRom)).unwrap(), Region::Ntsc) }` "
      "(imports: `puce16_core::bus::SysBus`, `puce16_core::cart::Cart`, `puce16_core::Region`).")

# ------------------------------------------------------------------ E03 part D: SysBus and Snes (E03-26..35)

T("E03-26", "bus/mod.rs: struct SysBus (possession des composants), new, reset, hash",
  size=120, deps=["E03-25"],
  context="""Le bus systeme possede tous les composants (spec 09 section 1) et implementera `cpu::Bus`. Cette tache cree seulement la structure et ses constructeurs.
Les champs prives ont des accesseurs `pub` pour les tests. """ + CHAIN,
  embed=["09_SYSTEM.md#1. Possession des composants", "02_BUS_TIMING.md#7. Reset et power-on"],
  write=[SRC + "/bus/mod.rs", TST + "/e03_26_sysbus.rs"],
  spec="""Dans `bus/mod.rs` (garde `access_clocks`, `pub mod wram; pub mod mmio;`) ajoute:
```rust
pub struct SysBus {
    pub master: u64, pub region: Region,
    pub cart: Cart, pub wram: Wram, pub mmio: Mmio, pub dma: Dma, pub ppu: Ppu, pub apu: Apu, pub joy: Joypads, pub timing: Timing,
    pub mdr: u8, pub memsel: bool,
    ppu_t: u64, pending_dma: u8, dma_wait: bool, pending_hdma_init: bool, pending_hdma_run: bool, vblank_flag: bool, vblank_count: u64,
}
impl SysBus {
    pub fn new(cart: Cart, region: Region) -> SysBus;     // tout a l'etat de power-on; master 0; ppu_t 0; mdr 0; memsel false; flags faux; vblank_count 0
    pub fn reset(&mut self);                              // reset bouton: mmio.reset(), ppu.reset(), apu.reset(), wram.reset_addr(), timing = Timing::new(), pending_* = faux, master = 0, ppu_t = 0, mdr = 0, memsel = false, vblank_flag = false, vblank_count = 0 ; WRAM, cart (SRAM), dma, joy conserves
    pub fn take_vblank_flag(&mut self) -> bool;           // renvoie vblank_flag puis le met a false
    pub fn vblank_count(&self) -> u64;  pub fn pending_dma(&self) -> u8;
    pub fn hash_into(&self, h: &mut crate::hash::Fnv64);  // master (u64), mdr, memsel (u8), ppu_t (u64), pending_dma, dma_wait, pending_hdma_init, pending_hdma_run, vblank_flag (u8), vblank_count (u64), puis cart, wram, mmio, dma, ppu, apu, joy, timing
}
```
Imports: `crate::cart::Cart`, `crate::dma::Dma`, `crate::ppu::Ppu`, `crate::apu::Apu`, `crate::joypad::Joypads`, `crate::timing::Timing`, `crate::region::Region`, `wram::Wram`, `mmio::Mmio`.""",
  tests=[("power_on", MK + " `mk(&[0xEA])`: master==0, mdr==0, memsel==false, `wram.read(0)==0x55`, `ppu.v()==0`, `ppu.dot()==0`, `vblank_count()==0`, `take_vblank_flag()==false`, `pending_dma()==0`, `region==Ntsc`."),
         ("reset_keeps_wram", "`bus.wram.write(5,1); bus.mmio.nmitimen=0x80; bus.master=1000; bus.mdr=7; bus.reset()`: master==0, mdr==0, nmitimen==0, `wram.read(5)==1`, `wram.addr==0`, ppu position (0,0)."),
         ("reset_keeps_cart_sram", "ROM avec SRAM (`make_rom_with(&[],LoRom,15,3,0x02,1)`): `bus.cart.write(0x70,0,0x5A)` puis `reset()`: `bus.cart.read(0x70,0)==Some(0x5A)`."),
         ("take_vblank_initial", "`bus.vblank_count()==0` et `take_vblank_flag()==false` deux fois de suite (le flag n'est leve que par le temps: voir E03-29)."),
         ("hash_equal", "deux `mk(&[0xEA])` ont le meme hash (`Fnv64::new()` puis `hash_into`) ; apres `bus.wram.write(0,1)` le hash differe ; apres `bus.master = 5` il differe."),
         ("pal_region", "`SysBus::new(cart, Region::Pal)`: `ppu.last_line()==311`.")])

T("E03-27", "bus/route.rs: read_inner (routage des lectures, open bus)",
  size=140,
  context="""Table de routage des LECTURES (spec 09 section 6). `read_inner(addr)` ne fait avancer aucun temps ni ne modifie `mdr` (c'est le role de `Bus::read`, E03-29).
Ajoute `mod route;` a `bus/mod.rs`. """ + CHAIN,
  embed=["09_SYSTEM.md#6. Open bus et routage (`read_inner` / `write_inner`)", "02_BUS_TIMING.md#4. Open bus"],
  write=[SRC + "/bus/mod.rs", SRC + "/bus/route.rs", TST + "/e03_27_read_inner.rs"],
  spec="""Dans `bus/route.rs`: `impl SysBus { pub fn read_inner(&mut self, addr: u32) -> u8 { ... } }` (declare `mod route;` dans `bus/mod.rs`).
`bank = (addr >> 16) as u8`, `a = (addr & 0xFFFF) as u16`.
- Si `bank & 0x7F <= 0x3F` (banques 00-3F et 80-BF), selon `a`:
  `0x0000..=0x1FFF` -> `wram.read(a as u32)` ;
  `0x2100..=0x213F` -> `ppu.read((a - 0x2100) as u8, self.mdr, self.mmio.wrio & 0x80 != 0)` ;
  `0x2140..=0x217F` -> `apu.port_read((a & 3) as usize)` ;
  `0x2180` -> `wram.data_read()` ;
  `0x4016` -> `(self.mdr & 0xFC) | joy.read_port(0)` ; `0x4017` -> `(self.mdr & 0xE0) | 0x1C | joy.read_port(1)` ;
  `0x4210..=0x421F` -> `mmio.read(a, self.mdr, self.master, HvState { vblank: ppu.in_vblank(), hblank: ppu.in_hblank(), autoread_busy: timing.autoread_busy(self.master) })` ;
  `0x4300..=0x437F` -> `dma.read(a).unwrap_or(self.mdr)` ;
  `0x6000..=0xFFFF` -> `cart.read(bank, a).unwrap_or(self.mdr)` ;
  tout autre `a` -> `self.mdr`.
- Sinon si `bank == 0x7E || bank == 0x7F`: `wram.read((((bank & 1) as u32) << 16) | a as u32)`.
- Sinon: `cart.read(bank, a).unwrap_or(self.mdr)`.
""" + MK,
  tests=[("wram_mirror", "`bus.wram.write(0x10,0xAB)`: `read_inner(0x000010)`, `read_inner(0x3F0010)`, `read_inner(0x800010)`, `read_inner(0xBF0010)`, `read_inner(0x7E0010)` valent tous 0xAB. `bus.wram.write(0x10010,0x77)`: `read_inner(0x7F0010)==0x77`. `read_inner(0x001FFF)==0x55`. `read_inner(0x7E2000)==0x55`."),
         ("rom_banks", "`mk(&[0xA9,0x01])`: `read_inner(0x008000)==0xA9`, `0x808000 -> 0xA9`, `0x008001 -> 0x01`, `0x00FFFC -> 0x00`, `0x00FFFD -> 0x80`, `0x400000 -> 0xA9`, `0xC00000 -> 0xA9`."),
         ("wmdata_port", "`bus.wram.write(0x200,0x11); write(0x201,0x22); wram.set_addr_l(0); wram.set_addr_m(2)`: `read_inner(0x002180)==0x11`, puis `0x22`, et `wram.addr==0x202`."),
         ("joyser", "mdr=0xFF (champ pub): `read_inner(0x004016)==0xFD` ; `read_inner(0x004017)==0xFD` (0xE0|0x1C|1). mdr=0x00: `0x004016 -> 0x01`, `0x004017 -> 0x1D`."),
         ("joyser_serial", "`bus.joy.set_buttons(0,0x8000)`; `bus.joy.write_out(true); write_out(false)`; mdr=0: 16 lectures de 0x004016 donnent 1,0,0,...(15 zeros) ; la 17eme donne 1."),
         ("rdnmi_hvbjoy", "`bus.mmio.rdnmi_flag=true`, mdr=0: `read_inner(0x004210)==0x82`, puis `0x02`. Au power-on, mdr=0: `read_inner(0x004212)==0x40` (HBlank car dot 0, pas de VBlank, pas d'auto-read)."),
         ("apu_ports", "`bus.apu.debug_set_from_apu(1,0x66)`: `read_inner(0x002141)==0x66`, `0x002145 -> 0x66`, `0x802141 -> 0x66` ; `read_inner(0x002140)==0x00`."),
         ("ppu_open_bus", "mdr=0x5A: `read_inner(0x002137)==0x5A` (le PPU de E03 renvoie le mdr) ; `read_inner(0x00213F)==0x5A`."),
         ("dma_regs", "`bus.dma.write(0x4300,0x12)`: `read_inner(0x004300)==0x12` ; `read_inner(0x004372)==0xFF` (power-on) ; mdr=0x99: `read_inner(0x00430C)==0x99` (registre non mappe)."),
         ("open_bus_zones", "mdr=0x33: `read_inner` de 0x002000, 0x0020FF, 0x002184, 0x0021FF, 0x003000, 0x004000, 0x004015, 0x004018, 0x0041FF, 0x004200, 0x00420D, 0x004220, 0x004380, 0x005FFF vaut 0x33 ; `read_inner(0x007FFF)` (LoROM, pas de SRAM): 0x33."),
         ("math_regs_via_bus", "`bus.mmio.write(0x4202,7,0); bus.mmio.write(0x4203,9,0); bus.master=100`: `read_inner(0x004216)==0x3F`, `read_inner(0x004217)==0x00`.")])

T("E03-28", "bus/route.rs: write_inner (routage des ecritures et effets)",
  size=130,
  context="Table de routage des ECRITURES. Les effets de `Mmio::write` sont appliques ici (MDMAEN -> `pending_dma`). " + CHAIN,
  embed=["09_SYSTEM.md#6. Open bus et routage (`read_inner` / `write_inner`)"],
  write=[SRC + "/bus/route.rs", TST + "/e03_28_write_inner.rs"],
  spec="""Dans `bus/route.rs`: `impl SysBus { pub fn write_inner(&mut self, addr: u32, v: u8) { ... } }` (ne touche pas `mdr`, n'avance pas le temps).
- Banques 00-3F / 80-BF (`bank & 0x7F <= 0x3F`), selon `a`:
  `0x0000..=0x1FFF` -> `wram.write(a as u32, v)` ; `0x2100..=0x213F` -> `ppu.write((a - 0x2100) as u8, v)` ; `0x2140..=0x217F` -> `apu.port_write((a & 3) as usize, v)` ;
  `0x2180` -> `wram.data_write(v)` ; `0x2181` -> `wram.set_addr_l(v)` ; `0x2182` -> `set_addr_m` ; `0x2183` -> `set_addr_h` ;
  `0x4016` -> `joy.write_out(v & 1 != 0)` ;
  `0x4200..=0x420D` -> `match mmio.write(a, v, self.master) { Mdmaen(m) => { self.pending_dma = m; self.dma_wait = true; } Hdmaen(_) => {} Memsel(b) => self.memsel = b, Wrio{old,new} => self.ppu.wrio_write(old, new), None => {} }` ;
  `0x4300..=0x437F` -> `dma.write(a, v)` ;
  `0x6000..=0xFFFF` -> `cart.write(bank, a, v)` (resultat ignore) ; autre: rien.
- Banques 7E/7F: `wram.write(((bank & 1) << 16) | a, v)`.
- Autres banques: `cart.write(bank, a, v)`.
(`dma_wait` sert a E03-32: le DMA demarre apres le cycle SUIVANT l'ecriture.)""" + " " + MK,
  tests=[("wram_writes", "`write_inner(0x7E0020,0x12)` -> `wram.read(0x20)==0x12` ; `write_inner(0x000020,0x34)` -> 0x34 ; `write_inner(0x3F0021,0x56)` -> `wram.read(0x21)==0x56` ; `write_inner(0x7F0000,5)` -> `wram.read(0x10000)==5` ; `write_inner(0x7FFFFF,9)` -> `wram.read(0x1FFFF)==9`."),
         ("wmadd_port", "`write_inner(0x002181,0x34); 0x002182 <- 0x12; 0x002183 <- 0x01; 0x002180 <- 0xCC`: `wram.read(0x11234)==0xCC` et `wram.addr==0x11235`."),
         ("nmitimen_effects", "`write_inner(0x004200,0x81)` -> `mmio.nmitimen==0x81` ; `write_inner(0x00420C,0x05)` -> `mmio.hdmaen==5` ; `write_inner(0x00420D,0x01)` -> `memsel==true` ; `write_inner(0x00420D,0x00)` -> false."),
         ("mdmaen_pending", "`write_inner(0x00420B,0x05)` -> `pending_dma()==5` ; `write_inner(0x00420B,0x00)` -> `pending_dma()==0`."),
         ("wrio", "`write_inner(0x004201,0x7F)` -> `mmio.wrio==0x7F`."),
         ("joyout", "`bus.joy.set_buttons(0,0x8000)`; `write_inner(0x004016,1)`: `read_inner(0x004016)&1` vaut 1 trois fois de suite (out0 vrai) ; `write_inner(0x004016,0)` puis lecture suivante: bit 15 (1) puis les lectures suivantes donnent 0."),
         ("dma_regs", "`write_inner(0x004300,0x01)` -> `dma.ch[0].dmap==1` ; `write_inner(0x004372,0x34)` -> `dma.ch[7].a1t & 0xFF==0x34` (partie basse)."),
         ("ppu_apu", "`write_inner(0x002100,0x0F)` -> `ppu.forced_blank()==false` ; `write_inner(0x002100,0x80)` -> true ; `write_inner(0x002142,0x09)` -> `apu.cpu_to_apu(2)==9` ; `write_inner(0x002146,0x07)` -> `apu.cpu_to_apu(2)==7`."),
         ("rom_and_sram", "ROM sans SRAM: `write_inner(0x008000,0x77)`: `read_inner(0x008000)` inchange (0xEA). ROM avec SRAM (`make_rom_with(&[0xEA],LoRom,15,3,0x02,1)`): `write_inner(0x700000,0x5A)` -> `cart.read(0x70,0)==Some(0x5A)` et `read_inner(0x700000)==0x5A`."),
         ("ignored_zones", "`write_inner` sur 0x002000, 0x0021FF, 0x004100, 0x004210, 0x004380, 0x005000 ne panique pas et ne change pas le hash du bus (comparer `hash_into` avant/apres)."),
         ("unknown_mmio_reg", "`write_inner(0x00420E,0xFF)` et `write_inner(0x00420F,0xFF)` ne changent rien (hash identique).")])

T("E03-29", "bus/cycle.rs: avance du temps, catch_up et impl cpu::Bus",
  size=140,
  context="""Flux de temps (spec 09 section 2): chaque acces CPU fait avancer `master` puis rattrape le PPU dot par dot (`catch_up`); `Timing::on_dot` produit les evenements.
Cette tache ne gere NI le rafraichissement, NI DMA, NI HDMA (E03-30..33): `begin_cycle`/`end_cycle` sont des fonctions vides a remplir. Ajoute `mod cycle;` a `bus/mod.rs`. """ + CHAIN,
  embed=["09_SYSTEM.md#2. Flux de temps", "09_SYSTEM.md#2.1 Cycle CPU (impl Bus pour SysBus)", "02_BUS_TIMING.md#3.4 Avance du temps par le bus"],
  write=[SRC + "/bus/mod.rs", SRC + "/bus/cycle.rs", TST + "/e03_29_bus_cycle.rs"],
  spec="""Dans `bus/cycle.rs`:
```rust
impl SysBus {
    pub fn advance(&mut self, clocks: u32) { self.master += clocks as u64; self.catch_up(); }
    pub fn catch_up(&mut self) {
        while self.ppu_t < self.master {
            let c = self.ppu.dot_clocks() as u64;
            self.ppu.step_dot();
            self.ppu_t += c;
            let out = self.timing.on_dot(&mut self.mmio, &mut self.joy, &self.ppu, self.ppu_t);
            if out.hdma_init { self.pending_hdma_init = true; }
            if out.hdma_run { self.pending_hdma_run = true; }
            if out.vblank_start { self.vblank_flag = true; self.vblank_count += 1; }
        }
        self.apu.step_to(self.master);
    }
    fn begin_cycle(&mut self) {}      // E03-30, E03-33
    fn end_cycle(&mut self) {}        // E03-32
}
impl crate::cpu::Bus for SysBus {
    fn read(&mut self, addr: u32) -> u8 { let addr = addr & 0xFF_FFFF; self.begin_cycle(); self.advance(access_clocks(addr, self.memsel)); let v = self.read_inner(addr); self.mdr = v; self.end_cycle(); v }
    fn write(&mut self, addr: u32, v: u8) { let addr = addr & 0xFF_FFFF; self.begin_cycle(); self.advance(access_clocks(addr, self.memsel)); self.mdr = v; self.write_inner(addr, v); self.end_cycle(); }
    fn idle(&mut self) { self.begin_cycle(); self.advance(6); self.end_cycle(); }
    fn nmi_pending(&mut self) -> bool { self.timing.take_nmi_edge() }
    fn irq_asserted(&mut self) -> bool { self.mmio.irq_line }
}
```
Les tests importent le trait: `use puce16_core::cpu::Bus;`. Rappel de la regle d'ordonnancement: un dot s'execute quand `master > debut du dot` (STRICT).""" + " " + MK,
  tests=[("idle_time", "`Bus::idle(&mut bus)` -> master==6 et `ppu.dot()==2` (dots 0 et 1 executes) ; un 2eme idle -> master==12, `ppu.dot()==3`."),
         ("read_costs", "`Bus::read(&mut bus, 0x008000)==0xA9` (ROM `mk(&[0xA9])`) -> master==8, mdr==0xA9 ; ensuite `read(0x002100)` -> master==14 ; `read(0x004016)` -> 26 ; `read(0x000000)` -> 34."),
         ("write_costs_and_mdr", "`Bus::write(&mut bus, 0x000010, 0x44)` -> `wram.read(0x10)==0x44`, mdr==0x44, master==8 ; `Bus::write(&mut bus, 0x002100, 0x80)` -> master==14."),
         ("memsel_fast", "`bus.memsel=true`: `Bus::read(&mut bus, 0x808000)` -> master==6 ; `Bus::read(&mut bus, 0x008000)` -> +8."),
         ("addr_masked", "`Bus::read(&mut bus, 0x01008000)` se comporte comme `0x008000` (masque 24 bits)."),
         ("line_boundary", "`bus.advance(1360)`: `ppu.v()==0` et `ppu.dot()==339` (le dot 339 commence a 1360, il n'est pas encore execute) ; `bus.advance(1)` de plus : `ppu.v()==1` et `ppu.dot()==0` (le dot 339 est execute des que master > 1360). Sur un autre bus neuf: `advance(1364)` d'un coup donne aussi (1,0)."),
         ("vblank_flag_strict", "`bus.advance(306_904)`: `take_vblank_flag()==false` (le dot (225,1) commence a 306904, il n'est pas encore execute) ; `bus.advance(1)`: `take_vblank_flag()==true` puis false ; `vblank_count()==1` et `mmio.rdnmi_flag==true`."),
         ("nmi_pending_consumed", "`bus.mmio.nmitimen=0x80`; `advance(306_904)`: `Bus::nmi_pending(&mut bus)==false` ; `advance(1)`: `nmi_pending()==true` ; 2eme appel: false."),
         ("irq_level", "`bus.mmio.irq_line=true`: `Bus::irq_asserted(&mut bus)==true` deux fois de suite (niveau, non consomme) ; `irq_line=false` -> false."),
         ("frame_length", "`bus.advance(357_368)`: `ppu.v()==0`, `ppu.dot()==0`, `ppu.field()==true`, `vblank_count()==1`."),
         ("time_is_monotonic", "60 appels `Bus::read(&mut bus, 0x008000)`: master==480 exactement (480 < 538: aucun rafraichissement DRAM meme apres E03-30) ; `ppu.v()==0`, `ppu.dot()==120`.")])

T("E03-30", "bus/cycle.rs: rafraichissement DRAM (40 clocks par ligne)",
  size=60,
  context="Le CPU est arrete 40 clocks par ligne a hclock >= 538 (`UNKNOWN - to confirm`, constante `DRAM_REFRESH_HCLOCK`). " + CHAIN,
  embed=["02_BUS_TIMING.md#3.3 Rafraichissement DRAM", "09_SYSTEM.md#2.1 Cycle CPU (impl Bus pour SysBus)"],
  write=[SRC + "/bus/cycle.rs", TST + "/e03_30_refresh.rs"],
  spec="""Dans `begin_cycle`: `if self.timing.refresh_due(self.master) { self.timing.mark_refreshed(); self.advance(DRAM_REFRESH_CLOCKS); }` avec `use crate::consts::DRAM_REFRESH_CLOCKS` (=40).
IMPORTANT: marquer AVANT d'avancer: `advance` peut franchir la fin de ligne, et `on_dot` (dot 0) remet alors le drapeau a faux; marquer apres l'avance annulerait ce reset.
""" + " " + MK,
  tests=[("refresh_once_per_line", "`bus.advance(536)` ; `Bus::idle` (hclock 536 < 538: pas de refresh) -> master==542 ; 2eme `idle` : refresh (hclock 542 >= 538): +40 +6 -> master==588 ; 3eme `idle` -> 594 (une seule fois)."),
         ("refresh_next_line", "apres le test precedent: `bus.advance(1364 + 538 - bus.master)` (master==1902) ; `Bus::idle` -> refresh a nouveau: master==1902+40+6==1948."),
         ("no_refresh_early", "sur la ligne 0 depuis master 0: 80 `idle` (480 clocks) -> master==480 exactement."),
         ("refresh_crossing_line", "bus neuf, aucun cycle encore: `bus.advance(1360)` (hclock 1360 sur la ligne 0, refresh de la ligne 0 non consomme) ; `Bus::idle` : refresh (+40 -> 1400, franchit la fin de ligne 1364) puis +6 -> master==1406 ; `ppu.v()==1`. Le refresh a ete marque AVANT l'avance donc `on_dot` (dot 0 de la ligne 1) a remis le drapeau a faux : `bus.timing.refresh_due(bus.master)` est FAUX car hclock vaut 42 (<538), et apres `bus.advance(538 - 42)` (hclock 538) il est VRAI."),
         ("read_triggers_refresh", "`bus.advance(540)` ; `Bus::read(&mut bus, 0x008000)` -> master==540+40+8==588."),
         ("hundred_reads_with_refresh", "100 lectures ROM depuis master 0: master==840 (les 68 premieres lectures finissent a 544 ; la 69eme commence a 544 >= 538 -> +40 ; au total 800 + 40) ; `ppu.v()==0`.")])

T("E03-31", "bus/mod.rs: impl DmaBus pour SysBus",
  size=90,
  context="""Le moteur DMA acces au bus via `DmaBus`: l'A-bus ne peut pas viser les registres systeme (zones ignorees, lecture = open bus), le B-bus vise `$2100 + reg`.
Le temps n'est PAS avance par ces methodes (sauf `elapse`). """ + CHAIN,
  embed=["03_DMA.md#3. DMA general (MDMAEN $420B)", "09_SYSTEM.md#3.7 Dma (`dma/mod.rs`, `dma/hdma.rs`)"],
  write=[SRC + "/bus/mod.rs", TST + "/e03_31_dmabus_impl.rs"],
  spec="""Dans `bus/mod.rs` (ou un sous-module): `impl crate::dma::DmaBus for SysBus`:
- `a_read(addr)`: `bank = (addr >> 16) as u8; a = addr as u16`. Si `bank & 0x7F <= 0x3F` ET `a` dans `0x2100..=0x21FF | 0x4000..=0x41FF | 0x4200..=0x421F | 0x4300..=0x437F`: renvoie `self.mdr` (sans modifier rien). Sinon `v = self.read_inner(addr); self.mdr = v; v`.
- `a_write(addr, v)`: meme zone interdite -> ne fait rien ; sinon `self.mdr = v; self.write_inner(addr, v)`.
- `b_read(reg)`: `v = self.read_inner(0x00_2100 | reg as u32); self.mdr = v; v`. `b_write(reg, v)`: `self.mdr = v; self.write_inner(0x00_2100 | reg as u32, v)`.
- `elapse(clocks)`: `self.advance(clocks)`. `master()`: `self.master`.""" + " " + MK,
  tests=[("a_read_ok_zones", "`bus.wram.write(0x30,0x9A)`: `a_read(0x7E0030)==0x9A`, `a_read(0x000030)==0x9A` ; ROM `mk(&[0xA9])`: `a_read(0x008000)==0xA9` ; `bus.mdr` vaut alors 0xA9."),
         ("a_read_forbidden", "mdr=0x77: `a_read` de 0x002100, 0x0021FF, 0x004000, 0x0041FF, 0x004200, 0x00421F, 0x004300, 0x00437F, 0x802100 vaut 0x77 ; `a_read(0x004016)` aussi (zone 4000-41FF) ; `a_read(0x002000)` (hors liste, 'open bus' via read_inner) == mdr."),
         ("a_write", "`a_write(0x7E0040,9)` -> `wram.read(0x40)==9` ; `a_write(0x002100,0x00)` ignore (`ppu.forced_blank()` reste true) ; `a_write(0x00420B,0x01)` ignore (`pending_dma()==0`) ; `a_write(0x004300,5)` ignore (`dma.ch[0].dmap==0xFF`)."),
         ("b_bus", "`b_write(0x80,0xAB)`: `wram.read(0)==0xAB`, `wram.addr==1` ; `b_read(0x80)` lit l'octet a l'adresse 1 et incremente ; `b_write(0x00,0x00)` -> `ppu.forced_blank()==false` ; `b_write(0x42,7)` -> `apu.cpu_to_apu(2)==7` ; `b_write(0xFF,1)` ne panique pas."),
         ("b_read_updates_mdr", "`bus.apu.debug_set_from_apu(3,0x5E)`: `b_read(0x43)==0x5E` et `bus.mdr==0x5E`."),
         ("no_time_except_elapse", "apres `a_read`, `a_write`, `b_read`, `b_write`: `bus.master==0` ; `elapse(10)` -> `master()==10` ; `elapse(1364)` fait avancer le PPU (`ppu.v()==1`)."),
         ("dma_through_trait", "utilise `Dma::new()` + `run_general(&mut bus_ref, 1)` en retirant le DMA du bus (`let mut d = std::mem::take(&mut bus.dma); ...`): ROM `mk` avec donnees [1,2,3] a 0x8000 (code = ces 3 octets), canal 0: dmap=0, bbad=0x80, a1b=0, a1t=0x8000, das=3, `wram.addr=0x100`: apres, `wram.read(0x100..0x103)==[1,2,3]` et `bus.master==8+8+24==40`.")])

T("E03-32", "bus/cycle.rs: execution du DMA general (apres le cycle suivant MDMAEN)",
  size=70,
  context="""Le DMA demarre APRES le cycle qui suit l'ecriture de MDMAEN (en pratique apres la lecture de l'opcode suivant, spec 03 section 3.1).
`write_inner` a deja pose `pending_dma = v` et `dma_wait = true`. """ + CHAIN,
  embed=["09_SYSTEM.md#2.1 Cycle CPU (impl Bus pour SysBus)", "09_SYSTEM.md#4. DMA general: temps (decisions provisoires, spec 03 section 3.1)"],
  write=[SRC + "/bus/cycle.rs", TST + "/e03_32_general_dma.rs"],
  spec="""Dans `end_cycle`:
```rust
if self.pending_dma != 0 {
    if self.dma_wait { self.dma_wait = false; }                 // cycle de l'ecriture MDMAEN: on arme seulement
    else {
        let m = self.pending_dma; self.pending_dma = 0;
        let mut d = std::mem::take(&mut self.dma);
        d.run_general(self, m);                                 // utilise l'impl DmaBus de SysBus
        self.dma = d;
    }
}
```
Pendant `run_general`, `self.dma` est un `Dma::default()` temporaire: c'est voulu (le moteur travaille sur `d`). Les ecritures CPU ne peuvent pas avoir lieu pendant ce temps.""" + " " + MK,
  tests=[("dma_after_next_cycle", "ROM: `mk(...)` dont le code est l'identite d'octets: construis la ROM par `make_rom(&data_block, LoRom)` avec data_block[0x200..0x204]=[0x11,0x22,0x33,0x44] (vec de 0x210 octets, donnees a l'offset 0x200 -> adresse 0x8200). `bus.write_inner(0x002181,0x00)`, `0x002182 <- 0x02` (WMADD=0x200). Canal 0 via `bus.dma.write`: 0x4300<-0x00, 0x4301<-0x80, 0x4302<-0x00, 0x4303<-0x82, 0x4304<-0x00, 0x4305<-0x04, 0x4306<-0x00. `Bus::write(&mut bus, 0x00420B, 0x01)` : master==6, `bus.pending_dma()==1`, `wram.read(0x200)==0x55` (rien copie encore)."),
         ("dma_runs_on_next_cycle", "suite: `Bus::read(&mut bus, 0x008000)`: apres cet appel `pending_dma()==0`, `wram.read(0x200..0x204)==[0x11,0x22,0x33,0x44]`, `wram.addr==0x204`, `dma.ch[0].das==0`, `dma.ch[0].a1t==0x8204`, et `bus.master==64` (6 + 8 -> 14 ; alignement +2 -> 16 ; +8 depart -> 24 ; +8 canal -> 32 ; 4 octets x 8 -> 64). `ppu.dot()==16`."),
         ("dma_regs_restored", "apres le DMA, `bus.dma` est bien le vrai objet (`bus.dma.ch[0].dmap==0x00` ecrit par le test, `bus.dma.ch[3].dmap==0xFF`)."),
         ("mdmaen_zero", "`Bus::write(&mut bus, 0x00420B, 0x00)` puis une lecture : aucun transfert, master==6+8."),
         ("two_channels_order", "mask 0x06 (canaux 1 et 2) : les canaux s'executent dans l'ordre 1 puis 2 : canal 1 copie 2 octets vers WMADD=0x300, canal 2 copie 2 octets (WMADD continue) ; WRAM[0x300..0x304] = les 4 octets dans l'ordre."),
         ("no_double_run", "apres le DMA, une 2eme lecture ne relance rien: `master` n'augmente que de 8."),
         ("write_then_idle", "meme sequence avec `Bus::idle` comme cycle suivant : le DMA part aussi apres l'idle (end_cycle de l'idle)."),
         ("rewrite_before_start", "deux ecritures MDMAEN successives (0x01 puis 0x02) avant le 1er cycle suivant: seul le canal 1 (valeur 0x02) s'execute ; mais `dma_wait` est re-arme par la 2eme ecriture : le DMA part apres le cycle suivant la 2eme.")])

T("E03-33", "bus/cycle.rs: execution HDMA (init et run) en debut de cycle",
  size=80,
  context="HDMA est declenche par `Timing` (flags `pending_hdma_*`) et execute au debut du cycle CPU suivant, avant le rafraichissement? NON: apres le rafraichissement (spec 09 section 2.1). " + CHAIN,
  embed=["09_SYSTEM.md#5. HDMA: declenchement", "09_SYSTEM.md#2.1 Cycle CPU (impl Bus pour SysBus)"],
  write=[SRC + "/bus/cycle.rs", TST + "/e03_33_hdma_exec.rs"],
  spec="""Dans `begin_cycle`, APRES le bloc de rafraichissement de E03-30:
```rust
if self.pending_hdma_init || self.pending_hdma_run {
    let (init, run) = (self.pending_hdma_init, self.pending_hdma_run);
    self.pending_hdma_init = false; self.pending_hdma_run = false;
    let m = self.mmio.hdmaen;
    let mut d = std::mem::take(&mut self.dma);
    if init { d.hdma_init(self, m); }
    if run  { d.hdma_run_line(self, m); }
    self.dma = d;
}
```
(L'avance du temps faite par HDMA peut redeclencher des flags; ils seront traites au cycle suivant.)""" + " " + MK,
  tests=[("init_after_dot3", "ROM de test avec table a 0x8300 = [0x01, 0xA1, 0x00] (vec de 0x310 octets). `bus.write_inner(0x00420C,0x01)` (hdmaen=1) ; WMADD=0x300 (`write_inner` 0x2181/0x2182) ; canal 0: dmap=0x00, bbad=0x80, a1t=0x8300, a1b=0 (via `bus.dma.write`). `bus.advance(20)` (position (0,5)): `hdma_state(0)` encore `(0xFFFF,0,false,true)` (rien n'a ete execute: HDMA ne part qu'en debut de cycle). `Bus::idle`: `bus.dma.hdma_state(0)==(0x8301, 1, true, false)` et `bus.master==20+26+6==52`."),
         ("run_line_writes", "suite: `bus.advance(1101 - bus.master)` (master==1101, position (0,276), flag run pose) puis `Bus::idle` : d'abord le rafraichissement de la ligne 0 (hclock 1101 >= 538, jamais consomme: +40 -> 1141), puis HDMA run (18+8+8 = 34 -> 1175), puis le cycle idle (+6) : `bus.master==1181` ; `wram.read(0x300)==0xA1`, `wram.addr==0x301`, `hdma_state(0).3==true` (l'octet suivant 0x00 termine la table)."),
         ("done_channel_no_more", "ensuite on avance jusqu'apres le dot 276 de la ligne 1 (`bus.advance(1364 + 1101 - bus.master)`) puis `Bus::idle` : rien n'est ecrit (`wram.addr` toujours 0x301, `hdma_state(0).3` toujours vrai) : le flag run est consomme mais aucun canal n'est actif, donc aucun surcout HDMA (master n'augmente que du refresh 40 + idle 6)."),
         ("hdmaen_zero", "sans `write_inner(0x00420C,..)`: sur une frame entiere (`advance(357_368)` en plusieurs `idle`), jamais d'ecriture WRAM par HDMA."),
         ("init_each_frame", "avec hdmaen=1: a la frame suivante (apres `advance` jusqu'a (1 frame + 5 dots)) et un `idle`, le canal est re-initialise (`hdma_state(0).3==false`)."),
         ("mark_refreshed_skips_refresh", "meme scenario que `run_line_writes` mais avec `bus.timing.mark_refreshed()` appele juste avant le `idle` : `bus.master==1101+34+6==1141` (pas de rafraichissement).")])

T("E03-34", "snes.rs: Snes reel (new_with, reset) a cote du motif de test",
  size=140,
  context="""Le `Snes` du motif de test (E01-06) devient un enum interne `Inner { Pattern, Real }`. L'API publique de E01-06 ne change pas.
La sortie video d'un `Snes` reel est un cadre NOIR 256x224 jusqu'au rendu PPU (E05). """ + CHAIN,
  embed=["01_ARCHITECTURE.md#5. API publique de `puce16-core` (stable, ne pas changer sans question)", "09_SYSTEM.md#7. Boucle d'une frame"],
  write=[SRC + "/snes.rs", TST + "/e03_34_snes_new.rs", TST + "/e01_06_snes_test_pattern.rs"],
  spec="""Dans `snes.rs`:
```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum LoadError { Cart(CartError) }
impl From<CartError> for LoadError { ... }
pub struct Snes { region: Region, frame: u64, joy: [u16; 2], inner: Inner }
enum Inner { Pattern { fb: FrameBuf, compact: Vec<u32> }, Real { cpu: Cpu, sys: Box<SysBus>, fb: FrameBuf, compact: Vec<u32> } }
impl Snes {
    pub fn new_test_pattern(region: Region) -> Snes;                 // inchange (E01-06)
    pub fn new(rom: &[u8], region: Option<Region>) -> Result<Snes, LoadError>;           // = new_with(rom, region, None)
    pub fn new_with(rom: &[u8], region: Option<Region>, map: Option<MapMode>) -> Result<Snes, LoadError>;
    pub fn reset(&mut self);
    pub fn region(&self) -> Region;
}
```
`new_with`: `cart = Cart::from_bytes_forced(rom, map, region)?` ; `region = cart.region()` ; `sys = Box::new(SysBus::new(cart, region))` ; `cpu = Cpu::new()` ; `cpu.reset(&mut *sys)` (lit le vecteur RESET et charge PC) ; `fb = FrameBuf::new()` ; `compact = fb.compact()`. `frame` = 0 ; `joy` = [0;2].
`reset`: motif de test: `frame = 0`, tampon noir et re-dessin comme a la creation. Reel: `sys.reset(); cpu.reset(&mut *sys)`; `frame = 0`; `joy` conserve.
Toutes les autres methodes de E01-06 (`run_frame`, `frame`, `frame_count`, `set_joypad`, `joypad`, `take_audio`, `state_hash`, `region`) doivent continuer a fonctionner pour le motif; pour le reel elles sont rendues en E03-35 (ici: `unimplemented!()` autorise pour le variant `Real`).""",
  tests=[("new_ok", "`Snes::new(&make_rom(&[0xEA],Map::LoRom), None).is_ok()` ; `region()==Ntsc` (pays 1)."),
         ("region_from_header", "`make_rom_with(&[0xEA],Map::LoRom,15,0,0,2)` (pays 2) -> `region()==Pal` ; avec `Some(Region::Ntsc)` -> Ntsc ; `make_rom` (pays 1) avec `Some(Region::Pal)` -> Pal."),
         ("errors", "`Snes::new(&[0u8;100], None).err()==Some(LoadError::Cart(CartError::TooSmall))` ; ROM coprocesseur (chipset 0x13) -> `Some(LoadError::Cart(CartError::UnsupportedCoprocessor(\"GSU\")))`."),
         ("forced_map", "`Snes::new_with(&make_rom(&[0xEA],Map::LoRom), None, Some(MapMode::HiRom)).is_err()` (en-tete HiRom absent dans 32 Kio)."),
         ("pattern_still_works", "`Snes::new_test_pattern(Ntsc)`: apres `run_frame()` pixel (0,0) == 0xFF010040 (non-regression E01-06) ; `reset()` remet `frame_count()==0`."),
         ("existing_e01_06_tests", "le fichier `tests/e01_06_snes_test_pattern.rs` existant doit rester vert sans modification (n'y touche que si une signature a change).")])

T("E03-35", "snes.rs: run_frame / run_master / step_instruction / hash / SRAM / debug (Snes reel)",
  size=140,
  context="Complete les methodes de `Snes` pour le variant `Real` (spec 09 section 7 et API publique de la spec 01 section 5). " + CHAIN,
  embed=["09_SYSTEM.md#7. Boucle d'une frame", "01_ARCHITECTURE.md#5. API publique de `puce16-core` (stable, ne pas changer sans question)"],
  write=[SRC + "/snes.rs", TST + "/e03_35_snes_run.rs"],
  spec="""Ajoute/complete dans `snes.rs` (le variant `Pattern` garde son comportement):
```rust
#[derive(Clone, Debug, PartialEq, Eq)] pub struct DebugSnapshot { pub pc: u16, pub pb: u8, pub a: u16, pub x: u16, pub y: u16, pub s: u16, pub d: u16, pub db: u8, pub p: u8, pub e: bool, pub v: u16, pub dot: u16, pub master: u64, pub frame: u64 }
#[derive(Clone, Debug, PartialEq, Eq)] pub struct CartInfo { pub title: String, pub map: MapMode, pub region: Region, pub rom_len: usize, pub sram_len: usize, pub battery: bool }
impl Snes {
    pub fn run_frame(&mut self);                  // Reel: boucle `cpu.step(&mut *sys)` jusqu'a `sys.take_vblank_flag()`; garde-fou: apres 2_000_000 instructions sans VBlank -> panic!("run_frame: no vblank"); puis frame += 1
    pub fn run_master(&mut self, master_clocks: u64);   // Reel: end = sys.master + n ; while sys.master < end { cpu.step }
    pub fn step_instruction(&mut self);           // Reel: un cpu.step ; Motif: ne fait rien
    pub fn frame_count(&self) -> u64;             // Reel: sys.vblank_count() ; Motif: frame
    pub fn frame(&self) -> FrameView<'_>;         // Reel: tampon noir 256x224 (compact)
    pub fn set_joypad(&mut self, port: usize, buttons: u16);   // met a jour joy[] ET sys.joy.set_buttons (port 0/1; autre: ignore)
    pub fn state_hash(&self) -> u64;              // Reel: Fnv64: region (u8), frame (u64), cpu.hash_into, sys.hash_into
    pub fn sram(&self) -> Option<&[u8]>;          // Reel: Some(sram) si la SRAM n'est pas vide ; sinon None ; Motif: None
    pub fn load_sram(&mut self, data: &[u8]);
    pub fn debug(&self) -> DebugSnapshot;         // Reel: registres CPU + (ppu.v, ppu.dot) + master ; Motif: tous zeros sauf frame
    pub fn cart_info(&self) -> Option<CartInfo>;  // Reel uniquement
    pub fn master(&self) -> u64;                  // Reel: sys.master ; Motif: 0
    pub fn peek_wram(&self, idx: u32) -> u8;      // Reel: sys.wram.read(idx) ; Motif: 0
}
```
`pub use` : `DebugSnapshot`, `CartInfo`, `LoadError` exportes depuis `lib.rs`? `lib.rs` ne doit PAS etre modifie : ils sont accessibles par `puce16_core::snes::{DebugSnapshot, CartInfo, LoadError}`.
Programme de test (LoRom, code a 0x8000): `78 18 FB A9 42 8D 10 00 80 FE` = SEI ; CLC ; XCE ; LDA #$42 ; STA $0010 ; BRA -2.""",
  tests=[("run_frame_basic", "ROM `make_rom(&[0x78,0x18,0xFB,0xA9,0x42,0x8D,0x10,0x00,0x80,0xFE], LoRom)`: `run_frame()` : `peek_wram(0x10)==0x42`, `frame_count()==1`, `debug().pb==0`, `debug().pc==0x8008`, `debug().e==false` (XCE a passe en natif), `debug().v==225`, `debug().dot <= 20`."),
         ("frame_master_range", "apres le 1er `run_frame`: `306_905 <= master() && master() <= 307_000` ; apres le 2eme : `master() - m1` entre 357_268 et 357_468 ; apres le 3eme (frame avec ligne 240 courte) : delta entre 357_264 et 357_464."),
         ("run_master_exact", "`m0 = master()` (le reset du CPU a deja consomme quelques cycles) ; `run_master(1000)` : `master() >= m0 + 1000` et `master() < m0 + 1000 + 80` (la derniere instruction peut depasser) ; `run_master(0)` ne change pas `master()`."),
         ("step_instruction", "juste apres `new`, 3 `step_instruction()` executent SEI, CLC, XCE : `debug().e==false` apres le 3eme et `debug().pc==0x8003`."),
         ("state_hash_determinism", "deux `Snes::new` identiques : meme `state_hash` apres `run_frame()` x3 ; des hash differents entre 2 et 3 frames ; `run_master(1000)` des deux cotes : meme hash."),
         ("joypad_api", "`set_joypad(0,0x8000)`: `joypad(0)==0x8000`, `joypad(1)==0` ; `set_joypad(2,1)` ignore ; le hash change apres `set_joypad(0,1)`."),
         ("sram_option", "ROM sans SRAM : `sram()==None`. ROM `make_rom_with(&[0xEA],LoRom,15,3,0x02,1)` : `sram().unwrap().len()==8192`; `load_sram(&[1,2,3])` puis `sram().unwrap()[..4]==[1,2,3,0]`."),
         ("cart_info", "`cart_info().unwrap()` pour `make_rom` LoRom: title==\"PUCE16 TEST ROM\", map==LoRom, region==Ntsc, rom_len==32768, sram_len==0, battery==false. Pour `new_test_pattern`: `cart_info()==None`."),
         ("pattern_debug_default", "`Snes::new_test_pattern(Ntsc).debug()`: tous les champs 0 sauf `frame` (== frame_count)."),
         ("frame_view_real", "`frame().width==256`, `height==224`, `pixels.len()==256*224`, tous `0xFF000000`."),
         ("stp_still_reaches_vblank", "ROM `[0x78,0x18,0xFB,0xDB]` (SEI, CLC, XCE, STP) : STP arrete le CPU mais, selon 01b section 4, un CPU arrete execute des cycles `idle` qui font avancer le temps : `run_frame()` revient (VBlank atteint) et `frame_count()==1`, `debug().pc==0x8004`.")])
