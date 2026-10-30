# SPEC 05b - PPU (partie 2/2): rendu pixel par pixel

Source: wiki SNESdev (Backgrounds, Tiles, Tilemaps, Offset_per_tile, Sprites, Mode_7_transform, Windows, Color_math, PPU_registers, Errata) + algorithmes de reference publics (bsnes) pour les formules de mode 7.
Suite de `05_PPU.md` (registres, memoires, timing). Fichiers cibles: `puce16-core/src/ppu/{bg,mode7,obj,window,colormath,output}.rs`.
Entiers uniquement. Chaque fonction "pure" (sans etat PPU) est testable seule avec des tableaux ecrits a la main.
Notation: `v` = compteur de ligne (1 = premiere ligne visible), `r = v - 1` = rangee de sortie, `x` = colonne 0..255.

## 1. Types de base
```rust
pub struct Pix { pub rgb: u16, pub prio: u8, pub obj_pal: u8 }   // rgb = BGR555 final (0bbbbbgggggrrrrr); obj_pal = palette sprite 0..7 (seulement pour OBJ)
// Couches: 0..=3 = BG1..BG4, 4 = OBJ, 5 = BACKDROP
pub struct Pick { pub rgb: u16, pub layer: u8, pub obj_pal: u8 }
```
Convention couleur: `r5 = w & 31`, `g5 = (w >> 5) & 31`, `b5 = (w >> 10) & 31`. Index 0 d'une tuile (2/4/8 bpp) = transparent (BG et OBJ), meme en couleur directe.

## 2. Modes BG: profondeur de bits par couche et tables de priorite

`BPP[mode][bg]` (0 = couche non rendue):
| Mode | BG1 | BG2 | BG3 | BG4 | Remarque |
|---|---|---|---|---|---|
| 0 | 2 | 2 | 2 | 2 | base CGRAM BGn = (n-1)*32 |
| 1 | 4 | 4 | 2 | 0 | |
| 2 | 4 | 4 | 0 | 0 | BG3 = donnees offset-per-tile (non affichee) |
| 3 | 8 | 4 | 0 | 0 | BG1 couleur directe possible |
| 4 | 8 | 2 | 0 | 0 | BG3 = OPT (un seul mot) |
| 5 | 4 | 2 | 0 | 0 | hi-res force, tuiles 16 px de large |
| 6 | 4 | 0 | 0 | 0 | hi-res force, BG3 = OPT |
| 7 | 8 | (EXTBG 7 bits) | 0 | 0 | voir section 8 |

Ordre de priorite, de l'AVANT vers l'ARRIERE. Notation: `Sp` = sprites de priorite p (0..3); `nH`/`nL` = BG n avec bit de priorite de tuile 1 / 0; en mode 7: `1` = BG1 (une seule priorite), EXTBG: `2H`/`2L` = bit 7 de l'indice.
```
Mode 0:  S3 1H 2H S2 1L 2L S1 3H 4H S0 3L 4L
Mode 1 (BGMODE bit3 = 0): S3 1H 2H S2 1L 2L S1 3H S0 3L
Mode 1 (BGMODE bit3 = 1): 3H S3 1H 2H S2 1L 2L S1 S0 3L
Mode 2,3,4,5: S3 1H S2 2H S1 1L S0 2L
Mode 6:  S3 1H S2 S1 1L S0
Mode 7:  S3 S2 S1 1 S0
Mode 7 + EXTBG: S3 S2 2H S1 1 S0 2L
```
Implementation: `const PRIORITY: [&[(u8 /*couche*/, u8 /*niveau*/)]; ...]` (niveau = priorite 0..3 pour OBJ, 0/1 pour BG; mode 6: BG1 H/L; mode 7: BG1 niveau 0 ou "tous"). Le fond (backdrop) vient toujours en dernier.
Pour une couche BG non rendue (BPP = 0), ou dont le bit TM/TS est a 0, l'entree est ignoree.

## 3. Palette et couleur directe
Base CGRAM d'un pixel BG d'indice `i != 0`, palette de tuile `p` (3 bits), couche `n` (0..3):
- 2 bpp: mode 0: `n*32 + p*4 + i`; autres modes: `p*4 + i`.
- 4 bpp: `p*16 + i`.
- 8 bpp: `i` (p ignore). Couleur directe si `direct_color` (CGWSEL bit0) et BG1 mode 3/4 (8 bpp) -> pas de CGRAM:
  `r5 = ((i & 7) << 2) | ((p & 1) << 1)`, `g5 = (((i >> 3) & 7) << 2) | (p & 2)`, `b5 = (((i >> 6) & 3) << 3) | ((p & 4))`.
