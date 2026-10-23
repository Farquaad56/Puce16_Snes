# SPEC 05 - PPU (partie 1/2): memoires, registres, acces, timing dot par dot, sortie

Source: wiki SNESdev (PPU_registers, Timing, Backgrounds, Sprites, Errata) + decisions D-002 (PPU dot par dot), D-003, D-008, D-009, D-010.
Partie 2 (rendu: BG, OPT, mode 7, sprites, fenetres, color math): `05b_PPU_RENDER.md`.
Fichiers cibles: `puce16-core/src/ppu/{mod,vram,cgram,oam,dot,output}.rs` (le rendu est dans bg.rs, mode7.rs, obj.rs, window.rs, colormath.rs, voir 05b).
Aucune ROM n'est necessaire pour tester: les tests ecrivent directement dans VRAM/CGRAM/OAM via les registres.
Entiers uniquement (D-009). Code et commentaires en anglais ASCII.

## 0. Modele general (a lire d'abord)

- Le PPU est avance **dot par dot** par `Ppu::step_dot()`, appele par `scheduler.catch_up` dans l'ordre chronologique. Les composants qui ont besoin de la position (timing/NMI/IRQ/HDMA) lisent `ppu.v()` / `ppu.dot()` apres chaque `step_dot` (voir 02_BUS_TIMING.md).
- A chaque dot visible, le PPU **compose un pixel complet** (BG, sprites, fenetres, color math, luminosite) a partir de l'etat COURANT des registres et de la VRAM/CGRAM. Une ecriture de registre PPU par le CPU est donc effective a partir du dot suivant celui deja compose. C'est ce qu'on appelle "pixel-accurate" (D-002).
- Regle d'ordonnancement: le dot `d` d'une ligne est execute quand `master > debut_du_dot(d)` (strictement). Une ecriture CPU terminee a `master_end == debut_du_dot(d)` affecte donc le dot `d`. (`UNKNOWN - to confirm`: la phase exacte vis-a-vis du CPU; ajustable plus tard par un decalage constant `PPU_WRITE_PHASE` dans consts.rs, defaut 0.)
- Le PPU ne "prefetch" pas les tuiles BG au dot pres comme le vrai materiel. Les BG sont evalues pixel par pixel dans VRAM au dot courant. Les sprites sont evalues UNE fois par ligne (voir 05b section 6). `UNKNOWN - to confirm`: effets de bord de fetch sub-tuile; couverts plus tard par des ROM de test.
- Performance: un cache de tuile par BG est autorise ssi le hash du framebuffer reste identique (test dedie). Jamais avant que les tests de rendu passent.

## 1. Memoires du PPU

| Memoire | Taille | Type Rust | Notes |
|---|---|---|---|
| VRAM | 64 Kio = 32768 mots de 16 bits | `[u16; 0x8000]` | adresses en MOTS; octet bas = VMDATAL, octet haut = VMDATAH |
| CGRAM | 512 octets = 256 mots | `[u16; 256]` | mot = `0bbbbbgg gggrrrrr` (bit 15 toujours 0) |
| OAM | 544 octets | `[u8; 544]` | 0..511: table basse (4 octets x 128), 512..543: table haute (2 bits x 128) |

Etat initial (power-on ET reset): tout a 0 (le vrai materiel est aleatoire; `UNKNOWN - to confirm`, defaut 0, deterministe).

## 2. Registres PPU en ecriture ($2100-$2133)

`Ppu::write(reg, v)` avec `reg = addr & 0xFF`. Les registres en ecriture seule lus par le CPU retournent le bus ouvert du CPU (`cpu_mdr`).

