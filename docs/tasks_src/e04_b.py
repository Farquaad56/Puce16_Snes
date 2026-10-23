from _dsl import T

CORE = "crates/puce16-core"
SRC = CORE + "/src"
TST = CORE + "/tests"

CHAIN = ("Execution strictement sequentielle (D-020): cette tache suppose que toutes les precedentes sont terminees. "
         "Ne modifie QUE les fichiers de 'Write only in'.")

# Aide commune aux tests: placer un Ppu NTSC (frame 0, sans entrelace) a la position (v, dot) par des step_dot.
AT = ("Dans les tests, utilise l'aide locale `fn at(p: &mut Ppu, v: u32, dot: u32) { for _ in 0..(v * 340 + dot) { p.step_dot(); } }` "
      "(valide pour NTSC, frame 0, sans entrelace: toutes les lignes ont 340 dots; position atteinte = (v, dot) apres ces steps).")

# ------------------------------------------------------------------ E04 part B: registres de configuration, aiguillage du Ppu, lecture, latch H/V (E04-06..12)

T("E04-06", "ppu/regs.rs: registres de configuration des fonds et des sprites",
  size=110,
  context="""Structure `Regs` (pure) qui stocke les registres de configuration. Cette tache: OBJSEL ($2101), BGMODE ($2105), MOSAIC ($2106), BG1SC..BG4SC ($2107-$210A), BG12NBA/BG34NBA ($210B/$210C).
Les fenetres, TM/TS, color math et COLDATA viennent en E04-07. Les champs sont `pub` (le rendu E05 les lit directement). """ + CHAIN,
  embed=["05_PPU.md#2. Registres PPU en ecriture"],
  write=[SRC + "/ppu/regs.rs", TST + "/e04_06_regs_bg.rs"],
  spec="""Dans `ppu/regs.rs` (garde `Scroll` de E04-04):
```rust
pub struct Regs {
    pub obj_size: u8, pub obj_name_gap: u16, pub obj_base: u16,        // OBJSEL
    pub bg_mode: u8, pub bg3_priority: bool, pub tile16: [bool; 4],    // BGMODE (tile16[0] = BG1)
    pub mosaic_raw: u8, pub mosaic_en: u8,                             // MOSAIC: raw = bits 7-4 ; en = bits 3-0 (bit0 = BG1)
    pub map_base: [u16; 4], pub map_x2: [bool; 4], pub map_y2: [bool; 4],   // BGnSC (indice 0 = BG1)
    pub chr_base: [u16; 4],                                            // BG12NBA / BG34NBA
}
impl Regs {
    pub fn new() -> Regs;                               // tout a 0 / false
    pub fn write(&mut self, reg: u8, v: u8) -> bool;    // reg = adresse - 0x2100. Gere 0x01 et 0x05..=0x0C. Renvoie true si gere, false sinon (sans rien modifier)
    pub fn mosaic_size(&self) -> u8;                    // mosaic_raw + 1 (1..=16)
    pub fn hash_into(&self, h: &mut crate::hash::Fnv64);
}
```
Decodage (spec 05 section 2):
- `0x01` OBJSEL : `obj_size = v >> 5` ; `obj_name_gap = ((((v >> 3) & 3) as u16) + 1) << 12` ; `obj_base = ((v & 7) as u16) << 13`.
- `0x05` BGMODE : `bg_mode = v & 7` ; `bg3_priority = v & 8 != 0` ; `tile16[n] = (v >> (4 + n)) & 1 != 0` pour n = 0..4.
- `0x06` MOSAIC : `mosaic_raw = v >> 4` ; `mosaic_en = v & 0x0F`.
- `0x07..=0x0A` BGnSC (n = reg - 7) : `map_base[n] = ((v >> 2) as u16) << 10` ; `map_x2[n] = v & 1 != 0` ; `map_y2[n] = v & 2 != 0`.
- `0x0B` BG12NBA : `chr_base[0] = ((v & 0xF) as u16) << 12` ; `chr_base[1] = ((v >> 4) as u16) << 12`. `0x0C` BG34NBA : memes formules pour `chr_base[2]` et `chr_base[3]`.
`hash_into` (ordre fixe, u8 pour les bools): obj_size, obj_name_gap (u16), obj_base (u16), bg_mode, bg3_priority, tile16[0..4], mosaic_raw, mosaic_en, map_base[0..4] (u16), map_x2[0..4], map_y2[0..4], chr_base[0..4] (u16).""",
  tests=[("power_on", "`Regs::new()`: obj_size==0, obj_name_gap==0, bg_mode==0, `mosaic_size()==1`, map_base==[0;4], chr_base==[0;4], tile16==[false;4]."),
         ("objsel", "`write(0x01,0xE3)`: obj_size==7, obj_name_gap==0x1000, obj_base==0x6000. `write(0x01,0x5A)`: obj_size==2, obj_name_gap==0x4000, obj_base==0x4000. `write(0x01,0x00)`: 0, 0x1000, 0."),
         ("bgmode", "`write(0x05,0xB5)`: bg_mode==5, bg3_priority==false, tile16==[true,true,false,true]. `write(0x05,0x09)`: bg_mode==1, bg3_priority==true, tile16==[false;4]. `write(0x05,0x07)`: bg_mode==7."),
         ("mosaic", "`write(0x06,0x35)`: mosaic_raw==3, mosaic_size()==4, mosaic_en==5. `write(0x06,0xF0)`: mosaic_size()==16, mosaic_en==0. `write(0x06,0x0F)`: mosaic_size()==1, mosaic_en==0x0F."),
         ("bgsc", "`write(0x07,0x7B)`: map_base[0]==0x7800, map_x2[0], map_y2[0]. `write(0x08,0x04)`: map_base[1]==0x0400, map_x2[1]==false, map_y2[1]==false. `write(0x09,0xFC)`: map_base[2]==0xFC00. `write(0x0A,0x02)`: map_base[3]==0, map_x2[3]==false, map_y2[3]==true."),
         ("nba", "`write(0x0B,0x42)`: chr_base[0]==0x2000, chr_base[1]==0x4000. `write(0x0C,0xF1)`: chr_base[2]==0x1000, chr_base[3]==0xF000."),
         ("return_value", "`write(0x01,0)`, `write(0x05,0)`, `write(0x0C,0)` renvoient true ; `write(0x00,0xFF)`, `write(0x0D,1)`, `write(0x23,1)`, `write(0x33,1)` renvoient false et ne changent pas le hash."),
         ("hash", "hash identique pour deux `Regs::new()` ; differe apres chacune des ecritures `write(0x01,0x20)`, `write(0x05,0x01)`, `write(0x06,0x10)`, `write(0x07,0x04)`, `write(0x0B,0x01)`.")])

