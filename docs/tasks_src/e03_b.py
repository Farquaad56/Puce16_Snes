from _dsl import T

CORE = "crates/puce16-core"
SRC = CORE + "/src"
TST = CORE + "/tests"

CHAIN = ("Execution strictement sequentielle (D-020): cette tache suppose que toutes les precedentes sont terminees. "
         "Ne modifie QUE les fichiers de 'Write only in'.")

# ------------------------------------------------------------------ E03 part B: PPU timing skeleton, WRAM, joypad, APU skeleton, access_clocks (E03-07..13)

T("E03-07", "ppu/dot.rs: tables de dots et types de ligne",
  size=100, deps=["E03-06"],
  context="""Le PPU avance dot par dot. Cette tache ne contient que des fonctions pures sur les durees (en clocks maitres) des dots d'une ligne.
Decision D-008 (provisoire): 4 dots de 5 clocks aux H = 323, 324, 327, 328. """ + CHAIN,
  embed=["05_PPU.md#5.1 Longueur des lignes et des dots", "02_BUS_TIMING.md#3.1 Position dans la ligne"],
  write=[SRC + "/ppu/dot.rs", TST + "/e03_07_dot.rs"],
  spec="""Dans `ppu/dot.rs` (module deja declare par `ppu/mod.rs`):
```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum LineType { Normal, Short, Long }
pub fn line_type(region: Region, interlace: bool, field: bool, v: u16) -> LineType;
pub fn line_dots(lt: LineType) -> u16;                 // Long: 341 ; sinon 340
pub fn line_clocks(lt: LineType) -> u32;               // Normal 1364 ; Short 1360 ; Long 1368
pub fn dot_clocks(lt: LineType, dot: u16) -> u32;      // 5 si dot dans {323,324,327,328} et lt != Short ; sinon 4 (dot >= line_dots: panic)
pub fn dot_start_clock(lt: LineType, dot: u16) -> u32; // somme des dot_clocks de 0..dot ; accepte dot == line_dots(lt) (= fin de ligne)
```
`line_type`: `Short` si Ntsc ET `!interlace` ET `field` ET `v == 240` ; `Long` si Pal ET `interlace` ET `field` ET `v == 311` ; sinon `Normal`.
`dot_start_clock` peut se calculer en boucle (pas de table necessaire). Utilise les constantes de `crate::consts` (CLOCKS_PER_LINE, CLOCKS_SHORT_LINE, CLOCKS_LONG_LINE).""",
  tests=[("line_type", "`line_type(Ntsc,false,true,240)==Short` ; `(Ntsc,false,false,240)==Normal` ; `(Ntsc,true,true,240)==Normal` ; `(Ntsc,false,true,239)==Normal` ; `(Pal,true,true,311)==Long` ; `(Pal,true,false,311)==Normal` ; `(Pal,false,true,311)==Normal` ; `(Pal,false,true,240)==Normal`."),
         ("dots_clocks_sums", "`line_dots`: Normal 340, Short 340, Long 341. `line_clocks`: 1364, 1360, 1368. Pour chaque type, la somme des `dot_clocks(lt,d)` pour d in 0..line_dots(lt) == `line_clocks(lt)`."),
         ("five_clock_dots", "Normal: `dot_clocks` vaut 5 pour d in {323,324,327,328} et 4 pour d in {0,1,322,325,326,329,339}. Short: 4 partout. Long: 5 aux memes dots, `dot_clocks(Long,340)==4`."),
         ("start_clock_normal", "Normal: `dot_start_clock` de 0->0, 1->4, 22->88, 274->1096, 323->1292, 324->1297, 325->1302, 326->1306, 327->1310, 328->1315, 329->1320, 340->1364."),
         ("start_clock_short_long", "Short: 323->1292, 324->1296, 340->1360. Long: 323->1292, 329->1320, 340->1364, 341->1368."),
         ("visible_window", "`dot_start_clock(Normal,22)==88` (consts::VISIBLE_CLOCK_FIRST) et `dot_start_clock(Normal,278)==1112` (consts::VISIBLE_CLOCK_END)."),
         ("panic_oob", "`#[should_panic]`: `dot_clocks(Normal, 340)`.")])