| Reg | Nom | Effet exact |
|---|---|---|
| $2100 | INIDISP | bit7 `forced_blank`; bits3-0 `brightness` (0..15). Etat initial: `forced_blank = true`, brightness 0. |
| $2101 | OBJSEL | `SSSN NbBB`: `obj_size = v>>5`; `obj_name_gap = ((v>>3)&3)+1) << 12` (mots); `obj_base = (v&7) << 13` (mots). |
| $2102 | OAMADDL | `oamadd_reg = (oamadd_reg & 0x100) | v`; puis `internal_oamadd = (oamadd_reg & 0x1FF) << 1` (adresse OCTET 10 bits). |
| $2103 | OAMADDH | bit0 -> bit 8 de `oamadd_reg`; bit7 -> `obj_priority_rotation`; puis meme recalcul de `internal_oamadd`. |
| $2104 | OAMDATA | voir 2.1. |
| $2105 | BGMODE | bits2-0 `bg_mode`; bit3 `bg3_priority`; bits7-4: taille de tuile BG4..BG1 (1 = 16x16). |
| $2106 | MOSAIC | bits7-4 `mosaic_size` (taille = valeur+1), bits3-0 activation par BG (bit0 = BG1). |
| $2107-$210A | BGnSC | `map_base = (v >> 2) << 10` (mots); bit0 `map_x2` (largeur 64), bit1 `map_y2` (hauteur 64). |
| $210B | BG12NBA | `chr_base[0] = (v & 0xF) << 12`, `chr_base[1] = (v >> 4) << 12` (mots). |
| $210C | BG34NBA | `chr_base[2] = (v & 0xF) << 12`, `chr_base[3] = (v >> 4) << 12`. |
| $210D | BG1HOFS / M7HOFS | voir 2.2. |
| $210E | BG1VOFS / M7VOFS | voir 2.2. |
| $210F-$2114 | BG2HOFS, BG2VOFS, BG3HOFS, BG3VOFS, BG4HOFS, BG4VOFS | voir 2.2 (pas de mode 7). |
| $2115 | VMAIN | bits1-0: increment (0 -> 1, 1 -> 32, 2 ou 3 -> 128); bits3-2: remap (0..3); bit7: `vram_inc_on_high` (0 = increment apres VMDATAL / lecture low; 1 = apres VMDATAH / lecture high). |
| $2116 | VMADDL | `vmadd = (vmadd & 0xFF00) | v` puis prefetch (2.3). |
| $2117 | VMADDH | `vmadd = (vmadd & 0x00FF) | ((v & 0x7F) << 8)` puis prefetch. |
| $2118 | VMDATAL | voir 2.3. |
| $2119 | VMDATAH | voir 2.3. |
| $211A | M7SEL | bit0 `m7_flip_x`, bit1 `m7_flip_y`, bit6 `m7_fill` (tuile 0 hors ecran), bit7 `m7_outside` (R). |
| $211B-$211E | M7A, M7B, M7C, M7D | `m7x = ((v << 8) | mode7_latch) as i16`; `mode7_latch = v`. (M7B garde aussi `m7b_last_byte = v` pour MPY.) |
| $211F | M7X | `m7_cx = sign_extend_13(((v & 0x1F) << 8) | mode7_latch)`; `mode7_latch = v`. |
| $2120 | M7Y | idem -> `m7_cy`. |
| $2121 | CGADD | `cgram_addr = v` (mot 0..255); `cgram_byte_toggle = 0`. |
| $2122 | CGDATA | voir 2.4. |
| $2123 | W12SEL | `win_sel[0] = v` (BG1 = bits 3-0, BG2 = bits 7-4). |
| $2124 | W34SEL | `win_sel[1] = v` (BG3 = bits 3-0, BG4 = bits 7-4). |
| $2125 | WOBJSEL | `win_sel[2] = v` (OBJ = bits 3-0, couleur = bits 7-4). |
| $2126-$2129 | WH0..WH3 | `wh[0..4] = v` (WH0 = gauche fenetre 1, WH1 = droite 1, WH2 = gauche 2, WH3 = droite 2). |
| $212A | WBGLOG | `win_log_bg = v` (BG1 bits1-0, BG2 bits3-2, BG3 bits5-4, BG4 bits7-6). |
| $212B | WOBJLOG | `win_log_obj = v & 3`, `win_log_col = (v >> 2) & 3`. |
| $212C | TM | `tm = v & 0x1F` |
| $212D | TS | `ts = v & 0x1F` |
| $212E | TMW | `tmw = v & 0x1F` |
| $212F | TSW | `tsw = v & 0x1F` |
| $2130 | CGWSEL | bit0 `direct_color`; bit1 `addend_subscreen`; bits5-4 `math_region` (SS); bits7-6 `black_region` (MM). |
| $2131 | CGADSUB | bits4-0 activation couleur math par couche (bit0 BG1, 1 BG2, 2 BG3, 3 BG4, 4 OBJ), bit5 backdrop, bit6 `math_half`, bit7 `math_sub`. |
| $2132 | COLDATA | `c = v & 0x1F`; si bit5: `fixed_r = c`; bit6: `fixed_g = c`; bit7: `fixed_b = c`. (plusieurs bits possibles dans la meme ecriture.) |
| $2133 | SETINI | bit0 `interlace`; bit1 `obj_interlace`; bit2 `overscan`; bit3 `hires` (pseudo); bit6 `extbg`; bit7 ignore (ext sync). |