T("E04-07", "ppu/regs.rs: fenetres, TM/TS, color math, COLDATA",
  size=130,
  context="Suite de `Regs` (E04-06): registres $2123-$2132. Seul le stockage est fait ici (le rendu des fenetres et de la color math est en E05). " + CHAIN,
  embed=["05_PPU.md#2. Registres PPU en ecriture"],
  write=[SRC + "/ppu/regs.rs", TST + "/e04_07_regs_win_math.rs"],
  spec="""Dans `ppu/regs.rs`, ajoute a `Regs` les champs `pub` suivants (valeur initiale 0 / false) et traite les registres `0x23..=0x32` dans `Regs::write` (qui renvoie alors true):
```rust
pub win_sel: [u8; 3], pub wh: [u8; 4], pub win_log_bg: u8, pub win_log_obj: u8, pub win_log_col: u8,
pub tm: u8, pub ts: u8, pub tmw: u8, pub tsw: u8,
pub direct_color: bool, pub addend_subscreen: bool, pub math_region: u8, pub black_region: u8,
pub math_mask: u8, pub math_half: bool, pub math_sub: bool,
pub fixed_r: u8, pub fixed_g: u8, pub fixed_b: u8,
```
Decodage:
- `0x23` W12SEL -> `win_sel[0] = v` ; `0x24` W34SEL -> `win_sel[1] = v` ; `0x25` WOBJSEL -> `win_sel[2] = v`.
- `0x26..=0x29` WH0..WH3 -> `wh[reg - 0x26] = v`.
- `0x2A` WBGLOG -> `win_log_bg = v` ; `0x2B` WOBJLOG -> `win_log_obj = v & 3 ; win_log_col = (v >> 2) & 3`.
- `0x2C` TM -> `tm = v & 0x1F` ; `0x2D` TS ; `0x2E` TMW ; `0x2F` TSW : meme masque.
- `0x30` CGWSEL -> `direct_color = v & 1 != 0 ; addend_subscreen = v & 2 != 0 ; math_region = (v >> 4) & 3 ; black_region = (v >> 6) & 3`.
- `0x31` CGADSUB -> `math_mask = v & 0x3F` (bits 0-4 couches BG1 BG2 BG3 BG4 OBJ, bit 5 backdrop) ; `math_half = v & 0x40 != 0 ; math_sub = v & 0x80 != 0`.
- `0x32` COLDATA -> `c = v & 0x1F` ; si bit 5: `fixed_r = c` ; si bit 6: `fixed_g = c` ; si bit 7: `fixed_b = c` (plusieurs bits possibles dans la meme ecriture ; les canaux non selectionnes ne changent pas).
`hash_into`: AJOUTE a la fin (apres les champs de E04-06), dans cet ordre: win_sel[0..3], wh[0..4], win_log_bg, win_log_obj, win_log_col, tm, ts, tmw, tsw, direct_color, addend_subscreen, math_region, black_region, math_mask, math_half, math_sub, fixed_r, fixed_g, fixed_b (u8 chacun).""",
  tests=[("win_sel", "`write(0x23,0xA5)`, `write(0x24,0x5A)`, `write(0x25,0xFF)` -> win_sel==[0xA5,0x5A,0xFF]."),
         ("wh", "`write(0x26,1)`, `write(0x27,2)`, `write(0x28,3)`, `write(0x29,4)` -> wh==[1,2,3,4]."),
         ("win_log", "`write(0x2A,0xE4)` -> win_log_bg==0xE4. `write(0x2B,0xFE)` -> win_log_obj==2, win_log_col==3. `write(0x2B,0x04)` -> win_log_obj==0, win_log_col==1."),
         ("layer_masks", "`write(0x2C,0xFF)` -> tm==0x1F ; `write(0x2D,0xE5)` -> ts==0x05 ; `write(0x2E,0xBF)` -> tmw==0x1F ; `write(0x2F,0x81)` -> tsw==0x01."),
         ("cgwsel", "`write(0x30,0xF3)`: direct_color, addend_subscreen, math_region==3, black_region==3. `write(0x30,0x44)`: direct_color==false, addend_subscreen==false, math_region==0, black_region==1. `write(0x30,0x24)`: math_region==2, black_region==0."),
         ("cgadsub", "`write(0x31,0xE5)`: math_mask==0x25, math_half, math_sub. `write(0x31,0x3F)`: math_mask==0x3F, math_half==false, math_sub==false. `write(0x31,0x40)`: math_mask==0, math_half, math_sub==false."),
         ("coldata", "`write(0x32,0x2A)` -> (r,g,b)==(10,0,0) ; `write(0x32,0x5F)` -> (10,31,0) ; `write(0x32,0x85)` -> (10,31,5) ; `write(0x32,0xFF)` -> (31,31,31) ; `write(0x32,0xC1)` -> (31,1,1)."),
         ("returns", "`write(0x23,0)` ... `write(0x32,0)` renvoient tous true (verifie pour chaque reg 0x23..=0x32) ; `write(0x33,1)` et `write(0x22,1)` renvoient false."),
         ("hash", "hash differe apres chacune de: `write(0x23,1)`, `write(0x26,1)`, `write(0x2A,1)`, `write(0x2C,1)`, `write(0x30,1)`, `write(0x31,1)`, `write(0x32,0x21)`.")])

