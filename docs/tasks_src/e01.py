from _dsl import T

CORE = "crates/puce16-core"
CLI = "crates/puce16-cli"
DESK = "crates/puce16-desktop"

# ------------------------------------------------------------------ E01: skeleton, tooling, minimal desktop (D0, D1)

T("E01-01", "Workspace Cargo et squelette de puce16-core",
  size=90,
  context="""Premiere tache du depot (vide). Cree le workspace Cargo et TOUS les fichiers de module de puce16-core
(vides ou presque) pour que les taches suivantes n'aient qu'a remplir un fichier existant, sans modifier lib.rs.""",
  embed=["01_ARCHITECTURE.md#2. Workspace Cargo", "01_ARCHITECTURE.md#3. Arborescence"],
  write=["Cargo.toml", CORE + "/Cargo.toml", CORE + "/src/**", CORE + "/tests/e01_01_smoke.rs"],
  spec="""1. `Cargo.toml` racine: `[workspace]`, `resolver = "2"`, `members = ["crates/*"]`. (Les crates cli et desktop sont creees a la tache E01-02: tant qu'elles n'existent pas, `members` doit quand meme fonctionner; si Cargo refuse un glob vide, liste `members = ["crates/puce16-core"]` et E01-02 completera.)
2. `crates/puce16-core/Cargo.toml`: package `puce16-core`, edition 2021, version 0.1.0, AUCUNE dependance.
3. `crates/puce16-core/src/lib.rs`: commence par `#![forbid(unsafe_code)]`, puis `pub mod` pour chaque module de l'arborescence (consts, region, snes, scheduler, cart, bus, cpu, dma, timing, ppu, apu, joypad, savestate, hash), puis:
   `pub const VERSION: &str = env!("CARGO_PKG_VERSION");`
   `pub use region::Region;` `pub use snes::Snes;` `pub use ppu::output::FrameView;`
4. Cree TOUS les fichiers de l'arborescence (section 3 ci-dessous), y compris les `mod.rs` qui declarent leurs sous-modules (`pub mod addr;` etc.). Un fichier encore vide contient la seule ligne `// stub: filled by a later task`.
   - `cart/mod.rs` declare `header` et `mapper`; `bus/mod.rs` declare `wram`, `mmio`; `cpu/mod.rs` declare `addr alu mem flow misc interrupts table testbus`; `dma/mod.rs` declare `hdma`; `ppu/mod.rs` declare `vram cgram oam bg mode7 obj window colormath output dot`; `apu/mod.rs` declare `spc700` et `dsp`; `apu/spc700/mod.rs` declare `ops table testbus`; `apu/dsp/mod.rs` declare `voice envelope brr echo tables`. `timing/mod.rs` sans sous-module.
5. Contenus non vides imposes (pour que lib.rs compile):
   - `region.rs`: `#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Region { Ntsc, Pal }`
   - `snes.rs`: `pub struct Snes {}`
   - `ppu/output.rs`: `pub struct FrameView<'a> { pub width: usize, pub height: usize, pub pixels: &'a [u32] }`
6. `cargo build -p puce16-core` ne doit produire aucune erreur (les avertissements de code mort sont acceptes).""",
  tests=[("lib_exports", "dans `crates/puce16-core/tests/e01_01_smoke.rs`: `puce16_core::VERSION` n'est pas vide ; `puce16_core::Region::Ntsc != puce16_core::Region::Pal` ; une `FrameView { width: 1, height: 1, pixels: &[0] }` se construit.")],
  accept=["cargo build -p puce16-core", "cargo test -p puce16-core e01_01_"])

