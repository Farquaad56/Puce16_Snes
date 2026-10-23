from e06_a import CHAIN
from e03_f import TINY
from e08_b import DESK, DT, EMBD, TMP
from _dsl import T

DSRC = DESK + "/src"
CCORE = "crates/puce16-core"
CSRC = CCORE + "/src"
CTST = CCORE + "/tests"
UIDOC = ["08_INPUT_DESKTOP.md#4. Fenetre, video, filtres, plein ecran"]
DBGDOC = ["08_INPUT_DESKTOP.md#8. Debugger"]

# ------------------------------------------------------------------ E08 part C: video, config, manette (E08-25..31)

T("E08-25", "video.rs: filtres, ratio par nom, scanlines (pur)",
  size=100, crate="puce16-desktop",
  context="Fonctions pures du jalon D5: noms de filtre/ratio (config.json) et pre-traitement CPU `scanlines`. " + CHAIN,
  embed=UIDOC,
  write=[DSRC + "/video.rs", DT + "/e08_25_filters.rs"],
  spec="""Ajoute dans `video.rs` (sans modifier E01-10):
```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Filter { Nearest, Linear, Scanlines }
impl Filter {
    pub fn from_name(s: &str) -> Filter;      // "nearest" | "linear" | "scanlines" ; tout autre texte -> Nearest
    pub fn name(self) -> &'static str;        // les memes trois noms
    pub fn next(self) -> Filter;              // Nearest -> Linear -> Scanlines -> Nearest
}
pub fn ratio_from_name(s: &str) -> Ratio;     // "par87" -> Par87 ; tout autre texte -> Square
pub fn ratio_name(r: Ratio) -> &'static str;  // "square" | "par87"
pub fn scanlines_active(f: Filter, scale: usize) -> bool;     // vrai seulement si f == Scanlines et scale >= 2
/// assombrit les lignes IMPAIRES (y = 1, 3, 5...) de l'image RGBA: R, G, B -> (v * 3) / 4 (division entiere) ; alpha inchange. `width` en pixels ; hauteur = rgba.len() / (width * 4).
pub fn apply_scanlines(rgba: &mut [u8], width: usize);
```""",
  tests=[("filter_names", "`Filter::from_name(\"linear\")==Linear` ; `\"scanlines\"` -> Scanlines ; `\"nearest\"`, `\"\"`, `\"weird\"` -> Nearest ; `Linear.name()==\"linear\"`."),
         ("filter_cycle", "`Nearest.next()==Linear`, `Linear.next()==Scanlines`, `Scanlines.next()==Nearest`."),
         ("ratio_names", "`ratio_from_name(\"par87\")==Par87` ; `\"square\"`, `\"x\"` -> Square ; `ratio_name(Par87)==\"par87\"`."),
         ("scanlines_active", "`(Scanlines,1)` faux ; `(Scanlines,2)` vrai ; `(Scanlines,3)` vrai ; `(Nearest,3)` faux ; `(Linear,5)` faux."),
         ("apply_scanlines_values", "image 2x4 pixels (width 2) toute a `[200,100,40,255]` : lignes 0 et 2 inchangees ; lignes 1 et 3: `[150,75,30,255]` (200*3/4=150, 100*3/4=75, 40*3/4=30)."),
         ("apply_scanlines_alpha", "alpha reste 255 sur toutes les lignes, et 7 si alpha vaut 7 au depart."),
         ("apply_scanlines_odd_height", "image de 3 lignes : seule la ligne 1 est modifiee, pas de panique."),
         ("apply_scanlines_empty", "`apply_scanlines(&mut [], 256)` ne panique pas ; `width == 0` ne panique pas.")],
  accept=["cargo test -p puce16-desktop e08_25_", "cargo test -p puce16-desktop e01_10_"])

T("E08-26", "app.rs: configuration (chargement, application, ROM recentes, menus Video/Audio)",
  size=140, kind="ui", crate="puce16-desktop",
  context="Branche `config.rs` (E08-24) et `video.rs` (E08-25) dans l'interface: l'etat de la fenetre vient du fichier de config et y retourne. Code UI: pas de test automatique. " + CHAIN,
  embed=UIDOC + ["08_INPUT_DESKTOP.md#6. Fichiers"],
  write=[DSRC + "/app.rs", DSRC + "/main.rs"],
  spec="""- `App` gagne `cfg: config::Config` et `cfg_dirty: bool`. `main.rs` appelle `config::load()` au demarrage et le passe a `App::new`. Au constructeur: `scale`, `ratio` (via `video::ratio_from_name`), `filter` (`Filter::from_name`), `muted` viennent de la config ; `volume` est applique en E08-27.
- Chaque changement d'echelle, de ratio, de filtre, de volume, de muet ou d'option met `cfg` a jour et `cfg_dirty = true`. Quand `cfg_dirty` et qu'aucun changement n'a eu lieu depuis 1 s (ou a la fermeture), appeler `config::save(&cfg)` ; echec -> statut `ERROR: <message>` (pas de panique).
- Menu Fichier: sous-menu "ROM recentes" listant `cfg.recent_roms` (clic = `open_path`) ; `open_path` appelle `cfg.add_recent(path)` et `cfg.last_dir` = dossier de la ROM ; le dialogue `rfd` demarre dans `last_dir` s'il existe (`set_directory`).
- Menu Video: echelle 1..8, case "Ajuster a la fenetre" (`cfg.fit_window` : echelle = `video::fit_scale` sur la zone disponible), ratio carre / 8:7, filtre (nearest / linear / scanlines).
- Texture: `TextureOptions::LINEAR` si `Filter::Linear`, sinon `NEAREST` ; avant la creation de la texture, si `scanlines_active(filter, scale)` appeler `apply_scanlines` sur le RGBA (une copie, pas l'image d'origine).
- Menu Entrees: liste en lecture seule des touches par defaut (texte fixe tire de la spec 08 section 2). Menu Audio: curseur volume 0..100 et case Muet (leur effet reel vient en E08-27).
- Les options `block_opposite_dirs` (defaut vrai) et `pause_on_focus_loss` existent dans le menu Entrees / Emulation ; `block_opposite_dirs == false` n'appelle PAS `filter_opposites`.""",
  tests=[],
  accept=["cargo build -p puce16-desktop"],
  notes="Verification visuelle faite par l'utilisateur au GATE E08-31.")