`sign_extend_13(x) = ((x as i16) << 3) >> 3` (x deja masque sur 13 bits).

### 2.1 OAMDATA ($2104) et lecture OAMDATAREAD ($2138)
```
a = internal_oamadd                                  // 10 bits
if a & 0x200 != 0:                                   // table haute (32 octets, miroir tous les 32)
    oam[0x200 + (a & 0x1F)] = v                      // ecriture immediate (si acces autorise, voir 3)
else:
    if a & 1 == 0: oam_latch = v                     // octet pair: memorise
    else: oam[a-1] = oam_latch; oam[a] = v           // octet impair: ecrit le mot
internal_oamadd = (a + 1) & 0x3FF
```
Lecture $2138: `a = internal_oamadd`; octet = `oam[ if a&0x200 {0x200 + (a&0x1F)} else {a} ]`; puis `internal_oamadd = (a+1) & 0x3FF`.
Rechargement: a chaque debut de VBlank (V = vblank_start, dot 0), si `!forced_blank`: `internal_oamadd = (oamadd_reg & 0x1FF) << 1`. (`UNKNOWN - to confirm`: rechargement aussi quand forced_blank retombe a 0; defaut: non.)

### 2.2 Registres de scroll
Latchs partages: `bgofs_latch: u8` (tous les BGnHOFS/VOFS), `bghofs_latch: u8` (BGnHOFS seulement), `mode7_latch: u8` (M7*).
```
BGnHOFS write v:  hofs[n] = ((v << 8) | (bgofs_latch & !7) | (bghofs_latch & 7)) & 0x3FF
                  bgofs_latch = v ; bghofs_latch = v
BGnVOFS write v:  vofs[n] = ((v << 8) | bgofs_latch) & 0x3FF
                  bgofs_latch = v
```
$210D en plus: `m7_hofs = sign_extend_13((v << 8) | mode7_latch)`; `mode7_latch = v`. $210E: `m7_vofs` idem. (Les deux ecritures - BG1 et mode 7 - ont lieu a chaque ecriture de $210D / $210E.)

### 2.3 VRAM
```
fn remap(a: u16) -> u16:           // VMAIN bits 3-2
    0: a
    1: (a & 0xFF00) | ((a & 0x00E0) >> 5) | ((a & 0x001F) << 3)
    2: (a & 0xFE00) | ((a & 0x01C0) >> 6) | ((a & 0x003F) << 3)
    3: (a & 0xFC00) | ((a & 0x0380) >> 7) | ((a & 0x007F) << 3)
```
Toute lecture/ecriture de VRAM via les registres utilise `ra = remap(vmadd) & 0x7FFF`.
- Prefetch (ecriture de VMADDL ou VMADDH): `vram_latch = vram[remap(vmadd) & 0x7FFF]` (toujours, meme hors blank).
- VMDATAL ecriture: si `vram_ok()` alors `vram[ra].low = v`. Si `!vram_inc_on_high`: `vmadd = (vmadd + step) & 0x7FFF` (l'increment a lieu MEME si l'ecriture est bloquee).
- VMDATAH ecriture: idem sur l'octet haut; increment si `vram_inc_on_high`.
- Lecture $2139 (low): retourne `vram_latch & 0xFF`; si `!vram_inc_on_high`: `vram_latch = vram[ra]; vmadd += step`. Lecture $213A (high): retourne `vram_latch >> 8`; si `vram_inc_on_high`: meme rechargement et increment. (Le rechargement lit a l'adresse AVANT l'increment.)
- Lecture hors blank: `vram_latch` reste utilisable (valeur du dernier prefetch); `UNKNOWN - to confirm`: sur le vrai materiel lire pendant l'affichage actif donne 0. Defaut v1: si `!vram_ok()` la lecture retourne `0`.

### 2.4 CGRAM
```
CGDATA write v:  if cgram_byte_toggle == 0 { cgram_latch = v }
                 else { cgram[cgram_addr] = (((v & 0x7F) as u16) << 8) | cgram_latch as u16; cgram_addr = (cgram_addr + 1) & 0xFF }
                 cgram_byte_toggle ^= 1
CGDATAREAD ($213B): w = cgram[cgram_addr]
                 toggle == 0 -> byte = w & 0xFF
                 toggle == 1 -> byte = ((w >> 8) as u8 & 0x7F) | (ppu2_mdr & 0x80); cgram_addr = (cgram_addr + 1) & 0xFF
                 cgram_byte_toggle ^= 1 ; ppu2_mdr = byte
```
Les acces CGRAM sont toujours autorises dans v1 (les jeux ecrivent la CGRAM en HBlank par HDMA). `UNKNOWN - to confirm`: ecritures pendant l'affichage actif sur le vrai materiel.