T("E04-08", "ppu/mod.rs: aiguillage des ecritures $2100-$2133 vers les memoires et registres",
  size=150,
  context="""Le `Ppu` (E03-08/09) recoit les memoires (E04-01..03), les scrolls (E04-04), le mode 7 (E04-05) et les registres de configuration (E04-06/07).
Cette tache ne fait QUE les ecritures (`Ppu::write`); les lectures sont en E04-09/10. Les regles d'acces sont dans la spec 05 section 3. """ + CHAIN,
  embed=["05_PPU.md#2. Registres PPU en ecriture", "05_PPU.md#3. Regles d'acces pendant l'affichage"],
  write=[SRC + "/ppu/mod.rs", TST + "/e04_08_ppu_write.rs"],
  spec="""Dans `ppu/mod.rs`: ajoute au `Ppu` les champs `brightness: u8`, `vram: Vram`, `cgram: Cgram`, `oam: Oam`, `scroll: Scroll`, `m7: M7Regs`, `regs: Regs` (tous a l'etat `new()` dans `Ppu::new`) et les methodes:
```rust
pub fn brightness(&self) -> u8;
pub fn vram_ok(&self) -> bool;     // forced_blank || v >= vblank_start()    (V = 0 NON accessible)
pub fn oam_ok(&self) -> bool;      // identique a vram_ok
pub fn vram(&self) -> &[u16];  pub fn cgram(&self) -> &[u16];  pub fn oam(&self) -> &[u8];
pub fn vram_unit(&self) -> &Vram;  pub fn cgram_unit(&self) -> &Cgram;  pub fn oam_unit(&self) -> &Oam;
pub fn regs(&self) -> &Regs;  pub fn scroll(&self) -> &Scroll;  pub fn m7(&self) -> &M7Regs;
```
`Ppu::write(reg, v)` (reg = adresse - 0x2100) aiguille:
- `0x00`: `forced_blank = v & 0x80 != 0 ; brightness = v & 0x0F`.
- `0x01`, `0x05..=0x0C`, `0x23..=0x32`: `self.regs.write(reg, v)`.
- `0x02` -> `oam.set_addr_l(v)` ; `0x03` -> `oam.set_addr_h(v)` ; `0x04` -> `oam.write(v, self.oam_ok())`.
- `0x0D..=0x14`: `n = ((reg - 0x0D) / 2) as usize` ; reg impair (0x0D, 0x0F, 0x11, 0x13) -> `scroll.write_hofs(n, v)`, reg pair -> `scroll.write_vofs(n, v)`. ET en plus: `0x0D` -> `m7.write_scroll(0, v)` ; `0x0E` -> `m7.write_scroll(1, v)`.
- `0x15` -> `vram.set_vmain(v)` ; `0x16` -> `vram.set_addr_l(v)` ; `0x17` -> `vram.set_addr_h(v)` ; `0x18` -> `vram.write_low(v, self.vram_ok())` ; `0x19` -> `vram.write_high(v, self.vram_ok())`.
- `0x1A` -> `m7.write_sel(v)` ; `0x1B..=0x1E` -> `m7.write_matrix((reg - 0x1B) as usize, v)` ; `0x1F` -> `m7.write_center(0, v)` ; `0x20` -> `m7.write_center(1, v)`.
- `0x21` -> `cgram.set_addr(v)` ; `0x22` -> `cgram.write(v)`.
- `0x33`: inchange (E03-09: interlace bit0, overscan bit2; le reste en E04-11).
- Tout autre `reg` (>= 0x34): ignore.
Une ecriture ne change jamais `v`, `dot`, `field`.
`hash_into`: AJOUTE a la fin (apres `forced_blank`): `brightness` (u8), puis `vram.hash_into`, `cgram.hash_into`, `oam.hash_into`, `scroll.hash_into`, `m7.hash_into`, `regs.hash_into`.""",
  tests=[("inidisp", "`Ppu::new(Ntsc)`: brightness()==0, forced_blank()==true. `write(0x00,0x0F)`: brightness()==15, forced_blank()==false. `write(0x00,0x85)`: brightness()==5, forced_blank()==true."),
         ("vram_write_forced_blank", "forced blank (par defaut) : `write(0x15,0x80); write(0x16,0x00); write(0x17,0x01); write(0x18,0xCD); write(0x19,0xAB)` -> `vram()[0x100]==0xABCD` ; `vram_unit().vmadd()==0x101`."),
         ("vram_ok_boundaries", AT + " `write(0x00,0x0F)` (display on), CHAQUE position sur un `Ppu` neuf : a (0,0): `vram_ok()==false` et `oam_ok()==false` ; a (224,339): false ; a (225,0): true. Avec `write(0x33,0x04)` (overscan): faux a (239,339), vrai a (240,0). Forced blank vrai a (0,0): `vram_ok()==true`."),
         ("vram_blocked_then_ok", AT + " display on, `write(0x15,0x80); write(0x16,0x00); write(0x17,0x02)` (adresse 0x200) ; `write(0x19,0x12)` a v=0 est bloque : `vram()[0x200]==0` et `vram_unit().vmadd()==0x201` ; apres `for _ in 0..225*340 { p.step_dot(); }` (position (225,0)) : `write(0x18,0x34); write(0x19,0x56)` -> `vram()[0x201]==0x5634` et `vram()[0x200]==0`."),
         ("oam_write", "forced blank : `write(0x02,0x00); write(0x03,0x00); write(0x04,0x11); write(0x04,0x22)` -> `oam()[0]==0x11`, `oam()[1]==0x22`. Display on a (0,0) : `write(0x02,0x00); write(0x04,0x33); write(0x04,0x44)` ne modifient pas `oam()[0]` (reste 0x11) mais `oam_unit().internal()==2`."),
         ("cgram_write", "`write(0x21,3); write(0x22,0x1F); write(0x22,0x03)` -> `cgram()[3]==0x031F`, `cgram_unit().addr()==4`. Les ecritures CGRAM fonctionnent aussi display on a (0,0) : `write(0x00,0x0F); write(0x21,4); write(0x22,0x01); write(0x22,0x02)` -> `cgram()[4]==0x0201`."),
         ("scroll_and_mode7", "`write(0x0D,0x34); write(0x0D,0x01)` -> `scroll().hofs(0)==0x134` ET `m7().hofs==308`. `write(0x0E,0x34); write(0x0E,0x02)` -> `scroll().vofs(0)==0x234` ET `m7().vofs==564`. `write(0x11,0x34); write(0x11,0x01)` -> `scroll().hofs(2)==0x134` (BG3). `write(0x14,0x34); write(0x14,0x02)` -> `scroll().vofs(3)==0x234` (BG4) sans changer `m7().vofs`."),
         ("mode7_regs", "`write(0x1B,0x00); write(0x1B,0x01)` -> `m7().a==256`. `write(0x1F,0x34); write(0x1F,0x01)` -> `m7().cx==308`. `write(0x1A,0xC3)` -> `m7().flip_x`, `flip_y`, `fill`, `outside` vrais."),
         ("config_regs", "`write(0x05,0xB5)` -> `regs().bg_mode==5` ; `write(0x07,0x7B)` -> `regs().map_base[0]==0x7800` ; `write(0x2C,0xFF)` -> `regs().tm==0x1F` ; `write(0x30,0xF3)` -> `regs().direct_color` ; `write(0x32,0xFF)` -> `regs().fixed_r==31`."),
         ("write_keeps_position", "apres `at(p,3,100)` puis une serie d'ecritures sur 0x00, 0x05, 0x15, 0x18, 0x22, 0x33 : `v()==3`, `dot()==100`, `field()==false`."),
         ("ignored_regs", "`write(0x34,0xFF)`, `write(0x3F,0xFF)`, `write(0xFF,0xFF)` ne changent pas le hash. `read(0x05,0xAB,false)==0xAB` et `read(0x37,0xCD,true)==0xCD` (lecture: inchange, open bus)."),
         ("hash", "hash differe apres: `write(0x00,0x05)` (brightness) ; une ecriture VRAM (forced blank) ; `write(0x22,1)` ; `write(0x04,1)` ; `write(0x0D,1)` ; `write(0x1A,1)` ; `write(0x05,1)`. Deux `Ppu` recevant la meme suite d'ecritures ont le meme hash.")])