T("E01-02", "Crates puce16-cli et puce16-desktop (manifestes + main minimal)",
  deps=["E01-01"], size=70,
  context="""Ajoute les deux binaires au workspace avec toutes leurs dependances declarees d'avance (pour detecter tout de suite
un probleme de telechargement de crates). Aucune logique encore.""",
  embed=["01_ARCHITECTURE.md#2. Workspace Cargo", "08_INPUT_DESKTOP.md#7. Architecture desktop"],
  write=["Cargo.toml", CLI + "/Cargo.toml", CLI + "/src/main.rs", CLI + "/tests/e01_02_cli.rs", DESK + "/Cargo.toml", DESK + "/src/main.rs", DESK + "/src/lib.rs",
         DESK + "/src/app.rs", DESK + "/src/emu_thread.rs", DESK + "/src/input.rs", DESK + "/src/video.rs", DESK + "/src/audio.rs", DESK + "/src/files.rs", DESK + "/src/config.rs", DESK + "/src/debugger.rs"],
  spec="""1. `Cargo.toml` racine: `members = ["crates/*"]` (ou liste explicite des trois crates).
2. `crates/puce16-cli`: package `puce16-cli`, binaire `puce16-cli`, dependances: `puce16-core` (path), `clap` (feature `derive`), `serde` (derive), `serde_json`, `toml`. `src/main.rs`: struct clap `Cli` avec `#[command(version)]` et sous-commande vide; sans argument, affiche l'aide; `--version` affiche `puce16-cli <version>`.
3. `crates/puce16-desktop`: package `puce16-desktop`, binaire `puce16-desktop`, dependances: `puce16-core` (path), `eframe` (egui), `cpal`, `rfd`, `zip`, `gilrs`, `serde` (derive), `serde_json`. `src/lib.rs` declare `pub mod app; pub mod audio; pub mod config; pub mod debugger; pub mod emu_thread; pub mod files; pub mod input; pub mod video;` et chacun de ces fichiers existe avec la seule ligne `// stub: filled by a later task` (les taches suivantes ne modifient plus lib.rs). `src/main.rs`: `fn main() { println!("puce16-desktop {}", puce16_core::VERSION); }` (la fenetre vient en E01-11). Dans ce projet, egui s'utilise via `eframe::egui` (pas de dependance `egui` separee).
4. Versions: laisse Cargo choisir les dernieres versions compatibles puis ECRIS les versions resolues dans les `Cargo.toml`. Si une crate ne se telecharge pas (reseau): question `tooling`, ne pas la retirer.
5. Ne pas lancer `cargo update` sur les dependances deja resolues apres cette tache sans demande de Claude.""",
  tests=[("version_flag", "`crates/puce16-cli/tests/e01_02_cli.rs` lance le binaire (`env!(\"CARGO_BIN_EXE_puce16-cli\")`) avec `--version`; code de sortie 0 et la sortie contient `puce16-cli`.")],
  accept=["cargo build -p puce16-cli", "cargo build -p puce16-desktop", "cargo test -p puce16-cli e01_02_"],
  crate="puce16-cli")

T("E01-03", "consts.rs et region.rs (constantes d'horloge et de ligne)",
  deps=["E01-01"], size=60,
  context="Constantes uniques de timing utilisees par tout le core (aucune autre valeur de ce type ailleurs).",
  embed=["02_BUS_TIMING.md#1. Constantes"],
  write=[CORE + "/src/consts.rs", CORE + "/src/region.rs", CORE + "/tests/e01_03_consts.rs"],
  spec="""Dans `consts.rs`: recopie EXACTEMENT les constantes de la section ci-dessous (memes noms, memes types, memes valeurs), puis ajoute:
- `pub const PPU_WRITE_PHASE: u32 = 0; // UNKNOWN - to confirm` (voir 05_PPU.md section 0)
- `pub const HV_LATCH_DOT_OFFSET: u32 = 0; // UNKNOWN - to confirm`
- `pub const WRAM_SIZE: usize = 0x20000; pub const VRAM_WORDS: usize = 0x8000; pub const CGRAM_WORDS: usize = 256; pub const OAM_BYTES: usize = 544; pub const APU_RAM_SIZE: usize = 0x10000;`
- `pub const FB_WIDTH: usize = 512; pub const FB_HEIGHT: usize = 478;`
Dans `region.rs` (garde l'enum existant) ajoute des methodes: `impl Region { pub const fn master_hz(self) -> u64` (NTSC -> `MASTER_HZ_NTSC`, PAL -> `MASTER_HZ_PAL`) ; `pub const fn last_line(self) -> u32` (NTSC 261, PAL 311) ; `pub const fn is_pal(self) -> bool` }.
Les constantes `LINES_NTSC`/`LINES_PAL` valent 262 / 312 (corrige le commentaire "263 en entrelace": le nombre de lignes en entrelace est gere par le PPU, 05_PPU.md section 5.2).""",
  tests=[("master_hz", "`Region::Ntsc.master_hz() == 21_477_272`, `Region::Pal.master_hz() == 21_281_370`."),
         ("last_line", "`Region::Ntsc.last_line() == 261`, `Region::Pal.last_line() == 311`, `is_pal` faux/vrai."),
         ("line_sums", "`CLOCKS_SHORT_LINE == 1360`, `CLOCKS_PER_LINE == 1364`, `CLOCKS_LONG_LINE == 1368`, `LINES_NTSC * CLOCKS_PER_LINE == 357_368` (u64), `LINES_PAL * CLOCKS_PER_LINE == 425_568`."),
         ("visible", "`VISIBLE_CLOCK_FIRST == 88`, `VISIBLE_DOT_FIRST * 4 == VISIBLE_CLOCK_FIRST`, `VISIBLE_CLOCK_END - VISIBLE_CLOCK_FIRST == 1024`.")])