T("E03-08", "ppu/mod.rs: Ppu (compteurs v/dot/field, step_dot, evenements)",
  size=140,
  context="""Squelette de timing du PPU (sans rendu: E04/E05). Les compteurs et les evenements suffisent a faire tourner NMI, IRQ, HDMA et auto-read.
Interface complete: spec 05 section 8; ce qui est fourni ici: spec 09 section 3.4. Interlace/overscan restent a `false` jusqu'a E03-09. """ + CHAIN,
  embed=["05_PPU.md#5.2 Frame", "05_PPU.md#5.3 Evenements par (v, dot)", "09_SYSTEM.md#3.4 Ppu"],
  write=[SRC + "/ppu/mod.rs", TST + "/e03_08_ppu_timing.rs"],
  spec="""Dans `ppu/mod.rs` (garde les `pub mod` existants; n'ecris pas encore le rendu):
```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)] pub struct DotEvents(pub u8);
impl DotEvents { pub const LINE_START: u8 = 1; pub const HBLANK_START: u8 = 2; pub const HBLANK_END: u8 = 4;
                 pub const VBLANK_START: u8 = 8; pub const VBLANK_END: u8 = 16;
                 pub fn has(self, m: u8) -> bool { self.0 & m != 0 } }
pub struct Ppu { region: Region, v: u16, dot: u16, field: bool, interlace: bool, overscan: bool, forced_blank: bool }
impl Ppu {
    pub fn new(region: Region) -> Ppu;     // v=0, dot=0, field=false, interlace=false, overscan=false, forced_blank=true
    pub fn reset(&mut self);               // meme etat que new (region conservee)
    pub fn v(&self) -> u16;  pub fn dot(&self) -> u16;  pub fn field(&self) -> bool;
    pub fn line_dots(&self) -> u16;        // dot::line_dots(type de la ligne courante)
    pub fn dot_clocks(&self) -> u32;       // duree du dot COURANT (avant step_dot)
    pub fn last_line(&self) -> u16;        // (Pal 311 / Ntsc 261) + (1 si interlace && !field)
    pub fn vblank_start(&self) -> u16;     // 240 si overscan sinon 225
    pub fn in_vblank(&self) -> bool;       // v >= vblank_start()
    pub fn in_hblank(&self) -> bool;       // dot >= 274 || dot < 1
    pub fn step_dot(&mut self) -> DotEvents;
}
```
`step_dot` (evenements calcules sur la position COURANTE, avant d'avancer):
1. `dot == 0`: si `v == 0` -> VBLANK_END ; si `v == vblank_start()` -> VBLANK_START ; toujours LINE_START.
2. `dot == 274` -> HBLANK_START ; `dot == 1` -> HBLANK_END.
3. Avancer: `dot += 1` ; si `dot == line_dots()` (calcule AVANT d'incrementer, avec la ligne courante): `dot = 0; v += 1;` et si `v > last_line()` : `v = 0; field = !field`.
Ordre important: `last_line()` est evalue avec le `field` de la frame qui se termine.""",
  tests=[("power_on", "`Ppu::new(Ntsc)`: v()==0, dot()==0, field()==false, dot_clocks()==4, line_dots()==340, last_line()==261, vblank_start()==225, in_vblank()==false, in_hblank()==true (dot 0 < 1). `Ppu::new(Pal).last_line()==311`."),
         ("dot_clocks_walk", "apres 323 `step_dot`: dot()==323 et dot_clocks()==5 ; apres 324: dot_clocks()==5 ; apres 325: 4 ; apres 327: 5 ; apres 329: 4."),
         ("line_sum", "somme des `dot_clocks()` relevee AVANT chaque `step_dot` sur 340 steps == 1364 ; ensuite v()==1 et dot()==0."),
         ("events_line0", "Ppu::new: 1er `step_dot()` renvoie `DotEvents(17)` (LINE_START|VBLANK_END) ; 2eme (dot 1) renvoie `DotEvents(4)` ; les steps des dots 2..=273 renvoient `DotEvents(0)` ; le step du dot 274 renvoie `DotEvents(2)`."),
         ("vblank_event", "apres exactement 225*340 steps: v()==225, dot()==0, in_vblank()==true ; le step suivant renvoie `DotEvents(9)` (LINE_START|VBLANK_START). Aucun step avant n'a renvoye VBLANK_START (bit 8)."),
         ("frame_ntsc_field0", "NTSC: somme des `dot_clocks()` sur exactement 262*340 steps == 357368 ; apres: v()==0, dot()==0, field()==true."),
         ("frame_ntsc_field1_short", "enchainement: apres la frame 0, les 262*340 steps suivants somment 357364 (ligne 240 courte de 1360 clocks) ; apres: v()==0, field()==false."),
         ("frame_pal", "PAL: frame 0 (field false): somme sur 312*340 steps == 425568 ; la frame 1 (field true, interlace off) somme aussi 425568 (pas de ligne longue sans interlace)."),
         ("vblank_end_event", "a la fin de la frame NTSC (apres 262*340 steps), le step suivant renvoie `DotEvents(17)` (v==0 de nouveau).")])

