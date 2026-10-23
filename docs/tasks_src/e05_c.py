from _dsl import T

CORE = "crates/puce16-core"
SRC = CORE + "/src"
TST = CORE + "/tests"

CHAIN = ("Execution strictement sequentielle (D-020): cette tache suppose que toutes les precedentes sont terminees. "
         "Ne modifie QUE les fichiers de 'Write only in'.")

AT = ("Aide locale de test: `fn at(p: &mut Ppu, v: u32, dot: u32) { for _ in 0..(v * 340 + dot) { p.step_dot(); } }` (NTSC, frame 0, sans entrelace).")

HELP = ("Aides de test (a recopier dans le fichier de test): "
        "`fn put_vram(p: &mut Ppu, addr: u16, w: u16) { p.write(0x15,0x80); p.write(0x16,(addr & 0xFF) as u8); p.write(0x17,(addr >> 8) as u8); p.write(0x18,(w & 0xFF) as u8); p.write(0x19,(w >> 8) as u8); }` ; "
        "`fn put_cgram(p: &mut Ppu, i: u8, w: u16) { p.write(0x21,i); p.write(0x22,(w & 0xFF) as u8); p.write(0x22,(w >> 8) as u8); }` ; "
        "`fn set_scroll(p: &mut Ppu, reg: u8, v: u16) { p.write(reg,(v & 0xFF) as u8); p.write(reg,(v >> 8) as u8); }`. "
        "Un `Ppu::new(Region::Ntsc)` neuf est en forced blank: toutes ces ecritures sont autorisees. Chaque test construit SA scene sur un `Ppu` neuf.")

# Scene H: mode 1, BG1 4 bpp (decrite une fois, reutilisee par E05-14 et E05-16)
SCENE_H = ("SCENE H (fonds) : `write(0x05,0x01)` (mode 1) ; `write(0x07,0x04)` (carte BG1 a 0x400, 32x32) ; `write(0x0B,0x01)` (chr BG1 a 0x1000) ; "
           "tuile 2 a 0x1020 : `put_vram(0x1020,0x0080)` (rangee 0, indice 1 au pixel 0) et `put_vram(0x1021,0x8000)` (rangee 1, indice 2 au pixel 0) ; "
           "`put_vram(0x400,0x0402)` (entree (0,0) : tuile 2, palette 1, prio 0) ; `put_vram(0x401,0x2402)` (entree (1,0) : prio 1) ; "
           "`put_cgram(p,17,0x1234)` ; `put_cgram(p,18,0x2345)` ; `put_cgram(p,0,0x7C00)` (backdrop bleu).")

# ------------------------------------------------------------------ E05 part C: choix du pixel, color math, composition (E05-14..16)

