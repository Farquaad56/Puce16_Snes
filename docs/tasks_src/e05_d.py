from _dsl import T

CORE = "crates/puce16-core"
SRC = CORE + "/src"
TST = CORE + "/tests"

CHAIN = ("Execution strictement sequentielle (D-020): cette tache suppose que toutes les precedentes sont terminees. "
         "Ne modifie QUE les fichiers de 'Write only in'.")

HELP = ("Aides de test (a recopier dans le fichier de test): "
        "`fn put_vram(p: &mut Ppu, addr: u16, w: u16) { p.write(0x15,0x80); p.write(0x16,(addr & 0xFF) as u8); p.write(0x17,(addr >> 8) as u8); p.write(0x18,(w & 0xFF) as u8); p.write(0x19,(w >> 8) as u8); }` ; "
        "`fn put_cgram(p: &mut Ppu, i: u8, w: u16) { p.write(0x21,i); p.write(0x22,(w & 0xFF) as u8); p.write(0x22,(w >> 8) as u8); }` ; "
        "`fn set_scroll(p: &mut Ppu, reg: u8, v: u16) { p.write(reg,(v & 0xFF) as u8); p.write(reg,(v >> 8) as u8); }`. "
        "Un `Ppu::new(Region::Ntsc)` neuf est en forced blank: toutes ces ecritures sont autorisees. Chaque test construit SA scene sur un `Ppu` neuf.")

AT = ("Aide locale de test: `fn at(p: &mut Ppu, v: u32, dot: u32) { for _ in 0..(v * 340 + dot) { p.step_dot(); } }` (NTSC, frame 0, sans entrelace). "
      "Capture de la frame 0 = `at(p,225,1)`; la frame suivante est capturee 262*340 steps plus tard (sans entrelace).")

SCENE_H = ("SCENE H (mode 1) : `write(0x05,0x01)` ; `write(0x07,0x04)` (carte BG1 a 0x400, 32x32) ; `write(0x0B,0x01)` (chr BG1 a 0x1000) ; "
           "tuile 2 a 0x1020 : `put_vram(0x1020,0x0080)` (rangee 0, indice 1 au pixel 0) et `put_vram(0x1021,0x8000)` (rangee 1, indice 2 au pixel 0) ; "
           "`put_vram(0x400,0x0402)` (entree (0,0) : tuile 2, palette 1, prio 0) ; `put_vram(0x401,0x2402)` (entree (1,0) : prio 1) ; "
           "`put_cgram(p,17,0x1234)` ; `put_cgram(p,18,0x2345)` ; `put_cgram(p,0,0x7C00)` (backdrop bleu) ; `set_scroll(&mut p,0x0E,0x3FF)` (VOFS BG1 = -1, la rangee de sortie 0 montre la rangee 0 de la carte).")

# Scene M5 : mode 5, BG1 4 bpp, carte (0,0) = tuile 2
SCENE_M5 = ("SCENE M5 (mode 5) : `write(0x05,0x05)` ; `write(0x07,0x04)` (carte BG1 a 0x400) ; `write(0x0B,0x01)` (chr BG1 a 0x1000) ; "
            "`put_vram(0x400,0x0402)` (entree (0,0) : tuile 2, palette 1, prio 0 ; couvre hx 0..=15 : moitie gauche = tuile 2, moitie droite = tuile 3) ; "
            "`put_vram(0x1020,0x0080)` (tuile 2 rangee 0 : indice 1 au pixel 0) ; `put_vram(0x1030,0x8000)` (tuile 3 rangee 0 : indice 2 au pixel 0) ; "
            "`put_cgram(p,17,0x1111)` ; `put_cgram(p,18,0x2222)`. Les `v` des appels directs valent 0 (py = v + vofs = 0).")

# ------------------------------------------------------------------ E05 part D: hi-res, pseudo hi-res, entrelacement, scenes (E05-17..19)

