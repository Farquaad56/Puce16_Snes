from _dsl import T

CORE = "crates/puce16-core"
SRC = CORE + "/src"
TST = CORE + "/tests"

CHAIN = ("Execution strictement sequentielle (D-020): cette tache suppose que toutes les precedentes sont terminees. "
         "Ne modifie QUE les fichiers de 'Write only in'.")

HELP = ("Aides de test (a recopier dans le fichier de test): "
        "`fn put_vram(p: &mut Ppu, addr: u16, w: u16) { p.write(0x15,0x80); p.write(0x16,(addr & 0xFF) as u8); p.write(0x17,(addr >> 8) as u8); p.write(0x18,(w & 0xFF) as u8); p.write(0x19,(w >> 8) as u8); }` ; "
        "`fn put_cgram(p: &mut Ppu, i: u8, w: u16) { p.write(0x21,i); p.write(0x22,(w & 0xFF) as u8); p.write(0x22,(w >> 8) as u8); }`. "
        "Un `Ppu::new(Region::Ntsc)` neuf est en forced blank: toutes ces ecritures sont autorisees. Chaque test construit SA scene sur un `Ppu` neuf.")

HELP_OAM = ("Aides OAM (a recopier): "
            "`fn load_oam(p: &mut Ppu, img: &[u8; 544]) { p.write(0x02,0); p.write(0x03,0); for b in img.iter() { p.write(0x04,*b); } }` ; "
            "`fn spr(img: &mut [u8; 544], i: usize, x: i16, y: u8, tile: u8, attr: u8, large: bool) { img[i*4] = (x & 0xFF) as u8; img[i*4+1] = y; img[i*4+2] = tile; img[i*4+3] = attr; "
            "let sh = (i % 4) * 2; let hb = 512 + i / 4; img[hb] &= !(3u8 << sh); let x8 = ((x >> 8) & 1) as u8; img[hb] |= (x8 | if large { 2 } else { 0 }) << sh; }`. "
            "Une image OAM neuve est `[0u8; 544]`: ATTENTION, un sprite a zero a x = 0, y = 0, 8x8: il est visible aux rangees 1..=8. Pour sortir les sprites inutilises de la zone visible, "
            "utilise `fn hide_all(img: &mut [u8; 544]) { for i in 0..128 { spr(img, i, -200, 0, 0, 0, false); } }` (x = -200 : hors ecran pour toutes les tailles, donc jamais dans la liste, et different de -256 qui compte toujours); "
            "puis place les sprites voulus. Les regles d'attribut sont : `attr = V H PP CCC t` (bit 7 = V flip, bit 6 = H flip, bits 5-4 = priorite, bits 3-1 = palette, bit 0 = bit de table de noms).")

# ------------------------------------------------------------------ E05 part B: sprites et mode 7 (E05-10..13)