T("E03-09", "ppu/mod.rs: registres $2100 et $2133 (minimal), read, hash",
  size=100,
  context="""Seuls INIDISP (bit 7: forced blank) et SETINI (bit 0: interlace, bit 2: overscan) sont necessaires au timing. Le reste des registres est ecrit en E04.
La lecture d'un registre PPU renvoie pour l'instant `cpu_mdr` (open bus). """ + CHAIN,
  embed=["05_PPU.md#8. Interface Rust (ppu/mod.rs)", "05_PPU.md#5.2 Frame"],
  write=[SRC + "/ppu/mod.rs", TST + "/e03_09_ppu_regs.rs"],
  spec="""Ajoute a `impl Ppu`:
```rust
pub fn write(&mut self, reg: u8, v: u8);                         // reg = adresse - 0x2100
pub fn read(&mut self, reg: u8, cpu_mdr: u8, wrio7: bool) -> u8;   // E03: renvoie cpu_mdr
pub fn wrio_write(&mut self, old: u8, new: u8);                  // E03: ne fait rien
pub fn forced_blank(&self) -> bool;  pub fn interlace(&self) -> bool;  pub fn overscan(&self) -> bool;
pub fn hash_into(&self, h: &mut crate::hash::Fnv64);             // region (u8), v, dot (u16), field, interlace, overscan, forced_blank (u8 chacun)
```
`write`: `0x00` -> `forced_blank = v & 0x80 != 0` ; `0x33` -> `interlace = v & 1 != 0 ; overscan = v & 4 != 0` ; tout autre `reg` -> ignore. L'ecriture ne change PAS `v`, `dot`, `field`.""",
  tests=[("inidisp", "`Ppu::new`: forced_blank()==true ; `write(0x00,0x0F)` -> false ; `write(0x00,0x8F)` -> true."),
         ("setini_flags", "`write(0x33,0x05)`: interlace()==true, overscan()==true, vblank_start()==240 ; `write(0x33,0x00)` -> false/false, vblank_start()==225 ; `write(0x33,0x04)` -> overscan seul."),
         ("ignored_regs", "`write(0x01,0xFF)`, `write(0x2F,0xFF)`, `write(0x34,0xFF)` ne changent ni `forced_blank` ni interlace/overscan (verifie par hash identique avant/apres)."),
         ("read_open_bus", "`read(0x37,0xAB,true)==0xAB` ; `read(0x3F,0x12,false)==0x12`."),
         ("interlace_ntsc_frames", "NTSC, `write(0x33,0x01)` avant le debut: frame 0 (field false): last_line()==262, somme des dot_clocks sur 263*340 steps == 358732 ; ensuite v()==0, field()==true ; frame 1 (field true): last_line()==261, somme sur 262*340 steps == 357368 (pas de ligne courte en entrelace)."),
         ("interlace_pal_frames", "PAL, `write(0x33,0x01)`: frame 0: last_line()==312, somme sur 313*340 steps == 426932 ; frame 1 (field true): last_line()==311, somme sur 312*340 + 1 steps (la ligne 311 a 341 dots) == 425572."),
         ("overscan_vblank_event", "NTSC, `write(0x33,0x04)`: apres 240*340 steps v()==240, le step suivant renvoie VBLANK_START ; `in_vblank()` faux a v==239 et vrai a v==240."),
         ("hash", "deux `Ppu::new(Ntsc)` ont le meme hash (`Fnv64`) ; apres un `step_dot` les hash different ; `Ppu::new(Ntsc)` et `Ppu::new(Pal)` different.")])

