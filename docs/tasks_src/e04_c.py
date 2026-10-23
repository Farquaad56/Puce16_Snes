from _dsl import T

CORE = "crates/puce16-core"
SRC = CORE + "/src"
TST = CORE + "/tests"

CHAIN = ("Execution strictement sequentielle (D-020): cette tache suppose que toutes les precedentes sont terminees. "
         "Ne modifie QUE les fichiers de 'Write only in'.")

AT = ("Dans les tests, utilise l'aide locale `fn at(p: &mut Ppu, v: u32, dot: u32) { for _ in 0..(v * 340 + dot) { p.step_dot(); } }` "
      "(valide pour NTSC, frame 0, sans entrelace; position atteinte = (v, dot) apres ces steps) et `fn set_backdrop(p: &mut Ppu, w: u16) { p.write(0x21, 0); p.write(0x22, (w & 0xFF) as u8); p.write(0x22, (w >> 8) as u8); }`.")

# ------------------------------------------------------------------ E04 part C: sortie video du PPU (E04-13..15)

T("E04-13", "ppu/output.rs: conversion couleur BGR555 -> XRGB8888 et rangee de sortie",
  size=70,
  context="Fonctions pures de la sortie video (spec 05 section 6, decision D-010). Pas de flottants (D-009). " + CHAIN,
  embed=["05_PPU.md#6. Sortie: framebuffer"],
  write=[SRC + "/ppu/output.rs", TST + "/e04_13_color.rs"],
  spec="""Dans `ppu/output.rs` (garde `FrameView`, `FrameBuf`, `pixel_double`, `make_view` de E01-05):
```rust
pub fn channel8(c5: u8, brightness: u8) -> u8;          // c5 0..=31, brightness 0..=15 ; c5b = c5 as u32 * brightness as u32 / 15 ; ((c5b << 3) | (c5b >> 2)) as u8
pub fn bgr555_to_xrgb(rgb: u16, brightness: u8) -> u32; // r5 = rgb & 31, g5 = (rgb >> 5) & 31, b5 = (rgb >> 10) & 31 ; 0xFF000000 | r8 << 16 | g8 << 8 | b8
pub fn output_row(r: usize, field: bool, interlaced: bool) -> usize;   // interlaced: 2 * r + field as usize ; sinon r
```
`brightness > 15` est traite comme 15 (`min`). Les bits 15 de `rgb` sont ignores.""",
  tests=[("channel8_extremes", "`channel8(31,15)==255` ; `channel8(31,0)==0` ; `channel8(0,15)==0` ; `channel8(1,15)==8` ; `channel8(16,15)==132` ; `channel8(31,1)==16` ; `channel8(31,7)==115` ; `channel8(31,8)==132` ; `channel8(10,5)==24`."),
         ("channel8_clamp", "`channel8(31,200)==255` (brightness plafonnee a 15)."),
         ("primary_colors", "`bgr555_to_xrgb(0x7FFF,15)==0xFFFFFFFF` ; `(0x001F,15)==0xFFFF0000` (rouge) ; `(0x03E0,15)==0xFF00FF00` (vert) ; `(0x7C00,15)==0xFF0000FF` (bleu) ; `(0x0000,15)==0xFF000000`."),
         ("brightness_scaling", "`bgr555_to_xrgb(0x7FFF,0)==0xFF000000` ; `(0x7FFF,8)==0xFF848484` ; `(0x0421,15)==0xFF080808` (r=g=b=1)."),
         ("bit15_ignored", "`bgr555_to_xrgb(0x801F,15)==bgr555_to_xrgb(0x001F,15)`."),
         ("output_row", "`output_row(0,false,false)==0` ; `(5,true,false)==5` ; `(5,false,true)==10` ; `(5,true,true)==11` ; `(237,true,true)==475` ; `(223,false,true)==446`.")])