T("E05-10", "ppu/obj.rs: tailles, decodage d'un sprite et liste des 32 sprites d'une rangee",
  size=140,
  context="""Debut des sprites (spec 05b section 6). Cette tache fait l'EVALUATION de la liste (quels sprites touchent la rangee `r`); la recuperation des pixels est en E05-11.
La rangee `r = v - 1` est la rangee de sortie (0 = premiere ligne visible). """ + CHAIN + " " + HELP + " " + HELP_OAM,
  embed=["05b_PPU_RENDER.md#6.1 Tables", "05b_PPU_RENDER.md#6.2 Evaluation de la liste (32 sprites max)"],
  write=[SRC + "/ppu/obj.rs", TST + "/e05_10_sprite_list.rs"],
  spec="""Dans `ppu/obj.rs`:
```rust
pub fn obj_dims(obj_size: u8, large: bool) -> (u8, u8);       // (largeur, hauteur) en pixels ; obj_size 0..=7 (> 7: comme 7)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Sprite { pub x: i16, pub y: u8, pub tile: u8, pub tbit: bool, pub pal: u8, pub prio: u8, pub hflip: bool, pub vflip: bool, pub large: bool }
impl Ppu {
    pub fn sprite(&self, i: usize) -> Sprite;                           // decode le sprite i (0..128) depuis l'OAM
    pub fn sprite_list(&self, r: u16) -> (Vec<u8>, bool);               // (indices dans l'ordre d'evaluation, depassement de 32)
}
```
`obj_dims(size, large)` (petit / grand) : 0 -> 8x8 / 16x16 ; 1 -> 8x8 / 32x32 ; 2 -> 8x8 / 64x64 ; 3 -> 16x16 / 32x32 ; 4 -> 16x16 / 64x64 ; 5 -> 32x32 / 64x64 ; 6 -> 16x32 / 32x64 ; 7 -> 16x32 / 32x32.
`sprite(i)` : octets de l'OAM basse `oam.byte(i*4 + k)` : k=0 X bits 7-0, k=1 Y, k=2 tuile, k=3 attribut `VHPP CCCt`. OAM haute : octet `512 + i/4`, bit `(i%4)*2` = X bit 8, bit `(i%4)*2 + 1` = grand. `x` = valeur 9 bits en signe : `x9 = lo | x8 << 8` ; `x = if x9 >= 256 { x9 - 512 } else { x9 }`. `tbit = attr & 1`, `pal = (attr >> 1) & 7`, `prio = (attr >> 4) & 3`, `hflip = attr & 0x40`, `vflip = attr & 0x80`.
`sprite_list(r)` (spec 6.2, taille lue avec `regs.obj_size`) :
```
first = if oam.rotation() { ((oam.internal() >> 2) & 0x7F) as usize } else { 0 }
list = [] ; overflow = false
pour k in 0..128 : i = (first + k) & 0x7F ; s = sprite(i) ; (w, h) = obj_dims(regs.obj_size, s.large)
    row = ((r as i32) - (s.y as i32) - 1) & 0xFF ; si row >= h : continuer
    si !(s.x == -256 || (s.x as i32) > -(w as i32)) : continuer
    si list.len() == 32 : overflow = true ; sortir de la boucle
    list.push(i)
```""",
  tests=[("obj_dims_table", "`obj_dims(0,false)==(8,8)`, `(0,true)==(16,16)` ; `(1,false)==(8,8)`, `(1,true)==(32,32)` ; `(2,true)==(64,64)` ; `(3,false)==(16,16)`, `(3,true)==(32,32)` ; `(4,true)==(64,64)` ; `(5,false)==(32,32)`, `(5,true)==(64,64)` ; `(6,false)==(16,32)`, `(6,true)==(32,64)` ; `(7,false)==(16,32)`, `(7,true)==(32,32)`."),
         ("decode_sprite", "`spr(&mut img,5,-1,0x12,0x34,0xE5,true)` puis `load_oam` : `p.sprite(5)==Sprite{x:-1,y:0x12,tile:0x34,tbit:true,pal:2,prio:2,hflip:true,vflip:true,large:true}`. Un autre sprite du meme octet haut (indice 4, 6, 7) n'est pas modifie : `p.sprite(4).large==false`."),
         ("decode_x_range", "`spr(.., 0, 255, ..)` -> `sprite(0).x==255` ; `spr(.., 1, -256, ..)` -> `sprite(1).x==-256` ; `spr(.., 2, 0, ..)` -> 0 ; `spr(.., 3, -2, ..)` -> -2 ; `spr(.., 4, 100, .., large=true)` : `sprite(4).x==100`, large vrai."),
         ("row_visibility_y255", "sprite 0 (x=10, y=255, 8x8) : `sprite_list(0)` contient 0 ; `sprite_list(7)` contient 0 ; `sprite_list(8)` ne contient pas 0 (rangee 8 >= hauteur 8). Les autres sprites sont cachees par `hide_all`."),
         ("row_visibility_y0", "sprite 0 (x=10, y=0) : absent de `sprite_list(0)` ; present pour r = 1..=8 ; absent pour r = 9. Sprite (y=10) : present pour r = 11..=18 uniquement."),
         ("large_height", "`write(0x01,0x00)` (taille 0), sprite 0 grand (16x16) y=255 : present pour r = 0..=15, absent pour r = 16."),
         ("x_range_small", "sprites 8x8 sur la rangee r=1 (y=0) : x=-7 present ; x=-8 absent ; x=-9 absent ; x=-256 present ; x=255 present ; x=0 present."),
         ("x_range_large", "sprite grand 16x16 : x=-15 present ; x=-16 absent."),
         ("limit_32", "33 sprites (indices 0..=32) identiques (x=10 + i, y=0, 8x8), les autres caches ; `sprite_list(1)` : liste de 32 indices `[0,1,...,31]` et overflow == true. Avec 32 sprites seulement (indices 0..=31) : liste de 32, overflow == false."),
         ("invisible_do_not_count", "40 sprites dont seuls les indices pairs touchent la rangee (les impairs ont y=100) : `sprite_list(1)` contient 20 indices pairs, overflow == false."),
         ("rotation_first_sprite", "sprites 0 et 5 visibles (rangee 1), les autres caches. Sans rotation : `sprite_list(1).0==[0,5]`. Apres `load_oam` puis `write(0x02,10); write(0x03,0x80)` (OAMADD 10 -> internal 20 -> first = 5, rotation vrai) : `sprite_list(1).0==[5,0]`. Avec `write(0x02,12); write(0x03,0x80)` (internal 24 -> first = 6, l'ordre est 6..127 puis 0..5) : `[0,5]`.")])