T("E04-09", "ppu/mod.rs: registres de lecture MPY, OAM, VRAM, CGRAM et open bus PPU",
  size=120,
  context="""Lecture des registres $2134-$2136 (MPY), $2138 (OAMDATAREAD), $2139/$213A (VMDATAxREAD), $213B (CGDATAREAD) et gestion de l'open bus interne du PPU (`ppu1_mdr`, `ppu2_mdr`).
Les registres $2137 (SLHV) et $213C-$213F (compteurs H/V, STAT77/78) sont traites en E04-10: ici ils renvoient `cpu_mdr` sans effet. """ + CHAIN,
  embed=["05_PPU.md#4. Registres PPU en lecture"],
  write=[SRC + "/ppu/mod.rs", TST + "/e04_09_ppu_read.rs"],
  spec="""Dans `ppu/mod.rs`: ajoute au `Ppu` les champs `ppu1_mdr: u8`, `ppu2_mdr: u8` (0 au power-on) et les accesseurs `pub fn ppu1_mdr(&self) -> u8`, `pub fn ppu2_mdr(&self) -> u8`.
`Ppu::read(reg, cpu_mdr, wrio7) -> u8`:
- `0x34 | 0x35 | 0x36` (MPYL, MPYM, MPYH): `val = ((m7.mpy_bits() >> (8 * (reg - 0x34))) & 0xFF) as u8` ; `ppu1_mdr = val` ; renvoie `val`.
- `0x38`: `val = oam.read(self.oam_ok())` ; `ppu1_mdr = val`.
- `0x39`: `val = vram.read_low(self.vram_ok())` ; `ppu1_mdr = val`. `0x3A`: `vram.read_high(self.vram_ok())` ; `ppu1_mdr = val`.
- `0x3B`: `val = cgram.read(self.ppu2_mdr)` ; `ppu2_mdr = val`.
- tout le reste (`0x00..=0x33`, `0x37`, `0x3C..=0x3F`, et reg > 0x3F): renvoie `cpu_mdr` (open bus CPU) sans modifier le Ppu. (E04-10 complete `0x37` et `0x3C..=0x3F`.)
`hash_into`: AJOUTE a la fin: `ppu1_mdr`, `ppu2_mdr` (u8).""",
  tests=[("mpy_bytes", "`write(0x1B,0x00); write(0x1B,0x01)` (a=256) ; `write(0x1C,0xFF)` -> `read(0x34,0,false)==0x00`, `read(0x35,0,false)==0xFF`, `read(0x36,0,false)==0xFF` (produit -256 = 0xFFFF00) ; `ppu1_mdr()==0xFF`."),
         ("mpy_negative_small", "a=-1 (`write(0x1B,0xFF)` deux fois), `write(0x1C,0x7F)` -> `read(0x34,..)==0x81`, `read(0x35,..)==0xFF`, `read(0x36,..)==0xFF` (-127 = 0xFFFF81)."),
         ("mpy_positive", "a=0x0100 (ecritures 0x00 puis 0x01 sur 0x1B), `write(0x1C,0x7F)` -> `read(0x34,..)==0x00`, `read(0x35,..)==0x7F`, `read(0x36,..)==0x00`."),
         ("oam_read", "forced blank : `write(0x02,0); write(0x03,0)`, quatre ecritures `write(0x04, 0x11/0x22/0x33/0x44)` ; `write(0x02,0)` (adresse 0) : `read(0x38,0,false)` renvoie 0x11, 0x22, 0x33, 0x44 successivement ; `ppu1_mdr()==0x44`. Display on a (0,0) : `read(0x38,0x99,false)==0` (bloque, adresse incrementee)."),
         ("vram_read_sequence", "forced blank : `write(0x15,0x80); write(0x16,0x10); write(0x17,0x00)` ; deux mots ecrits: `write(0x18,0xCD); write(0x19,0xAB); write(0x18,0x34); write(0x19,0x12)` (mots 0x10 = 0xABCD, 0x11 = 0x1234) ; `write(0x16,0x10)` (re-adresse, prefetch) : `read(0x39)==0xCD` (vmadd inchange), `read(0x3A)==0xAB`, `read(0x3A)==0xAB` (meme mot, rechargement avant increment), `read(0x39)==0x34` (latch = 0x1234, aucun changement), `read(0x3A)==0x12`. `ppu1_mdr()==0x12`."),
         ("vram_read_blocked", "display on (`write(0x00,0x0F)`) a (0,0), VMAIN 0x00 : `read(0x39,0x77,false)==0` (bloque) et `vram_unit().vmadd()==1`."),
         ("cgram_read_bit7_open_bus", "`write(0x21,5); write(0x22,0xBC); write(0x22,0x7A)` ; `write(0x21,5)` : `read(0x3B,0,false)==0xBC` (ppu2_mdr==0xBC), puis `read(0x3B,0,false)==0xFA` (0x7A | (0xBC & 0x80)). Ensuite mot 6 = 0 : `read(0x3B,..)==0x00`, `read(0x3B,..)==0x00` (bit 7 de ppu2_mdr est 0)."),
         ("mdr_independent", "`ppu1_mdr()` n'est pas modifie par `read(0x3B,..)` et `ppu2_mdr()` n'est pas modifie par `read(0x34,..)`, `read(0x38,..)`, `read(0x39,..)`."),
         ("open_bus_cpu", "`read(0x05,0xAB,false)==0xAB` ; `read(0x00,0x12,true)==0x12` ; `read(0x37,0xCD,false)==0xCD` ; `read(0x40,0x55,false)==0x55` ; ces appels ne changent pas le hash."),
         ("hash", "hash differe apres `read(0x34,..)` quand le produit est non nul (ppu1_mdr change de 0), et differe apres `read(0x3B,..)` (toggle/addr et ppu2_mdr).")])