T("E05-14", "ppu: pick_layers (ordre de priorite), fenetres par ecran, pick_screen",
  size=140,
  context="""Choix, pour un ecran (principal ou secondaire) et une colonne, de la couche visible (spec 05b 11.1 et 11.2).
`pick_layers` est une fonction PURE (facile a tester); `Ppu::sample_layers` echantillonne les couches actives; `Ppu::pick_screen` assemble le tout (pour tests et pour E05-16). """ + CHAIN + " " + HELP,
  embed=["05b_PPU_RENDER.md#11.1 Echantillons de couches (une fois par x)", "05b_PPU_RENDER.md#11.2 Choix d'un ecran"],
  write=[SRC + "/ppu/bg.rs", SRC + "/ppu/window.rs", TST + "/e05_14_pick.rs"],
  spec="""Dans `ppu/bg.rs` :
```rust
pub fn pick_layers(order: &[(u8, u8)], layers: &[Option<Pix>; 5], en: u8, hidden: [bool; 5], backdrop: u16) -> Pick;
impl Ppu {
    pub fn sample_layers(&self, x: usize, v: usize) -> [Option<Pix>; 5];
    pub fn pick_screen(&self, main: bool, x: usize, v: usize) -> Pick;
}
```
`pick_layers` : pour `(layer, level)` dans `order` (avant -> arriere) : ignorer si `en >> layer & 1 == 0` ; ignorer si `hidden[layer]` ; si `layers[layer] == Some(p)` et `p.prio == level` : retourner `Pick { rgb: p.rgb, layer, obj_pal: p.obj_pal }`. Sinon `Pick { rgb: backdrop, layer: LAYER_BACKDROP, obj_pal: 0 }`. (Pour les BG le niveau est la priorite de tuile 0/1 ; pour OBJ la priorite 0..3 ; en mode 7 le BG1 a toujours `prio 0`.)
`sample_layers(x, v)` : `mask = regs.tm | regs.ts` ; mode 7 : couche 0 = `m7_pixel(0,x,v)`, couche 1 = `m7_pixel(1,x,v)` (None sans EXTBG), couches 2-3 None ; modes 0-6 : couche n = `bg_pixel(n,x,v)` pour n in 0..4 ; dans tous les modes : couche 4 = `obj_pixel(x)`. Une couche dont le bit n'est pas dans `mask` vaut None.
`pick_screen(main, x, v)` : `layers = sample_layers(x,v)` ; `order = priority_order(regs.bg_mode, regs.bg3_priority, extbg)` ; `en = if main { tm } else { ts }` ; `hidden[l] = layer_hidden(main, l, x)` ; `backdrop = if main { cgram[0] } else { fixed_color }` ou `fixed_color = fixed_r | fixed_g << 5 | fixed_b << 10`.
Dans `ppu/window.rs` ajoute : `impl Ppu { pub fn layer_hidden(&self, main: bool, layer: u8, x: usize) -> bool }` = `((if main { regs.tmw } else { regs.tsw }) >> layer) & 1 != 0 && window_hides(layer, x)` (couches 0..=4).""",
  tests=[("pick_priority_order_mode1", "ordre mode 1 = `priority_order(1,false,false)` (S3 1H 2H S2 1L 2L S1 3H S0 3L), `en = 0x1F`, `hidden = [false;5]`, backdrop 0x0BAD. (a) BG1 prio 1 (rgb 1) + BG2 prio 1 (rgb 2) + OBJ prio 2 (rgb 3) : 1H est avant S2 -> `Pick{rgb:1,layer:0,obj_pal:0}`. (b) BG1 prio 0 (rgb 1) + OBJ prio 2 (rgb 3) : S2 est avant 1L -> `Pick{rgb:3,layer:4,..}`. (c) OBJ prio 0 (rgb 3) + BG3 prio 0 (rgb 4) : S0 est avant 3L -> rgb 3. (d) OBJ prio 0 (rgb 3) + BG3 prio 1 (rgb 4) : 3H est avant S0 -> `Pick{rgb:4,layer:2,..}`. (e) BG3 prio 1 (rgb 4) + OBJ prio 1 (rgb 3) : S1 est avant 3H -> `Pick{rgb:3,layer:4,..}`."),
         ("pick_none_is_backdrop", "aucune couche : `Pick{rgb:0x0BAD,layer:5,obj_pal:0}`. Toutes les couches presentes mais `en = 0` : backdrop."),
         ("pick_level_must_match", "BG1 avec `prio: 1` seul, mode 1 : trouve a l'entree (0,1) ; si le `Pix` a `prio: 5` (valeur invalide) il n'est jamais choisi -> backdrop."),
         ("pick_en_mask", "BG1 (rgb 1) et BG2 (rgb 2), toutes prio 1, `en = 0b11110` (BG1 desactive) : `Pick{rgb:2,layer:1,..}`. `en = 0b11101` : BG1 gagne."),
         ("pick_hidden", "BG1 (rgb 1) et BG2 (rgb 2) prio 1 ; `hidden = [true,false,false,false,false]` : BG2 gagne ; `hidden` tout vrai : backdrop."),
         ("pick_mode1_bg3_priority", "ordre `priority_order(1,true,false)` : BG3 prio 1 (rgb 4) + OBJ prio 3 (rgb 3) : BG3 gagne (entree (2,1) en tete) ; avec l'ordre `priority_order(1,false,false)` le meme jeu donne OBJ."),
         ("pick_obj_pal_kept", "OBJ `Pix{rgb:7,prio:3,obj_pal:5}` : `Pick{rgb:7,layer:4,obj_pal:5}`."),
         ("pick_screen_bg_and_backdrop", SCENE_H + " `write(0x2C,0x01)` (TM = BG1) : `pick_screen(true,0,0)==Pick{rgb:0x1234,layer:0,obj_pal:0}` ; `pick_screen(true,1,0)==Pick{rgb:0x7C00,layer:5,obj_pal:0}` (pixel transparent) ; `pick_screen(true,8,0)==Pick{rgb:0x1234,layer:0,obj_pal:0}` (BG1 prio 1). Avec TM = 0 : `pick_screen(true,0,0)` = backdrop 0x7C00."),
         ("pick_screen_sub", SCENE_H + " TM = 0, `write(0x2D,0x01)` (TS = BG1) : `pick_screen(false,0,0)` rgb 0x1234 layer 0 ; `pick_screen(false,1,0)` = backdrop du sous-ecran = couleur fixe : sans COLDATA `Pick{rgb:0,layer:5,obj_pal:0}` ; apres `write(0x32,0x2A)` (rouge fixe = 10) : `Pick{rgb:0x000A,layer:5,obj_pal:0}`. `pick_screen(true,0,0)` (ecran principal, TM = 0) = backdrop principal 0x7C00."),
         ("pick_screen_window", SCENE_H + " `write(0x2C,0x01)` ; `write(0x2E,0x01)` (TMW BG1) ; `write(0x23,0x02)` (fenetre 1 sur BG1) ; `write(0x26,0)`, `write(0x27,7)` : `pick_screen(true,0,0)` = backdrop 0x7C00 (BG1 cache par la fenetre) ; `pick_screen(true,8,0)` rgb 0x1234 (hors fenetre). `layer_hidden(true,0,3)==true`, `layer_hidden(true,0,8)==false`, `layer_hidden(false,0,3)==false` (TSW = 0), `layer_hidden(true,1,3)==false` (BG2 : TMW bit 1 = 0)."),
         ("pick_screen_obj_over_bg", SCENE_H + " + sprite : OAM neuve, `fn load_oam`/`spr` de E05-10 (sprites inutilises a x = -200), sprite 0 : x=0, y=255, tuile 0, attr 0x30 (prio 3, palette 0), petit ; `put_vram(0,0x0080)` (tuile 0 rangee 0 indice 1) ; `put_cgram(p,129,0x03E0)` ; `write(0x2C,0x11)` (TM = BG1 + OBJ) ; `p.render_obj_line(0)` : `pick_screen(true,0,0)==Pick{rgb:0x03E0,layer:4,obj_pal:0}` (OBJ prio 3 devant BG1 prio 0) ; `pick_screen(true,8,0)` = BG1 (rgb 0x1234).")])

