from _dsl import T

CORE = "crates/puce16-core"
SRC = CORE + "/src"
TST = CORE + "/tests"

CHAIN = ("Execution strictement sequentielle (D-020): cette tache suppose que toutes les precedentes sont terminees. "
         "Ne modifie QUE les fichiers de 'Write only in'.")

# Aides de test communes (recopier dans chaque fichier de test qui en a besoin).
HELP = ("Aides de test (a recopier dans le fichier de test): "
        "`fn put_vram(p: &mut Ppu, addr: u16, w: u16) { p.write(0x15,0x80); p.write(0x16,(addr & 0xFF) as u8); p.write(0x17,(addr >> 8) as u8); p.write(0x18,(w & 0xFF) as u8); p.write(0x19,(w >> 8) as u8); }` ; "
        "`fn put_cgram(p: &mut Ppu, i: u8, w: u16) { p.write(0x21,i); p.write(0x22,(w & 0xFF) as u8); p.write(0x22,(w >> 8) as u8); }` ; "
        "`fn set_scroll(p: &mut Ppu, reg: u8, v: u16) { p.write(reg,(v & 0xFF) as u8); p.write(reg,(v >> 8) as u8); }` (reg = 0x0D HOFS BG1, 0x0E VOFS BG1, 0x0F/0x10 BG2, 0x11/0x12 BG3, 0x13/0x14 BG4). "
        "Un `Ppu::new(Region::Ntsc)` neuf est en forced blank: toutes ces ecritures sont autorisees. Chaque test construit SA scene sur un `Ppu` neuf.")

# ------------------------------------------------------------------ E05 part A: fonds (modes 0-4), fenetres (E05-01..09)
# bg.rs / window.rs sont des sous-modules de `ppu`: leurs `impl Ppu` accedent aux champs PRIVES de Ppu.

T("E05-01", "ppu/bg.rs: types Pix/Pick, tables BPP et ordres de priorite",
  deps=["E04-15"], size=120,
  context="""Debut de l'epic E05 (rendu). Tout ce qui est dans cette tache est constant: tables de la spec 05b sections 1 et 2.
Les couches sont numerotees: 0..=3 = BG1..BG4, 4 = OBJ, 5 = BACKDROP. Un ordre de priorite est une liste de (couche, niveau) de l'AVANT vers l'ARRIERE. """ + CHAIN,
  embed=["05b_PPU_RENDER.md#1. Types de base", "05b_PPU_RENDER.md#2. Modes BG: profondeur de bits par couche et tables de priorite"],
  write=[SRC + "/ppu/bg.rs", TST + "/e05_01_tables.rs"],
  spec="""Dans `ppu/bg.rs`:
```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub struct Pix { pub rgb: u16, pub prio: u8, pub obj_pal: u8 }
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub struct Pick { pub rgb: u16, pub layer: u8, pub obj_pal: u8 }
pub const LAYER_OBJ: u8 = 4;  pub const LAYER_BACKDROP: u8 = 5;
pub fn bpp(mode: u8, bg: usize) -> u8;      // table BPP[mode][bg] ; mode > 7 ou bg > 3: 0
pub fn priority_order(mode: u8, bg3_priority: bool, extbg: bool) -> &'static [(u8, u8)];
```
`bpp` : mode 0 -> [2,2,2,2] ; 1 -> [4,4,2,0] ; 2 -> [4,4,0,0] ; 3 -> [8,4,0,0] ; 4 -> [8,2,0,0] ; 5 -> [4,2,0,0] ; 6 -> [4,0,0,0] ; 7 -> [8,0,0,0].
`priority_order` (couche, niveau): sprites `Sp` = (4, p) ; fond `nH` = (n-1, 1), `nL` = (n-1, 0). Tables (declare-les en `const`/`static`):
- mode 0 : S3 1H 2H S2 1L 2L S1 3H 4H S0 3L 4L (12 entrees)
- mode 1, `bg3_priority == false` : S3 1H 2H S2 1L 2L S1 3H S0 3L (10) ; `bg3_priority == true` : 3H S3 1H 2H S2 1L 2L S1 S0 3L (10)
- modes 2, 3, 4, 5 : S3 1H S2 2H S1 1L S0 2L (8)
- mode 6 : S3 1H S2 S1 1L S0 (6) -> (4,3) (0,1) (4,2) (4,1) (0,0) (4,0)
- mode 7, `extbg == false` : S3 S2 S1 1 S0 -> (4,3) (4,2) (4,1) (0,0) (4,0) ; `extbg == true` : S3 S2 2H S1 1 S0 2L -> (4,3) (4,2) (1,1) (4,1) (0,0) (4,0) (1,0)
`bg3_priority` ne compte qu'en mode 1 ; `extbg` ne compte qu'en mode 7. Le BACKDROP n'est pas dans les tables (toujours en dernier).""",
  tests=[("bpp_table", "`bpp(0,*)==[2,2,2,2]` ; `bpp(1,*)==[4,4,2,0]` ; `bpp(2,*)==[4,4,0,0]` ; `bpp(3,*)==[8,4,0,0]` ; `bpp(4,*)==[8,2,0,0]` ; `bpp(5,*)==[4,2,0,0]` ; `bpp(6,*)==[4,0,0,0]` ; `bpp(7,*)==[8,0,0,0]` (pour bg 0..4) ; `bpp(8,0)==0` ; `bpp(0,4)==0`."),
         ("order_mode0", "`priority_order(0,false,false)==[(4,3),(0,1),(1,1),(4,2),(0,0),(1,0),(4,1),(2,1),(3,1),(4,0),(2,0),(3,0)]`."),
         ("order_mode1", "`priority_order(1,false,false)==[(4,3),(0,1),(1,1),(4,2),(0,0),(1,0),(4,1),(2,1),(4,0),(2,0)]` ; `priority_order(1,true,false)==[(2,1),(4,3),(0,1),(1,1),(4,2),(0,0),(1,0),(4,1),(4,0),(2,0)]`."),
         ("order_modes_2_to_5", "pour m in 2..=5 : `priority_order(m,false,false)==[(4,3),(0,1),(4,2),(1,1),(4,1),(0,0),(4,0),(1,0)]` (et identique avec `bg3_priority` vrai)."),
         ("order_mode6", "`priority_order(6,false,false)==[(4,3),(0,1),(4,2),(4,1),(0,0),(4,0)]`."),
         ("order_mode7", "`priority_order(7,false,false)==[(4,3),(4,2),(4,1),(0,0),(4,0)]` ; `priority_order(7,false,true)==[(4,3),(4,2),(1,1),(4,1),(0,0),(4,0),(1,0)]`."),
         ("bg3_priority_only_mode1", "`priority_order(0,true,false)==priority_order(0,false,false)` ; `priority_order(2,true,false)==priority_order(2,false,false)` ; `priority_order(1,true,false)!=priority_order(1,false,false)`."),
         ("types", "`Pix{rgb:1,prio:2,obj_pal:3}` et `Pick{rgb:1,layer:5,obj_pal:0}` se construisent, sont `Copy` et comparables ; `LAYER_OBJ==4`, `LAYER_BACKDROP==5`.")])