T("E08-27", "app.rs / main.rs: audio reel (anneau, flux cpal, volume, muet)",
  size=120, kind="ui", crate="puce16-desktop",
  context="Jalon D2: relie `AudioOut` (E08-18) et `spawn_with_audio` (E08-19) a l'application. Sans peripherique audio l'emulation continue avec le minuteur. " + CHAIN,
  embed=["08_INPUT_DESKTOP.md#5. Audio"],
  write=[DSRC + "/app.rs", DSRC + "/main.rs"],
  spec="""- `main.rs`: creer `let ring = audio::new_shared_ring();` puis `match audio::AudioOut::new(ring.clone())` : `Ok(out)` -> `spawn_with_audio(region, false, Some(ring))` et `App.audio = Some(out)` ; `Err(e)` -> `spawn_with_audio(region, false, None)`, `App.audio = None`, statut `No audio: <e> (timer sync)`.
- `App`: `set_volume(cfg.volume as f32 / 100.0)` et `set_muted(cfg.muted)` a la creation et a chaque changement du curseur / de la case Muet / de la touche M (`Hotkey::Mute`, E08-23) ; la config est mise a jour (E08-26).
- A l'ouverture d'une ROM (`open_path`) vider l'anneau (`ring.lock().clear()`) pour eviter un reste de son de la ROM precedente. L'`App` garde un clone de `SharedRing`.
- Barre d'etat: remplissage de l'anneau en pourcentage (`len * 100 / capacity`) et `audio off` si pas de peripherique.""",
  tests=[],
  accept=["cargo build -p puce16-desktop"],
  notes="Verification a l'oreille faite par l'utilisateur au GATE E08-31. Si l'audio craque, ne rien deviner: l'utilisateur le signale et Claude cree une tache TUNE-nn.")

T("E08-28", "app.rs: plein ecran (F11), curseur masque, pause a la perte du focus",
  size=100, kind="ui", crate="puce16-desktop",
  context="Fin de la fenetre desktop (jalon D5). " + CHAIN,
  embed=UIDOC,
  write=[DSRC + "/app.rs"],
  spec="""- `Hotkey::Fullscreen` (F11) et le menu Video basculent `cfg.fullscreen` et envoient `ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(cfg.fullscreen))`. Au demarrage, si `cfg.fullscreen` est vrai, l'appliquer a la premiere frame UI.
- En plein ecran: la barre de menu est masquee sauf si la souris est dans les 8 pixels du haut de l'ecran ; la zone de jeu est centree, echelle = `fit_scale` (entier) sur l'ecran, ratio respecte (`display_size`). Echap en plein ecran quitte d'abord le plein ecran (le comportement `Quit` de la spec est reserve au mode fenetre).
- Curseur: `ctx.set_cursor_icon(egui::CursorIcon::None)` si la souris n'a pas bouge depuis 2 s en plein ecran (garder `last_mouse_move: Instant`).
- `cfg.pause_on_focus_loss`: quand la fenetre perd le focus (`ctx.input(|i| i.focused)` passe a faux) envoyer `Pause(true)` et memoriser `paused_by_focus = true` ; au retour du focus, `Pause(false)` seulement si `paused_by_focus` (une pause manuelle n'est jamais levee automatiquement).""",
  tests=[],
  accept=["cargo build -p puce16-desktop"],
  notes="Verification visuelle faite par l'utilisateur au GATE E08-31.")

T("E08-29", "input.rs: tables manette (PadButton), stick, mappings personnalises (pur)",
  size=140, crate="puce16-desktop",
  context="Partie pure du jalon D4: le peripherique `gilrs` est lu en E08-30; ici seulement les tables de conversion, testables sans manette. " + CHAIN,
  embed=["08_INPUT_DESKTOP.md#3. Manette physique", "08_INPUT_DESKTOP.md#2. Clavier (AZERTY) - touches LOGIQUES"],
  write=[DSRC + "/input.rs", DT + "/e08_29_pad.rs"],
  spec="""Ajoute dans `input.rs` (sans modifier `Key`, `default_buttons`, `Hotkey` de E01-13):
```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PadButton { South, East, West, North, LeftTrigger, RightTrigger, Start, Select, DPadUp, DPadDown, DPadLeft, DPadRight, Other }
pub fn pad_buttons(pressed: &[PadButton]) -> u16;            // tableau section 3: South->B, East->A, West->Y, North->X, LeftTrigger->L, RightTrigger->R, Start->START, Select->SELECT, DPad*->directions, Other->0
/// stick gauche: x positif = droite, y positif = HAUT (convention gilrs). Seuil strict: abs(axe) > 0.5.
pub fn stick_buttons(x: f32, y: f32) -> u16;
pub fn snes_button_bit(name: &str) -> Option<u16>;           // "B","A","Y","X","L","R","Start","Select","Up","Down","Left","Right" (casse exacte) ; autre -> None
pub fn pad_button_name(b: PadButton) -> &'static str;        // "South","East","West","North","LeftTrigger","RightTrigger","Start","Select","DPadUp","DPadDown","DPadLeft","DPadRight","Other"
/// `map`: bouton SNES -> nom de bouton manette (config.pad_map). Map vide = defauts. Pour chaque entree valide de `map`, ce bouton SNES est active si le bouton manette nomme est dans `pressed` ; un bouton SNES present dans `map` NE suit PLUS le defaut. Noms inconnus ignores.
pub fn pad_buttons_mapped(pressed: &[PadButton], map: &std::collections::BTreeMap<String, String>) -> u16;
pub fn key_name(k: Key) -> &'static str;                     // "ArrowUp","ArrowDown","ArrowLeft","ArrowRight","Z","S","Q","D","X","K","C","L","W","J","V","I","A","U","E","O","Enter","Backspace","ShiftRight","Other"
/// `map`: bouton SNES -> liste de noms de touches (config.keymap). Meme regle: map vide = defauts ; un bouton SNES present dans `map` ne suit plus le defaut.
pub fn keyboard_buttons_mapped(pressed: &[Key], map: &std::collections::BTreeMap<String, Vec<String>>) -> u16;
```
Algorithme mapped (commun): `bits = defaut(pressed) & !(OU des bits des boutons SNES presents dans map)` puis OU des bits des entrees de `map` dont une touche / un bouton nomme est presse.""",
  tests=[("pad_defaults", "`pad_buttons(&[South])==BTN_B` ; `[East]==BTN_A` ; `[West]==BTN_Y` ; `[North]==BTN_X` ; `[LeftTrigger]==BTN_L` ; `[RightTrigger]==BTN_R` ; `[Start]==BTN_START` ; `[Select]==BTN_SELECT` ; `[DPadUp,DPadLeft]==BTN_UP|BTN_LEFT` ; `[Other]==0` ; `[]==0`."),
         ("stick_threshold", "`stick_buttons(0.5,0.0)==0` (seuil strict) ; `(0.51,0.0)==BTN_RIGHT` ; `(-0.9,0.0)==BTN_LEFT` ; `(0.0,0.9)==BTN_UP` ; `(0.0,-0.9)==BTN_DOWN` ; `(0.7,0.7)==BTN_RIGHT|BTN_UP` ; `(0.1,-0.2)==0`."),
         ("snes_names", "`snes_button_bit(\"B\")==Some(BTN_B)` ; `\"Select\"` -> `Some(BTN_SELECT)` ; `\"Up\"` -> `Some(BTN_UP)` ; `\"b\"` -> `None` ; `\"\"` -> `None`."),
         ("pad_names", "`pad_button_name(South)==\"South\"` ; `pad_button_name(DPadRight)==\"DPadRight\"` ; `pad_button_name(Other)==\"Other\"`."),
         ("pad_mapped_empty_is_default", "map vide : `pad_buttons_mapped(&[South,Start], &map)==pad_buttons(&[South,Start])`."),
         ("pad_mapped_override", "map `{\"B\": \"West\"}` : `[West]` -> `BTN_B` (B est active par la map) et aussi `BTN_Y` (par le defaut, car Y n'est pas dans la map): resultat `BTN_B|BTN_Y` ; `[South]` -> `0` (B ne suit plus le defaut South)."),
         ("pad_mapped_unknown_ignored", "map `{\"B\": \"Nope\", \"Zed\": \"South\"}` : `[South]` -> `0` (B est dans la map donc ne suit plus South, mais `Nope` n'existe pas) ; aucune panique."),
         ("key_names", "`key_name(Key::Up)==\"ArrowUp\"` ; `key_name(Key::Enter)==\"Enter\"` ; `key_name(Key::ShiftRight)==\"ShiftRight\"` ; `key_name(Key::Z)==\"Z\"`."),
         ("keyboard_mapped_empty", "map vide : `keyboard_buttons_mapped(&[Key::X,Key::Enter], &map)==default_buttons(&[Key::X,Key::Enter])`."),
         ("keyboard_mapped_override", "map `{\"A\": [\"Q\", \"Enter\"]}` : `[Key::Q]` -> `BTN_A|BTN_LEFT` (Q reste Gauche par le defaut, et active A) ; `[Key::C]` -> `0` (A ne suit plus C) ; `[Key::Enter]` -> `BTN_START|BTN_A`.")],
  accept=["cargo test -p puce16-desktop e08_29_", "cargo test -p puce16-desktop e01_13_"])