T("E03-10", "bus/wram.rs: WRAM 128 Kio et registres $2180-$2183",
  size=90,
  context="WRAM (valeur de power-on 0x55, D-011) et port d'acces $2180 avec adresse WMADD de 17 bits. " + CHAIN,
  embed=["02_BUS_TIMING.md#5. WRAM et registres $2180-$2183", "09_SYSTEM.md#3.1 Wram"],
  write=[SRC + "/bus/wram.rs", TST + "/e03_10_wram.rs"],
  spec="""Dans `bus/wram.rs`:
```rust
pub struct Wram { mem: Vec<u8> /* 0x20000 octets a 0x55 */, pub addr: u32 }
impl Wram {
    pub fn new() -> Wram;  pub fn reset_addr(&mut self);                 // addr = 0 (la memoire est conservee)
    pub fn read(&self, idx: u32) -> u8;  pub fn write(&mut self, idx: u32, v: u8);     // idx & 0x1FFFF
    pub fn data_read(&mut self) -> u8;   pub fn data_write(&mut self, v: u8);          // $2180: acces a addr puis addr = (addr + 1) & 0x1FFFF
    pub fn set_addr_l(&mut self, v: u8); pub fn set_addr_m(&mut self, v: u8); pub fn set_addr_h(&mut self, v: u8);
    pub fn hash_into(&self, h: &mut crate::hash::Fnv64);               // toute la memoire puis addr (u32)
}
```
`set_addr_l`: `addr = (addr & !0xFF) | v` ; `set_addr_m`: `addr = (addr & !0xFF00) | (v << 8)` ; `set_addr_h`: `addr = (addr & 0xFFFF) | ((v & 1) << 16)`.""",
  tests=[("power_on", "`Wram::new()`: `read(0)==0x55`, `read(0x1FFFF)==0x55`, `addr==0`."),
         ("mask", "`write(0x20005, 9)` puis `read(5)==9` ; `read(0x40005)==9`."),
         ("addr_regs", "`set_addr_l(0x34); set_addr_m(0x12); set_addr_h(0x01)` -> addr==0x11234 ; `set_addr_l(0x99)` -> 0x11299 ; `set_addr_h(0xFE)` -> bit 16 a 0 (addr==0x1299) ; `set_addr_h(0x03)` -> bit16 a 1."),
         ("data_write_wrap", "addr=0x1FFFE (l=0xFE, m=0xFF, h=1) ; `data_write(1); data_write(2); data_write(3)` -> read(0x1FFFE)==1, read(0x1FFFF)==2, read(0)==3, addr==1."),
         ("data_read", "apres `write(0x10,0xAA); write(0x11,0xBB)`, addr=0x10: `data_read()==0xAA`, `data_read()==0xBB`, addr==0x12."),
         ("reset_addr", "`reset_addr()` met addr a 0 sans modifier la memoire (read(0x10) inchange)."),
         ("hash", "hash identique pour deux `Wram::new()` ; differe apres `write(5,1)` ; differe aussi apres un simple changement de `addr`.")])