T("E05-11", "ppu/obj.rs: tranches de sprites, tampon de ligne, limites et erratum de flip",
  size=150,
  context="""Suite des sprites (spec 05b 6.3): pour une rangee `r`, remplit un tampon de 256 pixels `line_obj`. Ajoute au `Ppu` le champ `line_obj: [Option<Pix>; 256]` (None) -- il est DERIVE (calcule a dot 0 de chaque ligne) donc exclu du hash; les etats sauvegardes se prennent entre deux frames (E08).
Le lecture du pixel et les drapeaux `range_over` / `time_over` sont exposes pour le test. """ + CHAIN + " " + HELP + " " + HELP_OAM,
  embed=["05b_PPU_RENDER.md#6.3 Recuperation des tranches (slivers, 34 max) et tampon de ligne"],
  write=[SRC + "/ppu/obj.rs", SRC + "/ppu/mod.rs", TST + "/e05_11_sprite_pixels.rs"],
  spec="""Ajoute a `ppu/obj.rs` :
```rust
impl Ppu {
    pub fn render_obj_line(&mut self, r: u16);       // evalue la rangee r et remplit line_obj
    pub fn obj_pixel(&self, x: usize) -> Option<Pix>; // line_obj[x] (None si x >= 256)
}
```
`render_obj_line(r)` :
1. `line_obj = [None; 256]` ; `(list, overflow) = sprite_list(r)` ; si `overflow` : `range_over = true` (ne repasse JAMAIS a false ici).
2. `slivers = 0` ; pour `i` dans `list.iter().rev()` (du dernier au premier) : `s = sprite(i)` ; `(w, h)` ; `row = ((r as i32) - (s.y as i32) - 1) & 0xFF` ; pour `c` dans `0..(w/8)` : `sx = s.x as i32 + c * 8` ; `counts = (sx > -8 && sx < 256) || s.x == -256` ; si `!counts` continuer ; si `slivers == 34` : `time_over = true` et ARRETER toute la boucle (fin) ; `slivers += 1` ; `fetch_sliver(s, c, row, sx)`.
3. `fetch_sliver` : `tc = if s.hflip { w/8 - 1 - c } else { c }` ; `rv = row` ; si `s.vflip` : si `w == h` (carre) `rv = h - 1 - row` ; sinon (16x32, 32x64) : `half = h / 2 ; rv = (row / half) * half + (half - 1 - row % half)` (erratum materiel : chaque moitie est retournee sur place) ; `tile_row = rv >> 3 ; fine_y = rv & 7`.
   `tx = ((s.tile & 15) as u32 + tc) & 15 ; ty = ((s.tile >> 4) as u32 + tile_row) & 15 ; tile8 = ty * 16 + tx` ; `addr = (regs.obj_base + (if s.tbit { regs.obj_name_gap } else { 0 }) + tile8 * 16 + fine_y) & 0x7FFF` (calcul en u32) ; `lo = vram[addr]` ; `hi = vram[(addr + 8) & 0x7FFF]`.
   Pour `px` in `0..8` : `bit = if s.hflip { px } else { 7 - px }` ; `i = ((lo >> bit) & 1) | (((lo >> (8 + bit)) & 1) << 1) | (((hi >> bit) & 1) << 2) | (((hi >> (8 + bit)) & 1) << 3)` ; `dest = sx + px` ; si `i != 0 && 0 <= dest < 256` : `line_obj[dest] = Some(Pix { rgb: cgram[obj_palette_index(s.pal, i)], prio: s.prio, obj_pal: s.pal })` (ecrase : le sprite d'indice le plus bas gagne car il est traite en dernier).
Accesseurs publics : `range_over()`, `time_over()` (existent depuis E04-10).""",
  tests=[("basic_pixel", "OBJSEL 0 ; sprite 0 (x=10, y=255, tuile 0, attr 0, petit) ; `put_vram(&mut p,0,0x0080)` (rangee 0, indice 1 au pixel 0) ; `put_cgram(&mut p,129,0x1111)` ; `render_obj_line(0)` : `obj_pixel(10)==Some(Pix{rgb:0x1111,prio:0,obj_pal:0})` ; `obj_pixel(9)==None` ; `obj_pixel(11)==None`."),
         ("row_selects_data", "meme scene + `put_vram(&mut p,1,0x8000)` (rangee 1 : indice 2) + `put_cgram(&mut p,130,0x2222)` : `render_obj_line(1)` -> `obj_pixel(10)==Some(Pix{rgb:0x2222,..})` ; `render_obj_line(2)` -> `obj_pixel(10)==None` (rangee 2 vide)."),
         ("hflip_8x8", "attr 0x40 : le pixel du bord gauche des donnees apparait a droite : `obj_pixel(17)==Some(..0x1111..)` et `obj_pixel(10)==None`."),
         ("palette_and_priority", "attr 0x28 (prio 2, palette 4) avec `put_cgram(&mut p,193,0x3333)` : `obj_pixel(10)==Some(Pix{rgb:0x3333,prio:2,obj_pal:4})`."),
         ("lowest_index_wins", "sprites 0 (attr 0, palette 0) et 1 (attr 0x02 = palette 1) a la meme position x=10, y=255, meme tuile ; `put_cgram(129,0x1111)`, `put_cgram(145,0x4444)` : `obj_pixel(10)==Some(Pix{rgb:0x1111,..})` (sprite 0 gagne)."),
         ("vflip_8x8", "sprite attr 0x80 (V flip), donnees seulement en rangee 0 : `render_obj_line(7)` -> `obj_pixel(10)` Some (rv = 7 - 7 = 0) ; `render_obj_line(0)` -> None (rv = 7)."),
         ("large_two_tiles", "OBJSEL 0 (16x16 grand) ; sprite 0 grand, x=20, y=255, tuile 0x01 : `put_vram(&mut p,16,0x0080)` (tuile 1 rangee 0 indice 1), `put_vram(&mut p,32,0x8000)` (tuile 2 rangee 0 indice 2), `put_vram(&mut p,272,0x0080)` (tuile 17 = 1 + 16, rangee 0) ; cgram 129 = 0x1111, 130 = 0x2222. `render_obj_line(0)` : `obj_pixel(20)` rgb 0x1111 ; `obj_pixel(28)` rgb 0x2222. `render_obj_line(8)` : `obj_pixel(20)` rgb 0x1111 (tuile 17, tile_row 1) ; `obj_pixel(28)` None (tuile 18 vide)."),
         ("large_hflip", "meme scene avec attr 0x40 : `render_obj_line(0)` : `obj_pixel(27)` rgb 0x2222 (colonne c=0 lit la tuile 2 retournee : le pixel de donnees 0 passe en position 7) et `obj_pixel(35)` rgb 0x1111 (c=1 lit la tuile 1 retournee)."),
         ("name_table_bit", "attr bit 0 = 1 : l'adresse ajoute `obj_name_gap` (0x1000 avec OBJSEL 0) : `put_vram(&mut p,0x1000,0x0080)` -> `obj_pixel(10)` Some. Avec `write(0x01,0x02)` (obj_base = 2 << 13 = 0x4000) et attr 0 : `put_vram(&mut p,0x4000,0x0080)` -> Some."),
         ("clipping_left_right", "sprite 8x8 x=-4, donnees `put_vram(&mut p,0,0x0001)` (pixel de donnees 7) -> dest = -4 + 7 = 3 : `obj_pixel(3)` Some, `obj_pixel(0)` None. Sprite x=250 avec pixel 7 -> dest 257 : aucune panique, `obj_pixel(255)==None`."),
         ("time_over_35_slivers", "`write(0x01,0x40)` (taille 2 : 8x8 / 64x64) ; sprites 0..=3 grands (64x64) en x = 0, 64, 128, 192 (y=255) = 4*8 = 32 tranches ; sprites 4, 5, 6 petits en x = 100, 110, 120 = 3 tranches ; total 35. `put_vram(&mut p,0,0x0080)` (tuile 0 rangee 0 indice 1), `put_vram(&mut p,112,0x0080)` (tuile 7 rangee 0), cgram 129 = 0x1111. `render_obj_line(0)` : `time_over()==true` ; `obj_pixel(0)` Some (colonne c=0 du sprite 0) ; `obj_pixel(56)==None` (la 35e tranche, c=7 du sprite 0, est abandonnee)."),
         ("no_time_over_34", "meme scene SANS le sprite 6 (34 tranches) : `time_over()==false` et `obj_pixel(56)` Some (tuile 7 du sprite 0)."),
         ("x_minus_256_counts_slivers", "`write(0x01,0x40)` ; 5 sprites grands (64x64) tous en x=-256, y=255 : `render_obj_line(0)` : `time_over()==true` (5*8 = 40 tranches comptees) ; aucun pixel n'est ecrit (`obj_pixel(x)==None` pour x = 0, 100, 255) ; `range_over()==false`."),
         ("range_over_flag", "33 sprites visibles : apres `render_obj_line(1)`, `range_over()==true` (et `read(0x3E,0,false)==0x81`) ; une rangee suivante sans depassement ne remet PAS le drapeau a faux."),
         ("erratum_tall_vflip", "`write(0x01,0xC0)` : taille 6 (petit 16x32). Sprite 0 petit, x=10, y=255, tuile 0, attr 0x80 (V flip). `put_vram(&mut p,263,0x0080)` (tuile 16 rangee 7, indice 1), `put_vram(&mut p,775,0x8000)` (tuile 48 rangee 7, indice 2), cgram 129 = 0x1111, 130 = 0x2222. `render_obj_line(0)` : `obj_pixel(10)` rgb 0x1111 (row 0 : rv = (0/16)*16 + (15 - 0) = 15 -> tile_row 1, fine_y 7). `render_obj_line(16)` : `obj_pixel(10)` rgb 0x2222 (row 16 : rv = 16 + 15 = 31 -> tile_row 3, fine_y 7 ; un flip complet donnerait rv = 15)."),
         ("square_vflip_full", "taille 0, sprite 0 grand (16x16), V flip (attr 0x80), x=10, y=255, tuile 0x01 ; `put_vram(&mut p,279,0x0080)` (tuile 17 = 1 + 16, rangee 7 : adresse 17*16 + 7) ; `render_obj_line(0)` : row = 0 -> rv = 15 - 0 = 15 -> tile_row 1, fine_y 7 ; la colonne c=0 lit la tuile 17 : `obj_pixel(10)` Some (flip complet des sprites carres).")])