T("E08-30", "input.rs / app.rs: lecture des manettes avec gilrs (PadPoller)",
  size=130, kind="ui", crate="puce16-desktop",
  context="Jalon D4: lit les manettes reelles. Le code qui touche au peripherique n'est pas teste automatiquement; seule la conversion `gilrs::Button -> PadButton` l'est. " + CHAIN,
  embed=["08_INPUT_DESKTOP.md#3. Manette physique"],
  write=[DSRC + "/input.rs", DSRC + "/app.rs", DT + "/e08_30_gilrs.rs"],
  spec="""Lis la doc de la version de `gilrs` resolue (`cargo doc -p gilrs --no-deps`, ou ses sources dans `~/.cargo/registry`): ne devine pas l'API (noms `Button`, `Axis`, `EventType`, `GamepadId`).
Dans `input.rs`:
```rust
pub fn pad_button_from_gilrs(b: gilrs::Button) -> PadButton;   // South,East,West,North,LeftTrigger,RightTrigger,Start,Select,DPadUp/Down/Left/Right ; tout le reste -> Other
pub struct PadPoller { /* Option<gilrs::Gilrs>, ports: [Option<GamepadId>; 2], pressed: [Vec<PadButton>; 2], stick: [(f32, f32); 2] */ }
impl PadPoller {
    pub fn new() -> PadPoller;                 // si Gilrs::new() echoue: poller vide (aucune manette), jamais de panique
    pub fn poll(&mut self) -> [u16; 2];        // vide les evenements en attente et renvoie, pour chaque port, pad_buttons(pressed) | stick_buttons(x, y)
    pub fn connected(&self) -> [bool; 2];
}
```
Attribution des ports: la premiere manette connectee prend le port 0, la deuxieme le port 1 (au demarrage: parcourir `gilrs.gamepads()` dans l'ordre ; ensuite sur `Connected`). `Disconnected` libere le port et remet ses boutons / son stick a zero (le port retombe a 0 sans erreur) ; une manette suivante peut reprendre le port libre le plus bas. Evenements `ButtonPressed` / `ButtonReleased` ajoutent / retirent dans `pressed` ; `AxisChanged(Axis::LeftStickX / LeftStickY, v)` met a jour `stick`.
Dans `app.rs`: a chaque frame UI, `let pads = self.poller.poll();` ; port 0 = `keyboard_bits | pads[0]` (OU logique, puis `filter_opposites` si l'option est active), port 1 = `pads[1]` ; envoyer `SetJoypad(port, bits)` seulement quand la valeur change (garder les dernieres valeurs par port). Utiliser `pad_buttons_mapped` / `keyboard_buttons_mapped` avec `cfg.pad_map` / `cfg.keymap` (E08-24) a la place des fonctions par defaut. Statut: `Pad 1 connected` / `Pad 1 disconnected` lors d'un changement.""",
  tests=[("button_conversion", "`pad_button_from_gilrs(gilrs::Button::South)==PadButton::South` ; `East`, `West`, `North` idem ; `LeftTrigger -> LeftTrigger` ; `RightTrigger -> RightTrigger` ; `Start -> Start` ; `Select -> Select` ; `DPadUp -> DPadUp` ... `DPadRight`."),
         ("button_other", "`gilrs::Button::Mode`, `LeftTrigger2`, `C`, `Unknown` -> `PadButton::Other`."),
         ("poller_without_device_is_safe", "`PadPoller::new().poll()` ne panique pas et renvoie `[0, 0]` quand aucune manette n'est branchee (sur la machine de test, `connected()` peut etre vrai si une manette est branchee: dans ce cas le test verifie seulement l'absence de panique)."),
         ("poller_poll_twice", "deux appels successifs de `poll()` ne paniquent pas.")],
  accept=["cargo test -p puce16-desktop e08_30_", "cargo build -p puce16-desktop"])

T("E08-31", "GATE D5: desktop jouable (audio, SRAM, save states, manette, filtres, config)",
  kind="gate", gate=True, size=0,
  context="Fin des jalons D2..D5 du desktop (spec 08 section 10). L'utilisateur verifie avec une vraie ROM et ses propres oreilles / yeux / manette. Aucun code a ecrire.",
  spec="""Criteres de validation (humains):
1. `cargo test --workspace` est vert.
2. Une ROM du commerce (ou homebrew) s'ouvre par Fichier > Ouvrir (et Ctrl+O): l'image s'affiche, le son est audible et sans craquements continus ; M coupe le son, le curseur de volume agit.
3. Les touches AZERTY decrites en spec 08 section 2 controlent le jeu (fleches ou ZQSD, X/C/W/V, A/E, Entree, Retour arriere).
4. SRAM: dans un jeu avec sauvegarde interne, sauvegarder en jeu, fermer l'emulateur, rouvrir la ROM: la sauvegarde est la ; le fichier `<rom>.srm` existe a cote de la ROM.
5. Save states: F5 sauvegarde, F7 recharge, PageUp / PageDown changent le slot, un etat d'une autre ROM est refuse avec un message.
6. Tab (maintenu) accelere, F2 cree `<rom>_1.ppm`, F11 bascule le plein ecran (le curseur disparait apres 2 s).
7. Manette: une manette de type Xbox branchee controle le jeu ; la debrancher ne plante pas.
8. Video: echelle 1..8, 8:7, filtres nearest / linear / scanlines (scanlines visible a partir de l'echelle 2).
9. Fermer puis rouvrir l'emulateur conserve echelle, ratio, filtre, volume et la liste des ROM recentes (`%APPDATA%\\Puce16\\config.json`).
10. Note: si le son ou la cadence est mauvais(e), le dire a Claude (creation d'une tache TUNE-nn): ce n'est pas un blocage du jalon si le reste fonctionne.""",
  user_cmds=["cargo test --workspace", "cargo run --release -p puce16-desktop", "python docs/tools/task.py approve E08-31"],
  accept=[])

