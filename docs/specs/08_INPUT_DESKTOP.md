# SPEC 08 - ENTREES, DESKTOP, FICHIERS, SAVE STATES

Decisions liees: D-009 (flottants autorises seulement dans desktop), D-015 (desktop minimal tres tot), D-017 (save states+SRAM, manette, debugger, filtres/plein ecran), D-019 (dependance `gilrs`).
Fichiers: core `savestate.rs`, `joypad.rs`; desktop `crates/puce16-desktop/src/{main,app,emu_thread,input,video,audio,files,debugger,config}.rs`.
Utilisateur: Windows, clavier AZERTY.

## 1. Manettes (core, `joypad.rs`)

Format 16 bits de `Snes::set_joypad(port, buttons)` (identique a JOY1H:JOY1L, spec 02 section 6.5):
```
bit15 B   bit14 Y   bit13 Select  bit12 Start  bit11 Up   bit10 Down  bit9 Left  bit8 Right
bit7  A   bit6  X   bit5  L       bit4  R      bits3-0 = 0
```
`const BTN_B: u16 = 1 << 15; BTN_Y = 1 << 14; BTN_SELECT = 1 << 13; BTN_START = 1 << 12; BTN_UP = 1 << 11; BTN_DOWN = 1 << 10; BTN_LEFT = 1 << 9; BTN_RIGHT = 1 << 8; BTN_A = 1 << 7; BTN_X = 1 << 6; BTN_L = 1 << 5; BTN_R = 1 << 4;`
Contraintes: le core n'empeche pas Haut+Bas / Gauche+Droite simultanes (c'est le desktop qui peut les filtrer, option `block_opposite_dirs` active par defaut: si Gauche et Droite sont appuyes, aucun des deux).

## 2. Clavier (AZERTY) - touches LOGIQUES

Defaut modifiable dans `config.json` (touche -> bouton, plusieurs touches par bouton). Les noms sont ceux d'`egui::Key`.
| Bouton SNES | Touches par defaut |
|---|---|
| Haut / Bas / Gauche / Droite | fleches ; Z / S / Q / D |
| B | X ; K |
| A | C ; L |
| Y | W ; J |
| X | V ; I |
| L | A ; U |
| R | E ; O |
| Start | Entree |
| Select | Retour arriere ; Maj droite |

Raccourcis emulateur (aucune touche chiffre: sur AZERTY elles exigent Maj):
| Action | Touche |
|---|---|
| Pause / reprise | P |
| Avancer d'une frame (en pause) | N |
| Reset (bouton) | F12 |
| Sauvegarder l'etat (slot courant) | F5 |
| Charger l'etat (slot courant) | F7 |
| Slot precedent / suivant (0..9) | Page precedente / Page suivante |
| Capture d'ecran (PNG non requis: PPM) | F2 |
| Plein ecran | F11 |
| Debugger (fenetre) | F10 |
| Muet | M |
| Avance rapide (maintenu) | Tab |
| Ouvrir une ROM | Ctrl+O |
| Quitter | Echap (avec confirmation si un fichier SRAM est en attente: ecriture automatique d'abord) |

## 3. Manette physique (`gilrs`, dependance autorisee par D-019)

Mapping par defaut (disposition "Xbox", noms `gilrs::Button`):
| Bouton SNES | Bouton manette |
|---|---|
| B | South (A) |
| A | East (B) |
| Y | West (X) |
| X | North (Y) |
| L / R | LeftTrigger / RightTrigger (bumpers) |
| Start / Select | Start / Select (Back) |
| Directions | DPad ; stick gauche avec zone morte 0.5 (seuil `abs(axis) > 0.5`) |
Premiere manette connectee = port 0 (joueur 1) ; deuxieme = port 1. Le clavier alimente toujours le port 0 (OU logique avec la manette 0). Deconnexion: le port retombe a 0 sans erreur. Mapping modifiable dans `config.json` (`pad_map`).

## 4. Fenetre, video, filtres, plein ecran

- Fenetre egui (`eframe`), titre `Puce16 - <nom de la ROM>`. Zone de jeu = `FrameView` (256x224 .. 512x478, spec 05 section 6) affichee dans une texture RGBA (conversion XRGB8888 -> `Color32`).
- Echelle: entier `1..=8` (defaut 3) ou "ajuster a la fenetre" ; ratio: pixels carres (defaut) ou 8:7 (largeur multipliee par 8/7).
- Filtres (option `filter`): `nearest` (defaut, `TextureOptions::NEAREST`), `linear` (`TextureOptions::LINEAR`), `scanlines` (nearest + une ligne sur deux assombrie a 75 % par pre-traitement CPU, uniquement en echelle >= 2).
- Plein ecran: `ViewportCommand::Fullscreen(true)` (sans bordure), memes regles d'echelle, curseur masque apres 2 s.
- Cadence: PAL 50.0070 Hz, NTSC 60.0988 Hz (valeurs de reference, flottants autorises ici). Synchronisation principale = audio (section 5) ; sans audio: minuteur monotone. Pause automatique quand la fenetre perd le focus (option, defaut: non).
- Menu minimal (barre egui): Fichier (Ouvrir, ROM recentes, Quitter), Emulation (Pause, Reset, Save/Load state, slot), Video (echelle, ratio, filtre, plein ecran), Audio (volume 0-100, muet), Entrees (liste des touches, lecture seule en v1), Debug (ouvrir le debugger).

## 5. Audio (`cpal`)

- Le core produit des echantillons stereo `[i16; 2]` a ~32 040 Hz (`Snes::take_audio`). Le thread audio consomme un anneau (`Vec<[i16;2]>` circulaire, capacite 8192 trames).
- Re-echantillonnage vers la frequence du peripherique par interpolation sinc fenetree (Hann), 16 points, en `f32` (autorise ici). Volume 0..100 % applique en sortie. Muet = silence.
- Latence cible 60 ms (anneau ~ 2000 trames pleines). Sous-alimentation: repeter le dernier echantillon ; sur-alimentation (anneau > `RING_WAIT_ABOVE` = 3072 trames, ~96 ms; constante reglable): le thread d'emulation attend par pas de 1 ms, au plus 100 ms par frame (c'est l'horloge de synchronisation).
- Pas de peripherique audio: l'emulation continue avec le minuteur (message dans la barre d'etat).