T("E05-12", "ppu/mode7.rs: echantillonnage mode 7 (m7_sample, clip13)",
  size=100,
  context="Fonctions PURES de la transformation affine du mode 7 (spec 05b section 8), sans VRAM. Entiers 32 bits, decalages arithmetiques. " + CHAIN,
  embed=["05b_PPU_RENDER.md#8. Mode 7 (fonction pure + pixel)"],
  write=[SRC + "/ppu/mode7.rs", TST + "/e05_12_m7_sample.rs"],
  spec="""Ajoute a `ppu/mode7.rs` (garde `M7Regs` de E04-05; ses champs `a b c d cx cy hofs vofs flip_x flip_y fill outside` sont `pub`):
```rust
pub fn clip13(n: i32) -> i32;                        // si n & 0x2000 != 0 { n | !0x3FF } sinon { n & 0x3FF }
pub fn m7_sample(r: &M7Regs, x: i32, v: i32) -> (i32, i32);     // (px, py) dans la carte 1024x1024
```
`m7_sample` : `sx = if r.flip_x { 255 - x } else { x }` ; `sy = if r.flip_y { 255 - v } else { v }` ; `a,b,c,d = r.a as i32, ...` ; `dh = clip13(r.hofs as i32 - r.cx as i32)` ; `dv = clip13(r.vofs as i32 - r.cy as i32)` ;
`ox = ((a * dh) & !63) + ((b * dv) & !63) + ((b * sy) & !63) + ((r.cx as i32) << 8)` ;
`oy = ((c * dh) & !63) + ((d * dv) & !63) + ((d * sy) & !63) + ((r.cy as i32) << 8)` ;
`px = (ox + a * sx) >> 8` ; `py = (oy + c * sx) >> 8` (decalage arithmetique) ; retourne `(px, py)`.""",
  tests=[("clip13_cases", "`clip13(5)==5` ; `clip13(0x3FF)==1023` ; `clip13(0x400)==0` ; `clip13(0x1FFF)==1023` ; `clip13(-1)==-1` ; `clip13(-1024)==-1024` ; `clip13(0x2005)==-1019`."),
         ("identity", "a=d=0x100, b=c=0, cx=cy=hofs=vofs=0 : `m7_sample(&r,7,9)==(7,9)` ; `(0,0)->(0,0)` ; `(255,223)->(255,223)`."),
         ("scroll", "identite + `hofs=10`, `vofs=5` : `m7_sample(&r,7,9)==(17,14)`."),
         ("flips", "identite + `flip_x` : `m7_sample(&r,7,9)==(248,9)` ; identite + `flip_y` : `(7,246)` ; les deux : `(248,246)`."),
         ("zoom_x2", "a=d=0x80 : `m7_sample(&r,7,9)==(3,4)` ((896)>>8 = 3 ; oy = (128*9=1152)&!63 = 1152 -> 1152>>8 = 4) ; `m7_sample(&r,8,9)==(4,4)`."),
         ("rotation_90", "a=0, b=0x100, c=-256, d=0 (cx=cy=hofs=vofs=0) : `m7_sample(&r,7,9)==(9,-7)`."),
         ("center_offset", "identite + `cx=100, cy=50, hofs=100, vofs=50` : `m7_sample(&r,7,9)==(107,59)`."),
         ("mask_63_matters", "a=63, d=0x100, b=c=0, hofs=1, vofs=0, cx=cy=0 : ox = (63*1)&!63 = 0 -> `m7_sample(&r,4,0)==(0,0)` (sans le masquage on obtiendrait (63 + 252) >> 8 = 1)."),
         ("negative_shift_is_arithmetic", "a=0, b=0, c=-256, d=0x100, reste a 0 : `m7_sample(&r,3,0)==(0,-3)`."),
         ("hofs_minus_cx_clipped", "identite, cx=0, hofs=0x1005 (bit 12 a 1, bit 13 a 0) : dh = clip13(0x1005) = 0x005 -> `m7_sample(&r,0,0)==(5,0)`. hofs=-1 (0xFFFF as i16 = -1) : dh = clip13(-1) = -1 -> `m7_sample(&r,0,0)==(-1,0)`.")])