T("E01-04", "hash.rs: Fnv64 (hash d'etat deterministe)",
  deps=["E01-01"], size=50,
  context="Hash unique pour framebuffer et etat complet (suite de ROM, tests de determinisme, save states).",
  embed=["06_TEST_ROMS.md#6. Commande"],
  write=[CORE + "/src/hash.rs", CORE + "/tests/e01_04_hash.rs"],
  spec="""`pub struct Fnv64 { h: u64 }` avec `Clone, Copy, Debug`:
- `pub fn new() -> Fnv64` (h = 0xcbf29ce484222325) ; `impl Default`.
- `pub fn write(&mut self, bytes: &[u8])` : pour chaque octet `h ^= b as u64; h = h.wrapping_mul(0x100000001b3)`.
- `pub fn write_u8/u16/u32/u64(&mut self, v)` : ecrit `v.to_le_bytes()`.
- `pub fn write_u32_slice(&mut self, s: &[u32])` et `write_u16_slice(&mut self, s: &[u16])` : chaque element en little-endian.
- `pub fn finish(&self) -> u64`.""",
  tests=[("empty", "`Fnv64::new().finish() == 0xcbf29ce484222325`."),
         ("vector_a", "write(b\"a\") -> finish == 0xaf63dc4c8601ec8c."),
         ("vector_foobar", "write(b\"foobar\") -> finish == 0x85944171f73967e8."),
         ("u64_le", "write_u64(1) donne le meme hash que write(&[1,0,0,0,0,0,0,0]) == 0x89cd31291d2aefa4."),
         ("incremental", "write(b\"foo\") puis write(b\"bar\") == write(b\"foobar\").")])

T("E01-05", "FrameBuf 512x478 et FrameView compacte",
  deps=["E01-03"], size=90,
  context="Tampon de sortie video et vue compacte exposee a l'exterieur (CLI, desktop, hash). Le PPU l'ecrira plus tard.",
  embed=["05_PPU.md#6. Sortie: framebuffer"],
  write=[CORE + "/src/ppu/output.rs", CORE + "/tests/e01_05_framebuf.rs"],
  spec="""Dans `ppu/output.rs` (conserve `FrameView<'a>` existant):
- `pub struct FrameBuf { pub pixels: Vec<u32> /* FB_WIDTH*FB_HEIGHT */, pub view_w: usize, pub view_h: usize }` ; `FrameBuf::new()` remplit de `0xFF000000`, view_w=256, view_h=224.
- `pub fn set(&mut self, col: usize, row: usize, argb: u32)` (ignore hors tampon) ; `pub fn get(&self, col, row) -> u32`.
- `pub fn set_view(&mut self, w: usize, h: usize)` (w dans {256,512}, h <= 478; sinon panic avec message).
- Le tampon interne est TOUJOURS large de `FB_WIDTH` (512) colonnes (stride 512). `pub fn compact(&self) -> Vec<u32>` copie la zone utile `view_w x view_h` depuis le coin haut-gauche (colonnes `0..view_w`, lignes `0..view_h`) vers un `Vec` de stride `view_w`. (Les frames 256 colonnes sont ecrites par le PPU dans les colonnes 0..255.)
- `pub fn pixel_double(src: &[u32], w: usize, h: usize) -> Vec<u32>` (fonction libre) : duplique chaque pixel horizontalement (`w` -> `2*w` colonnes), utilisee quand la frame est hi-res mais la ligne ne l'est pas.
- `pub fn make_view(w: usize, h: usize, pixels: &[u32]) -> FrameView<'_>` : construit la vue (panique si `pixels.len() != w*h`).
- `pub fn hash(&self, h: &mut crate::hash::Fnv64)` : ecrit `view_w`, `view_h` (u32) puis les pixels compacts (u32 little-endian, ligne par ligne).""",
  tests=[("new_black", "`FrameBuf::new()`: 512*478 pixels egaux a 0xFF000000, `view_w==256`, `view_h==224`."),
         ("set_get", "set(3,2,0xFF112233) puis get(3,2)==0xFF112233 ; set hors tampon (600,0) ne panique pas."),
         ("compact_256", "avec view 256x2, set(0,0,A), set(255,0,B), set(0,1,C), set(255,1,D): compact() == [A, ..., B, C, ..., D] de longueur 512, compact()[255]==B, compact()[256]==C."),
         ("pixel_double", "src = 256 pixels 0xFF000000|i (w=256,h=1): `pixel_double(&src,256,1)` a 512 elements et [2*i]==[2*i+1]==0xFF000000|i."),
         ("make_view", "`make_view(2,2,&[1,2,3,4])` a width 2, height 2, pixels == [1,2,3,4] ; avec 3 pixels -> panique (`#[should_panic]`)."),
         ("hash_changes", "deux FrameBuf identiques ont le meme hash ; apres set(0,0,1) le hash differe.")])