T("E05-02", "ppu/bg.rs: indices de palette et couleur directe",
  size=90,
  context="Fonctions pures de couleur (spec 05b section 3). Le `rgb` final d'un pixel est lu en CGRAM par l'appelant avec l'index calcule ici. " + CHAIN,
  embed=["05b_PPU_RENDER.md#3. Palette et couleur directe"],
  write=[SRC + "/ppu/bg.rs", TST + "/e05_02_palette.rs"],
  spec="""Ajoute a `ppu/bg.rs`:
```rust
pub fn bg_palette_index(mode: u8, bg: usize, bpp: u8, pal: u8, i: u8) -> u8;   // index CGRAM 0..=255 d'un pixel BG d'indice i != 0
pub fn obj_palette_index(pal: u8, i: u8) -> u8;                                // 128 + pal * 16 + i
pub fn direct_color_bg(i: u8, pal: u8) -> u16;                                 // BGR555, BG1 8 bpp couleur directe (modes 3/4)
pub fn direct_color_m7(i: u8) -> u16;                                          // BGR555, mode 7 BG1 couleur directe
```
`bg_palette_index` : 2 bpp -> si `mode == 0` : `bg * 32 + pal * 4 + i`, sinon `pal * 4 + i` ; 4 bpp -> `pal * 16 + i` ; 8 bpp -> `i` (pal ignore). Calcul en `u32`/`wrapping`, resultat tronque a `u8` (les cas valides ne depassent pas 255).
`direct_color_bg(i, p)` : `r5 = ((i & 7) << 2) | ((p & 1) << 1)` ; `g5 = (((i >> 3) & 7) << 2) | (p & 2)` ; `b5 = (((i >> 6) & 3) << 3) | (p & 4)` ; resultat `r5 | g5 << 5 | b5 << 10`.
`direct_color_m7(i)` : `r5 = (i & 7) << 2` ; `g5 = ((i >> 3) & 7) << 2` ; `b5 = (i >> 6) << 3` ; meme assemblage.""",
  tests=[("bg_palette_2bpp", "`bg_palette_index(0,2,2,3,2)==78` (2*32+3*4+2) ; `bg_palette_index(0,0,2,0,1)==1` ; `bg_palette_index(0,3,2,7,3)==96+28+3=127` ; `bg_palette_index(1,2,2,3,2)==14` ; `bg_palette_index(5,1,2,7,3)==31`."),
         ("bg_palette_4bpp_8bpp", "`bg_palette_index(1,0,4,5,9)==89` ; `bg_palette_index(2,1,4,7,15)==127` ; `bg_palette_index(3,0,8,5,200)==200` ; `bg_palette_index(3,0,8,0,1)==1`."),
         ("obj_palette", "`obj_palette_index(7,15)==255` ; `obj_palette_index(0,1)==129` ; `obj_palette_index(4,1)==193` ; `obj_palette_index(0,15)==143`."),
         ("direct_bg", "`direct_color_bg(0x00,0)==0` ; `(0x07,0)==0x001C` ; `(0x38,0)==0x0380` ; `(0xC0,0)==0x6000` ; `(0x01,1)==6` ; `(0x00,2)==0x0040` ; `(0x00,4)==0x1000` ; `(0xFF,7)==0x73DE` ; `(0x47,3)==0x205E`."),
         ("direct_m7", "`direct_color_m7(0x00)==0` ; `(0x07)==0x001C` ; `(0x38)==0x0380` ; `(0xC0)==0x6000` ; `(0xFF)==0x639C` ; `(0x47)==0x1C | (0 << 5) | (8 << 10)==0x201C`.")])