T("E05-13", "ppu/mode7.rs: pixel mode 7 (BG1) et EXTBG (BG2)",
  size=130,
  context="""Utilise `m7_sample` pour fabriquer les pixels du mode 7 (spec 05b section 8). `Ppu::m7_pixel(n, x, v)` renvoie le pixel de BG1 (n = 0) ou de BG2 en mode EXTBG (n = 1). Pour le mode 7 la VRAM est lue par OCTETS : la carte 128x128 dans les octets BAS des mots 0..16383, les pixels (8 bits) dans les octets HAUTS. """ + CHAIN + " " + HELP,
  embed=["05b_PPU_RENDER.md#8. Mode 7 (fonction pure + pixel)", "05b_PPU_RENDER.md#9. Mosaique"],
  write=[SRC + "/ppu/mode7.rs", TST + "/e05_13_m7_pixel.rs"],
  spec="""Ajoute a `ppu/mode7.rs`:
```rust
impl Ppu { pub fn m7_pixel(&self, n: usize, x: usize, v: usize) -> Option<Pix>; }
```
1. `None` si `regs.bg_mode != 7` ; `None` si `n == 1 && !extbg` ; `None` si `n > 1`.
2. mosaique : `(xm, vm) = mosaic_xy(regs.mosaic_size(), (regs.mosaic_en >> n) & 1 != 0, x, v)`.
3. `(px, py) = m7_sample(&m7, xm as i32, vm as i32)`.
4. Si `(px | py) & !0x3FF != 0` (hors 0..1023) : si `!m7.outside` : `px &= 0x3FF ; py &= 0x3FF` puis comme dans la boucle normale ; sinon si `!m7.fill` : `None` ; sinon `tile = 0` et `fx = px & 7`, `fy = py & 7` (avec px/py non masques).
   Cas normal : `tile = vram[((py >> 3) << 7) + (px >> 3)] & 0xFF` ; `fx = px & 7`, `fy = py & 7`.
5. `idx = (vram[tile * 64 + fy * 8 + fx] >> 8) & 0xFF` ; `idx == 0` -> `None`. (adresses & 0x7FFF)
6. BG1 (`n == 0`) : `rgb = if regs.direct_color { direct_color_m7(idx) } else { cgram[idx] }`, `prio = 0`. BG2 EXTBG (`n == 1`) : `prio = idx >> 7`, `i7 = idx & 0x7F` ; `i7 == 0` -> `None` ; `rgb = cgram[i7]`.
`obj_pal` vaut toujours 0.""",
  tests=[("scene_g_identity", "Scene G : `write(0x05,0x07)` ; matrice identite : `write(0x1B,0x00); write(0x1B,0x01); write(0x1E,0x00); write(0x1E,0x01)` ; carte (VMAIN 0, ecriture octet bas) : `fn put_m7_map(p,idx,tile) { p.write(0x15,0x00); p.write(0x16,(idx & 0xFF) as u8); p.write(0x17,(idx >> 8) as u8); p.write(0x18,tile); }` ; pixels (VMAIN 0x80, octet haut) : `fn put_m7_pix(p,tile,fy,fx,c) { let a = tile*64+fy*8+fx; p.write(0x15,0x80); p.write(0x16,(a & 0xFF) as u8); p.write(0x17,(a >> 8) as u8); p.write(0x19,c); }` (tile, fy, fx en u16). `put_m7_map(0,1)`, `put_m7_pix(1,0,0,5)`, `put_cgram(5,0x0555)` : `m7_pixel(0,0,0)==Some(Pix{rgb:0x0555,prio:0,obj_pal:0})`."),
         ("other_positions", "Scene G + `put_m7_map(1,2)`, `put_m7_pix(2,0,1,7)`, `put_cgram(7,0x0777)`, `put_m7_map(128,3)`, `put_m7_pix(3,0,0,9)`, `put_cgram(9,0x0999)` : `m7_pixel(0,9,0)` rgb 0x0777 (px=9 -> carte (1,0) = tuile 2, fx = 1) ; `m7_pixel(0,0,8)` rgb 0x0999 (py=8 -> carte index 128) ; `m7_pixel(0,1,0)==None` (tuile 1 pixel (0,1) = 0)."),
         ("not_mode7_or_wrong_layer", "Scene G avec `write(0x05,0x01)` : `m7_pixel(0,0,0)==None`. En mode 7 sans EXTBG : `m7_pixel(1,0,0)==None` ; `m7_pixel(2,0,0)==None`."),
         ("wrap_outside_false", "Scene G + `write(0x0D,0xFC); write(0x0D,0x03)` (M7HOFS = 1020) : `m7_pixel(0,4,0)` = pixel (px 1024 -> repete a 0, py 0) = `Some(Pix{rgb:0x0555,..})`."),
         ("outside_transparent", "meme scene + `write(0x1A,0x80)` (outside=1, fill=0) : `m7_pixel(0,4,0)==None`, mais `m7_pixel(0,0,0)` (a l'interieur) reste Some."),
         ("outside_fill_tile0", "meme scene + `write(0x1A,0xC0)` (outside=1, fill=1) + `put_m7_pix(0,0,0,9)` + cgram 9 = 0x0999 : `m7_pixel(0,4,0)` rgb 0x0999 (tuile 0, fx = 1024 & 7 = 0, fy = 0)."),
         ("flip_x", "Scene G + `write(0x1A,0x01)` (flip_x) + `put_m7_map(31,4)` + `put_m7_pix(4,0,7,3)` + `put_cgram(3,0x0333)` : `m7_pixel(0,0,0)` rgb 0x0333 (sx = 255 -> px = 255, carte (31,0) = tuile 4, fx = 7)."),
         ("extbg", "Scene G + `write(0x33,0x40)` (EXTBG) + `put_m7_pix(1,0,0,0x85)` + `put_cgram(0x85,0x0222)` + `put_cgram(5,0x0111)` : `m7_pixel(0,0,0)==Some(Pix{rgb:0x0222,prio:0,obj_pal:0})` ; `m7_pixel(1,0,0)==Some(Pix{rgb:0x0111,prio:1,obj_pal:0})`. Avec l'indice 0x80 (`put_m7_pix(1,0,0,0x80)`, cgram 0x80 = 0x0444) : BG1 rgb 0x0444 et `m7_pixel(1,0,0)==None` (i7 = 0). Avec l'indice 0x05 : BG2 prio 0."),
         ("direct_color", "Scene G + `write(0x30,0x01)` + `put_m7_pix(1,0,0,0xFF)` : `m7_pixel(0,0,0)` rgb 0x639C. En EXTBG, BG2 reste indexe (cgram[0x7F])."),
         ("mosaic", "Scene G + `write(0x06,0x11)` (taille 2, BG1) : `m7_pixel(0,1,0)` = pixel de (xm=0, vm=0) = `Some(Pix{rgb:0x0555,..})` alors que sans mosaique `m7_pixel(0,1,0)==None`.")])