# ------------------------------------------------------------------ E08 part D: debugger (E08-32..39)

T("E08-32", "snes.rs: accesseurs de debug (VRAM, CGRAM, OAM, registres PPU/SMP/DMA, page WRAM, run_until_pc)",
  size=130, crate="puce16-core",
  context="Le debugger du desktop (spec 08 section 8) ne lit que ces accesseurs publics du core. Variante motif de test: tableaux vides et `None`. " + CHAIN,
  embed=DBGDOC,
  write=[CSRC + "/snes.rs", CSRC + "/apu/mod.rs", CTST + "/e08_32_debug_access.rs"],
  spec="""Dans `snes.rs`:
```rust
#[derive(Clone, Debug, PartialEq, Eq)] pub struct SmpSnapshot { pub a: u8, pub x: u8, pub y: u8, pub sp: u8, pub pc: u16, pub psw: u8 }
#[derive(Clone, Debug, PartialEq, Eq)] pub struct DmaChanSnapshot { pub dmap: u8, pub bbad: u8, pub a1t: u16, pub a1b: u8, pub das: u16, pub dasb: u8 }
impl Snes {
    pub fn debug_vram(&self) -> &[u16];           // 32768 mots (Motif: &[])
    pub fn debug_cgram(&self) -> &[u16];          // 256 mots
    pub fn debug_oam(&self) -> &[u8];             // 544 octets
    pub fn debug_ppu_regs(&self) -> Option<crate::ppu::PpuRegsSnapshot>;   // Ppu::regs_snapshot() ; Motif: None
    pub fn debug_smp(&self) -> Option<SmpSnapshot>;
    pub fn debug_dma(&self) -> Option<[DmaChanSnapshot; 8]>;
    pub fn debug_wram_page(&self, page: u16) -> Vec<u8>;       // 256 octets de la WRAM, page 0..=511 (page & 0x1FF), via peek_wram ; Motif: 256 zeros
    /// execute des instructions (step_instruction) jusqu'a ce que PB:PC == (pb, pc) AVANT d'executer l'instruction a cette adresse ; au plus `max_instructions`. Renvoie vrai si l'adresse est atteinte. Si PB:PC vaut deja l'adresse au depart: renvoie vrai SANS rien executer.
    pub fn debug_run_until_pc(&mut self, pb: u8, pc: u16, max_instructions: u32) -> bool;
}
```
Dans `apu/mod.rs`: `impl Apu { pub fn smp_regs(&self) -> (u8, u8, u8, u8, u16, u8) }` = `(a, x, y, sp, pc, psw)` du `Spc700` (champs publics). `debug_smp` s'appuie dessus. Le chemin d'import de `PpuRegsSnapshot` est celui defini en E04-12; ne modifie pas `lib.rs`.""",
  tests=[("pattern_variant_empty", "`Snes::new_test_pattern(Ntsc)`: `debug_vram().is_empty()`, `debug_cgram().is_empty()`, `debug_oam().is_empty()`, `debug_ppu_regs().is_none()`, `debug_smp().is_none()`, `debug_dma().is_none()`, `debug_wram_page(0)==vec![0u8;256]`."),
         ("real_sizes", "ROM `make_rom(&[0x80,0xFE], LoRom)` : `debug_vram().len()==32768`, `debug_cgram().len()==256`, `debug_oam().len()==544`, `debug_ppu_regs().is_some()`, `debug_smp().is_some()`, `debug_dma().is_some()`."),
         ("wram_page_content", "programme `78 18 FB A9 42 8D 10 00 80 FE` (SEI CLC XCE LDA #$42 STA $0010 BRA -2), apres `run_frame()` : `debug_wram_page(0)[0x10]==0x42` ; `debug_wram_page(1)[0x10]==0` ; la page renvoyee a 256 octets ; `debug_wram_page(512)` == `debug_wram_page(0)` (masque 0x1FF)."),
         ("run_until_pc_reaches", "meme programme, au reset: `debug_run_until_pc(0, 0x8008, 100)==true`, `debug().pc==0x8008`, `debug().pb==0`, `peek_wram(0x10)==0x42`."),
         ("run_until_pc_already_there", "au reset (PC 0x8000): `debug_run_until_pc(0, 0x8000, 100)==true` et `debug().pc==0x8000` (rien execute)."),
         ("run_until_pc_limit", "adresse jamais atteinte `debug_run_until_pc(0, 0x9000, 50)==false` ; le CPU a avance (`debug().pc` est dans `0x8000..=0x8009`) sans panique."),
         ("smp_snapshot_after_boot", "apres 2 `run_frame()` de la ROM `make_rom(&[0x80,0xFE], LoRom)`: `debug_smp()` est `Some` et sa lecture ne panique pas (aucune valeur precise: l'etat exact du SMP est couvert par E06).")],
  accept=["cargo test -p puce16-core e08_32_", "cargo test -p puce16-core e03_35_"])