T("E05-03", "ppu/bg.rs: decode_row (tuiles 2/4/8 bpp)",
  size=70,
  context="Decodage d'une rangee de 8 pixels d'une tuile (spec 05b section 4). Fonction pure sur un tableau de mots VRAM. " + CHAIN,
  embed=["05b_PPU_RENDER.md#4. Format des tuiles (fonction pure `decode_row`)"],
  write=[SRC + "/ppu/bg.rs", TST + "/e05_03_decode_row.rs"],
  spec="""Ajoute a `ppu/bg.rs`:
```rust
pub fn decode_row(vram: &[u16], row_word_addr: u16, bpp: u8) -> [u8; 8];
```
`a = row_word_addr`. `m0 = vram[a & 0x7FFF]` (octet bas = plan 0, octet haut = plan 1). Si `bpp >= 4`: `m1 = vram[(a + 8) & 0x7FFF]` (plans 2 et 3). Si `bpp == 8`: `m2 = vram[(a + 16) & 0x7FFF]` (plans 4 et 5) et `m3 = vram[(a + 24) & 0x7FFF]` (plans 6 et 7). Pixel `px` (0 = gauche): bit `7 - px` de chaque plan ; indice = plan0 | plan1 << 1 | plan2 << 2 | ... Les plans non lus valent 0. `bpp` autre que 2/4/8: panic avec un message.""",
  tests=[("bpp2_pattern", "`vram[0] = 0xCCAA` (plan0 = 0xAA, plan1 = 0xCC) : `decode_row(&vram,0,2)==[3,2,1,0,3,2,1,0]`."),
         ("bpp2_zero_and_full", "`vram[0]=0x0000` -> [0;8] ; `vram[0]=0xFFFF` -> [3;8] ; `vram[0]=0x0080` -> [1,0,0,0,0,0,0,0] ; `vram[0]=0x0001` -> [0,0,0,0,0,0,0,1] ; `vram[0]=0x8000` -> [2,0,0,0,0,0,0,0]."),
         ("bpp4_planes", "`vram[0]=0xCCAA`, `vram[8]=0x00F0` (plan2 = 0xF0, plan3 = 0) : `decode_row(&vram,0,4)==[7,6,5,4,3,2,1,0]` ; avec `vram[8]=0xFFF0` : `[15,14,13,12,11,10,9,8]`."),
         ("bpp8_planes", "`vram` entierement a 0 sauf `vram[0]=0x0000`, `vram[16]=0x0180` (plan4 = 0x80, plan5 = 0x01), `vram[24]=0x0040` (plan6 = 0x40, plan7 = 0) : `decode_row(&vram,0,8)==[16,64,0,0,0,0,0,32]`."),
         ("row_offset", "`vram[0x0100]=0x0080` : `decode_row(&vram,0x0100,2)[0]==1` ; `decode_row(&vram,0x0101,2)==[0;8]`. En 4 bpp, les plans 2/3 sont lus a `row + 8` : `vram[0x0108]=0x0080` -> `decode_row(&vram,0x0100,4)[0]==4`."),
         ("wrap_at_end", "`vram` de 0x8000 mots : `vram[0x0007]=0x0080` et `a=0x7FFF` en 4 bpp -> les plans 2/3 viennent de `(0x7FFF+8)&0x7FFF == 7` : `decode_row(&vram,0x7FFF,4)[0]==4`."),
         ("bad_bpp", "`#[should_panic]` : `decode_row(&vram,0,3)`.")])

T("E05-04", "ppu/bg.rs: entree de tilemap et adresse (map_entry_addr)",
  size=80,
  context="Fonctions pures de la spec 05b section 5: decodage d'une entree de 16 bits et adresse d'une entree dans une carte de 32x32, 64x32, 32x64 ou 64x64 tuiles. " + CHAIN,
  embed=["05b_PPU_RENDER.md#5. Tilemap (fonction pure `map_entry_addr`)"],
  write=[SRC + "/ppu/bg.rs", TST + "/e05_04_tilemap.rs"],
  spec="""Ajoute a `ppu/bg.rs`:
```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub struct TileEntry { pub tile: u16, pub pal: u8, pub prio: u8, pub hflip: bool, pub vflip: bool }
pub fn decode_entry(w: u16) -> TileEntry;     // bit15 V, bit14 H, bit13 P (prio 0/1), bits 12-10 palette, bits 9-0 tuile
pub fn map_entry_addr(map_base: u16, x2: bool, y2: bool, tx: u16, ty: u16) -> u16;
```
`map_entry_addr` : `tx &= if x2 { 63 } else { 31 }` ; `ty &= if y2 { 63 } else { 31 }` ; `scr = ((tx >> 5) & (x2 as u16)) + ((ty >> 5) & (y2 as u16)) * (if x2 { 2 } else { 1 })` ; `addr = (map_base + scr * 0x400 + (ty & 31) * 32 + (tx & 31)) & 0x7FFF` (calcul en `u32` puis masque).""",
  tests=[("decode_entry", "`decode_entry(0xFFFF)`: tile 0x3FF, pal 7, prio 1, hflip, vflip vrais. `decode_entry(0x2000)`: prio 1, tile 0, pal 0, flips faux. `decode_entry(0x0C00)`: pal 3. `decode_entry(0x1400)`: pal 5. `decode_entry(0x03FF)`: tile 0x3FF, prio 0. `decode_entry(0x4001)`: hflip vrai, vflip faux, tile 1. `decode_entry(0x8001)`: vflip vrai, hflip faux."),
         ("map32x32", "base 0x1000, x2=false, y2=false : `(3,5) -> 0x10A3` ; `(35,5) -> 0x10A3` (tx masque) ; `(0,0) -> 0x1000` ; `(31,31) -> 0x13FF` ; `(3,37) -> 0x10A3` (ty masque)."),
         ("map64x32", "x2=true, y2=false : `(35,5) -> 0x14A3` ; `(64,5) -> 0x10A0` (wrap a 64) ; `(31,5) -> 0x10BF` ; `(32,5) -> 0x14A0`."),
         ("map32x64", "x2=false, y2=true : `(3,37) -> 0x14A3` (ecran 1 sous l'ecran 0) ; `(3,5) -> 0x10A3` ; `(35,37) -> 0x14A3` (tx masque a 31)."),
         ("map64x64", "x2=true, y2=true : `(35,37) -> 0x1CA3` (ecran 3) ; `(35,5) -> 0x14A3` (ecran 1) ; `(3,37) -> 0x18A3` (ecran 2) ; `(63,63) -> 0x1FFF`."),
         ("addr_wraps_15_bits", "base 0x7C00, x2=y2=true, `(63,63)` -> `0x0BFF` (masque 0x7FFF)."),
         ("tx_beyond_127", "tx est deja en unites de tuile mais peut depasser 63 (position en pixels / 8 jusqu'a 127) : base 0, x2=false : `(127,0) -> 31` ; x2=true : `(100,0) -> (100 & 63 = 36) -> 0x400 + 4 = 0x404`.")])