- OBJ: `128 + pal*16 + i`.
- Mode 7 couleur directe (BG1 seulement): `r5 = (i & 7) << 2`, `g5 = ((i >> 3) & 7) << 2`, `b5 = (i >> 6) << 3`. EXTBG BG2: toujours indexe.
Le `rgb` du `Pix` est lu de CGRAM AU DOT courant (CGRAM change possible par HDMA).

## 4. Format des tuiles (fonction pure `decode_row`)
Une tuile 8x8 = 8 rangees. Adresse en mots: `tile_word = chr_base[n] + tile_number * words_per_tile` (masque 0x7FFF), `words_per_tile` = 8 (2 bpp), 16 (4 bpp), 32 (8 bpp).
Rangee `fy` (0..7): mot `m0 = vram[tile_word + fy]` : octet bas = plan 0, octet haut = plan 1. Si 4 bpp: `m1 = vram[tile_word + 8 + fy]` : plans 2 et 3. Si 8 bpp: plans 4,5 en `+16 + fy`, plans 6,7 en `+24 + fy`.
Pixel `px` (0 = gauche): bit `7 - px` de chaque plan; `indice = plan0 | plan1 << 1 | plan2 << 2 | ...`.
`fn decode_row(vram: &[u16], row_word_addr: u16, bpp: u8) -> [u8; 8]` (prend l'adresse du premier mot de la rangee, relit `+8/+16/+24` selon bpp).

## 5. Tilemap (fonction pure `map_entry_addr`)
Entree de 16 bits: `VHPC CCTT TTTT TTTT`: bit15 V (flip vertical), bit14 H (flip horizontal), bit13 P (priorite), bits12-10 palette, bits9-0 numero de tuile.
Position d'entree `(tx, ty)` en unites de tuile de la carte (apres division par 8 ou 16 selon la taille):
```
scr = ((tx >> 5) & map_x2) + ((ty >> 5) & map_y2) * (if map_x2 {2} else {1})
addr = (map_base + scr * 0x400 + (ty & 31) * 32 + (tx & 31)) & 0x7FFF
```
`tx` est masque par la largeur: `tx & (if map_x2 {63} else {31})`, `ty & (if map_y2 {63} else {31})` avant le calcul ci-dessus.
Tuile 8x8: `tx = px >> 3`, `ty = py >> 3`. Tuile 16x16 (BGMODE bit pour ce BG): `tx = px >> 4`, `ty = py >> 4`; sous-tuile: `xs = ((px >> 3) & 1) ^ H`, `ys = ((py >> 3) & 1) ^ V`; `tuile = T + xs + ys * 16` (`& 0x3FF`). Flip dans la tuile 8x8: `fx = (px & 7) ^ (if H {7} else {0})`, `fy = (py & 7) ^ (if V {7} else {0})`.

## 6. Sprites (evalues UNE fois par ligne, a `dot == 0` de la ligne `v`, rangee `r = v - 1`)

### 6.1 Tables
OAM basse: octet `i*4+0` = X bits 7-0; `+1` = Y; `+2` = tuile bits 7-0; `+3` = `VHPP CCCt` (V flip, H flip, PP priorite 0..3, CCC palette 0..7, t = bit de table de noms / bit 8 de tuile).
OAM haute: octet `512 + i/4`, bits `(i%4)*2` = X bit 8, `(i%4)*2 + 1` = taille (grand si 1). `x = sign_extend_9(...)` -> -256..255.
Tailles (petit / grand) selon `obj_size`: 0: 8/16; 1: 8/32; 2: 8/64; 3: 16/32; 4: 16/64; 5: 32/64; 6: 16x32 / 32x64; 7: 16x32 / 32x32 (les autres sont carres: largeur = hauteur).

### 6.2 Evaluation de la liste (32 sprites max)
```
first = if obj_priority_rotation { (internal_oamadd >> 2) & 0x7F } else { 0 }
list = []
for k in 0..128:
    i = (first + k) & 0x7F
    row = (r - y) & 0xFF            // le sprite apparait sur la ligne Y; Y=0 -> premiere rangee visible, Y=255 -> rangee 1 du sprite sur la ligne 0
    if row >= h: continue
    if !(x == -256 || x > -(w as i32)): continue    // bug materiel: x = -256 compte dans la limite
    if list.len() == 32 { range_over = true; break }
    list.push(i)
```
Historique E09-00: formule corrigee de `(r - y - 1)` a `(r - y)`. SMW cache ses sprites inutilises en Y=240 (OAM $F0, taille petite); avec l'ancienne formule un sprite 16x16 a Y=240 couvrait r=241..256 et la rangee 15 revenait se dessiner sur r=0 (artefact de debris colores). Avec `(r - y)` le sprite ne couvre que r=240..255, hors ecran.
(`obj_interlace`: ignore en v1: traiter comme 0. `UNKNOWN - to confirm`.)

### 6.3 Recuperation des tranches (slivers, 34 max) et tampon de ligne
```
line_obj = [None; 256]; slivers = 0
for i in list.iter().rev():                  // du plus grand index au plus petit
    for c in 0..(w/8):                        // ordre ecran gauche -> droite
        sx = x + c*8
        counts = (sx > -8 && sx < 256) || x == -256
        if !counts: continue
        if slivers == 34 { time_over = true; goto fin }
        slivers += 1
        fetch_sliver(i, c)  // ecrit dans line_obj les pixels non transparents (ecrase: l'index le plus bas gagne car traite en dernier)
```
`fetch_sliver`: `tc = if H { w/8 - 1 - c } else { c }`; rangee de sprite `rv = row`; si V flip: pour tailles carrees `rv = h - 1 - row`; pour 16x32 et 32x64 (tailles non carrees: indices 6 et 7 grand/petit rectangulaire) chaque moitie est retournee SUR PLACE: `half = h/2; rv = (row / half) * half + (half - 1 - row % half)` (erratum materiel). `tile_row = rv >> 3`, `fine_y = rv & 7`.
```
tx = ((t & 15) + tc) & 15 ; ty = ((t >> 4) + tile_row) & 15 ; tile8 = ty * 16 + tx
addr = (obj_base + (if tbit { obj_name_gap } else { 0 }) + tile8 * 16 + fine_y) & 0x7FFF
lo = vram[addr] ; hi = vram[(addr + 8) & 0x7FFF]       // plans 0,1 / plans 2,3
pour px in 0..8 (ordre ecran): bit = if H { px } else { 7 - px }; indice = 4 plans; dest = sx + px
si indice != 0 et 0 <= dest < 256: line_obj[dest] = Some(Pix { rgb: cgram[128 + CCC*16 + indice], prio: PP, obj_pal: CCC })
```
Les flags `range_over` (plus de 32 sprites, STAT77 bit 7) et `time_over` (plus de 34 tranches, STAT77 bit 6) sont remis a faux au debut de chaque frame (v == 0, dot 0) et mis a vrai ici; ils restent a vrai jusque-la.
La couleur est lue de CGRAM lors de l'evaluation (a dot 0); acceptable (`UNKNOWN - to confirm`).

## 7. Fonds (modes 0-6, hors hi-res): `bg_pixel(n, x, v) -> Option<Pix>`
```
bpp = BPP[mode][n]; si 0 -> None
(xm, vm) = mosaic(n, x, v)                       // section 9
hofs_use = hofs[n]; vofs_use = vofs[n]
si offset-per-tile applicable (section 7.1): remplacer hofs_use / vofs_use pour cette colonne
px = (xm + hofs_use) & 0x3FF ; py = (vm + vofs_use) & 0x3FF
entree = vram[map_entry_addr(...)] ; tuile, flips, palette, priorite selon section 5
i = decode_row(...)[fx]; si i == 0 -> None
Some(Pix { rgb: palette(section 3), prio: P, obj_pal: 0 })
```
Remarque de decalage vertical: `vm = v` (1 pour la premiere rangee visible), donc la rangee de carte affichee a la premiere ligne est `vofs + 1` (les jeux mettent VOFS = -1 pour aligner).

### 7.1 Offset-per-tile (modes 2, 4, 6; couche BG1 ou BG2)
```
n_col = (xm + (hofs[n] & 7)) >> 3          // colonne de 8 px depuis la gauche; 0 = tuile partielle de gauche, jamais modifiee
si n_col >= 1:
    ex = ((n_col - 1) + (hofs[2] >> 3))      // colonne dans la carte de BG3 (hofs/vofs de BG3)
    ey = vofs[2] >> 3
    mode 2: eh = entree_BG3(ex, ey)  ;  ev = entree_BG3(ex, ey + 1)     // ev = "comme si 8 etait ajoute a VOFS"
    mode 4: e = entree_BG3(ex, ey) ; si e.bit15 == 0 { eh = e ; ev = 0 } sinon { eh = 0 ; ev = e }
    mode 6: hi-res: voir UNKNOWN (colonnes de 16 px hi-res)
    bit_enable = 13 pour BG1 (n = 0), 14 pour BG2 (n = 1)
    si eh.bit(bit_enable): hofs_use = (eh & 0x3F8) | (hofs[n] & 7)
    si ev.bit(bit_enable): vofs_use = ev & 0x3FF
```
`entree_BG3` utilise la carte de BG3 (`map_base[2]`, tailles de BG3) via `map_entry_addr` (tuiles 8x8, tx/ty modulo la taille de la carte). En mode 4 et mode 6, eh ou ev vaut 0 quand l'autre est utilise.

## 8. Mode 7 (fonction pure + pixel)
Registres: `m7a, m7b, m7c, m7d: i16`; `m7_cx, m7_cy, m7_hofs, m7_vofs: i32` (13 bits signes). M7SEL: `flip_x` (bit0), `flip_y` (bit1), `outside` (bit7), `fill` (bit6).
```
fn clip(n: i32) -> i32 { if n & 0x2000 != 0 { n | !0x3FF } else { n & 0x3FF } }
fn m7_sample(x, v) -> (i32 px, i32 py):
    sx = if flip_x { 255 - x } else { x }
    sy = if flip_y { 255 - v } else { v }
    a, b, c, d as i32
    ox = ((a * clip(m7_hofs - m7_cx)) & !63) + ((b * clip(m7_vofs - m7_cy)) & !63) + ((b * sy) & !63) + (m7_cx << 8)
    oy = ((c * clip(m7_hofs - m7_cx)) & !63) + ((d * clip(m7_vofs - m7_cy)) & !63) + ((d * sy) & !63) + (m7_cy << 8)
    px = (ox + a * sx) >> 8        // decalage arithmetique
    py = (oy + c * sx) >> 8
```
Pixel mode 7 (avec mosaique appliquee a `x` et `v` avant): 
```
si (px | py) & !0x3FF != 0 (hors 0..1023):
    si !outside:      px &= 0x3FF ; py &= 0x3FF            // repetition
    sinon si !fill:   transparent
    sinon:            tuile = 0 ; fx = px & 7 ; fy = py & 7  // remplissage avec la tuile 0
sinon tuile = (vram[((py >> 3) << 7) + (px >> 3)] & 0xFF)  // carte 128x128 en octets BAS
indice = (vram[tuile * 64 + (fy * 8 + fx)] >> 8) & 0xFF     // pixels en octets HAUTS
```
(avec `fx = px & 7`, `fy = py & 7`). Indice 0 = transparent. BG1: `rgb = cgram[indice]` (ou couleur directe section 3). EXTBG (SETINI bit6): BG2 utilise le MEME echantillon: `prio = indice >> 7`, `idx7 = indice & 0x7F` (0 = transparent), `rgb = cgram[idx7]`. Priorite BG1 en mode 7: niveau unique (traite comme prio 0 dans la table; voir section 2).
`UNKNOWN - to confirm`: bits de remplacement de flip_y avec `v` (deja decale d'une ligne) et alignement horizontal en overscan; validation par ROM de test mode 7.

## 9. Mosaique
`size = mosaic_size + 1`. Pour un BG `n` avec bit d'activation et `size > 1` (aussi BG1 mode 7): `xm = x - (x % size)`, `vm = v - ((v - 1) % size)`. Sinon `(x, v)`. Pas de mosaique sur les sprites. (hi-res: ignoree en v1, `UNKNOWN`.)

## 10. Fenetres (fonctions pures)
Un groupe de 4 bits `g` par couche: bit0 = inversion fenetre 1, bit1 = activation fenetre 1, bit2 = inversion fenetre 2, bit3 = activation fenetre 2.
Sources: BG1 = `win_sel[0] & 15`, BG2 = `win_sel[0] >> 4`, BG3 = `win_sel[1] & 15`, BG4 = `win_sel[1] >> 4`, OBJ = `win_sel[2] & 15`, couleur = `win_sel[2] >> 4`. Logiques: BG1 `win_log_bg & 3`, BG2 `(>>2) & 3`, BG3 `(>>4) & 3`, BG4 `(>>6) & 3`, OBJ `win_log_obj`, couleur `win_log_col`.
```
fn window_out(g: u8, logic: u8, x: u8, wh: [u8; 4]) -> bool:
    w1 = (x >= wh[0] && x <= wh[1]) ^ (g & 1 != 0)
    w2 = (x >= wh[2] && x <= wh[3]) ^ (g & 4 != 0)
    en1 = g & 2 != 0 ; en2 = g & 8 != 0
    match (en1, en2):
        (false, false) -> false                 // fenetres inactives: inversion sans effet
        (true, false)  -> w1
        (false, true)  -> w2
        (true, true)   -> match logic { 0 => w1 | w2, 1 => w1 & w2, 2 => w1 ^ w2, _ => !(w1 ^ w2) }
```
`true` = la couche est CACHEE a ce pixel (si le bit TMW/TSW de la couche est actif pour l'ecran concerne). Si gauche > droite, la fenetre non inversee est vide.

## 11. Composition d'un pixel (fonction `compose_pixel(x)`, appelee a chaque dot visible)

### 11.1 Echantillons de couches (une fois par x)
Pour chaque couche activee dans `tm | ts` : `layer_pix[n] = bg_pixel(n, x, v)` (BG), `layer_pix[4] = line_obj[x]`. Mode 7: BG1 via section 8, BG2 si EXTBG. Les BG sans bpp (BPP = 0) restent `None`.

### 11.2 Choix d'un ecran
```
fn pick(screen, x) -> Pick:
    en = if main { tm } else { ts } ; wm = if main { tmw } else { tsw }
    pour (couche, niveau) dans PRIORITE[mode] (avant -> arriere):
        si en.bit(couche) == 0 : continuer
        si wm.bit(couche) && window_out(couche, x) : continuer
        si layer_pix[couche] = Some(p) avec p.prio == niveau : retourner Pick { rgb: p.rgb, layer: couche, obj_pal: p.obj_pal }
    si main : Pick { rgb: cgram[0], layer: BACKDROP }
    sinon   : Pick { rgb: fixed_color, layer: BACKDROP }      // le fond du sous-ecran est la couleur fixe (COLDATA)
```
Pour OBJ, `niveau` compare a `p.prio` (0..3). Pour BG, `niveau` 0/1 compare au bit de priorite de tuile.

### 11.3 Regions de la fenetre couleur
`color_in = window_out(couleur, ...)`; `fn region(t: u8, inside: bool) -> bool { match t { 0 => false, 1 => !inside, 2 => inside, _ => true } }`. Si les fenetres couleur sont desactivees, `color_in = false` (tout est "dehors").
- `black = region(black_region, color_in)` : l'ecran principal est remplace par noir (rgb 0) pour ce pixel.
- `prevent = region(math_region, color_in)` : pas de color math pour ce pixel.

### 11.4 Color math (mode non hi-res)
```
main = pick(Main, x) ; sub = pick(Sub, x) (seulement si addend_subscreen)
base = if black { 0 } else { main.rgb }
layer_ok = match main.layer { 0..=3 => cgadsub.bit(layer), 4 => cgadsub.bit(4) && main.obj_pal >= 4, _ => cgadsub.bit(5) }
si layer_ok && !prevent:
    (add, sub_is_back) = if addend_subscreen { (sub.rgb, sub.layer == BACKDROP) } else { (fixed_color, false) }
    halve = math_half && !(addend_subscreen && sub_is_back)
    pour chaque canal c5 de (base, add):
        t = if math_sub { base_c - add_c } else { base_c + add_c }
        si halve { t >>= 1 }                 // decalage arithmetique
        c = clamp(t, 0, 31)
sinon c = base
sortie = convert(c) (05 section 6: luminosite puis expansion 8 bits)
```
`fixed_color` = BGR555 forme de `fixed_r/g/b` (COLDATA). Si `forced_blank`: sortie noire (pas de calcul).

## 12. Hi-res et pseudo hi-res
`hires_line = (mode == 5 || mode == 6 || hires_bit)`.
- **Pseudo hi-res** (SETINI bit3, modes autres que 5/6): colonne `2x+1` = ecran principal (calcul 11.4 normal), colonne `2x` = ecran secondaire `pick(Sub, x)` avec color math UNIQUEMENT si `!addend_subscreen` et couche autorisee (`layer_ok` calcule sur `sub.layer`) et `!prevent`: `blend(sub.rgb, fixed_color)`; sinon `sub.rgb` brut. `black` s'applique aux deux colonnes. (`UNKNOWN - to confirm`.)
- **Modes 5 et 6**: les fonds sont evalues sur 512 colonnes: `bg_pixel_hires(n, hx, v)` avec `hx = 2x + 1` pour l'ecran principal et `hx = 2x` pour le secondaire. Les sprites sont a 256 colonnes (meme pixel pour les deux). Tuile BG: 16 px de large (carte), BGMODE choisit 16x8 ou 16x16. Position: `px = hx + (hofs[n] << 1)`; entree de carte `(px >> 4, py >> 3 ou >> 4)`; demi-tuile gauche/droite: `xs = ((px >> 3) & 1) ^ H`, numero `T + xs (+ ys*16 si tuile 16 haute)`; pixel `fx = (px & 7) ^ (H?7:0)`. Memes regles de priorite/palette/couleur que 7. Entrelacement (SETINI bit0): `py = vm_il + vofs[n]` avec `vm_il = (v << 1) | field` (sinon `v`) `UNKNOWN - to confirm`. OPT mode 6: `UNKNOWN`, a faire en dernier.
Ecriture dans le framebuffer: voir 05 section 6.

## 13. Resume des UNKNOWN de rendu (defauts ci-dessus)
Alignement vertical du mode 7 (v vs r), parite des colonnes hi-res, mosaique verticale (origine `v = 1`), OBJ interlace, OPT en mode 6, color math des colonnes secondaires en hi-res, moment de lecture de la couleur sprite (dot 0), borne de la fenetre en hi-res.

## 14. Decoupage conseille en taches (E05, 50-150 lignes chacune)
R01 tables BPP/priorites/couche activee; R02 couleur (palette, direct, convert 5->8); R03 `decode_row`; R04 `map_entry_addr`; R05 `bg_pixel` 8x8 sans effet; R06 tuiles 16x16 + flips; R07 mosaique; R08 offset-per-tile; R09 fenetres (pures); R10 sprites: evaluation de liste; R11 sprites: tranches + tampon de ligne + erratum flip; R12 mode 7 fonction pure `m7_sample`; R13 mode 7 pixel + EXTBG; R14 `pick` + parcours des priorites; R15 color math + regions fenetre couleur; R16 `compose_pixel` branche dans `step_dot`; R17 modes 5/6 hi-res; R18 pseudo hi-res + entrelacement; R19 tests de rendu de scenes synthetiques (hash attendu calcule par un script Python independant fourni dans la tache).

## 15. Tests (prefixe = id de tache; aucune ROM)
- `decode_row`: octets plans -> indices (cas 2/4/8 bpp avec motifs connus ex. plan0=0xAA, plan1=0xCC).
- `map_entry_addr`: 4 tailles de carte, wrap, 16x16 avec flips (tableau de cas calcules a la main).
- Sprites: sprite 8x8 en Y=0 visible a la rangee 0; Y=1 non visible a la rangee 0; x=-256 compte dans la limite 32; 33 sprites alignes -> `range_over`; 35 tranches -> `time_over`; priorite d'index (le plus bas gagne); flip H/V; erratum 16x32 flip V.
- Mode 7: matrice identite (a=d=0x0100, b=c=0): `px = x + hofs...` (cas calcules a la main); rotation 90 degres; hors-ecran: repetition / transparent / tuile 0.
- Fenetres: gauche > droite; inversion; logiques OR/AND/XOR/XNOR avec les 4 combinaisons de (w1, w2).
- Color math: add, sub, half, clamp; fond de sous-ecran = couleur fixe; pas de half si addend sous-ecran = backdrop; OBJ palettes 0-3 sans math; region noire/prevent selon fenetre couleur.
- Scenes: VRAM/CGRAM/OAM construites en code, N lignes rendues, `hash(framebuffer)` compare a une valeur calculee a part par un script Python de reference (la tache fournit le script).