T("E01-06", "Snes de test (motif degrade) pour le desktop minimal D0",
  deps=["E01-04", "E01-05"], size=110,
  context="""Jalon D0 (D-015): afficher quelque chose des le debut. `Snes` fournit un constructeur temporaire de motif de test
qui sera supprime/remplace quand le vrai PPU sortira une image. L'API publique finale (spec 01 section 5) est deja posee.""",
  embed=["01_ARCHITECTURE.md#5. API publique"],
  write=[CORE + "/src/snes.rs", CORE + "/tests/e01_06_snes_test_pattern.rs"],
  spec="""Dans `snes.rs`:
```rust
pub struct Snes { region: Region, frame: u64, fb: FrameBuf, joy: [u16; 2], test_pattern: bool }
impl Snes {
    pub fn new_test_pattern(region: Region) -> Snes;     // frame = 0, fb noir 256x224
    pub fn run_frame(&mut self);                         // frame += 1 puis, si test_pattern, dessine le motif
    pub fn frame(&self) -> FrameView<'_>;                // vue compacte 256x224 (voir ci-dessous pour la memoire du Vec)
    pub fn frame_count(&self) -> u64;
    pub fn set_joypad(&mut self, port: usize, buttons: u16);   // port 0 ou 1 (autre: ignore)
    pub fn joypad(&self, port: usize) -> u16;
    pub fn take_audio(&mut self, out: &mut Vec<[i16; 2]>);     // ne fait rien pour l'instant
    pub fn state_hash(&self) -> u64;                     // Fnv64 sur frame (u64), joy[0], joy[1], hash du FrameBuf
    pub fn region(&self) -> Region;
}
```
Motif (frame numerotee `f` = valeur de `frame` APRES l'increment de run_frame): pour x in 0..256, y in 0..224: `argb = 0xFF000000 | (((x + f) & 0xFF) << 16) | ((y & 0xFF) << 8) | 0x40` (f en u64 reduit modulo 256 avant l'addition). Si `buttons != 0` pour le port 0: le pixel (0,0) est ecrit en blanc `0xFFFFFFFF`.
`frame()` doit retourner une `FrameView` empruntant un tampon compact stocke dans `Snes` (ajoute un champ `compact: Vec<u32>` mis a jour par `run_frame`). Region PAL: meme motif (hauteur 224).""",
  tests=[("pixel_origin", "apres `new_test_pattern(Ntsc)` + 1 `run_frame()` (f=1): pixel (0,0) = ((0+1)&0xFF)<<16 | 0 | 0x40 | 0xFF000000 = 0xFF010040."),
         ("pixel_formula", "apres 3 `run_frame()` (f=3): pixel (10,5) == 0xFF0D0540 ; pixel (255,223) == 0xFF02DF40 (r=(255+3)&0xFF=2, g=223=0xDF, b=0x40)."),
         ("view_size", "`frame().width==256`, `frame().height==224`, `pixels.len()==256*224`."),
         ("deterministic", "deux `Snes` identiques apres 10 frames ont le meme `state_hash` ; ils different apres 11 vs 10 frames."),
         ("joypad_marker", "`set_joypad(0, 0x8000)` puis `run_frame()` -> pixel (0,0) == 0xFFFFFFFF ; `set_joypad(2, 1)` est ignore (pas de panique).")])

T("E01-07", "puce16-cli run: motif de test, --hash, --dump-ppm",
  deps=["E01-02", "E01-06"], size=120,
  context="Premiere commande CLI utile: executer N frames du motif de test, afficher le hash, ecrire un PPM. Sera etendue aux vraies ROM en E03.",
  embed=["06_TEST_ROMS.md#6. Commande", "01_ARCHITECTURE.md#6. CLI"],
  write=[CLI + "/src/main.rs", CLI + "/src/ppm.rs", CLI + "/tests/e01_07_run.rs"],
  spec="""- `src/ppm.rs`: `pub fn encode_ppm(width: usize, height: usize, pixels: &[u32]) -> Vec<u8>` : en-tete ASCII `P6\\n<w> <h>\\n255\\n` puis, pour chaque pixel XRGB, les 3 octets R, G, B (R = bits 23-16, G = bits 15-8, B = bits 7-0).
- `main.rs`: sous-commande `run` avec options: `--test-pattern` (flag, obligatoire pour l'instant: sans lui, erreur d'usage `rom path not supported yet`, code de sortie 2), `--frames <N>` (defaut 1), `--hash` (affiche une ligne `frame_hash=0x<16 hex minuscules>` = Fnv64 sur `width, height` (u32) puis les pixels de `frame()` en u32 little-endian), `--region ntsc|pal` (defaut ntsc), `--dump-ppm <FICHIER>` (ecrit le PPM de la derniere frame).
- Codes de sortie: 0 OK, 2 erreur d'usage/fichier.""",
  tests=[("ppm_2x1", "`encode_ppm(2,1,&[0xFF102030,0xFFFFFFFF])` == b\"P6\\n2 1\\n255\\n\" + [0x10,0x20,0x30,0xFF,0xFF,0xFF]."),
         ("run_hash_stable", "le binaire lance deux fois avec `run --test-pattern --frames 5 --hash` produit la meme ligne `frame_hash=0x...` (16 hex)."),
         ("run_hash_frames", "`--frames 5` et `--frames 6` produisent des hash differents."),
         ("dump_ppm", "`--dump-ppm <tmp>` cree un fichier dont les 11 premiers octets sont b\"P6\\n256 224\\n\" et dont la taille est 15 + 256*224*3."),
         ("no_pattern_usage", "sans `--test-pattern`: code de sortie 2.")],
  accept=["cargo test -p puce16-cli e01_07_"], crate="puce16-cli")