## 3. Regles d'acces pendant l'affichage
```
fn vram_ok() -> bool { forced_blank || v >= vblank_start }        // V=0 NON accessible (UNKNOWN - to confirm)
fn oam_ok()  -> bool { forced_blank || v >= vblank_start }
```
- VRAM: voir 2.3. OAM: si `!oam_ok()`: les ecritures $2104 sont ignorees (l'adresse s'incremente quand meme) et la lecture $2138 retourne 0 (`UNKNOWN - to confirm`).
- `vblank_start = if overscan {240} else {225}` (lu en direct a chaque usage).

## 4. Registres PPU en lecture ($2134-$213F)

`Ppu::read(reg, cpu_mdr, wrio7) -> u8`. `ppu1_mdr` et `ppu2_mdr` = derniere valeur retournee par un registre PPU1 / PPU2 respectivement.

| Reg | Nom | Valeur retournee | Mise a jour open bus |
|---|---|---|---|
| $2134 | MPYL | `mpy & 0xFF` | `ppu1_mdr = valeur` |
| $2135 | MPYM | `(mpy >> 8) & 0xFF` | `ppu1_mdr` |
| $2136 | MPYH | `(mpy >> 16) & 0xFF` | `ppu1_mdr` |
| $2137 | SLHV | si `wrio7`: verrouille les compteurs (4.1). Retourne `cpu_mdr`. | aucune |
| $2138 | OAMDATAREAD | voir 2.1 | `ppu1_mdr` |
| $2139 | VMDATALREAD | voir 2.3 | `ppu1_mdr` |
| $213A | VMDATAHREAD | voir 2.3 | `ppu1_mdr` |
| $213B | CGDATAREAD | voir 2.4 | `ppu2_mdr` |
| $213C | OPHCT | voir 4.1 | `ppu2_mdr` |
| $213D | OPVCT | voir 4.1 | `ppu2_mdr` |
| $213E | STAT77 | `(range_over<<7) | (time_over<<6) | (ppu1_mdr & 0x10) | 0x01` (bit7 = T: plus de 32 sprites sur une ligne = `range_over`; bit6 = R: plus de 34 tranches = `time_over`; bit5 master/slave = 0; version PPU1 = 1) | `ppu1_mdr` |
| $213F | STAT78 | `(field<<7) | (counter_latched<<6) | (ppu2_mdr & 0x20) | (is_pal<<4) | 0x03` ; puis `counter_latched = false; ophct_byte = 0; opvct_byte = 0` | `ppu2_mdr` |

`mpy = (m7a as i16 as i32) * ((m7b_last_byte as i8) as i32)` (24 bits signe, calcule a la lecture).
Lecture de $2100-$2133 et de $2137: retourne `cpu_mdr` (open bus CPU).

### 4.1 Verrou des compteurs H/V
- Verrouillage (`latch_counters`): `latch_h = dot_counter_now()`, `latch_v = v`, `counter_latched = true`. `dot_counter_now()` = `self.dot` (0..339, ou 0..340 sur ligne longue); decalage `HV_LATCH_DOT_OFFSET` dans consts.rs, defaut 0 (`UNKNOWN - to confirm`).
- Declencheurs: lecture de $2137 si `wrio7`; `wrio_write(old, new)` quand le bit 7 passe de 1 a 0 (front descendant) [spec 02: WRIO.7 = /EXTLATCH].
- OPHCT ($213C): `ophct_byte == 0` -> retourne `latch_h & 0xFF`; sinon retourne `(latch_h >> 8) & 1 | (ppu2_mdr & 0xFE)`. Puis `ophct_byte ^= 1`.
- OPVCT ($213D): idem avec `latch_v` et `opvct_byte`.
- Avant tout verrouillage, `latch_h = latch_v = 0`.

## 5. Position, timing et evenements

Etat: `v: u16`, `dot: u16`, `field: bool`, `line_type`. Le PPU possede ces compteurs. `timing/mod.rs` (spec 02) les lit.