T("E05-15", "ppu/colormath.rs: color math, regions de la fenetre couleur",
  size=140,
  context="""Color math (spec 05b 11.3 et 11.4): fonctions pures + `Ppu::color_window_flags`. Les couleurs sont en BGR555 (5 bits par canal). Pas de flottants. """ + CHAIN,
  embed=["05b_PPU_RENDER.md#11.3 Regions de la fenetre couleur", "05b_PPU_RENDER.md#11.4 Color math (mode non hi-res)"],
  write=[SRC + "/ppu/colormath.rs", TST + "/e05_15_colormath.rs"],
  spec="""Dans `ppu/colormath.rs` :
```rust
pub fn region(t: u8, inside: bool) -> bool;                               // 0 -> false ; 1 -> !inside ; 2 -> inside ; _ -> true
pub fn fixed_color(r: u8, g: u8, b: u8) -> u16;                           // r | g << 5 | b << 10 (valeurs deja sur 5 bits)
pub fn blend(base: u16, add: u16, sub: bool, half: bool) -> u16;         // par canal : t = base_c +/- add_c ; si half : t >>= 1 (arithmetique) ; clamp 0..=31
pub fn math_layer_ok(layer: u8, obj_pal: u8, mask: u8) -> bool;           // layer 0..=3 : mask bit layer ; 4 (OBJ) : mask bit 4 && obj_pal >= 4 ; 5 (backdrop) : mask bit 5
pub fn color_math(main: &Pick, sub: &Pick, black: bool, prevent: bool, regs: &Regs) -> u16;
impl Ppu { pub fn color_window_flags(&self, x: usize) -> (bool, bool); }    // (black, prevent)
```
`color_math` (spec 11.4) : `base = if black { 0 } else { main.rgb }` ; `layer_ok = math_layer_ok(main.layer, main.obj_pal, regs.math_mask)` ; si `layer_ok && !prevent` : `(addend, sub_is_back) = if regs.addend_subscreen { (sub.rgb, sub.layer == LAYER_BACKDROP) } else { (fixed_color(regs.fixed_r, regs.fixed_g, regs.fixed_b), false) }` ; `halve = regs.math_half && !(regs.addend_subscreen && sub_is_back)` ; retourne `blend(base, addend, regs.math_sub, halve)` ; sinon retourne `base`.
`color_window_flags(x)` : `inside = window_hides(5, x)` (groupe/logique "couleur", couche 5 ; faux si les fenetres couleur sont inactives) ; `black = region(regs.black_region, inside)` ; `prevent = region(regs.math_region, inside)`.""",
  tests=[("region_table", "`region(0,true)==false`, `region(0,false)==false` ; `region(1,true)==false`, `region(1,false)==true` ; `region(2,true)==true`, `region(2,false)==false` ; `region(3,true)==true`, `region(3,false)==true`."),
         ("fixed_color_values", "`fixed_color(31,0,0)==0x001F` ; `(0,31,0)==0x03E0` ; `(0,0,31)==0x7C00` ; `(1,2,3)==0x0C41` ; `(31,31,31)==0x7FFF`."),
         ("blend_add", "base = 0x7A8A (r=10, g=20, b=30), add = 0x29E5 (r=5, g=15, b=10) : `blend(base,add,false,false)==0x7FEF` (15, 35 -> 31, 40 -> 31) ; `blend(base,add,false,true)==0x5227` (7, 17, 20)."),
         ("blend_sub", "memes valeurs : `blend(base,add,true,false)==0x50A5` (5, 5, 20) ; `blend(base,add,true,true)==0x2842` (2, 2, 10) ; `blend(add,base,true,false)==0x0000` (tout negatif -> 0) ; `blend(add,base,true,true)==0x0000`."),
         ("blend_neutral", "`blend(0x1234,0,false,false)==0x1234` ; `blend(0x1234,0,true,false)==0x1234` ; `blend(0x7FFF,0x7FFF,false,false)==0x7FFF` ; `blend(0x7FFF,0x7FFF,true,false)==0x0000` ; `blend(0x7FFF,0x7FFF,false,true)==0x7FFF`."),
         ("math_layer_ok_rules", "`math_layer_ok(0,0,0x01)==true`, `(1,0,0x01)==false`, `(3,0,0x08)==true` ; OBJ : `(4,4,0x10)==true`, `(4,7,0x10)==true`, `(4,3,0x10)==false`, `(4,4,0x00)==false` ; backdrop : `(5,0,0x20)==true`, `(5,0,0x1F)==false`."),
         ("math_off_by_default", "`Regs::new()`, `main = Pick{rgb:0x0001,layer:0,obj_pal:0}`, `sub = Pick{rgb:0,layer:5,obj_pal:0}` : `color_math(&main,&sub,false,false,&regs)==0x0001` (masque 0)."),
         ("math_fixed_add_half_sub", "regs : `write(0x31,0x01)` (math sur BG1), `write(0x32,0x43)` (vert fixe = 3) : `color_math(main (rgb 0x0001, layer 0),..)==0x0061` (r=1, g=3). Avec `write(0x31,0x41)` (half) : `0x0020` ((1+0)>>1 = 0, (0+3)>>1 = 1). Avec `write(0x31,0x81)` (soustraction) : `0x0001` (g = 0 - 3 -> 0)."),
         ("math_subscreen", "`write(0x30,0x02)` (addend = sous-ecran), `write(0x31,0x01)` ; sub = `Pick{rgb:0x0002,layer:1,obj_pal:0}` : resultat `0x0003`. Avec `write(0x31,0x41)` (half) : `(1+2)>>1 = 1` -> `0x0001`. Avec sub = `Pick{rgb:0x0002,layer:5,obj_pal:0}` (backdrop) et half : PAS de division -> `0x0003`."),
         ("math_prevent_black", "`write(0x31,0x01)`, `write(0x32,0x2A)` (rouge fixe = 10) : `prevent = true` -> `0x0001` ; `black = true` -> base 0 : `0x000A` ; `black = true` avec masque 0 (`write(0x31,0x00)`) -> `0x0000`."),
         ("math_obj_palettes", "main = `Pick{rgb:0x0005,layer:4,obj_pal:3}`, `write(0x31,0x10)`, `write(0x32,0x2A)` : resultat `0x0005` (palette < 4 : pas de math) ; avec `obj_pal:4` : `0x000F`."),
         ("math_backdrop_layer", "main = `Pick{rgb:0x0001,layer:5,obj_pal:0}`, rouge fixe 10 : avec `write(0x31,0x20)` -> `0x000B` ; avec `write(0x31,0x01)` -> `0x0001`."),
         ("math_clamp_white", "main rgb 0x7FFF layer 0, `write(0x31,0x01)`, `write(0x32,0xFF)` (fixe blanc 31,31,31) : `0x7FFF`."),
         ("color_window_flags", "`write(0x25,0x20)` (fenetre couleur : groupe 2 = fenetre 1 active), `write(0x26,10)`, `write(0x27,20)`, `write(0x30,0x60)` (black_region = 1 \"dehors\", math_region = 2 \"dedans\") : `color_window_flags(15)==(false,true)` ; `color_window_flags(5)==(true,false)`. Avec `write(0x30,0x00)` : `(false,false)` partout. Avec `write(0x30,0xF0)` (3 et 3) : `(true,true)` partout. Fenetre couleur inactive (`write(0x25,0x00)`) avec `write(0x30,0x60)` : x=15 -> `(true,false)` (inside = faux : dehors).")])