T("E01-08", "Helper de test make_rom (ROM synthetiques)",
  deps=["E01-01"], size=110,
  context="""Les tests d'integration sans ROM externe (bus, DMA, NMI, joypad...) ont besoin de fabriquer de petites ROM valides.
Ce helper est INDEPENDANT du code cart/ (il ecrit l'en-tete a partir de la spec 04 pour ne pas dependre d'un module encore inexistant).""",
  embed=["04_CART_HEADER.md#2. En-tete interne", "04_CART_HEADER.md#2.4 Checksum", "06_TEST_ROMS.md#7. Tests internes"],
  write=[CORE + "/tests/common/mod.rs", CORE + "/tests/e01_08_make_rom.rs"],
  spec="""Dans `tests/common/mod.rs` (module partage `mod common;` par les autres fichiers de test):
```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Map { LoRom, HiRom, ExHiRom }
/// Build a valid ROM. `code` is placed at the file offset that CPU address $00:8000 maps to:
/// LoRom file offset 0x0000 ; HiRom file offset 0x8000 ; ExHiRom file offset 0x408000.
pub fn make_rom(code: &[u8], map: Map) -> Vec<u8>;
pub fn make_rom_with(code: &[u8], map: Map, rom_log2: u8 /* 15=32 KiB .. */, sram_log2: u8, chipset: u8, country: u8) -> Vec<u8>;
```
Regles de `make_rom_with`:
- taille ROM = `1 << rom_log2` octets (panique si trop petit pour contenir l'en-tete: LoRom >= 32 Kio, HiRom >= 64 Kio, ExHiRom >= 8 Mio = `rom_log2 >= 23`), remplie d'octets 0x00 sauf code et en-tete.
- en-tete a: LoRom `0x7FC0`, HiRom `0xFFC0`, ExHiRom `0x40FFC0`. Titre = 21 octets ASCII `"PUCE16 TEST ROM      "`. Octet map: LoRom `0x20`, HiRom `0x21`, ExHiRom `0x25`. Chipset = param. Taille ROM = `rom_log2 - 10`. Taille RAM = `sram_log2` (0 = aucune; la valeur est l'exposant N de `1024 << N`, donc 3 = 8 Kio). Pays = param. Dev = 0x00. Version = 0.
- vecteurs (zone header_offset + 0x20 .. +0x3F): TOUS les mots a `0x8000` (little-endian). `pub fn set_vector(rom: &mut Vec<u8>, map: Map, name: &str, addr: u16)` ecrase un vecteur puis RECALCULE le checksum. Table (offsets relatifs au debut de l'en-tete): `cop` -> +0x24 (natif) et +0x34 (emulation); `brk` -> +0x26 (natif) et +0x3E (emulation); `nmi` -> +0x2A et +0x3A; `irq` -> +0x2E et +0x3E; `reset` -> +0x3C seulement. Nom inconnu: panique.
- code copie au debut du code reset (voir plus haut); checksum: complement et checksum mis a 0xFFFF / 0x0000 pendant le calcul, somme de tous les octets (u16 wrapping), puis `checksum` en header+0x1E (LE) et `!checksum` en header+0x1C (LE).
- `make_rom(code, map)` = `make_rom_with(code, map, 15 (LoRom) / 16 (HiRom) / 23 (ExHiRom, = 8 Mio), 0, 0x00, 0x01)`.""",
  tests=[("lorom_layout", "`make_rom(&[0xEA], Map::LoRom)`: len==32768 ; rom[0]==0xEA ; rom[0x7FFC..0x7FFE]==[0x00,0x80] ; rom[0x7FD5]==0x20 ; rom[0x7FC0..0x7FC6]==b\"PUCE16\"."),
         ("hirom_layout", "`make_rom(&[0xEA], Map::HiRom)`: len==65536 ; rom[0x8000]==0xEA ; rom[0xFFFC..0xFFFE]==[0x00,0x80] ; rom[0xFFD5]==0x21."),
         ("checksum_valid", "pour LoRom et HiRom: somme u16 (wrapping) de tous les octets == checksum lu en header+0x1E ; checksum ^ complement == 0xFFFF."),
         ("set_vector", "`set_vector(&mut rom, Map::LoRom, \"nmi\", 0x8123)`: rom[0x7FEA..0x7FEC]==[0x23,0x81] et rom[0x7FFA..0x7FFC]==[0x23,0x81] ; le checksum reste valide (set_vector recalcule)."),
         ("chipset_sram", "`make_rom_with(&[], Map::LoRom, 15, 3, 0x02, 1)`: rom[0x7FD6]==0x02 ; rom[0x7FD8]==3 ; rom[0x7FD9]==1.")])