T("E05-17", "ppu/bg.rs: fonds hi-res des modes 5 et 6 (bg_pixel_hires)",
  size=140,
  context="""Les modes 5 et 6 evaluent les fonds sur 512 colonnes (`hx` = 0..=511, deux colonnes par pixel de 256). Une entree de carte couvre 16 pixels hi-res de large (deux tuiles 8x8 cote a cote); la hauteur est 8 ou 16 selon BGMODE.
`UNKNOWN - to confirm`: parite des colonnes (`hx` impair = ecran principal), mosaique en hi-res, formule d'entrelacement `(v << 1) | field`, offset-per-tile du mode 6 (NON implemente ici: ne rien ajouter). """ + CHAIN + " " + HELP,
  embed=["05b_PPU_RENDER.md#12. Hi-res et pseudo hi-res", "05b_PPU_RENDER.md#2. Modes BG: profondeur de bits par couche et tables de priorite"],
  write=[SRC + "/ppu/bg.rs", TST + "/e05_17_hires_bg.rs"],
  spec="""Dans `ppu/bg.rs` :
```rust
impl Ppu {
    pub fn bg_pixel_hires(&self, n: usize, hx: usize, v: usize) -> Option<Pix>;
    pub fn sample_layers_hx(&self, x: usize, hx: usize, v: usize) -> [Option<Pix>; 5];
}
```
`bg_pixel_hires(n, hx, v)` (n = 0..=3) :
1. `mode = regs.bg_mode` ; si `mode != 5 && mode != 6` -> `None` ; `bpp = bpp(mode, n)` ; si `bpp == 0` -> `None`.
2. Mosaique (meme reglage que E05-07, active si `regs.mosaic_en >> n & 1 != 0`) : `x = hx >> 1` ; `(xm, vm) = mosaic_xy(regs.mosaic_size(), enabled, x, v)` ; `hxm = (xm << 1) | (hx & 1)`.
3. Entrelacement : `vm_il = if self.interlace { (vm << 1) | (self.field as usize) } else { vm }`.
4. `px = (hxm + ((scroll.hofs(n) as usize) << 1)) & 0x3FF` ; `py = (vm_il + scroll.vofs(n) as usize) & 0x3FF`.
5. `tall = regs.tile16[n]` ; `tx = px >> 4` ; `ty = if tall { py >> 4 } else { py >> 3 }` ; `entry = decode_entry(vram_word(map_entry_addr(regs.map_base[n], regs.map_x2[n], regs.map_y2[n], tx as u16, ty as u16)))`.
6. `xs = (((px >> 3) & 1) as u16) ^ (entry.hflip as u16)` ; `ys = if tall { (((py >> 3) & 1) as u16) ^ (entry.vflip as u16) } else { 0 }` ; `number = (entry.tile + xs + ys * 16) & 0x3FF` ; `fx = (px & 7) ^ (if entry.hflip { 7 } else { 0 })` ; `fy = (py & 7) ^ (if entry.vflip { 7 } else { 0 })`.
7. Meme lecture de tuile que `bg_pixel` (E05-05, `words = 16` pour 4 bpp, `8` pour 2 bpp, `tile_word = (chr_base[n] + number * words) & 0x7FFF`, `row = (tile_word + fy) & 0x7FFF`, `decode_row(..)[fx]`, indice 0 -> `None`) ; `rgb = cgram[bg_palette_index(mode, n, bpp, entry.pal, i)]` ; `Some(Pix { rgb, prio: entry.prio, obj_pal: 0 })`.
Modification de `Ppu::bg_pixel` (E05-05) : retourne `None` si `regs.bg_mode >= 5` (modes 5/6 : seul `bg_pixel_hires` est utilise ; mode 7 : voir E05-12/13).
`sample_layers_hx(x, hx, v)` = le contenu de `sample_layers` (E05-14) ou la couche BG `n` vaut `bg_pixel_hires(n, hx, v)` si `regs.bg_mode` est 5 ou 6, `bg_pixel(n, x, v)` sinon (mode 7 et OBJ inchanges ; une couche absente de `tm | ts` vaut `None`). `sample_layers(x, v)` devient `sample_layers_hx(x, 2 * x + 1, v)`.""",
  tests=[("left_right_halves", SCENE_M5 + " Attendu : `bg_pixel_hires(0,0,0)==Some(Pix{rgb:0x1111,prio:0,obj_pal:0})` ; `(0,1,0)==None` ; `(0,8,0)==Some(Pix{rgb:0x2222,prio:0,obj_pal:0})` (moitie droite = tuile 3) ; `(0,9,0)==None` ; `(0,16,0)==None` (entree (1,0) vide)."),
         ("bg_pixel_none_in_hires_modes", SCENE_M5 + " `bg_pixel(0,0,0)==None` (mode 5). Meme chose avec `write(0x05,0x06)`."),
         ("hofs_doubled", SCENE_M5 + " `set_scroll(&mut p,0x0D,4)` : `bg_pixel_hires(0,0,0)==Some(Pix{rgb:0x2222,..})` (px = 0 + 4*2 = 8 -> tuile 3) ; `set_scroll(&mut p,0x0D,8)` : `bg_pixel_hires(0,0,0)==None` (px = 16 -> entree (1,0) vide)."),
         ("tile16_vertical", SCENE_M5 + " `write(0x05,0x15)` (mode 5, BG1 en tuiles hautes de 16) ; `put_vram(0x1120,0x8080)` (tuile 18 rangee 0 : indice 3) ; `put_cgram(p,19,0x3333)` : `bg_pixel_hires(0,0,8)==Some(Pix{rgb:0x3333,prio:0,obj_pal:0})` (py = 8 -> ys = 1 -> tuile 2 + 0 + 16) ; `(0,8,8)` : moitie droite, tuile 3 + 16 = 19 (vide) -> `None` ; `(0,0,0)==Some(Pix{rgb:0x1111,..})` ; `(0,0,7)==None` (rangee 7 vide)."),
         ("tile8_vertical", SCENE_M5 + " (BGMODE 0x05 : tuiles de 8 de haut) : `bg_pixel_hires(0,0,8)==None` (entree (0,1) a 0x420 vide) ; `(0,0,0)==Some(Pix{rgb:0x1111,..})`."),
         ("hflip", SCENE_M5 + " `put_vram(0x400,0x4402)` (flip H) : `bg_pixel_hires(0,15,0)==Some(Pix{rgb:0x1111,..})` (xs = 1^1 = 0, fx = 7^7 = 0) ; `(0,7,0)==Some(Pix{rgb:0x2222,..})` (xs = 0^1 = 1 -> tuile 3, fx = 0) ; `(0,0,0)==None`."),
         ("vflip_8_high", SCENE_M5 + " `put_vram(0x400,0x8402)` (flip V, tuiles de 8 de haut) : `bg_pixel_hires(0,0,7)==Some(Pix{rgb:0x1111,..})` (fy = 7^7 = 0) ; `(0,0,0)==None` (fy = 7, vide)."),
         ("bg2_two_bpp", SCENE_M5 + " + BG2 : `write(0x08,0x08)` (carte BG2 a 0x800) ; `write(0x0B,0x21)` (chr BG1 a 0x1000, chr BG2 a 0x2000) ; `put_vram(0x800,0x0C01)` (tuile 1, palette 3) ; `put_vram(0x2008,0x0080)` (2 bpp : tuile 1 = 0x2000 + 8, rangee 0 pixel 0 = indice 1) ; `put_cgram(p,13,0x4444)` (3*4 + 1) : `bg_pixel_hires(1,0,0)==Some(Pix{rgb:0x4444,prio:0,obj_pal:0})` ; `(1,8,0)==None` (tuile 2 en 0x2010 : vide) ; `bg_pixel_hires(2,0,0)==None` (BG3 absent en mode 5)."),
         ("mode6_bg1_only", SCENE_M5 + " `write(0x05,0x06)` : `bg_pixel_hires(0,0,0)==Some(Pix{rgb:0x1111,..})` ; `(1,0,0)==None` (BG2 absent en mode 6) ; `(0,8,0)==Some(Pix{rgb:0x2222,..})`."),
         ("priority_bit", SCENE_M5 + " `put_vram(0x400,0x2402)` (prio 1) : `bg_pixel_hires(0,0,0)==Some(Pix{rgb:0x1111,prio:1,obj_pal:0})`."),
         ("interlace_field_false", SCENE_M5 + " `write(0x05,0x15)` ; `put_vram(0x1120,0x8080)` ; `put_cgram(p,19,0x3333)` ; `write(0x33,0x01)` (entrelace, field faux au debut) : `bg_pixel_hires(0,0,4)==Some(Pix{rgb:0x3333,..})` (vm_il = 8 -> ys = 1 -> tuile 18) ; `(0,0,0)==Some(Pix{rgb:0x1111,..})`. Sans entrelace (`write(0x33,0x00)`) : `(0,0,4)==None` (py = 4, rangee 4 vide)."),
         ("interlace_field_true", SCENE_M5 + " `write(0x05,0x15)` ; `put_vram(0x1121,0x0080)` (tuile 18 rangee 1 : indice 1 -> palette 17) ; `write(0x33,0x01)` ; `at(p,263,0)` (une frame entrelacee de 263 lignes : field vaut vrai ensuite) : `bg_pixel_hires(0,0,4)==Some(Pix{rgb:0x1111,..})` (vm_il = (4<<1)|1 = 9 -> ys = 1, fy = 1 -> tuile 18 rangee 1)."),
         ("mosaic_vertical", SCENE_M5 + " `put_vram(0x1021,0x8000)` (tuile 2 rangee 1 : indice 2) ; `write(0x06,0x11)` (taille 2, BG1) : `bg_pixel_hires(0,0,2)==Some(Pix{rgb:0x2222,..})` (vm = 2 - (1 % 2) = 1 -> rangee 1) ; sans mosaique (`write(0x06,0x00)`) `(0,0,2)==None`."),
         ("mosaic_horizontal", "SCENE M5 avec `put_vram(0x1020,0x0040)` a la place de la ligne tuile 2 (pixel 1 = indice 1, le pixel 0 est transparent) et `write(0x06,0x11)` : `bg_pixel_hires(0,3,0)==Some(Pix{rgb:0x1111,..})` (x = 1 -> xm = 0 -> hxm = 1 -> fx = 1) ; `(0,2,0)==None` (hxm = 0, fx = 0, transparent) ; sans mosaique `(0,3,0)==None`."),
         ("sample_layers_hx_mode5", SCENE_M5 + " `write(0x2C,0x01)` (TM = BG1) : `sample_layers_hx(0,0,0)[0]==Some(Pix{rgb:0x1111,..})` ; `sample_layers_hx(0,1,0)[0]==None` ; `sample_layers(0,0)[0]==None` (hx = 1) ; `sample_layers(4,0)[0]==None` (hx = 9) ; `sample_layers_hx(4,8,0)[0]==Some(Pix{rgb:0x2222,..})`. Avec TM = TS = 0 : `sample_layers_hx(0,0,0)` est `[None;5]`."),
         ("sample_layers_hx_ignored_in_mode1", SCENE_H + " `write(0x2C,0x01)` : `sample_layers_hx(0,5,0)==sample_layers(0,0)` et `sample_layers_hx(0,0,0)[0]==Some(Pix{rgb:0x1234,prio:0,obj_pal:0})`.")])