T("E08-33", "emu_thread: commandes de debug (RequestDebug, SetWramPage, StepInstruction, RunToPc) et Event::Debug",
  size=140, crate="puce16-desktop",
  context="Le thread d'emulation joint un `DebugData` a chaque frame quand le debugger est ouvert, et sait avancer d'une instruction ou jusqu'a un PC. " + CHAIN,
  embed=DBGDOC + ["08_INPUT_DESKTOP.md#7. Architecture desktop"],
  write=[DSRC + "/emu_thread.rs", DT + "/e08_33_debug_thread.rs"],
  spec="""Dans `emu_thread.rs`:
```rust
pub struct DebugData { pub snapshot: puce16_core::snes::DebugSnapshot, pub vram: Vec<u16>, pub cgram: Vec<u16>, pub oam: Vec<u8>, pub wram_page: Vec<u8>,
    pub ppu_regs: Option<puce16_core::ppu::PpuRegsSnapshot>, pub smp: Option<puce16_core::snes::SmpSnapshot>, pub dma: Option<[puce16_core::snes::DmaChanSnapshot; 8]> }
// Command gagne: RequestDebug(bool), SetWramPage(u16), StepInstruction, RunToPc(u32)   // RunToPc: (pb << 16) | pc
// Event gagne: Debug(Box<DebugData>)
```
- `RequestDebug(true)` active, `(false)` desactive (defaut: desactive). Actif: apres CHAQUE `Event::Frame` publiee, envoyer aussi un `Event::Debug` construit depuis les accesseurs `debug_*` (E08-32) ; `wram_page` = `debug_wram_page(page courante)` (page courante `SetWramPage`, defaut 0). `SetWramPage` alors actif renvoie aussitot un `Event::Debug`.
- `StepInstruction`: ignore si le thread n'est pas en pause (`Event::Status(\"pause first\")`) ; en pause: `snes.step_instruction()` puis `Event::Debug` si actif (PAS de nouvelle `Event::Frame`).
- `RunToPc(addr)`: ignore hors pause (meme statut). En pause: boucle de blocs de `debug_run_until_pc(pb, pc, RUN_TO_PC_BLOCK)` (constantes `pub const RUN_TO_PC_BLOCK: u32 = 100_000;` et `pub const RUN_TO_PC_MAX_BLOCKS: u32 = 100;`), entre deux blocs traiter les commandes en attente: `Quit` termine le thread ; `Pause(false)` ANNULE la recherche (`Event::Status("Breakpoint cancelled")`) et reprend l'execution normale. Atteint -> reste en pause, `Event::Status(format!("Breakpoint {:02X}:{:04X}", pb, pc))`, une `Event::Frame` (image courante) puis `Event::Debug` si actif. Blocs epuises -> `Event::Status("Breakpoint not reached")`, reste en pause.
- Les autres comportements sont inchanges ; le motif de test envoie un `DebugData` a vecteurs vides.""" + " " + TINY,
  tests=[("no_debug_by_default", "`spawn_with_audio(Ntsc,true,None)` + `LoadRom(tiny_rom(),\"t\")` + `FrameAdvance` : on recoit `Event::Frame` et aucun `Event::Debug` (attendre 200 ms)."),
         ("debug_after_frame", "`RequestDebug(true)` puis `FrameAdvance` : un `Event::Debug` arrive apres la `Event::Frame` ; `vram.len()==32768`, `cgram.len()==256`, `oam.len()==544`, `wram_page.len()==256`, `ppu_regs.is_some()`."),
         ("debug_off_again", "`RequestDebug(true)`, `RequestDebug(false)`, `FrameAdvance` : aucun `Event::Debug`."),
         ("pattern_debug_empty", "sans ROM (motif de test), `RequestDebug(true)` + `FrameAdvance` : `Event::Debug` avec `vram.is_empty()` et `ppu_regs.is_none()`."),
         ("step_instruction", "ROM `78 18 FB A9 42 8D 10 00 80 FE` = `tiny_rom()` dont les 10 premiers octets sont remplaces par `78 18 FB A9 42 8D 10 00 80 FE` (SEI CLC XCE LDA #$42 STA $0010 BRA -2), en pause, `RequestDebug(true)`, 3 x `StepInstruction` : le dernier `Event::Debug` a `snapshot.pc==0x8003` ; aucun `Event::Frame` recu pendant ces pas."),
         ("step_requires_pause", "thread NON en pause : `StepInstruction` -> `Event::Status` contenant `pause first`."),
         ("run_to_pc", "meme programme, en pause : `RunToPc(0x008008)` -> `Event::Status` contenant `Breakpoint 00:8008` ; `SetWramPage(0)` -> `Event::Debug` avec `wram_page[0x10]==0x42`."),
         ("run_to_pc_cancel", "ROM `tiny_rom()`, en pause : `RunToPc(0x009000)` (jamais atteinte) puis aussitot `Pause(false)` -> `Event::Status` contenant `cancelled` (timeout 10 s) ; ensuite des `Event::Frame` arrivent (l'emulation tourne)."),
         ("set_wram_page", "`SetWramPage(1)` actif -> `Event::Debug` avec `wram_page.len()==256`.")],
  accept=["cargo test -p puce16-desktop e08_33_", "cargo test -p puce16-desktop e08_22_"])