T("E01-09", "Desktop: thread d'emulation (Command / Event) sur le motif de test",
  deps=["E01-02", "E01-06"], size=140,
  context="Coeur non graphique du desktop, testable sans fenetre: un thread possede `Snes` et dialogue par canaux (spec 08 section 7).",
  embed=["08_INPUT_DESKTOP.md#7. Architecture desktop"],
  write=[DESK + "/src/emu_thread.rs", DESK + "/tests/e01_09_emu_thread.rs"],
  spec="""Dans `emu_thread.rs` (le module est deja declare dans lib.rs; les tests importent `puce16_desktop::emu_thread::*`):
```rust
pub enum Command { Pause(bool), FrameAdvance, SetJoypad(usize, u16), Reset, SetFastForward(bool), Quit }   // LoadRom/SaveState/... viendront plus tard
pub struct FrameData { pub width: usize, pub height: usize, pub pixels: Vec<u32>, pub frame_no: u64 }
pub enum Event { Frame(std::sync::Arc<FrameData>), Status(String) }
pub struct EmuHandle { pub tx: std::sync::mpsc::Sender<Command>, pub rx: std::sync::mpsc::Receiver<Event>, join: Option<std::thread::JoinHandle<()>> }
pub fn spawn_test_pattern(region: puce16_core::Region, start_paused: bool) -> EmuHandle;
impl EmuHandle { pub fn quit(self); }   // envoie Quit et join
```
Comportement:
- le thread cree `Snes::new_test_pattern(region)` et boucle: traite TOUTES les commandes en attente (`try_recv`); si non pause: `run_frame()`, envoie `Event::Frame` (copie de `frame()` + `frame_count`), puis attend ~16 ms (`thread::sleep`; cadence exacte non requise ici). Si pause: dort 5 ms.
- `FrameAdvance` en pause: execute exactement une frame et l'envoie. `Pause(false)` reprend. `SetJoypad(p, b)` appelle `set_joypad`. `Reset`: recree le `Snes` (frame_no repart a 1 au prochain run). `Quit` termine le thread.
- Le canal d'evenements est borne par la consommation de la UI (utilise `mpsc::channel` non borne mais n'envoie pas plus d'une frame par frame emulee).""",
  tests=[("start_paused_no_frame", "`spawn_test_pattern(Ntsc, true)`: apres 100 ms `rx.try_recv()` est vide ; puis `FrameAdvance` -> on recoit un `Event::Frame` avec `frame_no==1` (timeout 2 s)."),
         ("frame_advance_twice", "deux `FrameAdvance` en pause -> `frame_no` 1 puis 2 (ordre garanti)."),
         ("run_free", "`spawn_test_pattern(Ntsc,false)`: on recoit au moins 3 `Event::Frame` en < 2 s, `frame_no` strictement croissant."),
         ("joypad_visible", "en pause: `SetJoypad(0,0x8000)` puis `FrameAdvance` -> le pixel (0,0) de la frame recue vaut 0xFFFFFFFF."),
         ("quit_joins", "`quit()` retourne en < 1 s sans panique.")],
  accept=["cargo test -p puce16-desktop e01_09_"], crate="puce16-desktop")