T("E04-14", "ppu/mod.rs: framebuffer, pixel de fond (backdrop) et FrameView",
  size=140,
  context="""Le PPU ecrit maintenant une image. Sans rendu des fonds (E05), chaque pixel visible vaut la couleur du backdrop (CGRAM mot 0) a la luminosite courante, ou noir en forced blank.
Le point d'extension du rendu est la fonction privee `pixel_bgr555(&self, x: usize) -> u16` (renvoie ici `cgram.word(0)`): E05 la remplacera par la composition complete. """ + CHAIN + " " + AT,
  embed=["05_PPU.md#5.3 Evenements par (v, dot), dans CET ordre dans `step_dot`", "05_PPU.md#6. Sortie: framebuffer"],
  write=[SRC + "/ppu/mod.rs", TST + "/e04_14_ppu_frame.rs"],
  spec="""Dans `ppu/mod.rs`: ajoute au `Ppu` les champs `fb: FrameBuf`, `view: Vec<u32>` (copie compacte de la derniere frame complete), `view_w: usize`, `view_h: usize` (initialement 256x224, noir `0xFF000000`).
```rust
pub fn frame(&self) -> FrameView<'_>;     // output::make_view(view_w, view_h, &view)
fn pixel_bgr555(&self, x: usize) -> u16;  // E04: self.cgram.word(0)   (E05: composition)
```
`step_dot` (sur la position COURANTE, dans l'ordre de la section 5.3), apres les evenements du point 2:
- point 3: si `1 <= v <= vis_last` et `22 <= dot <= 277` (`vis_last = vblank_start() - 1`): `x = (dot - 22) as usize` ; `row = output_row((v - 1) as usize, field, interlace)` ; `argb = if forced_blank { 0xFF000000 } else { bgr555_to_xrgb(pixel_bgr555(x), brightness) }` ; `fb.set(x, row, argb)`. (Colonne `x` seulement: le mode hi-res vient en E05.)
- a `dot == 0` et `v == vblank_start()` (APRES le rechargement OAM): capture de la frame: `w = 256` ; `h = vis_last * (if interlace {2} else {1})` ; `fb.set_view(w, h)` ; `view = fb.compact()` ; `view_w = w` ; `view_h = h`.
`Ppu::reset()` remet aussi `fb = FrameBuf::new()` et `view` a 256x224 noir. Le framebuffer et `view` ne font PAS partie de `hash_into` (etat derive; ne les ajoute pas).""",
  tests=[("initial_frame", "`Ppu::new(Ntsc).frame()`: width==256, height==224, pixels.len()==57344, tous egaux a 0xFF000000."),
         ("forced_blank_black", "CGRAM mot 0 = 0x001F (`set_backdrop`), forced blank (defaut) ; `at(p,225,1)` : `frame()` 256x224, tous les pixels 0xFF000000."),
         ("backdrop_red", "`set_backdrop(&mut p,0x001F)` ; `write(0x00,0x0F)` ; `at(p,225,1)` : `frame()` 256x224 et tous les 57344 pixels == 0xFFFF0000."),
         ("frame_not_captured_early", "meme preparation ; `at(p,225,0)` (le step du dot 0 de la ligne 225 n'est pas encore execute) : `frame()` est encore l'image initiale noire (tous 0xFF000000) ; un `step_dot` de plus -> tous 0xFFFF0000."),
         ("brightness_second_frame", "apres la preparation rouge et `at(p,225,1)`, `write(0x00,0x08)` ; 262*340 steps supplementaires : tous les pixels == 0xFF840000."),
         ("overscan_height", "`write(0x33,0x04)`, backdrop rouge, display on ; `at(p,240,1)` : `frame().height==239`, `pixels.len()==256*239`, tous 0xFFFF0000."),
         ("interlace_fields", "`write(0x33,0x01)`, backdrop rouge, display on ; `at(p,225,1)` (frame 0, field faux) : `frame().height==448` ; pixel (0,0) rouge, pixel (0,1) 0xFF000000, pixel (255,446) rouge, pixel (255,447) 0xFF000000, nombre de pixels rouges == 224*256. Frame suivante (field vrai, 263 lignes a la frame 0 en entrelace NTSC): apres 263*340 steps supplementaires, tous les 448*256 pixels sont rouges."),
         ("line_change_vertical", "backdrop rouge, display on ; `at(p,100,0)` puis `set_backdrop(&mut p,0x7C00)` (bleu) ; continuer jusqu'a (225,1) (125*340+1 steps) : les rangees 0..=98 sont rouges, les rangees 99..=223 sont bleues (0xFF0000FF) (la ligne v=100 donne la rangee 99)."),
         ("mid_line_change", "backdrop rouge, display on ; `at(p,10,150)` puis `set_backdrop` bleu ; continuer jusqu'a (225,1) : dans la rangee 9 les colonnes 0..=127 sont rouges et 128..=255 sont bleues (le dot 150 dessine la colonne 128 et vient APRES l'ecriture) ; la rangee 8 est entierement rouge ; la rangee 10 est entierement bleue."),
         ("reset_clears_image", "apres une frame rouge, `reset()` : `frame()` est a nouveau 256x224 noir."),
         ("hash_ignores_framebuffer", "A: `set_backdrop(rouge 0x001F)`, `write(0x00,0x0F)`, `at(p,225,1)`, puis `set_backdrop(&mut p,0)`. B: `set_backdrop(&mut p,0)`, `write(0x00,0x0F)`, `at(p,225,1)`. Les registres, memoires et compteurs sont identiques: `hash(A)==hash(B)` alors que `A.frame()` est rouge et `B.frame()` noir (le framebuffer n'est pas dans le hash).")])