### 5.1 Longueur des lignes et des dots
```
fn line_clocks(v) -> u32:   // 1364 par defaut
    NTSC && !interlace && field && v == 240  -> 1360   // ligne courte
    PAL  &&  interlace && field && v == 311  -> 1368   // ligne longue
fn line_dots(v) -> u16:  340 (ligne courte aussi: 340 dots de 4 clocks); ligne longue: 341
fn dot_clocks(dot, line) -> u32:  5 si dot in {323, 324, 327, 328} et ligne non courte; sinon 4   // D-008 provisoire
```
`DOT_START_CLOCK[line_type][dot]` est precalcule (3 tables de 341+1 entrees); somme = `line_clocks`.

### 5.2 Frame
```
last_line = (if PAL {311} else {261}) + (if interlace && !field {1} else {0})   // UNKNOWN - to confirm quel field
```
A la fin de la derniere ligne: `v = 0`, **`field ^= 1` a chaque frame, interlace ou non** (le vrai materiel bascule toujours; STAT78 bit7 expose cette valeur).
`vis_last = if overscan {239} else {224}`. Lignes visibles: `v in 1..=vis_last`; ligne de sortie `r = v - 1`. V=0 est cachee (noire).

### 5.3 Evenements par (v, dot), dans CET ordre dans `step_dot`
1. `dot == 0`:
   - `v == 0`: debut de frame: `time_over = range_over = false`; evenement VBLANK_END.
   - `1 <= v <= vis_last`: **evaluation des sprites** pour la ligne (05b section 6) -> remplit `line_obj[256]`.
   - `v == vblank_start`: evenement VBLANK_START; rechargement OAM (2.1) si `!forced_blank`.
   - evenement LINE_START (toujours).
2. `dot == 274`: evenement HBLANK_START. `dot == 1`: HBLANK_END. (bornes `UNKNOWN - to confirm`, spec 02 3.1: 1096 clocks / 4.)
3. Si `1 <= v <= vis_last` et `22 <= dot <= 277`: `x = dot - 22`; **composer le pixel x** (05b section 11) et l'ecrire dans le framebuffer (section 6).
4. Avancer `dot += 1`; si `dot == line_dots(v)`: `dot = 0; v += 1; si v > last_line {v = 0; field ^= 1; FRAME_START}`.

`step_dot` retourne un petit ensemble de drapeaux (`DotEvents`: LINE_START, HBLANK_START, HBLANK_END, VBLANK_START, VBLANK_END) et `scheduler` gere NMI/IRQ/HDMA/auto-joypad a partir de la position (spec 02/03). Le PPU ne connait ni DMA ni NMI.

## 6. Sortie: framebuffer

- Tampon interne fixe: `[u32; 512 * 478]`, pixel XRGB8888 avec octet haut `0xFF`: `0xFF000000 | r8 << 16 | g8 << 8 | b8`.
- Conversion finale d'un canal 5 bits `c5` (apres math et luminosite): `c5b = c5 * brightness / 15` (division entiere; brightness 0..15), puis `c8 = (c5b << 3) | (c5b >> 2)` (D-010). `forced_blank` -> `0xFF000000` (noir).
- Rangee de sortie: `row = if frame_interlaced { 2 * r + field_as_u8 } else { r }`. Les rangees de l'autre field gardent leur contenu precedent (image entrelacee complete = 448 ou 478 lignes).
- Colonne: mode 256: `x` (largeur utile 256). Mode hi-res (SETINI.3 ou BG mode 5/6, voir 05b section 12): colonnes `2x` (sous-ecran) et `2x+1` (ecran principal) dans le tampon 512.
- Si la ligne n'est pas hi-res mais que la frame l'est (`frame_hires` = au moins une ligne hi-res dans la frame courante), chaque pixel est duplique sur `2x`, `2x+1` a l'export.
- `FrameView` (spec 01_ARCHITECTURE section 5) est calcule au debut de VBlank (`v == vblank_start`): `width = if frame_hires {512} else {256}`, `height = (vis_last) * (if frame_interlaced {2} else {1})`, avec `frame_interlaced = interlace` lu a ce moment. `pixels` = copie compacte de la zone utile (stride = width). `Ppu::frame()` retourne la derniere frame complete.
- Les lignes non visibles ou ligne 0: non ecrites.

## 7. Reset et power-on
- Power-on: tous les registres 0 sauf `forced_blank = true`; memoires a 0; `v = 0, dot = 0, field = false`; latchs a 0; `internal_oamadd = 0`.
- Reset bouton: memes registres PPU qu'au power-on (INIDISP force blank); VRAM/CGRAM/OAM CONSERVEES (`UNKNOWN - to confirm`).