T("E01-10", "Desktop: video.rs (conversion XRGB -> RGBA et calcul de taille)",
  deps=["E01-02"], size=80,
  context="Fonctions pures du rendu desktop (testables sans fenetre).",
  embed=["08_INPUT_DESKTOP.md#4. Fenetre, video"],
  write=[DESK + "/src/video.rs", DESK + "/tests/e01_10_video.rs"],
  spec="""Dans `video.rs` (le module est deja declare dans lib.rs; les tests importent `puce16_desktop::video::*`):
```rust
pub fn xrgb_to_rgba(src: &[u32]) -> Vec<u8>;       // chaque pixel -> [R, G, B, 255] ; R = bits 23-16, G = 15-8, B = 7-0 ; l'octet haut de src est ignore
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Ratio { Square, Par87 }
/// taille d'affichage en points ecran: largeur * scale (* 8/7 arrondi vers le bas si Par87 et largeur 256) ; hauteur * scale
pub fn display_size(w: usize, h: usize, scale: usize, ratio: Ratio) -> (usize, usize);
pub fn fit_scale(w: usize, h: usize, avail_w: usize, avail_h: usize) -> usize;   // plus grand entier >= 1 tel que w*s <= avail_w et h*s <= avail_h (min 1)
```
Pour le ratio 8:7 la largeur devient `w * scale * 8 / 7` (entier). Aucun flottant n'est necessaire.""",
  tests=[("rgba_basic", "`xrgb_to_rgba(&[0xFF102030, 0x00FFFFFF])` == [0x10,0x20,0x30,0xFF, 0xFF,0xFF,0xFF,0xFF]."),
         ("display_square", "`display_size(256,224,3,Square)==(768,672)`."),
         ("display_87", "`display_size(256,224,3,Par87)==(877,672)` (768*8/7 = 877)."),
         ("fit_scale", "`fit_scale(256,224,1000,700)==3` ; `fit_scale(256,224,100,100)==1` ; `fit_scale(512,448,2048,896)==2`.")],
  accept=["cargo test -p puce16-desktop e01_10_"], crate="puce16-desktop")

T("E01-11", "Desktop: fenetre eframe affichant le motif de test (jalon D0)",
  deps=["E01-09", "E01-10"], size=130, kind="ui",
  context="""Jalon D0 (D-015): une vraie fenetre montrant la sortie du core. Code UI: pas de test automatique,
mais la compilation et les fonctions pures doivent etre propres.""",
  embed=["08_INPUT_DESKTOP.md#4. Fenetre, video"],
  write=[DESK + "/src/app.rs", DESK + "/src/main.rs"],
  spec="""- `app.rs`: `pub struct App { handle: EmuHandle, texture: Option<egui::TextureHandle>, last: Option<Arc<FrameData>>, scale: usize, ratio: Ratio, paused: bool, status: String }` implementant `eframe::App`.
- A chaque `update`: vider `handle.rx` (garder la derniere `Event::Frame`), convertir en RGBA (`video::xrgb_to_rgba`), mettre a jour/creer la texture (`ctx.load_texture`, `TextureOptions::NEAREST`), l'afficher dans le panneau central a la taille `display_size(w,h,scale,ratio)`; `ctx.request_repaint()` en continu.
- Barre de menu minimale: Fichier -> Quitter ; Emulation -> Pause (case a cocher, envoie `Command::Pause`), Image suivante (envoie `FrameAdvance`, actif si pause), Reset ; Video -> echelle 1..6 (valeur) et Ratio (carre / 8:7). Barre d'etat: `status`, numero de frame, `scale`.
- `main.rs`: lance `eframe::run_native("Puce16", options, ...)` avec fenetre initiale 768x672 + barres; cree `App` avec `spawn_test_pattern(Region::Ntsc, false)`. A la fermeture: `handle.quit()`.
- Pas de ROM, pas d'audio, pas d'entrees clavier (E01-13).""",
  tests=[],
  accept=["cargo build -p puce16-desktop"],
  crate="puce16-desktop",
  notes="Verification visuelle faite par l'utilisateur au GATE E01-14.")

T("E01-12", "joypad.rs: constantes de boutons et filtre des directions opposees",
  deps=["E01-01"], size=50,
  context="Format 16 bits des manettes partage par core et desktop.",
  embed=["08_INPUT_DESKTOP.md#1. Manettes"],
  write=[CORE + "/src/joypad.rs", CORE + "/tests/e01_12_joypad_consts.rs"],
  spec="""Dans `joypad.rs` ajoute (pour l'instant seulement ceci; l'etat des ports vient en E03):
`pub const BTN_B: u16 = 1 << 15;` ... `BTN_R: u16 = 1 << 4;` (douze constantes, valeurs du texte ci-dessus) et
`pub fn filter_opposites(b: u16) -> u16` : si `BTN_LEFT` et `BTN_RIGHT` sont tous deux a 1, les deux sont mis a 0 ; idem `BTN_UP`/`BTN_DOWN`. Les autres bits inchanges.""",
  tests=[("values", "`BTN_B==0x8000`, `BTN_Y==0x4000`, `BTN_SELECT==0x2000`, `BTN_START==0x1000`, `BTN_UP==0x0800`, `BTN_DOWN==0x0400`, `BTN_LEFT==0x0200`, `BTN_RIGHT==0x0100`, `BTN_A==0x0080`, `BTN_X==0x0040`, `BTN_L==0x0020`, `BTN_R==0x0010`."),
         ("no_overlap", "toutes les constantes sont deux a deux disjointes et leur OU vaut 0xFFF0."),
         ("filter_lr", "`filter_opposites(BTN_LEFT|BTN_RIGHT|BTN_A) == BTN_A`."),
         ("filter_ud_keep", "`filter_opposites(BTN_UP|BTN_LEFT) == BTN_UP|BTN_LEFT` ; `filter_opposites(BTN_UP|BTN_DOWN|BTN_LEFT|BTN_RIGHT) == 0`.")])