T("E03-11", "joypad.rs: ports $4016/$4017 et valeurs d'auto-read",
  size=120, deps=["E01-12"],
  context="Etat des deux manettes et registres a decalage serie. Format des 16 bits: spec 02 section 6.5 (identique a E01-12). " + CHAIN,
  embed=["02_BUS_TIMING.md#6.5 JOYOUT / JOYSER0 / JOYSER1 ($4016/$4017)", "09_SYSTEM.md#3.2 Joypads (`joypad.rs`, ajouter a E01-12)"],
  write=[SRC + "/joypad.rs", TST + "/e03_11_joypad_ports.rs"],
  spec="""Ajoute a `joypad.rs` (garde les constantes et `filter_opposites` de E01-12):
```rust
pub struct Joypads { state: [u16; 2], shift: [u16; 2], out0: bool }
impl Joypads {
    pub fn new() -> Joypads;                                   // state 0, shift 0xFFFF, out0 false  (UNKNOWN - to confirm: valeur initiale du registre)
    pub fn set_buttons(&mut self, port: usize, v: u16);        // port >= 2: ignore
    pub fn buttons(&self, port: usize) -> u16;                 // 0 si port >= 2
    pub fn write_out(&mut self, bit0: bool);                   // if bit0 || self.out0 { shift = state (les 2 ports) } ; puis out0 = bit0
    pub fn read_port(&mut self, port: usize) -> u8;            // renvoie 0 ou 1 (bit D0) ; port >= 2: 1
    pub fn autoread_values(&mut self) -> [u16; 4];             // [state0, state1, 0, 0] ; puis shift = 0xFFFF pour les deux ports
    pub fn hash_into(&self, h: &mut crate::hash::Fnv64);       // state[0], state[1], shift[0], shift[1] (u16), out0 (u8)
}
```
`read_port(p)`: si `out0`: `(state[p] >> 15) & 1` sans decaler ; sinon `b = (shift[p] >> 15) & 1; shift[p] = (shift[p] << 1) | 1; b`.""",
  tests=[("initial", "`Joypads::new()`: `read_port(0)==1` (registre a 0xFFFF) ; `buttons(0)==0`."),
         ("latch_sequence", "`set_buttons(0, 0x8080)` ; `write_out(true); write_out(false)` ; 16 lectures `read_port(0)` donnent 1,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0 ; les lectures 17 et 18 donnent 1,1."),
         ("port1_independent", "`set_buttons(1, BTN_START)` (0x1000) ; latch ; `read_port(1)` x4 donne 0,0,0,1 ; `read_port(0)` (state 0) x2 donne 0,0."),
         ("hold_out0", "`set_buttons(0,0x8000)`; `write_out(true)` ; 3 lectures `read_port(0)` donnent 1,1,1 sans decaler ; `set_buttons(0,0)` -> la lecture suivante donne 0 immediatement ; `write_out(false)` puis lecture = bit 15 de l'etat (0)."),
         ("no_reload_when_idle", "`set_buttons(0,0xC000)` ; latch ; 2 lectures (1,1) ; `write_out(false)` (out0 deja faux, bit0 faux) ne recharge pas : la 3eme lecture donne 0 (bit 13)."),
         ("reload_on_write_true", "latch ; 3 lectures ; `write_out(true)` recharge : une lecture donne bit 15."),
         ("autoread", "`set_buttons(0,0x8001)`, `set_buttons(1,0x0F00)`: `autoread_values()==[0x8001,0x0F00,0,0]` ; ensuite `read_port(0)==1` et `read_port(1)==1` (registres a 0xFFFF)."),
         ("bad_port", "`set_buttons(2, 0xFFFF)` ne panique pas et `buttons(2)==0` ; `read_port(2)==1`."),
         ("hash", "hash differe apres `set_buttons(0,1)`, et differe apres une lecture `read_port(0)` (shift change).")])