T("E08-34", "debugger.rs: decodage de tuiles, couleurs, palettes, hexdump, drapeaux (pur)",
  size=130, crate="puce16-desktop",
  context="Fonctions pures du debugger (panneaux tuiles, palettes, memoire, registres). Aucune egui ici. " + CHAIN,
  embed=DBGDOC,
  write=[DSRC + "/debugger.rs", DT + "/e08_34_debugger_tiles.rs"],
  spec="""Dans `debugger.rs` (module deja declare):
```rust
pub fn bgr555_to_rgba(c: u16) -> [u8; 4];   // r5 = c & 31, g5 = (c >> 5) & 31, b5 = (c >> 10) & 31 ; c8 = (c5 << 3) | (c5 >> 2) (D-010) ; alpha 255
/// decode une tuile 8x8 de `bpp` (2, 4 ou 8) bits dont le premier mot est a `word_addr`; adresses modulo 0x8000. Sortie: 64 indices de couleur, ligne par ligne, pixel de gauche = bit 7.
pub fn decode_tile(vram: &[u16], word_addr: usize, bpp: u8, out: &mut [u8; 64]);
pub fn tile_words(bpp: u8) -> usize;       // 2 -> 8, 4 -> 16, 8 -> 32 ; autre -> 8
/// grille cols x rows de tuiles consecutives a partir de `base_word`, couleurs lues dans `cgram[(palette * (1 << bpp) + idx) & 0xFF]` (8 bpp: palette ignoree). Renvoie (largeur, hauteur, RGBA). Un `vram` vide donne une image noire opaque de la bonne taille.
pub fn render_tiles(vram: &[u16], cgram: &[u16], base_word: usize, bpp: u8, palette: u8, cols: usize, rows: usize) -> (usize, usize, Vec<u8>);
pub fn palette_rgba(cgram: &[u16]) -> Vec<[u8; 4]>;     // 256 couleurs (cgram plus court: complete par noir opaque)
pub fn hex_rows(data: &[u8], base: u32) -> Vec<String>;  // 16 octets par ligne: format!("{:06X}: ", adresse) puis les octets "XX" separes par un espace, puis deux espaces, puis l'ASCII (0x20..=0x7E sinon '.'); derniere ligne eventuellement plus courte, sans remplissage
pub fn flags_string(p: u8, e: bool) -> String;           // "NVMXDIZC": lettre MAJUSCULE si le bit (7..0) est 1 sinon minuscule, puis " e=1" ou " e=0"
```
Planaire SNES (spec 05 section tuiles): 2 bpp = 8 mots, mot `r` = (plan 0 = octet bas, plan 1 = octet haut) de la ligne `r` ; 4 bpp = 16 mots (mots 0-7 plans 0/1, mots 8-15 plans 2/3) ; 8 bpp = 32 mots (plans 0/1, 2/3, 4/5, 6/7 par blocs de 8 mots). Indice = OU des bits de plan, plan 0 = poids faible.""",
  tests=[("color_conversion", "`bgr555_to_rgba(0x0000)==[0,0,0,255]` ; `0x7FFF -> [255,255,255,255]` ; `0x001F -> [255,0,0,255]` ; `0x03E0 -> [0,255,0,255]` ; `0x7C00 -> [0,0,255,255]` ; `0x0010 -> [132,0,0,255]` (16<<3 | 16>>2 = 128+4)."),
         ("tile_words", "`tile_words(2)==8`, `4 -> 16`, `8 -> 32`, `3 -> 8`."),
         ("decode_2bpp", "vram[0]=0x00FF (plan0=0xFF, plan1=0x00) et vram[1]=0xFF00 (plan0=0x00, plan1=0xFF), le reste 0 ; `decode_tile(.., 0, 2)`: ligne 0 = huit fois 1 ; ligne 1 = huit fois 2 ; lignes 2..8 = 0."),
         ("decode_2bpp_bits", "vram[0]=0x0180 (plan0=0x80, plan1=0x01) : ligne 0 = `[1,0,0,0,0,0,0,2]`."),
         ("decode_4bpp_planes", "vram[8]=0x00FF a l'adresse 0 + 8 (plan 2 = 0xFF, plan 3 = 0x00), vram[0]=0x0000 : ligne 0 = huit fois 4 ; avec en plus vram[0]=0xFF00 (plan 1 = 0xFF) : ligne 0 = huit fois 6."),
         ("decode_8bpp_planes", "vram[24]=0x0001 (plan 6 = 0x01) : ligne 0, pixel 7 = 64 ; vram[16+1]=0x0080 (plan 4 = 0x80, ligne 1) : ligne 1, pixel 0 = 16."),
         ("decode_wraps", "tuile 2 bpp a `word_addr = 0x7FFF` : la ligne 0 vient de `vram[0x7FFF]` et la ligne 1 de `vram[0]` (modulo 0x8000), sans panique."),
         ("render_tiles_layout", "`render_tiles(.., base 0, bpp 2, palette 0, cols 2, rows 1)` : taille (16, 8), longueur 16*8*4 ; tuile 1 (a `base + 8`) rendue a partir de x=8 ; couleur lue dans cgram: `cgram[1]=0x001F` -> pixel d'indice 1 = `[255,0,0,255]`."),
         ("render_tiles_palette", "bpp 4, palette 2, indice 3 : couleur `cgram[2*16+3]` ; bpp 8: `cgram[idx]` quel que soit `palette`."),
         ("render_tiles_empty_vram", "`render_tiles(&[], &[], 0, 4, 0, 4, 4)` : taille (32, 32), tout noir opaque `[0,0,0,255]`, pas de panique."),
         ("palette_rgba", "cgram de 256 mots avec `cgram[5]=0x03E0` : `palette_rgba(..)[5]==[0,255,0,255]`, longueur 256 ; `palette_rgba(&[])` : 256 noirs."),
         ("hex_rows_format", "`hex_rows(&[0x48,0x69,0x00,0xFF], 0x7E0000)==[\"7E0000: 48 69 00 FF  Hi..\"]`."),
         ("hex_rows_multi", "20 octets 0..20 de base 0x10 : 2 lignes ; la 2eme commence par `\"000020: \"` et contient 4 octets."),
         ("flags_string", "`flags_string(0x34,true)==\"nvMXdIzc e=1\"` ; `flags_string(0xFF,false)==\"NVMXDIZC e=0\"` ; `flags_string(0x00,false)==\"nvmxdizc e=0\"`.")],
  accept=["cargo test -p puce16-desktop e08_34_"])

T("E08-35", "debugger.rs: rendu des cartes BG, carte mode 7 et decodage OAM (pur)",
  size=140, crate="puce16-desktop",
  context="Suite des fonctions pures du debugger (E08-34): cartes de tuiles et liste de sprites. " + CHAIN,
  embed=DBGDOC,
  write=[DSRC + "/debugger.rs", DT + "/e08_35_debugger_maps.rs"],
  spec="""Ajoute dans `debugger.rs` (sans modifier E08-34; reutilise `decode_tile`, `tile_words`, `bgr555_to_rgba`):
```rust
/// Carte d'un BG (mode 0-6). map_size: 0 = 32x32, 1 = 64x32, 2 = 32x64, 3 = 64x64 ecrans-tuiles (chaque ecran = 0x400 mots; disposition 64x32: ecrans a base et base+0x400 cote a cote; 32x64: base et base+0x400 l'un sous l'autre; 64x64: base, base+0x400 (droite), base+0x800 (bas gauche), base+0xC00 (bas droite)).
/// Entree de carte: bits 0-9 numero de caractere, 10-12 palette, 13 priorite (ignoree ici), 14 flip H, 15 flip V.
/// Tuile de caractere c: premier mot = (chr_base_word + c * tile_words(bpp)) & 0x7FFF. tile16 = vrai: la tuile affichee fait 16x16 et est composee des caracteres c, c+1 (ligne du haut) et c+16, c+17 (ligne du bas) ; flip H echange gauche/droite ET retourne chaque 8x8 ; flip V idem pour haut/bas.
/// Couleur: cgram[((palette << bpp) + idx) & 0xFF] (8 bpp: cgram[idx]). Simplification v1 documentee: pas de decalage de palette du mode 0, pas de transparence (l'indice 0 est dessine).
/// Renvoie (largeur, hauteur, RGBA) en pixels: largeur = ecrans_horizontaux * 32 * (8 ou 16).
pub fn render_bg_map(vram: &[u16], cgram: &[u16], map_base_word: usize, chr_base_word: usize, map_size: u8, tile16: bool, bpp: u8) -> (usize, usize, Vec<u8>);
/// Mode 7: carte 128x128 (octet BAS du mot `y * 128 + x` = numero de tuile), tuile n = 8x8 pixels 8 bpp dont le pixel (px, py) est l'octet HAUT du mot `n * 64 + py * 8 + px`. Image 1024x1024 RGBA, couleur cgram[indice] directe.
pub fn render_mode7_map(vram: &[u16], cgram: &[u16]) -> (usize, usize, Vec<u8>);
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SpriteInfo { pub index: u8, pub x: i16, pub y: u8, pub tile: u16, pub palette: u8, pub priority: u8, pub hflip: bool, pub vflip: bool, pub large: bool }
/// OAM de 544 octets: sprite i = octets 4i..4i+3 (x bas, y, tuile, attributs vhoopppN) ; table haute (octets 512..543): sprite i = 2 bits a l'octet 512 + i/4, decalage 2 * (i % 4): bit 0 = bit 8 de x, bit 1 = grande taille. x = 9 bits en complement a 2 (-256..255). tile = octet de tuile | (N << 8) ; palette = bits 1-3 ; priority = bits 4-5 ; flips bits 6 (H) et 7 (V). `oam` plus court que 544: sprites manquants a zero.
pub fn decode_oam(oam: &[u8]) -> Vec<SpriteInfo>;      // 128 elements
pub fn sprite_dims(obj_size: u8, large: bool) -> (u8, u8);   // largeur, hauteur: table de spec 05b section sprites (tailles 0..7)
```""",
  tests=[("bg_map_size", "`render_bg_map(.., map_size 0, tile16 false, bpp 2)` -> (256, 256) ; `map_size 1` -> (512, 256) ; `2` -> (256, 512) ; `3` -> (512, 512) ; `tile16 true`, map_size 0 -> (512, 512)."),
         ("bg_map_tile_lookup", "vram[0x100 + 0] = entree (caractere 1, palette 0) ; tuile 1 a `chr_base 0`: vram[8] = 0x00FF (ligne 0 = indice 1) ; cgram[1] = 0x001F : `render_bg_map(.., map_base 0x100, chr_base 0, 0, false, 2)` : pixels (0..8, 0) = `[255,0,0,255]`."),
         ("bg_map_hflip", "meme tuile mais plan0 = 0x80 sur la ligne 0 (pixel de gauche seul) et entree avec flip H (bit 14) : le pixel colore est a x = 7."),
         ("bg_map_vflip", "tuile dont seule la ligne 0 est non nulle, entree avec flip V (bit 15) : la ligne coloree est y = 7."),
         ("bg_map_palette_bits", "entree avec palette 3, bpp 2, indice 1 -> couleur `cgram[(3 << 2) + 1]`."),
         ("bg_map_screens_64x32", "map_size 1 : l'entree de l'ecran 2 (mot `map_base + 0x400`) est dessinee a x = 256..263, y = 0."),
         ("bg_map_tile16_layout", "tile16 vrai, entree caractere 0 sans flip : le sous-bloc du haut-droit (x 8..16, y 0..8) vient du caractere 1 et le bas-gauche (x 0..8, y 8..16) du caractere 16 ; avec flip H le haut-gauche vient du caractere 1 (retourne)."),
         ("bg_map_empty_vram", "`render_bg_map(&[], &[], 0, 0, 0, false, 2)` : taille (256,256), tout noir opaque, pas de panique."),
         ("mode7_map", "vram[0] = 0x0005 (tuile 5 en (0,0)) ; vram[5*64 + 0] = 0x0900 (octet haut 9 : pixel (0,0) de la tuile) ; cgram[9] = 0x7C00 : pixel (0,0) de l'image = `[0,0,255,255]` ; taille (1024, 1024)."),
         ("oam_decode_basic", "octets du sprite 0 = `[0x10,0x20,0x05,0x31]`, table haute 0 : `index 0, x 16, y 0x20, tile 0x105, palette 0, priority 3, hflip false, vflip false, large false`."),
         ("oam_decode_high_table", "sprite 1 avec x bas 0xFF et table haute octet 512, bits 2-3 = `0b11` (bit8=1, large=1) : `x == -1`, `large == true` ; sprite 0 du meme octet inchange."),
         ("oam_decode_flips_palette", "attributs 0xE6 (v=1,h=1,oo=2,ppp=3,N=0): `vflip`, `hflip`, `priority 2`, `palette 3`, `tile` sans bit 8."),
         ("oam_short_input", "`decode_oam(&[])` : 128 elements a zero (x 0, y 0, tile 0) ; `decode_oam(&[0;544]).len()==128`."),
         ("sprite_dims", "`sprite_dims(0,false)==(8,8)` ; `(0,true)==(16,16)` ; `(3,true)==(32,32)` ; `(5,false)==(32,32)` ; `(5,true)==(64,64)` ; `(6,false)==(16,32)` ; `(6,true)==(32,64)` ; `(7,false)==(16,32)` ; `(7,true)==(32,32)`.")],
  accept=["cargo test -p puce16-desktop e08_35_", "cargo test -p puce16-desktop e08_34_"])