T("E05-18", "ppu/mod.rs: hi-res et pseudo hi-res dans step_dot, export 512 colonnes",
  size=150,
  context="""Rendu des lignes hi-res: modes 5/6 et pseudo hi-res (SETINI bit 3). Une ligne hi-res ecrit DEUX colonnes par pixel de 256 (`2x` = ecran secondaire, `2x+1` = ecran principal).
La vue exportee fait 512 colonnes des qu'au moins une rangee de la frame est hi-res; les rangees non hi-res sont alors doublees horizontalement SANS modifier le tampon interne (`fb`). Rappel `UNKNOWN - to confirm`: color math des colonnes secondaires, bornes de fenetre en hi-res (on utilise `x`), OBJ en entrelace (sprites inchanges). """ + CHAIN + " " + HELP + " " + AT,
  embed=["05b_PPU_RENDER.md#12. Hi-res et pseudo hi-res", "05_PPU.md#6. Sortie: framebuffer"],
  write=[SRC + "/ppu/mod.rs", SRC + "/ppu/colormath.rs", TST + "/e05_18_hires_output.rs"],
  spec="""Dans `ppu/mod.rs` ajoute au `Ppu` (etat derive de l'image, NON hache, remis a zero par `reset`) : `line_hires: bool` et `row_hires: Vec<bool>` (478 elements, faux).
```rust
impl Ppu {
    pub fn hires_line(&self) -> bool;                              // regs.bg_mode == 5 || regs.bg_mode == 6 || self.pseudo_hires
    pub fn compose_hires(&self, x: usize, v: usize) -> (u16, u16);  // (colonne 2x, colonne 2x+1) en BGR555 avant luminosite
}
```
`compose_hires(x, v)` (hors du dernier point la logique est celle de `compose_pixel`, E05-16) :
- `order` et `extbg` comme `compose_pixel` ; `lm = sample_layers_hx(x, 2*x+1, v)` ; `ls = sample_layers_hx(x, 2*x, v)` ; `fixed = fixed_color(fixed_r, fixed_g, fixed_b)`.
- `main = pick_layers(order, &lm, regs.tm, [layer_hidden(true,l,x) ...], cgram[0])` ; `sub = pick_layers(order, &ls, regs.ts, [layer_hidden(false,l,x) ...], fixed)` (toujours calcule, meme si `addend_subscreen` est faux) ; `(black, prevent) = color_window_flags(x)`.
- colonne `2x+1` = `color_math(&main, &sub, black, prevent, &regs)`.
- colonne `2x` : `base = if black { 0 } else { sub.rgb }` ; si `!regs.addend_subscreen && !prevent && math_layer_ok(sub.layer, sub.obj_pal, regs.math_mask)` alors `blend(base, fixed, regs.math_sub, regs.math_half)` sinon `base`.
`step_dot` (point 3, E04-14/E05-16) :
- Au dot 22 de chaque ligne visible (`1 <= v <= vis_last`) : `row = output_row((v-1), field, interlace)` ; `line_hires = hires_line()` ; `row_hires[row] = line_hires`.
- Pour chaque dot visible : si `line_hires` : `(e, o) = compose_hires(x, v)` (noir 0xFF000000 pour les deux si `forced_blank`) ; `fb.set(2*x, row, argb(e))` ; `fb.set(2*x+1, row, argb(o))` ; sinon comportement de E05-16 (colonne `x` seulement).
- Capture (a `dot == 0`, `v == vblank_start()`, APRES le rechargement OAM) : `h = vis_last * (if interlace {2} else {1})` ; `frame_hires = row_hires[0..h].iter().any(|b| *b)` ; si `frame_hires` : `w = 512` ; `fb.set_view(512, h)` ; `view` = pour chaque rangee `r` de `0..h` : si `row_hires[r]` les 512 colonnes de `fb` ; sinon `pixel_double` des 256 premieres colonnes de `fb` ; si non : `w = 256` et `view = fb.compact()` comme avant. `view_w = w`, `view_h = h`.""",
  tests=[("pseudo_hires_sub_black", SCENE_H + " TM = BG1 (`write(0x2C,0x01)`), TS = 0 ; `write(0x33,0x08)` ; `write(0x00,0x0F)` ; `at(p,225,1)` : `frame().width==512`, `height==224`, `pixels.len()==512*224` ; ligne 0 (indices `0*512 + col`) : col 1 == 0xFFA58C21 (x=0 principal : BG1 0x1234) ; col 0 == 0xFF000000 (secondaire : couleur fixe 0) ; col 3 == 0xFF0000FF (x=1 : backdrop principal bleu) ; col 2 == 0xFF000000."),
         ("pseudo_hires_ts_bg1", SCENE_H + " TM = TS = BG1 (`write(0x2C,0x01)`, `write(0x2D,0x01)`), `write(0x33,0x08)`, `write(0x00,0x0F)`, `at(p,225,1)` : ligne 0 : col 0 == col 1 == 0xFFA58C21 ; col 2 == 0xFF000000 (sous-ecran transparent -> couleur fixe 0) ; col 3 == 0xFF0000FF."),
         ("pseudo_hires_math", SCENE_H + " TM = BG1, TS = 0 ; `write(0x33,0x08)` ; `write(0x31,0x01)` (math sur BG1) ; `write(0x32,0x2A)` (rouge fixe 10) ; display on ; `at(p,225,1)` : col 1 == 0xFFF78C21 (0x1234 + rouge 10 = 0x123E) ; col 0 == 0xFF520000 (secondaire = backdrop fixe 0x000A, couche 5 hors masque -> brut) ; col 3 == 0xFF0000FF (x=1 : principal = backdrop 0x7C00, couche 5 hors masque -> brut) ; col 2 == 0xFF520000 (secondaire = backdrop fixe 0x000A brut). Avec `write(0x31,0x21)` (math aussi sur le backdrop) : col 0 == 0xFFA50000 (0x000A + 0x000A = 0x0014) ; col 3 == 0xFF5200FF (0x7C00 + 0x000A) ; col 2 == 0xFFA50000 ; col 1 inchange."),
         ("pseudo_hires_addend_subscreen", SCENE_H + " TM = TS = BG1 ; `write(0x33,0x08)` ; `write(0x30,0x02)` (addend sous-ecran) ; `write(0x31,0x01)` ; display on ; `at(p,225,1)` : col 1 == 0xFFFFFF42 (0x1234 + 0x1234 : r 31, g 31, b 8 = 0x23FF) ; col 0 == 0xFFA58C21 (secondaire brut : pas de math quand l'addend est le sous-ecran)."),
         ("pseudo_hires_black_region", SCENE_H + " `write(0x33,0x08)` ; `write(0x30,0xC0)` (region noire = toujours) ; TM = BG1 ; display on ; `at(p,225,1)` : toutes les colonnes de la ligne 0 valent 0xFF000000. Avec `write(0x31,0x01)` et `write(0x32,0x2A)` en plus : col 1 == 0xFF520000 (base 0 + rouge 10) ; col 0 == 0xFF000000 ; col 3 == 0xFF000000 (backdrop principal, couche 5 hors masque : base 0 sans math)."),
         ("mode5_frame", SCENE_M5 + " + `set_scroll(&mut p,0x0E,0x3FF)` ; `put_cgram(p,0,0x7C00)` ; TM = TS = BG1 (`write(0x2C,0x01)`, `write(0x2D,0x01)`) ; display on ; `at(p,225,1)` : `frame().width==512`, `height==224` ; ligne 0 : col 0 == 0xFF8C4221 (x=0 : secondaire hx=0 -> BG1 0x1111 ; principal hx=1 -> transparent -> backdrop) ; col 1 == 0xFF0000FF ; col 8 == 0xFF108C42 (x=4 : secondaire hx=8 -> tuile 3 indice 2 = 0x2222) ; col 9 == 0xFF0000FF (principal hx=9 transparent) ; col 2 == 0xFF000000 (x=1 : secondaire hx=2 -> fx=2 transparent -> sous-ecran = couleur fixe 0 = noir) ; col 3 == 0xFF0000FF (x=1 : principal hx=3 transparent -> backdrop bleu)."),
         ("mixed_rows_doubled", SCENE_H + " TM = 0 (backdrop bleu seul) ; `write(0x33,0x08)` ; display on ; `at(p,100,0)` puis `write(0x33,0x00)` ; continuer jusqu'a (225,1) : `frame().width==512` ; rangees 0..=98 hi-res : col 0 == 0xFF000000 et col 1 == 0xFF0000FF (secondaire = couleur fixe 0, principal = backdrop) ; rangees 99..=223 non hi-res, doublees : col 0 == col 1 == 0xFF0000FF ; derniere rangee : col 510 == col 511 == 0xFF0000FF."),
         ("frame_back_to_256", "Suite du test precedent : `write(0x33,0x00)` est deja actif ; 262*340 steps de plus (frame suivante entierement non hi-res) : `frame().width==256` et tous les pixels == 0xFF0000FF."),
         ("interlace_pseudo_hires", SCENE_H + " TM = 0 ; `write(0x33,0x09)` (entrelace + pseudo hi-res) ; display on ; `at(p,225,1)` : `width==512`, `height==448` ; (col 1, rangee 0) == 0xFF0000FF ; (col 0, rangee 0) == 0xFF000000 ; (col 1, rangee 1) == 0xFF000000 (rangee de l'autre field, non encore dessinee) ; (col 1, rangee 446) == 0xFF0000FF ; (col 1, rangee 447) == 0xFF000000 ; nombre de pixels egaux a 0xFF0000FF == 224*256."),
         ("forced_blank_hires", SCENE_H + " `write(0x33,0x08)` mais SANS `write(0x00,0x0F)` (forced blank) ; `at(p,225,1)` : `width==512` et tous les pixels == 0xFF000000."),
         ("non_hires_frame_stays_256", SCENE_H + " TM = BG1 ; display on ; `at(p,225,1)` : `width==256`, `height==224` ; pixel (0,0) == 0xFFA58C21."),
         ("hash_ignores_hires_state", "A : SCENE_H, `write(0x33,0x08)`, display on, `at(p,225,1)`, puis `write(0x33,0x00)`. B : meme preparation, `write(0x33,0x00)`, display on, `at(p,225,1)`. Les registres sont identiques apres : `hash(A)==hash(B)` (`line_hires` et `row_hires` ne sont pas hachees), alors que `A.frame().width==512` et `B.frame().width==256`.")])