T("E05-05", "ppu/bg.rs: bg_pixel (tuiles 8x8, defilement, palette)",
  size=140,
  context="""Premier rendu d'un fond: `Ppu::bg_pixel(n, x, v)` pour les modes 0-6 (la partie mode 7 est en E05-12/13; en mode 7 cette fonction renvoie None).
Cette tache ne gere QUE les tuiles 8x8 SANS flips (E05-06), SANS mosaique (E05-07), SANS offset-per-tile (E05-08). `v` est le compteur de ligne du PPU (1 = premiere ligne visible): `py = v + vofs`, donc un jeu met VOFS = -1 pour afficher la rangee 0 de sa carte a la premiere ligne. """ + CHAIN + " " + HELP,
  embed=["05b_PPU_RENDER.md#7. Fonds (modes 0-6, hors hi-res)"],
  write=[SRC + "/ppu/bg.rs", TST + "/e05_05_bg_pixel.rs"],
  spec="""Ajoute a `ppu/bg.rs` (sous-module de `ppu`: `impl Ppu` y accede aux champs prives `regs`, `scroll`, `vram`, `cgram`, `m7`):
```rust
impl Ppu { pub fn bg_pixel(&self, n: usize, x: usize, v: usize) -> Option<Pix>; }
```
Algorithme (n = 0..=3 pour BG1..BG4 ; `mode = regs.bg_mode`) :
1. `bpp = bpp(mode, n)` ; si `bpp == 0` ou `mode == 7` -> `None`.
2. `px = (x + scroll.hofs(n) as usize) & 0x3FF` ; `py = (v + scroll.vofs(n) as usize) & 0x3FF`.
3. `tx = px >> 3 ; ty = py >> 3` ; `entry = decode_entry(vram_word(map_entry_addr(regs.map_base[n], regs.map_x2[n], regs.map_y2[n], tx, ty)))`.
4. `fx = px & 7 ; fy = py & 7` ; `words = 8 / 16 / 32` pour bpp 2 / 4 / 8 ; `tile_word = (regs.chr_base[n] + entry.tile * words) & 0x7FFF` ; `row = (tile_word + fy) & 0x7FFF` ; `i = decode_row(vram, row, bpp)[fx]` ; si `i == 0` -> `None`.
5. `rgb` : si `regs.direct_color && n == 0 && bpp == 8 && (mode == 3 || mode == 4)` : `direct_color_bg(i, entry.pal)` ; sinon `cgram[bg_palette_index(mode, n, bpp, entry.pal, i)]`.
6. `Some(Pix { rgb, prio: entry.prio, obj_pal: 0 })`.""",
  tests=[("scene_mode1_basic", "Scene A (mode 1, BG1 4 bpp) : `write(0x05,0x01)` ; `write(0x07,0x04)` (carte a 0x400, 32x32) ; `write(0x0B,0x01)` (chr BG1 a 0x1000) ; tuile 2 a 0x1020 : `put_vram(0x1020,0x0080)` (rangee 0, pixel 0 = indice 1), `put_vram(0x1021,0x8000)` (rangee 1, pixel 0 = indice 2) ; `put_vram(0x400,0x0402)` (tuile 2, pal 1, prio 0) ; `put_vram(0x401,0x2402)` (prio 1) ; `put_cgram(p,17,0x1234)` ; `put_cgram(p,18,0x2345)`. Attendu : `bg_pixel(0,0,0)==Some(Pix{rgb:0x1234,prio:0,obj_pal:0})` ; `bg_pixel(0,1,0)==None` ; `bg_pixel(0,0,1)==Some(Pix{rgb:0x2345,prio:0,obj_pal:0})` ; `bg_pixel(0,8,0)==Some(Pix{rgb:0x1234,prio:1,obj_pal:0})`."),
         ("vofs_minus_one", "Scene A + `set_scroll(&mut p,0x0E,0x3FF)` (VOFS BG1 = -1) : `bg_pixel(0,0,1)==Some(Pix{rgb:0x1234,prio:0,obj_pal:0})` (py = (1+1023)&1023 = 0) ; `bg_pixel(0,0,2)==Some(Pix{rgb:0x2345,..})` (py = 1)."),
         ("hofs_scroll", "Scene A + `set_scroll(&mut p,0x0D,8)` : `bg_pixel(0,0,0)==Some(Pix{rgb:0x1234,prio:1,obj_pal:0})` (px = 8 -> entree (1,0)) ; `bg_pixel(0,8,0)` = entree (2,0) (vide) -> `None`."),
         ("map_64_wide", "Scene A + `write(0x07,0x05)` (64 de large) + `put_vram(0x800,0x0402)` + `set_scroll(&mut p,0x0D,256)` : `bg_pixel(0,0,0)==Some(Pix{rgb:0x1234,prio:0,obj_pal:0})` (px = 256 -> tx = 32 -> ecran 1 a 0x800)."),
         ("unused_layers", "Scene A : `bg_pixel(3,0,0)==None` (BG4 en mode 1 : bpp 0) ; `bg_pixel(2,0,0)==None` (BG3 2 bpp : rien dans la VRAM) ; avec `write(0x05,0x07)` (mode 7) : `bg_pixel(0,0,0)==None`."),
         ("mode0_bg3_palette", "Scene B (mode 0, BG3 2 bpp) : `write(0x05,0x00)` ; `write(0x09,0x08)` (carte BG3 a 0x800) ; `write(0x0C,0x02)` (chr BG3 a 0x2000) ; `put_vram(0x800,0x0801)` (tuile 1, pal 2) ; `put_vram(0x2008,0x0080)` ; `put_cgram(p,73,0x7FFF)` (index 2*32 + 2*4 + 1) : `bg_pixel(2,0,0)==Some(Pix{rgb:0x7FFF,prio:0,obj_pal:0})`."),
         ("mode3_bg1_8bpp", "Scene C (mode 3) : `write(0x05,0x03)` ; `write(0x07,0x04)` ; `write(0x0B,0x00)` ; `put_vram(0x400,0x0C01)` (tuile 1, pal 3) ; tuile 1 a `32` : `put_vram(32,0x8080)` (plans 0 et 1), `put_vram(40,0x0080)` (plan 2), `put_vram(56,0x0080)` (plan 6) -> indice 1+2+4+64 = 71 ; `put_cgram(p,71,0x4321)` : `bg_pixel(0,0,0)==Some(Pix{rgb:0x4321,prio:0,obj_pal:0})` (la palette de tuile est ignoree en 8 bpp)."),
         ("mode3_direct_color", "Scene C + `write(0x30,0x01)` (CGWSEL bit 0) : `bg_pixel(0,0,0)==Some(Pix{rgb:0x205E,prio:0,obj_pal:0})` (direct_color_bg(71, 3)).")])