T("E08-36", "debugger.rs / app.rs: fenetre du debugger (F10), registres et commandes d'execution",
  size=140, kind="ui", crate="puce16-desktop",
  context="Premiere partie de l'interface du debugger (jalon D6): fenetre egui separee, panneau Registres, boutons d'execution. " + CHAIN,
  embed=DBGDOC,
  write=[DSRC + "/debugger.rs", DSRC + "/app.rs"],
  spec="""Dans `debugger.rs`:
```rust
pub struct DebuggerUi { pub open: bool, pub last: Option<Box<crate::emu_thread::DebugData>>, pub wram_page: u16, pub pc_text: String, /* champs des panneaux suivants: tiles_*, map_*, mem_* ajoutes en E08-37 / E08-38 */ }
impl DebuggerUi {
    pub fn new() -> DebuggerUi;
    pub fn show(&mut self, ctx: &eframe::egui::Context, tx: &std::sync::mpsc::Sender<crate::emu_thread::Command>, paused: bool);
}
```
- `Hotkey::Debugger` (F10) et le menu Debug basculent `open` ; a l'ouverture envoyer `Command::RequestDebug(true)`, a la fermeture `RequestDebug(false)`. L'`App` ranges chaque `Event::Debug` dans `debugger.last`.
- `show` ouvre une fenetre egui secondaire (`egui::Window` deplacable ou `ctx.show_viewport_immediate`, au choix, qui compile avec la version resolue) titree `Puce16 - Debugger`, avec des onglets: Registres | Tuiles | Cartes | Palettes | OAM | Memoire (les cinq derniers affichent `(a venir)` dans cette tache).
- Onglet Registres: CPU (`A X Y S D DB PB PC P` en hexadecimal + `flags_string`), position (`frame`, `V`, `dot`, `master`), SMP (A X Y SP PC PSW) si present, registres PPU principaux (`brightness`, `bg_mode`, `tm`, `ts`, `forced_blank`...) depuis `ppu_regs`, DMA: 8 lignes `ch n: dmap bbad a1t a1b das` depuis `dma`. Tout en police monospace.
- Commandes d'execution (boutons): Pause / Reprendre (`Command::Pause(!paused)`), "Image" (`FrameAdvance`, actif en pause), "Instruction" (`StepInstruction`, actif en pause), champ texte `pc_text` (hexadecimal `BBPPPP` 6 chiffres ou `PPPP` 4 chiffres = banque 00) + bouton "Executer jusqu'a" (`RunToPc`, actif en pause ; texte invalide -> message rouge, rien d'envoye). Utilise une fonction pure `pub fn parse_pc(text: &str) -> Option<u32>` (testee dans la tache E08-38).""",
  tests=[],
  accept=["cargo build -p puce16-desktop"],
  notes="La fonction pure `parse_pc` est ecrite ici mais testee en E08-38. Verification visuelle au GATE E08-39.")