## 6. Fichiers

- Ouverture: `rfd` (filtres `.sfc`, `.smc`, `.swc`, `.fig`, `.zip`). Un `.zip` (crate `zip`) : prendre la premiere entree dont l'extension est une des quatre precedentes.
- **SRAM**: `<dossier de la ROM>/<nom sans extension>.srm`, octets bruts de `Snes::sram()`. Chargee a l'ouverture de la ROM (`load_sram`) ; sauvee toutes les 5 s si le contenu a change, a la fermeture de la ROM et a la sortie ; ecriture atomique (ecrire `.srm.tmp` puis renommer). Pas de SRAM (`None`) -> aucun fichier.
- **Save states**: `<dossier de la ROM>/<nom>.<slot>.p16s`, slot 0..9. Contenu: en-tete `b"P16S"` + `u16` version du conteneur (1) + `u64` hash FNV-64 des octets de la ROM tels que charges (apres extraction d'un `.zip`, avant toute normalisation du core) + `u8` region, puis le blob `Snes::save_state()`. Au chargement: refuser (message) si magie, version ou hash ROM different.
- **Config**: `config.json` dans `%APPDATA%\Puce16\` (Windows) ; a defaut `$HOME/.config/puce16/`. JSON: `scale, ratio, filter, volume, muted, fullscreen, keymap, pad_map, recent_roms (max 8), last_dir, block_opposite_dirs`. Fichier absent ou invalide -> valeurs par defaut (jamais de plantage).
- Capture d'ecran F2: `<nom>_<n>.ppm` dans le dossier de la ROM (n = premier numero libre).

## 7. Architecture desktop

- Thread d'emulation (`emu_thread.rs`) possede `Snes` ; thread UI (egui) envoie des `Command` (`std::sync::mpsc`): `LoadRom(Vec<u8>, path)`, `Reset`, `Pause(bool)`, `FrameAdvance`, `SetJoypad(port, u16)`, `SaveState(slot)`, `LoadState(slot)`, `SetFastForward(bool)`, `RequestDebug(bool)`, `Quit`.
- Le thread d'emulation publie `Event` : `Frame(Arc<FrameData>)` (largeur, hauteur, pixels), `Status(String)`, `Debug(Box<DebugData>)` (seulement si demande), `SramDirty`, `Error(String)`.
- Boucle: tant que non en pause: `run_frame()` ; pousser l'audio ; publier la frame ; attendre selon la synchro. Une frame est publiee par frame emulee (la UI affiche la plus recente).
- Aucun etat d'emulation n'est partage par verrou avec la UI: tout passe par canaux.

## 8. Debugger (fenetre egui separee, ouverte par F10)

Envoie `RequestDebug(true)` ; le thread d'emulation joint a chaque frame un `DebugData` = `{ snapshot: DebugSnapshot, vram: Vec<u16> (32768), cgram: Vec<u16> (256), oam: Vec<u8> (544), wram_page: Vec<u8> (256 octets de la page choisie), ppu_regs: Option<PpuRegsSnapshot>, smp: Option<SmpSnapshot>, dma: Option<[DmaChanSnapshot; 8]> }` (les `Vec` sont vides et les `Option` sont `None` pour le motif de test).
Panneaux (v1):
1. **Registres**: CPU (A, X, Y, S, D, DB, PB, PC, P avec drapeaux N V M X D I Z C et e), SMP (A, X, Y, SP, PC, PSW), position (frame, V, H, master), registres PPU principaux (INIDISP, BGMODE, TM, TS, CGWSEL, CGADSUB, SETINI), registres DMA actifs.
2. **VRAM - tuiles**: grille de tuiles 8x8, choix profondeur 2/4/8 bpp, palette (0..15 ou 0..7), adresse de base en mots (curseur par pas de 0x400), zoom 1-4.
3. **VRAM - cartes**: rendu de la carte d'un BG (1..4) avec ses reglages courants (taille de carte, tuile 8/16) ; mode 7 : carte 128x128 en 1024x1024.
4. **Palettes**: 256 pastilles CGRAM, survol = indice + valeur.
5. **OAM**: tableau des 128 sprites (x, y, tuile, palette, priorite, taille, flips).
6. **Memoire**: vue hexadecimale de WRAM (page de 256 octets, navigation par 256).
Commandes d'execution: Pause, Reprendre, Avancer une frame, Avancer une instruction CPU (via commande `StepInstruction`), Executer jusqu'a un PC (un seul point d'arret `PC`, saisi en hexadecimal). Pas de desassembleur en v1 (hors perimetre).

## 9. Save states (core, `savestate.rs`)

- Format binaire maison: octets little-endian, `u32` version du blob (1 au debut). Aucun serde, aucune dependance.
- Ecriture/lecture par un trait: `trait State { fn save(&self, w: &mut StateWriter); fn load(&mut self, r: &mut StateReader) -> Result<(), StateError>; }` implemente par chaque composant, dans CET ordre: `master` (+ phase APU `apu_acc: i64`), Cpu, Bus (WRAM 128 Kio, registres $4200-$421F, mdr), DMA (8 canaux + etat HDMA), timing, Ppu (registres, VRAM, CGRAM, OAM, latchs, position, `line_obj`, framebuffer 512x478), Apu (RAM 64 Kio, SMP, timers, ports, DSP: registres, voix, echo, tampons), joypads, Cart (SRAM seulement ; la ROM n'est PAS sauvee).
- `StateWriter`: `u8/u16/u32/u64/i64/bool/bytes(&[u8])/slice_u16`; `StateReader` symetrique avec verification de fin de donnees (erreur `StateError::Truncated`/`Version`/`Mismatch`).
- Aller-retour: `load_state(save_state())` laisse `state_hash` inchange, et executer N frames apres un `load_state` donne le meme `state_hash` que sans passer par le save state.
- Taille attendue: ~1.3 Mio ; pas de compression en v1.

## 10. Jalons desktop (lies a D-015)

| Jalon | Contenu |
|---|---|
| D0 | fenetre + affichage d'un `FrameView` fourni par le core (degrade/motif de test genere), thread d'emulation, ouverture de ROM (rfd) |
| D1 | clavier AZERTY -> `set_joypad`, pause, reset, echelle, ratio |
| D2 | audio `cpal` + resample + synchro audio |
| D3 | SRAM + save states (UI + fichiers) |
| D4 | manette `gilrs` |
| D5 | filtres + plein ecran + config.json + menu |
| D6 | debugger (registres, tuiles, cartes, palettes, OAM, memoire) |
D0 et D1 sont planifies tres tot (apres le squelette du core), conformement a D-015 ; D2-D6 en E08 (D2 peut etre avance des que le DSP produit des echantillons).

## 11. Tests (sans ROM, sans fenetre)

- `joypad`: conversions et constantes (`BTN_*`), filtre directions opposees (fonction pure `filter_opposites(u16) -> u16`).
- `input.rs`: table de touches -> bits (fonction pure, entree = ensemble de touches appuyees, sortie `u16`) ; table manette idem (entree: ensemble de boutons `gilrs` representes par une enum locale, pas de peripherique).
- `files.rs`: chemins derives (`rom_path -> srm_path / state_path(slot)`), ecriture atomique dans un dossier temporaire, en-tete `.p16s` (magie/version/hash).
- `config.rs`: aller-retour JSON, fichier invalide -> defauts.
- `audio.rs` (partie pure): re-echantillonneur sur un signal connu (continu constant -> sortie constante), anneau circulaire (ecriture/lecture/debordement).
- `video.rs` (partie pure): conversion XRGB -> RGBA, filtre scanlines, calcul de taille de zone selon echelle/ratio.
- Le code UI (egui) lui-meme n'est pas teste automatiquement: GATE visuel de l'utilisateur.