T("E01-13", "Desktop: clavier AZERTY (input.rs) et raccourcis (jalon D1)",
  deps=["E01-11", "E01-12"], size=140,
  context="Entrees clavier de l'utilisateur (AZERTY) et raccourcis emulateur, avec une table pure testable.",
  embed=["08_INPUT_DESKTOP.md#2. Clavier"],
  write=[DESK + "/src/input.rs", DESK + "/src/app.rs", DESK + "/tests/e01_13_input.rs"],
  spec="""Dans `input.rs` (module deja declare dans lib.rs; egui = `eframe::egui`):
```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Key { Up, Down, Left, Right, Z, S, Q, D, X, K, C, L, W, J, V, I, A, U, E, O, Enter, Backspace, ShiftRight, Other }
pub fn default_buttons(pressed: &[Key]) -> u16;   // OU des BTN_* selon le tableau du texte (sans filtrage)
pub fn from_egui_key(k: egui::Key) -> Key;          // correspondance egui::Key -> Key (Other si absente)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Hotkey { Pause, FrameAdvance, Reset, SaveState, LoadState, SlotPrev, SlotNext, Screenshot, Fullscreen, Debugger, Mute, FastForward, OpenRom, Quit }
pub fn hotkey_for(k: egui::Key, ctrl: bool) -> Option<Hotkey>;   // P, N, F12, F5, F7, PageUp->SlotPrev, PageDown->SlotNext, F2, F11, F10, M, Tab, Ctrl+O, Escape
```
Dans `app.rs`: a chaque frame UI, lire les touches maintenues (`ctx.input`), calculer `filter_opposites(default_buttons(...))` et envoyer `Command::SetJoypad(0, bits)` seulement quand la valeur change. Raccourcis Pause / FrameAdvance / Reset / Quit branches sur les commandes existantes; les autres sont ignores pour l'instant (log dans la barre d'etat: `hotkey not implemented`).""",
  tests=[("buttons_dpad", "`default_buttons(&[Key::Up, Key::Right])==BTN_UP|BTN_RIGHT` ; `default_buttons(&[Key::Z, Key::D])==BTN_UP|BTN_RIGHT` (Z=haut, D=droite)."),
         ("buttons_face", "`default_buttons(&[Key::X])==BTN_B` ; `[Key::C]==BTN_A` ; `[Key::W]==BTN_Y` ; `[Key::V]==BTN_X` ; `[Key::A]==BTN_L` ; `[Key::E]==BTN_R` ; `[Key::K]==BTN_B` ; `[Key::L]==BTN_A`."),
         ("buttons_start_select", "`[Key::Enter]==BTN_START` ; `[Key::Backspace]==BTN_SELECT` ; `[Key::ShiftRight]==BTN_SELECT`."),
         ("buttons_combo", "`default_buttons(&[Key::X, Key::C, Key::Enter])==BTN_B|BTN_A|BTN_START` ; `default_buttons(&[Key::Other])==0` ; `default_buttons(&[])==0`."),
         ("hotkeys", "`hotkey_for(egui::Key::P,false)==Some(Hotkey::Pause)` ; `F12 -> Reset` ; `O` avec ctrl=true -> `OpenRom` ; `O` sans ctrl -> `None` ; `PageUp -> SlotPrev`.")],
  accept=["cargo test -p puce16-desktop e01_13_", "cargo build -p puce16-desktop"], crate="puce16-desktop")

T("E01-14", "GATE J0: le desktop affiche le motif et reagit au clavier",
  deps=["E01-07", "E01-08", "E01-13"], size=0, gate=True, kind="gate",
  context="""Fin de l'epopee E01. L'utilisateur verifie a la main que la chaine complete (core -> thread -> fenetre -> clavier) fonctionne.
Aucun code a ecrire.""",
  spec="""Criteres de validation (humains):
1. La fenetre s'ouvre, un degrade colore defile horizontalement.
2. Menu Emulation: Pause fige l'image ; "Image suivante" avance d'une frame ; Reset relance ; la touche P pause aussi.
3. Menu Video: echelle 1..6 et ratio 8:7 changent la taille de l'image.
4. En appuyant sur la fleche haut + X, le pixel en haut a gauche devient blanc (marqueur de manette).
5. `cargo test --workspace` est vert.""",
  user_cmds=["cargo test --workspace", "cargo run -p puce16-desktop", "cargo run -p puce16-cli -- run --test-pattern --frames 10 --hash --dump-ppm tests/out/pattern.ppm"],
  accept=[])