T("E04-10", "ppu/mod.rs: verrou des compteurs H/V, OPHCT/OPVCT, STAT77, STAT78",
  size=140,
  context="""Fin des registres de lecture: SLHV ($2137), OPHCT ($213C), OPVCT ($213D), STAT77 ($213E), STAT78 ($213F), et le declencheur par front descendant de WRIO bit 7 (`wrio_write`).
Les drapeaux `range_over` / `time_over` (STAT77) seront positionnes par le rendu des sprites (E05); ici ils ont seulement des setters de test. """ + CHAIN + " " + AT,
  embed=["05_PPU.md#4. Registres PPU en lecture", "05_PPU.md#4.1 Verrou des compteurs H/V"],
  write=[SRC + "/ppu/mod.rs", TST + "/e04_10_hv_latch.rs"],
  spec="""Dans `ppu/mod.rs`: ajoute au `Ppu` les champs `latch_h: u16`, `latch_v: u16`, `counter_latched: bool`, `ophct_byte: bool`, `opvct_byte: bool`, `range_over: bool`, `time_over: bool` (tous a 0 / false) et:
```rust
pub fn latch_counters(&mut self);                  // latch_h = (self.dot as u32 + crate::consts::HV_LATCH_DOT_OFFSET) as u16 ; latch_v = self.v ; counter_latched = true
pub fn counter_latched(&self) -> bool;
pub fn set_range_over(&mut self, b: bool);  pub fn set_time_over(&mut self, b: bool);
pub fn range_over(&self) -> bool;  pub fn time_over(&self) -> bool;
```
`wrio_write(old, new)`: si `old & 0x80 != 0 && new & 0x80 == 0` (front descendant du bit 7) -> `latch_counters()`.
`read` (complete E04-09):
- `0x37` SLHV: si `wrio7` -> `latch_counters()` ; renvoie `cpu_mdr` (pas de mise a jour d'open bus).
- `0x3C` OPHCT: `val = if !ophct_byte { (latch_h & 0xFF) as u8 } else { (((latch_h >> 8) & 1) as u8) | (ppu2_mdr & 0xFE) }` ; `ophct_byte = !ophct_byte` ; `ppu2_mdr = val`. `0x3D` OPVCT: idem avec `latch_v` et `opvct_byte`.
- `0x3E` STAT77: `val = (range_over as u8) << 7 | (time_over as u8) << 6 | (ppu1_mdr & 0x10) | 0x01` ; `ppu1_mdr = val`.
- `0x3F` STAT78: `val = (field as u8) << 7 | (counter_latched as u8) << 6 | (ppu2_mdr & 0x20) | (is_pal as u8) << 4 | 0x03` ; puis `counter_latched = false ; ophct_byte = false ; opvct_byte = false` ; `ppu2_mdr = val`.
`hash_into`: AJOUTE a la fin: `latch_h` (u16), `latch_v` (u16), `counter_latched`, `ophct_byte`, `opvct_byte`, `range_over`, `time_over` (u8 chacun).""",
  tests=[("before_latch", "`Ppu::new(Ntsc)` : `read(0x3C,0,false)==0` et `read(0x3D,0,false)==0` ; `counter_latched()==false`."),
         ("latch_via_slhv", "a (3,100): `read(0x37,0xEE,true)==0xEE` ; `counter_latched()==true` ; `read(0x3C,0,false)==100` ; `read(0x3D,0,false)==3` (ppu2_mdr valait 100, mais le 1er octet ne depend pas de l'open bus)."),
         ("slhv_needs_wrio7", "a (3,100): `read(0x37,0,false)` ne verrouille pas : `counter_latched()==false`."),
         ("ophct_two_bytes", "a (1,300): `read(0x37,0,true)` ; `read(0x3C,0,false)==0x2C` (300 = 0x12C, ppu2_mdr=0x2C) ; `read(0x3C,0,false)==0x2D` (bit 8 = 1 | (0x2C & 0xFE)) ; `read(0x3C,0,false)==0x2C` (le toggle est revenu au 1er octet)."),
         ("opvct_two_bytes", "a (261,5): `read(0x37,0,true)` ; `read(0x3D,0,false)==0x05` (261 = 0x105) ; `read(0x3D,0,false)==0x05` (bit 8 = 1 | (0x05 & 0xFE = 0x04) = 0x05) ; `read(0x3D,0,false)==0x05`."),
         ("latch_value_frozen", "a (2,10): `read(0x37,0,true)` ; puis 5 `step_dot` supplementaires : `read(0x3C,0,false)==10` (la valeur verrouillee ne bouge pas)."),
         ("stat78_ntsc", "`Ppu::new(Ntsc)`: `read(0x3F,0,false)==0x03`. Apres 262*340 steps (frame 0 terminee): `read(0x3F,0,false)==0x83` (field=1)."),
         ("stat78_pal", "`Ppu::new(Pal)`: `read(0x3F,0,false)==0x13`."),
         ("stat78_clears_latch", "a (3,100): `read(0x37,0,true)` ; `read(0x3F,0,false)==0x43` ; `counter_latched()==false` ; `read(0x3F,0,false)==0x03`."),
         ("stat78_resets_toggles", "a (1,300): latch ; `read(0x3C,0,false)==0x2C` (toggle vrai) ; `read(0x3F,0,false)==0x63` (0x40 latched | 0x20 (ppu2_mdr=0x2C) | 0x03) ; `read(0x3C,0,false)==0x2C` (toggle remis au 1er octet, latch_h conserve)."),
         ("stat77", "`Ppu::new(Ntsc)`: `read(0x3E,0,false)==0x01`. `set_range_over(true)`: 0x81. `set_time_over(true)` aussi: 0xC1. Open bus bit 4: a=16 (`write(0x1B,0x10); write(0x1B,0x00)`), `write(0x1C,0x01)` -> `read(0x34,..)==0x10` (ppu1_mdr=0x10) puis `read(0x3E,..)==0x11`."),
         ("wrio_falling_edge", "a (2,10): `wrio_write(0x80,0x00)` verrouille (`counter_latched()==true`) ; nouveau `Ppu` a (2,10): `wrio_write(0x00,0x80)`, `wrio_write(0x80,0x80)`, `wrio_write(0x00,0x00)` ne verrouillent pas ; `wrio_write(0xFF,0x7F)` verrouille."),
         ("hash", "hash differe apres `read(0x37,0,true)` (verrou) ; apres `set_range_over(true)` ; apres `set_time_over(true)`.")])