## 8. Interface Rust (ppu/mod.rs)
```rust
pub struct Ppu { /* prive */ }
bitflags-like (sans crate): pub struct DotEvents(pub u8);   // consts LINE_START=1, HBLANK_START=2, HBLANK_END=4, VBLANK_START=8, VBLANK_END=16
impl Ppu {
    pub fn new(region: Region) -> Ppu;
    pub fn reset(&mut self);
    pub fn step_dot(&mut self) -> DotEvents;
    pub fn dot_clocks(&self) -> u32;                 // duree en clocks du dot COURANT (avant step_dot)
    pub fn v(&self) -> u16;  pub fn dot(&self) -> u16;  pub fn field(&self) -> bool;
    pub fn line_dots(&self) -> u16;                  // nombre de dots de la ligne courante
    pub fn write(&mut self, reg: u8, v: u8);         // reg 0x00..=0x33
    pub fn read(&mut self, reg: u8, cpu_mdr: u8, wrio7: bool) -> u8;   // reg 0x34..=0x3F (hors plage: cpu_mdr)
    pub fn wrio_write(&mut self, old: u8, new: u8);
    pub fn in_vblank(&self) -> bool;                 // v >= vblank_start
    pub fn in_hblank(&self) -> bool;                 // dot >= 274 || dot < 1
    pub fn frame(&self) -> FrameView<'_>;
    pub fn vram(&self) -> &[u16];  pub fn cgram(&self) -> &[u16];  pub fn oam(&self) -> &[u8];
    pub fn regs_snapshot(&self) -> PpuRegsSnapshot;  // pour le debugger (desktop)
    pub fn hash_into(&self, h: &mut Fnv64);          // tout l'etat, ordre fixe
}
```

## 9. Tests (prefixe = id de tache; ils ne chargent aucune ROM)
- Ecriture/lecture VRAM: increment 1/32/128, remap des 3 modes (cas calcules a la main), prefetch, `vram_inc_on_high`, ecriture bloquee hors blank (sauf increment).
- CGRAM: ecriture par paires, bit 15 efface, lecture avec bit7 open bus, auto-increment.
- OAM: ecriture par paires, table haute (miroir 32), `internal_oamadd` apres $2102/$2103, rechargement a VBlank.
- Scroll: sequence d'ecritures $210D deux fois -> valeur 10 bits attendue, latchs partages (tableau de cas).
- MPY: M7A=0x0100 (ecrire 0x00 puis 0x01), M7B octet 0xFF -> -256 (0xFFFF00); M7A=-1, octet 0x7F -> -127.
- Compteurs: ligne normale = 1364 clocks (somme des `dot_clocks`), longue 1368, courte 1360; nombre de lignes par frame NTSC 262 / PAL 312 / +1 en entrelace field 0; `field` bascule chaque frame.
- Evenements: VBLANK_START a v=225 (et 240 avec overscan); HBLANK_START dot 274.
- STAT78: bit4 selon la region, version 3, effacement du verrou apres lecture; latch via $2137 seulement si `wrio7`; front descendant WRIO.
- Framebuffer: forced blank -> 0xFF000000; conversion couleur (`c5=31,b=15 -> 0xFFFFFF`, `c5=31,b=0 -> 0`, `c5=1,b=15 -> 8` octet `(1<<3)|(1>>2) = 8`).

## 10. Decoupage conseille en taches (E04, 50-150 lignes chacune)
P01 consts+types+dot tables; P02 VRAM+registres VMAIN/VMADD/VMDATA; P03 CGRAM; P04 OAM+OAMDATA; P05 registres de configuration (BGMODE..SETINI, fenetres, TM/TS); P06 scroll latchs; P07 registres de lecture (MPY, STAT77/78, open bus); P08 compteurs H/V + latch; P09 `step_dot` + evenements + frame/field; P10 framebuffer + conversion couleur + FrameView. Rendu: voir 05b section 14.

## 11. UNKNOWN recapitules (valeurs par defaut ci-dessus)
Phase d'ecriture (`PPU_WRITE_PHASE`), bornes HBlank, offset de latch H, field portant la ligne en plus, VRAM accessible a V=0, lecture VRAM/OAM pendant l'affichage, OAM rechargee sur sortie de forced blank, etat initial des memoires.