T("E05-06", "ppu/bg.rs: tuiles 16x16 et retournements (flips)",
  size=100,
  context="Complete `bg_pixel` (E05-05) avec les flips H/V de l'entree et les tuiles 16x16 (BGMODE bits 7-4). " + CHAIN + " " + HELP,
  embed=["05b_PPU_RENDER.md#5. Tilemap (fonction pure `map_entry_addr`)"],
  write=[SRC + "/ppu/bg.rs", TST + "/e05_06_bg_flips.rs"],
  spec="""Dans `Ppu::bg_pixel` :
- Tuile 8x8 (`!regs.tile16[n]`) : `tx = px >> 3`, `ty = py >> 3` ; `fx = (px & 7) ^ (if hflip { 7 } else { 0 })` ; `fy = (py & 7) ^ (if vflip { 7 } else { 0 })` ; numero de tuile = `entry.tile`.
- Tuile 16x16 (`regs.tile16[n]`) : `tx = px >> 4`, `ty = py >> 4` ; `xs = (((px >> 3) & 1) as u16) ^ (hflip as u16)` ; `ys = (((py >> 3) & 1) as u16) ^ (vflip as u16)` ; numero = `(entry.tile + xs + ys * 16) & 0x3FF` ; `fx`, `fy` comme pour 8x8 (a partir de `px & 7`, `py & 7`).
Le reste (palette, priorite, direct color) est inchange. L'entree est lue a `map_entry_addr(.., tx, ty)`.""",
  tests=[("scene_16x16", "Scene D (mode 1, BG1 16x16, 4 bpp) : `write(0x05,0x11)` ; `write(0x07,0x04)` ; `write(0x0B,0x01)` ; `put_vram(0x400,0x0402)` (entree (0,0) : tuile 2, pal 1) ; tuile 2 (0x1020) rangee 0 : `put_vram(0x1020,0x0080)` (indice 1 pixel 0) ; tuile 3 (0x1030) : `put_vram(0x1030,0x8000)` (indice 2) ; tuile 18 (0x1120) : `put_vram(0x1120,0x8080)` (indice 3) ; `put_cgram(p,17,0x0011)`, `(p,18,0x0022)`, `(p,19,0x0033)`. Attendu (v=0 -> py=0) : `bg_pixel(0,0,0)` rgb 0x0011 (tuile 2) ; `bg_pixel(0,8,0)` rgb 0x0022 (sous-tuile droite = tuile 3) ; `bg_pixel(0,0,8)` rgb 0x0033 (sous-tuile basse = tuile 18) ; `bg_pixel(0,1,0)==None`."),
         ("hflip_16x16", "Scene D avec entree 0x4402 (H flip) : `bg_pixel(0,15,0)` rgb 0x0011 (xs = 1^1 = 0 -> tuile 2, fx = 7^7 = 0) ; `bg_pixel(0,7,0)` rgb 0x0022 (xs = 0^1 = 1 -> tuile 3, fx = 0) ; `bg_pixel(0,0,0)==None` (tuile 3, fx = 7)."),
         ("vflip_16x16", "Scene D avec entree 0x8402 (V flip) : `bg_pixel(0,0,15)` rgb 0x0011 (ys = 1^1 = 0 -> tuile 2, fy = 7^7 = 0 mais la rangee 0 : py=15 -> fy = (15&7)^7 = 0) ; `bg_pixel(0,0,7)` rgb 0x0033 (ys = 0^1 = 1 -> tuile 18, fy = 7^7 = 0)."),
         ("flips_8x8", "Scene A de E05-05 avec entree (0,0) = 0x4402 (H flip, 8x8) : `bg_pixel(0,7,0)` rgb 0x1234 et `bg_pixel(0,0,0)==None`. Avec 0x8402 (V flip) : `bg_pixel(0,0,7)` rgb 0x1234 (fy = 7^7 = 0) et `bg_pixel(0,0,0)==None` (fy = 7)."),
         ("both_flips_8x8", "entree 0xC402 (H et V) : `bg_pixel(0,7,7)` rgb 0x1234 ; `bg_pixel(0,0,0)==None`."),
         ("tile_number_wrap_16x16", "entree avec tuile 0x3F0 (16x16, ys=1 -> + 16) : le numero utilise est `(0x3F0 + 16) & 0x3FF == 0` (tuile 0) : avec des donnees de tuile 0 non nulles a 0x1000 (`put_vram(0x1000,0x0080)`), `bg_pixel(0,0,8)` renvoie Some (indice 1)."),
         ("tile16_per_bg", "Scene D + BG2 en 8x8 : `write(0x08,0x08)` (carte BG2 a 0x800) ; `write(0x0B,0x11)` (chr BG1 et BG2 a 0x1000) ; `put_vram(0x801,0x0402)` (entree BG2 (1,0) : tuile 2, pal 1). `bg_pixel(1,8,0)` rgb 0x0011 (BG2 8x8 : px = 8 -> tuile 2 pixel 0) alors que `bg_pixel(0,8,0)` rgb 0x0022 (BG1 16x16 : sous-tuile droite = tuile 3).")])