T("E05-19", "tests: harness de scenes PPU (hash de frame compare a un renderer Python independant)",
  size=130, kind="code",
  context="""Derniere tache de l'epic E05: valide la COMPOSITION complete avec des scenes riches, dont la valeur attendue (hash de frame) est calculee par un renderer Python INDEPENDANT (`docs/tools/ppu_ref.py`, ecrit par Claude, deja fourni).
Tu ne crees que le harness Rust. Tu ne modifies JAMAIS les fichiers fournis (scenes, valeurs attendues). Si un hash ne correspond pas: ne cherche pas a "faire passer" le test en changeant le moteur au hasard; rends compte (le test echoue = question a Claude, qui compare le reference et la spec).
Les scenes evitent les comportements `UNKNOWN - to confirm` (offset-per-tile du mode 6, OBJ entrelace, bornes de fenetre en hi-res). """ + CHAIN,
  reads=["specs/data/ppu_scenes.txt (FOURNI, format ci-dessous)", "specs/data/ppu_scenes.expected (FOURNI: `nom largeur hauteur hash_hex16` par ligne)"],
  write=[TST + "/e05_19_scenes.rs", TST + "/data/ppu_scenes.txt", TST + "/data/ppu_scenes.expected"],
  spec="""Etape 0: copie `docs/specs/data/ppu_scenes.txt` et `ppu_scenes.expected` (fournis) vers `crates/puce16-core/tests/data/` sans les modifier.
Format de `ppu_scenes.txt` (texte, une directive par ligne, `#` = commentaire, valeurs hexa sans `0x` pour les mots/octets) :
```
scene <nom> <frames>              # debut de scene; frames = 1 ou 2 (2 seulement si pas d'entrelace)
vram <addr_hex> <w_hex> ...       # mots 16 bits consecutifs a partir de l'adresse mot
cgram <index_dec> <w_hex> ...     # mots a partir de l'index couleur
oam <addr_dec> <b_hex> ...        # octets a partir de l'adresse octet (0..543)
reg <reg_hex> <val_hex>           # ecriture registre PPU ($21xx -> reg = xx), dans l'ordre du fichier
at <v_dec> <reg_hex> <val_hex>    # ecriture appliquee a CHAQUE frame, juste avant le premier step du dot 0 de la ligne v
end
```
Harness (`tests/e05_19_scenes.rs`, API publique seulement) pour chaque scene :
1. `Ppu::new(Region::Ntsc)` (forced blank). Chargements memoire dans l'ordre du fichier, via registres : VRAM = `write(0x15,0x80)`, `write(0x16,addr&0xFF)`, `write(0x17,addr>>8)` puis pour chaque mot `write(0x18,low)`, `write(0x19,high)` ; CGRAM = `write(0x21,index)` puis `write(0x22,low)`, `write(0x22,high)` ; OAM = `write(0x02,(a/2)&0xFF)`, `write(0x03,((a/2)>>8)&1)` (OAMADD est une adresse MOT = adresse octet / 2) puis `write(0x04,b)` par octet (les adresses `oam` sont toujours paires et les longueurs paires). Puis les `reg`, dans l'ordre.
2. Boucle de steps: `n = 225*340 + 1 + (frames-1)*262*340` ; pour `s` de 0 a `n-1` : `line = (s / 340) % 262` ; `dot = s % 340` ; si `dot == 0` appliquer les `at` de la ligne `line` ; puis `p.step_dot()`.
3. `f = p.frame()` ; hash = `Fnv64` : `write_u32(width)`, `write_u32(height)`, `write_u32_slice(pixels)` ; compare a la ligne attendue (nom, largeur, hauteur, hash `{:016x}`).
4. Ecrit toujours `tests/out/ppu_scenes/<nom>.ppm` a la racine du depot (`env!("CARGO_MANIFEST_DIR")/../../tests/out/ppu_scenes/`; cree le dossier; format P6 binaire, RGB de chaque pixel `0xFFRRGGBB`), pour inspection visuelle humaine (dossier ignore par git).
Les erreurs de format du fichier de scenes font paniquer le test avec le numero de ligne.""",
  tests=[("scene_file_parses", "le fichier de scenes se lit sans erreur; il contient au moins 10 scenes; chaque scene a une ligne dans `ppu_scenes.expected` et inversement (memes noms)."),
         ("all_scenes_match", "pour chaque scene, largeur, hauteur et hash == valeurs attendues; en cas d'echec le message liste TOUTES les scenes en erreur (nom, attendu, obtenu) et pas seulement la premiere."),
         ("scenes_deterministic", "la premiere scene rejouee deux fois donne deux hashs egaux."),
         ("ppm_written", "apres `all_scenes_match`, le fichier `tests/out/ppu_scenes/<premiere scene>.ppm` existe, commence par `P6\\n` et sa taille est `header + largeur*hauteur*3`.")],
  notes="""Fichiers fournis par Claude (a recevoir AVANT cette tache): `docs/tools/ppu_ref.py` (generateur des scenes + renderer de reference + calcul des hash). Tu ne le lances pas et tu ne le modifies pas.""")