T("E08-37", "debugger.rs: onglets Tuiles, Cartes et Palettes",
  size=140, kind="ui", crate="puce16-desktop",
  context="Panneaux graphiques du debugger, qui utilisent les fonctions pures de E08-34 et E08-35. " + CHAIN,
  embed=DBGDOC,
  write=[DSRC + "/debugger.rs"],
  spec="""Dans `DebuggerUi` (E08-36) ajoute les champs d'etat et remplace les placeholders:
- **Tuiles**: choix `bpp` (2 / 4 / 8), `palette` (0..15 en 4 bpp, 0..63 en 2 bpp: borne `(256 >> bpp) - 1`), adresse de base en mots (`DragValue` ou boutons -0x400 / +0x400, borne 0..=0x7FFF), zoom 1..4 ; image 16 colonnes x 16 lignes via `render_tiles` ; texture egui recreee seulement quand les donnees ou les reglages changent (comparer un hash simple ou garder la derniere entree) ; `TextureOptions::NEAREST`.
- **Cartes**: choix BG 1..4, reglages initiaux tires de `ppu_regs` (`map_base[n]`, `chr_base[n]`, `tile16[n]`; la taille de carte n'est pas dans le snapshot: menu 32x32 / 64x32 / 32x64 / 64x64, defaut 32x32). `bpp` par defaut selon `bg_mode` (BG1, BG2, BG3, BG4): mode 0 = 2,2,2,2 ; mode 1 = 4,4,2,- ; mode 2 = 4,4,-,- ; mode 3 = 8,4,-,- ; mode 4 = 8,2,-,- ; mode 5 = 4,2,-,- ; mode 6 = 4,-,-,- ; un BG absent du mode est grise. Mode 7: case "Mode 7" -> `render_mode7_map`. Tous les reglages (base, bpp, taille) sont modifiables a la main pour explorer la VRAM ; zoom 1..2 et defilement (`ScrollArea::both`).
- **Palettes**: 256 pastilles 16x16 (16 par ligne) via `palette_rgba`, survol = `index 0xNN  valeur 0xNNNN` (valeur brute de `cgram`).
- Si `last` est `None` (pas de donnees): afficher `(pas de donnees: pause + ouvrir une ROM)`.
Cette tache n'ajoute pas de tests (le dessin est visuel); les fonctions pures sont deja couvertes.""",
  tests=[],
  accept=["cargo build -p puce16-desktop"],
  notes="Verification visuelle au GATE E08-39.")

T("E08-38", "debugger.rs: onglets OAM et Memoire, parse_pc (+ tests purs)",
  size=130, crate="puce16-desktop",
  context="Fin du debugger. Les tests portent sur la fonction pure `parse_pc` et sur un formateur de ligne de sprite. " + CHAIN,
  embed=DBGDOC,
  write=[DSRC + "/debugger.rs", DT + "/e08_38_debugger_misc.rs"],
  spec="""Dans `debugger.rs`:
```rust
pub fn parse_pc(text: &str) -> Option<u32>;        // hexadecimal, espaces autour ignores, prefixe optionnel `0x` ou `$` ; 4 chiffres (ou moins) -> banque 0 ; 6 chiffres -> BBPPPP ; plus de 6 chiffres ou caractere non hexadecimal ou vide -> None ; la valeur renvoyee est (banque << 16) | pc
pub fn sprite_line(s: &SpriteInfo, obj_size: u8) -> String;   // "NNN x=XXXX y=YYY t=TTT p=P pr=R hv=HV WxH" : voir exemples dans les tests
```
Format exact de `sprite_line`: `format!("{:03} x={:4} y={:3} t={:03X} p={} pr={} {}{} {}x{}", index, x, y, tile, palette, priority, if hflip {'H'} else {'-'}, if vflip {'V'} else {'-'}, w, h)` avec `(w, h) = sprite_dims(obj_size, large)`.
Onglets UI:
- **OAM**: tableau scrollable de 128 lignes `sprite_line` (monospace) issues de `decode_oam(&last.oam)` ; `obj_size` vient de `ppu_regs.obj_size`; les sprites hors ecran (y >= 224 et y < 240, ou x <= -w) sont affiches en gris.
- **Memoire**: bascule WRAM (par page de 256 octets: boutons -256 / +256, champ page hexadecimal ; envoie `SetWramPage(page)` a chaque changement) ; affiche `hex_rows(&last.wram_page, (page as u32) * 256)` (monospace, adresses de WRAM 0x000000..0x01FFFF).""",
  tests=[("parse_pc_forms", "`parse_pc(\"8008\")==Some(0x008008)` ; `\"008008\"` -> `Some(0x008008)` ; `\"7E1234\"` -> `Some(0x7E1234)` ; `\"0x8000\"` -> `Some(0x8000)` ; `\"$00FF00\"` -> `Some(0x00FF00)` ; `\"  12  \"` -> `Some(0x12)`."),
         ("parse_pc_invalid", "`\"\"`, `\"  \"`, `\"xyz\"`, `\"12345678\"`, `\"80 08\"`, `\"-1\"` -> `None`."),
         ("sprite_line_format", "sprite `{index 3, x -16, y 32, tile 0x105, palette 2, priority 1, hflip true, vflip false, large true}`, obj_size 0 : `\"003 x= -16 y= 32 t=105 p=2 pr=1 H- 16x16\"`."),
         ("sprite_line_small", "sprite `{index 127, x 255, y 0, tile 0, palette 7, priority 3, hflip false, vflip true, large false}`, obj_size 6 : `\"127 x= 255 y=  0 t=000 p=7 pr=3 -V 16x32\"`.")],
  accept=["cargo test -p puce16-desktop e08_38_", "cargo build -p puce16-desktop"])

T("E08-39", "GATE J8: save states, SRAM et debugger valides (jalon desktop final)",
  kind="gate", gate=True, size=0,
  context="Fin de l'epic E08. L'utilisateur valide le debugger et la robustesse des sauvegardes avec de vraies ROM. Aucun code a ecrire.",
  spec="""Criteres de validation (humains):
1. `cargo test --workspace` est vert (aucun test E01..E08 ignore).
2. F10 ouvre / ferme le debugger ; l'onglet Registres montre PC / A / X / Y qui bougent en jeu, et en pause le bouton \"Instruction\" avance le PC.
3. \"Executer jusqu'a\" avec une adresse atteinte par le jeu arrete l'emulation (message `Breakpoint BB:PPPP`).
4. Tuiles: avec un jeu en cours, une base VRAM bien choisie montre des graphismes reconnaissables en 4 bpp ; la palette correspond.
5. Cartes: le BG1 reflete l'ecran en cours (pour un jeu en mode 1) ; Palettes et OAM affichent des valeurs coherentes ; Memoire montre la WRAM.
6. Save state: sauvegarder, jouer, recharger ramene exactement la meme image ET le meme son (pas de note qui se coupe ou reste bloquee). Charger l'etat d'une autre ROM est refuse.
7. `cargo run --release -p puce16-cli -- run <rom> --frames 600 --save-state tests/out/s.bin`, puis `--load-state tests/out/s.bin --frames 60 --hash` donne le meme hash que `--frames 660 --hash` (determinisme; voir E08-13).
8. Aucun `unimplemented!` / `todo!` dans `crates/puce16-desktop/src` ni dans `crates/puce16-core/src/savestate.rs`.
Apres le jalon: les coprocesseurs (E09) sont HORS v1 (D-005); la v1 est terminee.""",
  user_cmds=["cargo test --workspace", "cargo run --release -p puce16-desktop", "python docs/tools/task.py approve E08-39"],
  accept=[])