T("E05-07", "ppu/bg.rs: mosaique",
  size=90,
  context="Mosaique des fonds (spec 05b section 9): fonction pure + branchement dans `bg_pixel`. Pas de mosaique sur les sprites. " + CHAIN + " " + HELP,
  embed=["05b_PPU_RENDER.md#9. Mosaique"],
  write=[SRC + "/ppu/bg.rs", TST + "/e05_07_mosaic.rs"],
  spec="""Ajoute a `ppu/bg.rs`:
```rust
pub fn mosaic_xy(size: u8, enabled: bool, x: usize, v: usize) -> (usize, usize);
```
`size` = `regs.mosaic_size()` (1..=16). Si `enabled && size > 1` : `xm = x - (x % size)` ; `vm = v - (v.saturating_sub(1) % size)` ; sinon `(x, v)`. (Les lignes visibles commencent a `v = 1`: avec size 4 les lignes 1-4 donnent vm = 1, les lignes 5-8 vm = 5.)
Dans `Ppu::bg_pixel`, avant le calcul de `px`/`py`: `(xm, vm) = mosaic_xy(regs.mosaic_size(), regs.mosaic_en >> n & 1 != 0, x, v)` et utiliser `xm`, `vm` a la place de `x`, `v`.""",
  tests=[("mosaic_xy_disabled", "`mosaic_xy(4,false,6,6)==(6,6)` ; `mosaic_xy(1,true,6,6)==(6,6)`."),
         ("mosaic_xy_size4", "`mosaic_xy(4,true,6,6)==(4,5)` ; x : 0..=3 -> 0, 4..=7 -> 4 ; v : 1..=4 -> 1, 5..=8 -> 5, 9 -> 9 : `mosaic_xy(4,true,0,1)==(0,1)`, `(3,4)->(0,1)`, `(7,5)->(4,5)`, `(7,8)->(4,5)`, `(8,9)->(8,9)`."),
         ("mosaic_xy_size16", "`mosaic_xy(16,true,255,224)` : xm = 255 - 15 = 240 ; vm = 224 - (223 % 16 = 15) = 209 -> `(240,209)`. `mosaic_xy(2,true,1,1)==(0,1)`, `mosaic_xy(2,true,1,2)==(0,1)`, `mosaic_xy(2,true,1,3)==(0,3)`."),
         ("mosaic_xy_v0", "`mosaic_xy(4,true,0,0)==(0,0)` (v = 0 : pas de panique)."),
         ("bg_pixel_mosaic_x", "Scene A de E05-05 + `write(0x06,0x11)` (taille 2, BG1) : `bg_pixel(0,1,0)` = `Some(Pix{rgb:0x1234,..})` (xm = 0 -> pixel 0 de la tuile) alors que sans mosaique `bg_pixel(0,1,0)==None`."),
         ("bg_pixel_mosaic_v", "Scene A + `write(0x06,0x11)` : `bg_pixel(0,0,2)` rgb 0x2345 (vm = 2 - (1 % 2) = 1 -> rangee 1 : indice 2) alors que sans mosaique `bg_pixel(0,0,2)==None` (rangee 2) ; `bg_pixel(0,1,1)` rgb 0x2345 (xm = 0, vm = 1)."),
         ("mosaic_per_bg", "Scene A + `write(0x06,0x12)` (taille 2, active pour BG2 seulement, bit 1) : `bg_pixel(0,1,0)==None` (BG1 non concerne)."),
         ("mosaic_size1_noop", "`write(0x06,0x01)` (taille 1, BG1 actif) : identique a l'absence de mosaique.")])