T("E04-15", "snes.rs: le Snes reel affiche la sortie du PPU (jalon video V1)",
  size=100,
  context="""Le variant `Real` de `Snes` cesse d'avoir son propre tampon noir: `frame()` renvoie `sys.ppu.frame()`. Premier test de bout en bout ou du vrai code 65816 ecrit un registre PPU et ou l'image change.
Les champs `fb` et `compact` du variant `Real` (E03-34) deviennent inutiles et sont supprimes. Le variant `Pattern` ne change pas. """ + CHAIN,
  embed=["01_ARCHITECTURE.md#5. API publique de `puce16-core` (stable, ne pas changer sans question)"],
  write=[SRC + "/snes.rs", TST + "/e04_15_snes_video.rs"],
  spec="""Dans `snes.rs`:
- `Snes::frame(&self)` pour `Inner::Real { sys, .. }`: `sys.ppu.frame()`.
- Retire `fb` et `compact` de `Inner::Real` (et de `new_with`/`reset`); le variant `Pattern` garde les siens.
Les tests existants (E03-34/35, `frame_view_real`: image noire 256x224) doivent rester verts sans modification.
Programmes de test (LoRom, a mettre dans le test; `HDR` habituel: `mod common; #[path = "common/prog.rs"] mod prog;`, `Snes::new(&build_rom(&[(0, PROG)], &[]), None)`):
- `PROG_RED` = `78 18 FB  A9 00 8D 21 21  A9 1F 8D 22 21  A9 00 8D 22 21  A9 0F 8D 00 21  80 FE` : SEI ; CLC ; XCE ; LDA #$00 ; STA $2121 (CGADD=0) ; LDA #$1F ; STA $2122 ; LDA #$00 ; STA $2122 (couleur 0 = 0x001F, rouge) ; LDA #$0F ; STA $2100 (display on, luminosite 15) ; BRA -2. (25 octets.)
- `PROG_BLUE` = `78 18 FB  A9 00 8D 21 21  A9 00 8D 22 21  A9 7C 8D 22 21  A9 0F 8D 00 21  80 FE` (couleur 0 = 0x7C00, bleu).
- `PROG_DARK` = `78 18 FB  A9 00 8D 21 21  A9 1F 8D 22 21  A9 00 8D 22 21  A9 08 8D 00 21  80 FE` (rouge, luminosite 8).""",
  tests=[("red_frame", "`Snes::new` sur PROG_RED ; `run_frame()` deux fois : `frame().width==256`, `frame().height==224`, tous les 57344 pixels == 0xFFFF0000."),
         ("blue_frame", "idem avec PROG_BLUE : tous les pixels == 0xFF0000FF."),
         ("dark_frame", "idem avec PROG_DARK : tous les pixels == 0xFF840000 (luminosite 8)."),
         ("black_before_display", "ROM `[0x78,0x18,0xFB,0x80,0xFE]` (jamais d'ecriture PPU) : apres `run_frame()` tous les pixels == 0xFF000000 (forced blank)."),
         ("first_frame_complete", "PROG_RED : apres le PREMIER `run_frame()` seulement, tous les pixels sont deja 0xFFFF0000 (le programme finit sa configuration pendant la ligne 0, avant la premiere ligne visible)."),
         ("determinism", "deux `Snes` PROG_RED identiques : meme `state_hash` apres 2 frames ; un `Snes` PROG_BLUE a un hash different."),
         ("pattern_unchanged", "`Snes::new_test_pattern(Ntsc)` : apres `run_frame()` le pixel (0,0) vaut toujours 0xFF010040."),
         ("reset_clears_picture", "PROG_RED, 2 frames ; `reset()` puis `run_frame()` une fois : l'image est rouge a nouveau (le programme se re-execute) et `frame_count()==1`.")])
