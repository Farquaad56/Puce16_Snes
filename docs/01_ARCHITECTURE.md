# 01 - ARCHITECTURE (Puce16)

## 1. Decisions de cadrage (resumees; detail dans docs/qa/DECISIONS.md)

| ID | Decision |
|---|---|
| D-002 | Precision: **cycle-accurate** pour CPU 65816, bus, DMA/HDMA, timers H/V, IRQ/NMI, SPC700, S-DSP. PPU: **dot par dot** (un pixel compose a chaque dot, ecritures de registres effectives au dot courant). Confirme par l'utilisateur. |
| D-003 | Regions: NTSC et PAL des la v1 (constantes dans `consts.rs`). |
| D-004 | (confirme) Horloge APU: 24 606 720 Hz (mesuree, compatible bsnes) -> DSP 32 040 Hz. Constante unique `APU_RESONATOR_HZ`; la valeur nominale du wiki (24 576 000 -> 32 000 Hz) est documentee mais non utilisee. Voir Q-001. |
| D-005 | Pas de coprocesseur en v1. Ordre ensuite: DSP-1, puis SuperFX, puis SA-1 (E09, taches differees, aucune specification pour l'instant). |
| D-006 | Jeux cibles de validation: Super Mario World, Zelda: A Link to the Past, Super Mario All-Stars (SMB3), plus les ROM de test (voir 06_TEST_ROMS.md). |
| D-007 | Depot neuf (template vierge + pipeline Puce8). |
| D-018..D-021 | DSP precis a l'echantillon (provisoire) ; `gilrs` et raccourcis AZERTY sans chiffres (provisoire) ; **execution strictement sequentielle, une tache a la fois** ; depot GitHub de sauvegarde (jeton jamais ecrit dans le depot). |
| D-012..D-017 | Assignee `Puce16_Snes`, depot `H:\__Emulator__\Puce16_Snes`, taches 50-150 lignes, desktop minimal tres tot, ROMs de test telechargees par Hermes, desktop v1 complet (save states, gamepad, debugger, filtres). |

## 2. Workspace Cargo

```
Cargo.toml                       [workspace] members = crates/*
crates/puce16-core/              lib, aucune dependance runtime (sauf serde optionnel feature "serde")
crates/puce16-cli/               binaire `puce16-cli` (clap), depend de core
crates/puce16-desktop/           binaire `puce16-desktop` (egui/eframe, cpal, rfd, zip), depend de core
tests/                           baseline.json, manifest.toml, roms/README.md (pas de ROM commitee)
docs/                            toute la documentation (ce dossier)
```

Dependances autorisees:
- core: aucune en v1 (`std` seulement). Feature optionnelle `serde` interdite en v1.
- cli: `clap` (derive), `serde`, `serde_json`, `toml`.
- desktop: `eframe`/`egui`, `cpal`, `rfd`, `zip`, `gilrs` (D-019), `serde`, `serde_json`.
- Dev-deps de core: `serde`, `serde_json` (chargement des tests JSON SingleStepTests).

Toute nouvelle dependance = question Q&R (QA) avant ajout.

## 3. Arborescence de `puce16-core/src/` (vue d'ensemble)

**Source de verite = le champ "Write only in" de chaque tache** (genere depuis `docs/tasks_src/`). Ce plan est un resume: les specs de conception `01b_CPU_DESIGN.md`, `07c_SPC_DESIGN.md`, `07d_DSP_DESIGN.md` et `09_SYSTEM.md` precisent les sous-modules reels (par exemple `apu/timer.rs`, `apu/ipl.rs`, `apu/io.rs`, les sous-modules `impl Ppu` / `impl Dsp`) et les noms des structures (`SysBus`). En cas d'ecart, la tache fait foi.

```
lib.rs              re-exports + #![forbid(unsafe_code)]
consts.rs           constantes d'horloge, tailles memoire, vecteurs
region.rs           enum Region { Ntsc, Pal }
snes.rs             struct Snes (assemble tout), run_frame, run_until, API publique
scheduler.rs        horloge maitre, rattrapage (catch-up) PPU/APU/DMA
cart/mod.rs         struct Cartridge, load(rom bytes)
cart/header.rs      parse + validation + scoring de l'en-tete
cart/mapper.rs      LoROM, HiROM, ExHiROM (fonction d'adresse -> offset ROM/SRAM)
bus/mod.rs          struct SysBus (possede cart, wram, mmio, dma, ppu, apu, joy, timing), read/write 24 bits, vitesse d'acces, open bus
bus/wram.rs         WRAM 128 KiB + registres WMDATA/WMADD
bus/mmio.rs         registres $4200-$421F et $4016/$4017 (cote CPU)
cpu/mod.rs          struct Cpu + registres + step_instruction
cpu/addr.rs         modes d'adressage (chaque mode = fonction qui emet des cycles de bus)
cpu/alu.rs          ADC SBC AND ORA EOR CMP CPX CPY BIT INC DEC ASL LSR ROL ROR TSB TRB
cpu/mem.rs          LDA LDX LDY STA STX STY STZ MVN MVP PEA PEI PER PHx PLx
cpu/flow.rs         BRA Bxx JMP JML JSR JSL RTS RTL RTI BRK COP WAI STP
cpu/misc.rs         transferts, REP SEP XCE XBA flags NOP WDM
cpu/interrupts.rs   NMI, IRQ, RESET, vecteurs
cpu/table.rs        table des 256 opcodes (mnemonic, mode), generee par docs/tools/gen_tables.py
cpu/testbus.rs      FlatBus (bus plat 24 bits enregistrant les cycles) pour les tests et l'oracle
dma/mod.rs          registres $43xx, MDMAEN, DMA general
dma/hdma.rs         HDMA init + transfert par scanline
timing/mod.rs       compteurs H/V, VBlank, NMI, IRQ H/V, auto-joypad, rafraichissement DRAM
ppu/mod.rs          struct Ppu, registres $2100-$213F (lecture/ecriture)
ppu/vram.rs cgram.rs oam.rs   memoires + latchs
ppu/bg.rs           modes 0-6 (tiles 2/4/8 bpp, tilemaps, scroll, offset-per-tile)
ppu/mode7.rs        mode 7 + EXTBG
ppu/obj.rs          sprites (evaluation 32 sprites / 34 slivers)
ppu/window.rs       fenetres et logique
ppu/colormath.rs    maths de couleur
ppu/output.rs       composition finale -> framebuffer, hi-res, entrelace, overscan
ppu/dot.rs          boucle dot par dot: avance H/V, declenche fetch/evaluation/composition a chaque dot
apu/mod.rs          ports $2140-$2143 <-> $F4-$F7, horloge APU
apu/spc700/mod.rs   etat + step
apu/spc700/ops.rs   instructions
apu/spc700/table.rs table opcodes + cycles (generee par docs/tools/gen_tables.py)
apu/spc700/testbus.rs FlatSmpBus (bus plat 16 bits) pour les tests et l'oracle
apu/dsp/mod.rs      registres + ordonnancement 32 pas par echantillon
apu/dsp/voice.rs envelope.rs brr.rs echo.rs tables.rs
joypad.rs           port manette, auto-read, JOYSER
savestate.rs        serialisation binaire maison (little-endian, versionnee)
hash.rs             hash d'etat (FNV-1a 64) du framebuffer et de l'etat complet
```

Fichiers de test de core: `crates/puce16-core/tests/<id_tache>_<nom>.rs`
(ex: `tests/e02_03_adc.rs`). Chaque fonction de test commence par l'ID: `e02_03_adc_imm_8bit`.

## 4. Modele de temps (coeur du design - lire attentivement)

- Unite: **horloge maitre** (`u64 master`). NTSC 21 477 272 Hz, PAL 21 281 370 Hz (entiers; valeur reelle NTSC = 945/44 MHz).
- `Snes` possede `master: u64` = date courante du CPU. Chaque acces bus du CPU
  (`bus.read`, `bus.write`, `bus.idle`) **avance `master` de 6, 8 ou 12** selon la region
  (voir specs/02_BUS_TIMING.md) et appelle `scheduler.catch_up(master)`.
- `catch_up(t)`: amene PPU (position H/V), timers, HDMA/DMA, APU (SMP + DSP), joypad auto-read a la date `t`.
  Les composants ne sont jamais "ticke un par un par le CPU": chacun garde sa propre date et avance jusqu'a `t`.
- APU: accumulateur entier `apu_acc += APU_RESONATOR_HZ * delta_master`;
  tant que `apu_acc >= 24 * MASTER_HZ` -> un cycle SMP (24 tics resonateur) et `apu_acc -= 24 * MASTER_HZ`.
  Le DSP produit 1 echantillon stereo tous les 32 cycles SMP.
- Interruptions: le CPU echantillonne les lignes NMI/IRQ **avant le dernier cycle de chaque instruction**
  (regle exacte dans specs/01_CPU_65816.md section Interruptions).
- DMA: une ecriture sur MDMAEN ($420B) bloque le CPU; `dma.run()` consomme du temps maitre et
  appelle `catch_up` octet par octet. HDMA s'execute a des dates fixes de la ligne (specs/03_DMA.md).
- Determinisme: aucune dependance a l'heure systeme, aucun flottant, aucune iteration de HashMap.
  Meme ROM + memes entrees => meme `state_hash` a chaque master clock.

## 5. API publique de `puce16-core` (stable, ne pas changer sans question)

```rust
pub enum Region { Ntsc, Pal }
pub struct Snes { /* prive */ }
impl Snes {
    pub fn new(rom: &[u8], region: Option<Region>) -> Result<Snes, LoadError>; // None = deduit du header
    pub fn reset(&mut self);
    pub fn run_frame(&mut self);                       // jusqu'au debut du prochain VBlank
    pub fn run_master(&mut self, master_clocks: u64);  // pour les tests de timing
    pub fn step_instruction(&mut self);                // une instruction CPU complete (debugger)
    pub fn frame_count(&self) -> u64;
    pub fn frame(&self) -> FrameView<'_>;              // { width, height, pixels: &[u32] } XRGB8888
    pub fn take_audio(&mut self, out: &mut Vec<[i16; 2]>); // echantillons DSP 32 kHz-ish
    pub fn set_joypad(&mut self, port: usize, buttons: u16); // port 0 ou 1; bits = format JOY1L/H
    pub fn load_sram(&mut self, data: &[u8]);
    pub fn sram(&self) -> Option<&[u8]>;               // Some si la cartouche a de la SRAM
    pub fn state_hash(&self) -> u64;
    pub fn save_state(&self) -> Vec<u8>;
    pub fn load_state(&mut self, data: &[u8]) -> Result<(), StateError>;
    pub fn debug(&self) -> DebugSnapshot;              // registres CPU/APU, H/V, master (pour rapports d'echec)
}
```
Accesseurs ajoutes par des taches (la tache qui les cree fait foi): `cart_info`, `master`, `peek_wram` (E03-35), `apu_port` (E07-07), `debug_vram/cgram/oam/ppu_regs/smp/dma/wram_page/run_until_pc` (E08-32). `Snes` n'a pas d'autre acces a l'etat interne.

Framebuffer: tampon fixe 512x478 u32; `FrameView` expose la zone utile
(256x224, 256x239, 512x224, 512x448, 512x478 selon SETINI). Details dans specs/05_PPU.md.

## 6. CLI (`puce16-cli`)

```
puce16-cli run <rom> [--frames N] [--region ntsc|pal] [--hash] [--trace FILE]
                     [--dump-ppm FILE] [--save-state FILE] [--load-state FILE]
puce16-cli suite <manifest.toml> [--update-baseline]
puce16-cli cpu-test <dossier_json_65816> [--filter NN]     # SingleStepTests
puce16-cli spc-test <dossier_json_spc700> [--filter NN]
```

Codes de sortie: 0 = OK, 1 = echec de test, 2 = erreur d'usage/fichier.
Tout echec imprime `DebugSnapshot` (PC/A/X/Y/S/D/DB/P, H/V, master).

## 7. Desktop (`puce16-desktop`)

- Thread d'emulation separe; communication par canaux `std::sync::mpsc` (`Command::{LoadRom, SetRomPath, Reset, Pause, FrameAdvance, SetJoypad, SaveState, LoadState, SetFastForward, FlushSram, RequestDebug, SetWramPage, StepInstruction, RunToPc, Quit}`, detail dans specs/08_INPUT_DESKTOP.md).
- Video: texture egui mise a jour a 60 Hz (50 Hz PAL), mise a l'echelle plus proche voisin, ratio 8:7 optionnel.
- Audio: anneau de samples via `cpal`, ressample 32 040 Hz -> frequence du peripherique par
  filtre sinc fenetre (entiers/virgule fixe cote core; flottants autorises UNIQUEMENT dans desktop).
- Entrees: clavier AZERTY (touches par defaut dans specs/08_INPUT_DESKTOP.md) + gamepad optionnel.
- Fichiers: `rfd` pour ouvrir ROM (.sfc/.smc/.zip via `zip`), SRAM `.srm` a cote de la ROM, save states `.p16s`.

## 8. Outils du pipeline (docs/tools)

- `task.py`: `next`, `status`, `list`, `show`, `start`, `scope`, `check`, `done`, `approve`, `reset`. Lit `docs/tasks/INDEX.tsv` et `PROGRESS.json`.
  `check <ID>` est NATIF (Python, Windows compris, pas besoin de bash): ordre strict D-020, scope (`git status` doit rester dans `Write only in`),
  presence des tests nommes (`fn <nom>(`), hygiene (ASCII, pas de `f32/f64`/`unsafe`/`HashMap`/`Instant` dans `puce16-core`), puis les commandes du bloc `acceptance`.
  Les scripts `docs/checks/<ID>.sh` (generes) sont un equivalent facultatif pour un shell bash.
- `qa.py`: `list`, `show`, `answer` (voir 02_QA_PROTOCOL.md).
- `kanban_import.py`: lit `docs/kanban/tasks.json` et appelle `hermes kanban create` (voir 03_KANBAN_HERMES.md).
- `build_tasks.py`: (maintenance Claude) regenere `docs/tasks/*.md`, `INDEX.tsv`, `ORDER.md`, `docs/kanban/tasks.json` depuis `docs/tasks_src/*.py`.
- `check_gauss.py`, `ppu_ref.py`: verifications de donnees et reference independante du rendu PPU (voir les taches concernees).