T("E05-08", "ppu/bg.rs: offset-per-tile (modes 2 et 4)",
  size=130,
  context="""Offset-per-tile: BG3 fournit des offsets de defilement par colonne de 8 pixels pour BG1/BG2 (spec 05b 7.1). Cette tache gere les modes 2 et 4 (fonds non hi-res). Le mode 6 (hi-res) est traite en E05-17.
La fonction renvoie `(hofs_use, vofs_use)` utilises par `bg_pixel` a la place de `scroll.hofs(n)` / `scroll.vofs(n)`. """ + CHAIN + " " + HELP,
  embed=["05b_PPU_RENDER.md#7.1 Offset-per-tile (modes 2, 4, 6; couche BG1 ou BG2)"],
  write=[SRC + "/ppu/bg.rs", TST + "/e05_08_opt.rs"],
  spec="""Ajoute a `ppu/bg.rs`:
```rust
impl Ppu { fn opt_offsets(&self, n: usize, xm: usize) -> (u16, u16); }     // prive
```
`hofs_use = scroll.hofs(n)` et `vofs_use = scroll.vofs(n)` par defaut. Si `mode` est 2 ou 4 et `n <= 1` :
```
n_col = (xm + (hofs(n) & 7)) >> 3                    // 0 = tuile partielle de gauche, jamais modifiee
si n_col >= 1:
    ex = (n_col - 1) + (hofs(2) >> 3) ; ey = vofs(2) >> 3        // coordonnees dans la carte de BG3 (tuiles 8x8)
    e(tx,ty) = vram_word(map_entry_addr(map_base[2], map_x2[2], map_y2[2], tx, ty))     // mot BRUT
    mode 2 : eh = e(ex, ey) ; ev = e(ex, ey + 1)
    mode 4 : e0 = e(ex, ey) ; si e0 & 0x8000 == 0 { eh = e0 ; ev = 0 } sinon { eh = 0 ; ev = e0 }
    bit = 0x2000 si n == 0 (BG1), 0x4000 si n == 1 (BG2)
    si eh & bit != 0 : hofs_use = (eh & 0x3F8) | (hofs(n) & 7)
    si ev & bit != 0 : vofs_use = ev & 0x3FF
```
Dans `bg_pixel`, apres la mosaique : `(hofs_use, vofs_use) = opt_offsets(n, xm)` et `px = (xm + hofs_use) & 0x3FF`, `py = (vm + vofs_use) & 0x3FF`. Hors modes 2/4, `opt_offsets` renvoie les scrolls normaux.""",
  tests=[("mode2_both_offsets", "Scene E (mode 2, BG1 4 bpp) : `write(0x05,0x02)` ; `write(0x07,0x04)` ; `write(0x0B,0x01)` ; `write(0x09,0x08)` (carte BG3 a 0x800) ; tuile 2 (0x1020) rangee 0 : `put_vram(0x1020,0x0080)` ; entree BG1 (3,1) a 0x423 : `put_vram(0x423,0x0402)` ; `put_cgram(p,17,0x1234)` ; entrees BG3 : `put_vram(0x800,0x2010)` (horizontal : bit 13 + 0x10) et `put_vram(0x820,0x2005)` (vertical : rangee suivante de la carte BG3, valeur 5). Attendu : `bg_pixel(0,8,3)==Some(Pix{rgb:0x1234,prio:0,obj_pal:0})` (colonne 1 : hofs_use = 16, vofs_use = 5 -> px = 24, py = 8 -> tuile (3,1))."),
         ("mode2_first_column_unmodified", "Scene E : `bg_pixel(0,0,3)==None` (n_col = 0 : pas d'offset ; entree (0,0) de BG1 vide) ; `bg_pixel(0,7,3)==None`."),
         ("mode2_horizontal_only", "Scene E avec `put_vram(0x800,0x0010)` (pas de bit 13 : horizontal ignore) : `bg_pixel(0,8,3)==None` (px = 8, py = 3 + 5 = 8 -> entree (1,1) vide)."),
         ("mode2_wrong_enable_bit", "Scene E avec entrees BG3 `0x4010` et `0x4005` (bit 14 = BG2 seulement) : BG1 inchange -> `bg_pixel(0,8,3)==None`."),
         ("mode2_fine_hofs_kept", "Scene E + `set_scroll(&mut p,0x0D,3)` : `bg_pixel(0,5,3)==Some(Pix{rgb:0x1234,..})` (n_col = (5+3)>>3 = 1 ; hofs_use = 0x10 | 3 = 19 ; px = 5 + 19 = 24)."),
         ("mode2_bg3_hofs_shifts_column", "Scene E mais les entrees BG3 en colonne 1 : `put_vram(0x800,0x0000)`, `put_vram(0x801,0x2010)`, `put_vram(0x821,0x2005)` ; `set_scroll(&mut p,0x11,8)` (HOFS BG3 = 8 -> ex = 0 + 1) : `bg_pixel(0,8,3)==Some(Pix{rgb:0x1234,..})`. Sans le scroll de BG3, `bg_pixel(0,8,3)==None`."),
         ("mode4_horizontal", "Scene F (mode 4, BG2 2 bpp) : `write(0x05,0x04)` ; `write(0x08,0x0C)` (carte BG2 a 0xC00) ; `write(0x0B,0x20)` (chr BG2 a 0x2000) ; `write(0x09,0x08)` ; `put_vram(0x2008,0x0080)` (tuile 1 rangee 0) ; `put_cgram(p,1,0x0AAA)` ; `put_vram(0xC23,0x0001)` (entree BG2 (3,1)) ; `put_vram(0x800,0x4010)` (bit 15 = 0 -> horizontal, bit 14 = BG2, valeur 0x10) : `bg_pixel(1,8,8)==Some(Pix{rgb:0x0AAA,prio:0,obj_pal:0})` (px = 24, py = 8)."),
         ("mode4_vertical", "Scene F' = Scene F mais SANS l'entree (3,1) et avec `put_vram(0xC21,0x0001)` (entree (1,1)) et `put_vram(0x800,0xC005)` (bit 15 = 1 -> vertical, bit 14 = BG2, valeur 5) : `bg_pixel(1,8,3)==Some(Pix{rgb:0x0AAA,..})` (py = 3 + 5 = 8 -> ligne de tuiles 1, px = 8 inchange). Sans l'entree BG3 (`put_vram(0x800,0)`), `bg_pixel(1,8,3)==None`."),
         ("non_opt_modes", "En mode 1, avec la meme VRAM BG3 que Scene E, `bg_pixel(0,8,3)` n'est pas modifie (les scrolls normaux sont utilises) : `None` si la carte BG1 n'a rien en (1,0).")])