T("E04-11", "ppu/mod.rs: INIDISP/SETINI complets, rechargement OAM et drapeaux au debut de frame",
  size=110,
  context="""Derniers morceaux du `Ppu` sans rendu: les bits restants de SETINI ($2133), le rechargement de l'adresse OAM au debut du VBlank (spec 05 section 2.1) et la remise a zero de `range_over` / `time_over` au debut de la frame (section 5.3).
Ces comportements vivent dans `step_dot` (sur la position COURANTE, avant d'avancer, dans l'ordre de la section 5.3). """ + CHAIN + " " + AT,
  embed=["05_PPU.md#5.3 Evenements par (v, dot), dans CET ordre dans `step_dot`", "05_PPU.md#2.1 OAMDATA ($2104) et lecture OAMDATAREAD ($2138)"],
  write=[SRC + "/ppu/mod.rs", TST + "/e04_11_ppu_events.rs"],
  spec="""Dans `ppu/mod.rs`:
- Champs `obj_interlace: bool`, `pseudo_hires: bool`, `extbg: bool` (false) et accesseurs `obj_interlace()`, `pseudo_hires()`, `extbg()`. `write(0x33, v)`: `interlace = bit0`, `obj_interlace = bit1`, `overscan = bit2`, `pseudo_hires = bit3`, `extbg = bit6` ; le bit 7 est ignore.
- Accesseur `pub fn oam_internal_addr(&self) -> u16` (= `oam.internal()`).
- `step_dot`, a `dot == 0`: si `v == 0`: `time_over = false ; range_over = false` (en plus de l'evenement VBLANK_END) ; si `v == vblank_start()` et `!forced_blank`: `oam.reload()` (en plus de l'evenement VBLANK_START). Rien d'autre ne change.
`hash_into`: AJOUTE a la fin: `obj_interlace`, `pseudo_hires`, `extbg` (u8 chacun).""",
  tests=[("setini_bits", "`write(0x33,0x4A)`: obj_interlace()==true, pseudo_hires()==true, extbg()==true, interlace()==false, overscan()==false. `write(0x33,0x87)`: interlace, obj_interlace, overscan vrais ; pseudo_hires et extbg faux (bit 7 ignore). `write(0x33,0x00)`: tout faux."),
         ("oam_reload_at_vblank", "forced blank : `write(0x02,0x07); write(0x03,0x00)` (internal 14) ; 3 x `write(0x04,1)` (internal 17) ; `write(0x00,0x0F)` (display on) ; `at(p,225,0)` : `oam_internal_addr()==17` (l'evenement n'a pas encore eu lieu : il se produit pendant le step execute A (225,0)) ; un `step_dot` de plus -> `oam_internal_addr()==14`."),
         ("oam_reload_skipped_in_forced_blank", "forced blank (par defaut) : `write(0x02,0x07)`, 3 x `write(0x04,1)` (internal 17) ; `at(p,225,1)` : `oam_internal_addr()==17` (pas de rechargement en forced blank)."),
         ("oam_write_in_vblank", "display on, `at(p,226,0)` : `write(0x02,0); write(0x04,0xAA); write(0x04,0xBB)` -> `oam()[0]==0xAA`, `oam()[1]==0xBB` (acces autorise en VBlank)."),
         ("oam_reload_overscan", "`write(0x33,0x04)`, `write(0x02,0x07)` en forced blank, 3 x `write(0x04,1)`, `write(0x00,0x0F)` ; `at(p,225,1)` : internal reste 17 (pas encore VBlank) ; puis 15*340 steps supplementaires (position (240,1)) : `oam_internal_addr()==14`."),
         ("flags_cleared_at_frame_start", "`set_range_over(true); set_time_over(true)` sur un `Ppu::new` (v=0,dot=0) ; un `step_dot` : `read(0x3E,0,false)==0x01`."),
         ("flags_persist_mid_frame", "`at(p,5,0)`, puis `set_range_over(true); set_time_over(true)` ; 256*340 steps supplementaires (position (261,0)) : `read(0x3E,0,false)==0xC1` ; encore 340 steps (position (0,0) de la frame suivante, l'evenement du dot 0 n'est pas encore execute) : `read(0x3E,0,false)==0xC1` ; UN step de plus : `read(0x3E,0,false)==0x01`."),
         ("hash", "hash differe apres `write(0x33,0x02)`, `write(0x33,0x08)`, `write(0x33,0x40)` (chacun sur un Ppu neuf).")])