T("E05-16", "ppu/mod.rs: compose_pixel branche dans step_dot (rendu non hi-res complet)",
  size=140,
  context="""Assemble tout le rendu non hi-res: a chaque dot visible, le pixel est calcule par `compose_pixel(x, v)` (echantillons -> ecran principal et sous-ecran -> color math -> BGR555), puis converti en XRGB avec la luminosite (E04-13/14).
Les sprites de la ligne sont evalues a `dot == 0` (ordre de la spec 05 5.3: AVANT le dessin des pixels de la ligne). Le hi-res (modes 5/6, pseudo hi-res) arrive en E05-17/18: ici on rend TOUJOURS une colonne par pixel. """ + CHAIN + " " + HELP + " " + AT,
  embed=["05_PPU.md#5.3 Evenements par (v, dot), dans CET ordre dans `step_dot`", "05b_PPU_RENDER.md#11.4 Color math (mode non hi-res)"],
  write=[SRC + "/ppu/mod.rs", TST + "/e05_16_compose.rs"],
  spec="""Dans `ppu/mod.rs` :
- Remplace la fonction privee `pixel_bgr555(&self, x)` de E04-14 par `pub fn compose_pixel(&self, x: usize, v: usize) -> u16` (BGR555 final avant luminosite) :
  `layers = sample_layers(x, v)` ; `order` et `extbg = self.extbg` ; `main = pick_layers(order, &layers, regs.tm, [layer_hidden(true,l,x) pour l in 0..5], cgram[0])` ; `fixed = fixed_color(regs.fixed_r, regs.fixed_g, regs.fixed_b)` ; `sub = if regs.addend_subscreen { pick_layers(order, &layers, regs.ts, [layer_hidden(false,l,x)...], fixed) } else { Pick { rgb: fixed, layer: LAYER_BACKDROP, obj_pal: 0 } }` ; `(black, prevent) = color_window_flags(x)` ; retourne `color_math(&main, &sub, black, prevent, &regs)`.
- `step_dot`, a `dot == 0` et `1 <= v <= vis_last` : appelle `render_obj_line((v - 1) as u16)` (dans l'ordre de la section 5.3 : avant l'evenement LINE_START... l'ordre entre cet appel et les autres evenements de dot 0 n'a pas d'effet observable ici).
- Dessin d'un pixel (point 3 de `step_dot`, E04-14) : `argb = if forced_blank { 0xFF000000 } else { bgr555_to_xrgb(compose_pixel(x, v as usize), brightness) }`.
Le reste de `step_dot` est inchange. Les tests ci-dessous utilisent la SCENE H (decrite plus bas), `VOFS BG1 = -1` pour que la rangee de sortie 0 montre la rangee 0 de la carte, et `at(p,225,1)` (225*340 + 1 steps) pour capturer la frame (`frame().pixels[row*256 + col]`).""",
  tests=[("bg1_scene", SCENE_H + " `set_scroll(&mut p,0x0E,0x3FF)` (VOFS BG1 = -1) ; `write(0x2C,0x01)` ; `write(0x00,0x0F)` ; `at(p,225,1)` : pixel (col 0,row 0) == 0xFFA58C21 (0x1234 : r=20 -> 165, g=17 -> 140, b=4 -> 33) ; pixel (1,0) == 0xFF0000FF (transparent -> backdrop bleu) ; pixel (8,0) == 0xFFA58C21 ; pixel (0,1) (rangee 1 de la carte = indice 2, 0x2345) == `bgr555_to_xrgb(0x2345,15)`."),
         ("tm_zero_all_backdrop", "SCENE H, TM = 0 : tous les 57344 pixels == 0xFF0000FF."),
         ("obj_over_bg", "SCENE H + VOFS -1 + `write(0x2C,0x11)` + sprite 0 : x=0, y=255, tuile 0, attr 0x30 (prio 3), `put_vram(0,0x0080)`, `put_cgram(129,0x03E0)` (OAM chargee comme en E05-10, autres sprites a x = -200) : pixel (0,0) == 0xFF00FF00 ; pixel (8,0) == 0xFFA58C21 (BG1 visible) ; le sprite n'a qu'un pixel (indice 1 au pixel 0) : pixel (1,0) == 0xFF0000FF. Rangee 1 : le sprite (rangee de donnees 1 vide) n'apparait pas."),
         ("fixed_add_on_backdrop", "TM = 0, `put_cgram(0,0x0001)`, `write(0x31,0x20)` (math sur backdrop), `write(0x32,0x2A)` (rouge fixe 10) : tous les pixels == 0xFF5A0000 (r = 11 -> (11<<3)|(11>>2) = 90 = 0x5A). Avec `write(0x31,0x60)` (+ half) : 0xFF290000 (r = 5 -> 41). Avec `write(0x31,0xA0)` (soustraction) : 0xFF000000 (1 - 10 -> 0)."),
         ("subscreen_add", SCENE_H + " VOFS -1 ; `put_cgram(0,0x0001)` ; TM = 0 ; `write(0x2D,0x01)` (TS = BG1) ; `write(0x30,0x02)` (addend sous-ecran) ; `write(0x31,0x20)` (math sur backdrop) : pixel (0,0) == 0xFFAD8C21 (backdrop r=1 + BG1 0x1234 -> r=21, g=17, b=4) ; pixel (1,0) == 0xFF080000 (sous-ecran transparent -> couleur fixe 0 : r = 1)."),
         ("window_hides_bg", SCENE_H + " VOFS -1 ; `write(0x2C,0x01)`, `write(0x2E,0x01)`, `write(0x23,0x02)`, `write(0x26,0)`, `write(0x27,7)` : pixel (0,0) == 0xFF0000FF (cache) ; pixel (8,0) == 0xFFA58C21."),
         ("mode7_pixel", "Mode 7 : `write(0x05,0x07)` ; matrice identite (`write(0x1B,0x00); write(0x1B,0x01); write(0x1E,0x00); write(0x1E,0x01)`) ; `write(0x0E,0xFF); write(0x0E,0x1F)` (M7VOFS = -1) ; carte index 0 = tuile 1 (VMAIN 0 : `write(0x15,0x00); write(0x16,0); write(0x17,0); write(0x18,1)`) ; pixel (fy 0, fx 0) de la tuile 1 = 5 (VMAIN 0x80, adresse 64 : `write(0x15,0x80); write(0x16,64); write(0x17,0); write(0x19,5)`) ; `put_cgram(5,0x0555)` ; `write(0x2C,0x01)` : pixel (0,0) == 0xFFAD5208."),
         ("brightness_applied", "SCENE H, TM = 0, `write(0x00,0x08)` : tous les pixels == 0xFF000084 (backdrop bleu 31 a luminosite 8 : 31*8/15 = 16 -> 132 = 0x84)."),
         ("forced_blank_stays_black", "SCENE H, `write(0x2C,0x01)` mais SANS `write(0x00,0x0F)` (forced blank) : tous les pixels == 0xFF000000."),
         ("sprites_evaluated_each_line", "sprite 0 en y=9 (visible rangees 10..=17), tuile 0 : `put_vram(0,0x0080)` rangee 0 -> la rangee de sortie 10 montre le pixel (col x) ; la rangee 9 et la rangee 11 (donnees vides) non."),
         ("range_over_flag_in_frame", "33 sprites sur une meme rangee de sortie (meme y = 20, tuile 0, donnees vides : sprites i = 0..31 a x = 8*i, sprite 32 a x = -8), les 95 autres sprites a x = -200 (y = 0) : apres `at(p,225,1)`, `read(0x3E,0,false) & 0x80 != 0` (range_over vrai). Apres exactement 262*340 + 1 steps depuis le debut (le dot 0 de v=0 de la frame suivante vient d'etre traite, aucune ligne de sprites evaluee depuis), `read(0x3E,0,false) & 0x80 == 0` (drapeau remis a faux).")])