T("E05-09", "ppu/window.rs: fenetres (window_out) et groupes de couche",
  size=110,
  context="Logique des fenetres (spec 05b section 10), fonctions pures + extraction des reglages depuis `Regs`. Le masquage par TMW/TSW est fait dans `pick` (E05-14). " + CHAIN,
  embed=["05b_PPU_RENDER.md#10. Fenetres (fonctions pures)"],
  write=[SRC + "/ppu/window.rs", TST + "/e05_09_window.rs"],
  spec="""Dans `ppu/window.rs`:
```rust
pub fn window_out(g: u8, logic: u8, x: u8, wh: [u8; 4]) -> bool;     // algorithme de la section 10 a suivre EXACTEMENT
pub fn layer_group(regs: &Regs, layer: u8) -> u8;     // layer 0..=3 BG1..BG4, 4 OBJ, 5 couleur
pub fn layer_logic(regs: &Regs, layer: u8) -> u8;
impl Ppu { pub fn window_hides(&self, layer: u8, x: usize) -> bool; }   // window_out(group, logic, x as u8, regs.wh)
```
`layer_group` : BG1 = `win_sel[0] & 15`, BG2 = `win_sel[0] >> 4`, BG3 = `win_sel[1] & 15`, BG4 = `win_sel[1] >> 4`, OBJ = `win_sel[2] & 15`, couleur (layer 5) = `win_sel[2] >> 4`. `layer_logic` : BG1 `win_log_bg & 3`, BG2 `(win_log_bg >> 2) & 3`, BG3 `(>> 4) & 3`, BG4 `(>> 6) & 3`, OBJ `win_log_obj`, couleur `win_log_col`. Couche > 5 : groupe 0, logique 0.
`window_out(g, logic, x, wh)` : `w1 = (x >= wh[0] && x <= wh[1]) ^ (g & 1 != 0)` ; `w2 = (x >= wh[2] && x <= wh[3]) ^ (g & 4 != 0)` ; `en1 = g & 2 != 0` ; `en2 = g & 8 != 0` ; (false,false) -> false ; (true,false) -> w1 ; (false,true) -> w2 ; (true,true) -> logique 0 `w1 | w2`, 1 `w1 & w2`, 2 `w1 ^ w2`, 3 `!(w1 ^ w2)`.""",
  tests=[("window1_only", "wh = [10,20,15,25], g = 2 : `window_out(2,0,x,wh)` vrai pour x = 10, 15, 20 ; faux pour x = 9, 21, 0, 255."),
         ("window1_inverted", "g = 3 (fenetre 1 active + inversee) : faux pour x = 10, 15, 20 ; vrai pour x = 9, 21, 0, 255."),
         ("window2_only", "g = 8 : vrai pour x = 15, 25 ; faux pour x = 14, 26. g = 12 (fenetre 2 active + inversee) : faux pour 15..=25, vrai pour 14."),
         ("left_greater_than_right", "wh = [20,10,0,0], g = 2 : faux pour tout x (fenetre vide) ; g = 3 : vrai pour tout x (0, 10, 15, 20, 255)."),
         ("inactive_inversion_no_effect", "g = 1 (inversion seule, aucune fenetre active) -> faux pour tout x ; g = 5 -> faux ; g = 0 -> faux."),
         ("logic_combinations", "wh = [10,20,15,25], g = 0xA (les deux actives, sans inversion). x = 12 (w1 vrai, w2 faux) : OR vrai, AND faux, XOR vrai, XNOR faux. x = 17 (vrai, vrai) : OR vrai, AND vrai, XOR faux, XNOR vrai. x = 22 (faux, vrai) : OR vrai, AND faux, XOR vrai, XNOR faux. x = 30 (faux, faux) : OR faux, AND faux, XOR faux, XNOR vrai. (logique 0 = OR, 1 = AND, 2 = XOR, 3 = XNOR)."),
         ("logic_with_inversion", "wh = [10,20,15,25], g = 0xB (fenetre 1 inversee + les deux actives), logique 0 (OR) : x = 12 -> w1 = faux (inverse), w2 = faux -> faux ; x = 30 -> w1 vrai (inverse) -> vrai."),
         ("layer_group_extraction", "`Regs` avec `win_sel = [0xA5,0x3C,0x71]` (ecrire par `regs.write(0x23,0xA5)`, `0x24`, `0x25`) : `layer_group(&r,0)==5`, `(1)==0xA`, `(2)==0xC`, `(3)==3`, `(4)==1`, `(5)==7` ; `layer_group(&r,6)==0`."),
         ("layer_logic_extraction", "`regs.write(0x2A,0xE4)` : `layer_logic(&r,0)==0`, `(1)==1`, `(2)==2`, `(3)==3` ; `regs.write(0x2B,0xFE)` : `layer_logic(&r,4)==2`, `layer_logic(&r,5)==3`."),
         ("ppu_window_hides", "`Ppu::new`: `write(0x23,0x02)` (BG1 fenetre 1 active), `write(0x26,10)`, `write(0x27,20)` : `window_hides(0,15)==true`, `window_hides(0,9)==false`, `window_hides(1,15)==false` (BG2 : groupe 0) ; `write(0x2C,0)` ne change rien (TMW n'est pas consulte ici).")])