T("E04-12", "ppu/mod.rs: reset vs power-on, regs_snapshot (debugger) et hash complet",
  size=140,
  context="""Spec 05 section 7: au reset bouton les registres reviennent a leur etat de power-on mais VRAM, CGRAM et OAM sont CONSERVEES. Cette tache fixe aussi `PpuRegsSnapshot` (utilise par le debugger du desktop en E08) et verifie que le hash couvre tout l'etat. """ + CHAIN,
  embed=["05_PPU.md#7. Reset et power-on", "05_PPU.md#8. Interface Rust (ppu/mod.rs)"],
  write=[SRC + "/ppu/mod.rs", TST + "/e04_12_ppu_reset.rs"],
  spec="""Dans `ppu/mod.rs`:
```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PpuRegsSnapshot {
    pub v: u16, pub dot: u16, pub field: bool, pub forced_blank: bool, pub brightness: u8,
    pub bg_mode: u8, pub bg3_priority: bool, pub tile16: [bool; 4], pub obj_size: u8,
    pub map_base: [u16; 4], pub chr_base: [u16; 4], pub hofs: [u16; 4], pub vofs: [u16; 4],
    pub tm: u8, pub ts: u8, pub tmw: u8, pub tsw: u8,
    pub direct_color: bool, pub addend_subscreen: bool, pub math_region: u8, pub black_region: u8,
    pub math_mask: u8, pub math_half: bool, pub math_sub: bool, pub fixed: [u8; 3],
    pub vmain: u8, pub vmadd: u16, pub cgram_addr: u8, pub oam_internal: u16,
    pub interlace: bool, pub obj_interlace: bool, pub overscan: bool, pub pseudo_hires: bool, pub extbg: bool,
    pub m7: [i16; 6],            // a, b, c, d, cx, cy
    pub m7_hofs: i16, pub m7_vofs: i16,
}
impl Ppu { pub fn regs_snapshot(&self) -> PpuRegsSnapshot;  /* copie des valeurs courantes */ }
```
`Ppu::reset()` (remplace celle de E03-08): conserve `region` ET les contenus de `vram`, `cgram`, `oam` (`mem`); appelle `reset_regs()` sur `vram`, `cgram`, `oam`; remplace `scroll`, `m7`, `regs` par `new()`; remet `v = 0, dot = 0, field = false, interlace = false, obj_interlace = false, overscan = false, pseudo_hires = false, extbg = false, forced_blank = true, brightness = 0, ppu1_mdr = ppu2_mdr = 0, latch_h = latch_v = 0, counter_latched = ophct_byte = opvct_byte = range_over = time_over = false`.
Le `Ppu::new(region)` equivaut a un power-on (toutes memoires a 0).""",
  tests=[("snapshot_power_on", "`Ppu::new(Ntsc).regs_snapshot()`: v==0, dot==0, field==false, forced_blank==true, brightness==0, bg_mode==0, tm==0, vmadd==0, oam_internal==0, m7==[0;6], fixed==[0;3]."),
         ("snapshot_values", "apres `write(0x00,0x0F)`, `write(0x05,0x07)`, `write(0x2C,0x13)`, `write(0x32,0x2A)`, `write(0x07,0x7B)`, `write(0x16,0x34)`, `write(0x17,0x12)`, `write(0x1B,0x00)`, `write(0x1B,0x01)` : brightness==15, forced_blank==false, bg_mode==7, tm==0x13, fixed==[10,0,0], map_base[0]==0x7800, vmadd==0x1234, m7[0]==256."),
         ("reset_keeps_memories", "forced blank : `write(0x16,0x05); write(0x17,0x00); write(0x18,0x7B)` (VRAM mot 5 bas = 0x7B), CGRAM mot 2 = 0x0155 (`write(0x21,2); write(0x22,0x55); write(0x22,0x01)`), OAM octets 0,1 = 0x11,0x22 ; `reset()` : `vram()[5]==0x007B`, `cgram()[2]==0x0155`, `oam()[0]==0x11`, `oam()[1]==0x22`."),
         ("reset_clears_registers", "apres de nombreuses ecritures (0x00 0x0F, 0x05 0x07, 0x15 0x80, 0x16 0x34, 0x2C 0x1F, 0x33 0x4F, 0x0D 0x12) et 12345 `step_dot`: `reset()` -> `regs_snapshot()` egal a celui d'un `Ppu::new` de meme region (memoires exclues du snapshot), `ppu1_mdr()==0`, `ppu2_mdr()==0`, `counter_latched()==false`, `v()==0`, `dot()==0`."),
         ("reset_keeps_region", "`Ppu::new(Pal)` puis `reset()` : `read(0x3F,0,false)==0x13` (toujours PAL) ; `last_line()==311`."),
         ("hash_covers_everything", "a partir d'un `Ppu::new(Ntsc)`, chacune des actions suivantes (sur une copie fraiche) change `hash_into`: `write(0x21,1)` ; `write(0x16,1)` ; `write(0x22,5)` seul ; `read(0x37,0,true)` ; `set_range_over(true)` ; `write(0x33,0x40)` ; `write(0x0D,3)` ; `write(0x1F,3)` ; `write(0x2C,1)` ; `write(0x01,0x20)` ; `write(0x00,0x01)` ; un `step_dot` ; `write(0x02,5)`."),
         ("hash_after_reset_equals_new_except_memories", "sur des `Ppu` ou seules des REGISTRES (pas de memoire) ont ete ecrits (ex.: 0x05, 0x2C, 0x33), `reset()` redonne exactement le hash de `Ppu::new` (meme region)."),
         ("reset_idempotent", "`reset()` deux fois de suite donne le meme hash qu'une seule fois.")])