T("E03-12", "apu/mod.rs: squelette Apu (ports $2140-$2143)",
  size=70,
  context="Le CPU SPC700 n'existe pas encore (E06). Ce squelette expose seulement les 4 ports de communication pour que le bus soit complet. " + CHAIN,
  embed=["09_SYSTEM.md#3.3 Apu (squelette, `apu/mod.rs`; sera complete en E06)"],
  write=[SRC + "/apu/mod.rs", TST + "/e03_12_apu_skeleton.rs"],
  spec="""Dans `apu/mod.rs` (garde les `pub mod spc700; pub mod dsp;` existants):
```rust
pub struct Apu { to_apu: [u8; 4], from_apu: [u8; 4] }
impl Apu {
    pub fn new() -> Apu;  pub fn reset(&mut self);             // les 8 octets a 0
    pub fn port_read(&self, i: usize) -> u8;                   // from_apu[i & 3]
    pub fn port_write(&mut self, i: usize, v: u8);             // to_apu[i & 3] = v
    pub fn step_to(&mut self, master: u64);                    // ne fait rien (E06)
    pub fn cpu_to_apu(&self, i: usize) -> u8;                  // debug/test: to_apu[i & 3]
    pub fn debug_set_from_apu(&mut self, i: usize, v: u8);     // debug/test: from_apu[i & 3] = v
    pub fn hash_into(&self, h: &mut crate::hash::Fnv64);       // les 8 octets
}
```""",
  tests=[("power_on", "`Apu::new()`: `port_read(i)==0` pour i in 0..4, `cpu_to_apu(i)==0`."),
         ("write_goes_to_apu", "`port_write(1,0x55)`: `cpu_to_apu(1)==0x55` ET `port_read(1)==0` (les deux sens sont separes)."),
         ("mirror", "`port_write(5,0x77)` -> `cpu_to_apu(1)==0x77` ; `debug_set_from_apu(2,0x99)` -> `port_read(6)==0x99` et `port_read(2)==0x99`."),
         ("reset", "apres ecritures, `reset()` remet tout a 0."),
         ("step_to_noop", "`step_to(1_000_000)` ne panique pas et ne change pas le hash."),
         ("hash", "hash differe apres `port_write(0,1)` et apres `debug_set_from_apu(0,1)`.")])

T("E03-13", "bus/mod.rs: access_clocks (6/8/12 clocks par acces)",
  size=70,
  context="Duree en clocks maitres d'un acces CPU selon l'adresse (algorithme de la spec 02 section 2, a implementer EXACTEMENT). Fonction pure. " + CHAIN,
  embed=["02_BUS_TIMING.md#2. Carte memoire CPU (24 bits: bank:addr)"],
  write=[SRC + "/bus/mod.rs", TST + "/e03_13_access_clocks.rs"],
  spec="""Dans `bus/mod.rs` (garde `pub mod wram; pub mod mmio;`): `pub fn access_clocks(addr: u32, memsel: bool) -> u32` avec l'algorithme de la section (arithmetique `u16` avec `wrapping_add` / `wrapping_sub`). `addr` = adresse 24 bits (banque << 16 | adresse).
```
a = (addr & 0xFFFF) as u16
if addr & 0x408000 != 0 { return if addr & 0x800000 != 0 && memsel { 6 } else { 8 } }
if (a.wrapping_add(0x6000)) & 0x4000 != 0 { return 8 }
if (a.wrapping_sub(0x4000)) & 0x7E00 != 0 { return 6 }
12
```""",
  tests=[("wram_low", "`access_clocks(0x000000,false)==8` ; `0x001FFF -> 8` ; `0x801234 -> 8` ; `0x3F0000 -> 8`."),
         ("io_6", "`0x002000 -> 6` ; `0x002100 -> 6` ; `0x00213F -> 6` ; `0x002140 -> 6` ; `0x002180 -> 6` ; `0x003FFF -> 6` ; `0x004200 -> 6` ; `0x00420B -> 6` ; `0x004300 -> 6` ; `0x005FFF -> 6`."),
         ("joyser_12", "`0x004000 -> 12` ; `0x004016 -> 12` ; `0x004017 -> 12` ; `0x0041FF -> 12` ; `0x804016 -> 12`."),
         ("sram_area", "`0x006000 -> 8` ; `0x007FFF -> 8` ; `0x706000 -> 8`."),
         ("rom_slow", "`0x008000 -> 8` ; `0x00FFFF -> 8` ; `0x018000 -> 8` ; `0x400000 -> 8` ; `0x7E0000 -> 8` ; `0x7FFFFF -> 8` ; `0xC00000 -> 8` (memsel faux)."),
         ("memsel", "memsel=true: `0x808000 -> 6` ; `0xC00000 -> 6` ; `0xFFFFFF -> 6` ; `0x008000 -> 8` ; `0x400000 -> 8` ; `0x7E0000 -> 8` ; `0x802100 -> 6` ; `0x800000 -> 8` (WRAM miroir, pas ROM)."),
         ("exhaustive_banks", "pour toute banque 0x00..=0x3F et 0x80..=0xBF : `access_clocks(bank<<16 | 0x4016, m)==12` et `access_clocks(bank<<16 | 0x2100, m)==6` pour m in {false,true}.")])
