from e06_a import CHAIN
from e03_f import TINY
from _dsl import T

CORE = "crates/puce16-core"
CLI = "crates/puce16-cli"
SU = CLI + "/src/suite"
TST = CORE + "/tests"
CTST = CLI + "/tests"

WPROG = ("Programme d'ecriture WRAM (a mettre en tete d'une copie de `tiny_rom()`, octets 0..10): `78 18 FB A9 42 8D 10 00 80 FE` = SEI ; CLC ; XCE ; LDA #$42 ; STA $0010 ; BRA -2 "
         "(`fn wram_rom() -> Vec<u8> { let mut r = tiny_rom(); r[..10].copy_from_slice(&[0x78,0x18,0xFB,0xA9,0x42,0x8D,0x10,0x00,0x80,0xFE]); r }`).")

# ------------------------------------------------------------------ E07: test campaign (ROM de test, harnais suite, jeux)

T("E07-01", "T-ROMS: telecharger les ROM de test homebrew (reseau)",
  kind="tooling", size=0,
  context="""Les ROM de test homebrew (N3) servent a valider le PPU, le DMA et l'APU contre des images connues. Elles ne sont jamais commitees (dossier ignore par git). Aucune ROM commerciale ici (D-006: celles-ci sont fournies par l'utilisateur).
Aucune URL n'est supposee: chaque depot est RETROUVE par recherche web.""",
  embed=["06_TEST_ROMS.md#4. ROM de test homebrew"],
  write=["tests/roms/homebrew/**", "tests/roms/README.md"],
  spec="""1. Pour chaque candidat du tableau (PeterLemon/SNES, gilyon/snes-tests, undisbeliever/snes-test-roms, tests SPC700/DSP de blargg, tests bsnes/higan): retrouver le depot (recherche web), cloner en `--depth 1` dans un dossier TEMPORAIRE hors du depot, puis COPIER seulement les ROM `.sfc` / `.smc` deja compilees dans `tests/roms/homebrew/<auteur>/` en gardant l'arborescence relative du depot. Si le depot ne fournit que des sources: ne pas compiler, noter `source only`. Limite: 300 Mo au total; si un depot est plus gros, utiliser un clone partiel/sparse (`git clone --filter=blob:none --sparse` puis `git sparse-checkout set <dossiers>`).
2. Ecrire `tests/roms/README.md` (commite) avec, pour chaque candidat: nom, URL reelle, date de telechargement, licence si indiquee, nombre de ROM copiees, et un tableau `chemin | taille | sha256` (calcule par script, jamais a la main). Un candidat introuvable: ligne `not found`.
3. Ajouter a la fin du README une section `## Jeux commerciaux (fournis par l'utilisateur)` qui dit: placer les ROM dans `tests/roms/games/` sous les noms exacts `smw.sfc`, `alttp.sfc`, `smb_allstars.sfc` (jamais commitees).
4. Verifier avec `git status` qu'aucune ROM n'apparait comme fichier a ajouter (le `.gitignore` les couvre).
5. Reseau bloque ou aucun candidat accessible: question `tooling` (docs/02_QA_PROTOCOL.md) et arrete-toi.""",
  accept=["python -c \"import os,sys; sys.exit(0 if os.path.isfile('tests/roms/README.md') else 1)\"",
          "python -c \"import sys; t=open('tests/roms/README.md',encoding='utf-8').read(); sys.exit(0 if 'sha256' in t and 'smw.sfc' in t else 1)\""],
  notes=CHAIN)

T("E07-02", "tests/manifest.toml et tests/baseline.json initiaux",
  kind="data", size=0,
  context="""Le manifeste liste les ROM de test a executer par `puce16-cli suite`. Il est ecrit a partir des fichiers REELLEMENT telecharges (README de E07-01), jamais de memoire. Le baseline est vide au depart: les hash sont fixes plus tard par l'utilisateur apres validation visuelle (E07-12).""",
  embed=["06_TEST_ROMS.md#5. Manifeste"],
  reads=["tests/roms/README.md", "specs/06_TEST_ROMS.md sections 4, 5, 6"],
  write=["tests/manifest.toml", "tests/baseline.json"],
  spec="""1. `tests/baseline.json` contient exactement `{\\n  "version": 1,\\n  "roms": {}\\n}\\n`.
2. `tests/manifest.toml`: une entree `[[rom]]` par ROM de test utile (viser 20 a 60 entrees, choisies par categorie). Champs comme la section 5: `name` (unique, `[a-z0-9_]+`, de la forme `<auteur>_<categorie>_<nom>`), `path` (relatif a la racine du depot, existe vraiment), `region = "auto"`, `frames`, `optional = false`, `epic`, `check = "hash"`.
   Choix de `epic` et `frames`: tests CPU -> `E02`, 120 frames ; DMA/HDMA/IRQ/timing -> `E03`, 120 ; affichage de base, VRAM, CGRAM, OAM -> `E04`, 120 ; modes graphiques 0-7, sprites, fenetres, mosaique, color math, HDMA graphique -> `E05`, 120 (mode 7: 240) ; SPC700/BRR/DSP -> `E06`, 300.
   Une ROM dont le nom ou le contenu suggere un resultat texte (PASS/FAIL a l'ecran) est quand meme `check = "hash"`.
3. Ajouter 3 entrees de jeux commerciaux `optional = true`: `game_smw` (`tests/roms/games/smw.sfc`, `epic = "E07"`, `frames = 600`, `check = "none"`), `game_alttp` (`alttp.sfc`), `game_allstars` (`smb_allstars.sfc`), memes champs.
4. Controle (script Python fourni dans la tache, a lancer): lire le TOML avec `tomllib` (Python >= 3.11), verifier noms uniques et conformes, `epic` de la forme `E` + 2 chiffres, `frames` > 0, et que chaque `path` existe SAUF les entrees `optional`. Afficher le nombre d'entrees par epopee.""",
  accept=["python -c \"import tomllib,os,re,sys; d=tomllib.load(open('tests/manifest.toml','rb')); r=d['rom']; n=[x['name'] for x in r]; ok=len(n)==len(set(n)) and all(re.fullmatch(r'[a-z0-9_]+',x) for x in n) and all(re.fullmatch(r'E[0-9][0-9]',x['epic']) for x in r) and all(x['frames']>0 for x in r) and all(os.path.isfile(x['path']) or x.get('optional') for x in r) and len(r)>=10; sys.exit(0 if ok else 1)\"",
          "python -c \"import json,sys; d=json.load(open('tests/baseline.json')); sys.exit(0 if d=={'version':1,'roms':{}} else 1)\""],
  notes=CHAIN)

T("E07-03", "GATE: revue du manifeste et des ROM de test (Claude)",
  kind="gate", gate=True, size=0,
  context="Claude relit README + manifeste: provenance des ROM (pas de ROM commerciale), choix des epopees et des frames, coherence des noms.",
  spec="""Criteres de validation:
1. `tests/roms/README.md` cite une URL reelle par candidat, aucune ROM commerciale n'est dans `tests/roms/homebrew/`.
2. `tests/manifest.toml` couvre au moins: CPU, DMA/HDMA, modes graphiques 0-7, sprites, fenetres, mosaique, SPC700/DSP (si trouves).
3. `git status` ne montre aucune ROM ni aucun gros fichier a ajouter.""",
  user_cmds=["git status", "python docs/tools/task.py approve E07-03"], accept=[])

T("E07-04", "suite/manifest.rs: lecture et validation du manifeste TOML",
  size=130, kind="code", crate="puce16-cli",
  context="Premier module du harnais `suite` (les 4 fichiers `suite/*.rs` existent deja vides depuis E02-11). Parse le manifeste de la section 5 de la spec 06. Utilise les dependances deja presentes (`serde`, `toml`). " + CHAIN,
  embed=["06_TEST_ROMS.md#5. Manifeste"],
  write=[SU + "/manifest.rs"],
  spec="""Dans `suite/manifest.rs` (tests unitaires `#[cfg(test)] mod tests` dans le fichier, noms `e07_04_...`):
```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum RegionSel { Auto, Ntsc, Pal }
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Check { Hash, Ram, Ports, None }
#[derive(Clone, PartialEq, Eq, Debug)] pub struct RamExpect { pub addr: u32, pub value: u8 }
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RomEntry { pub name: String, pub path: String, pub region: RegionSel, pub frames: u32, pub optional: bool, pub epic: String, pub check: Check, pub expect_ram: Vec<RamExpect>, pub expect_ports: Option<[u8; 4]> }
pub fn parse(text: &str) -> Result<Vec<RomEntry>, String>;       // ordre du fichier conserve
pub fn epic_number(s: &str) -> Option<u32>;                      // "E04" -> Some(4) ; exactement 'E' majuscule + 2 chiffres, sinon None
```
Deserialisation avec `serde` + `toml` (structure brute `Raw { rom: Option<Vec<RawRom>> }`, champs optionnels avec defauts: `region="auto"`, `optional=false`, `check="hash"`). `expect_ram` = tableau de tables `{ addr, value }` (entiers TOML, hexa autorise), `expect_ports` = tableau de 4 entiers 0..255.
Validation (message d'erreur commencant par le nom de la ROM: `<name>: <probleme>`): `name` vide ou hors `[a-z0-9_]+` ; nom duplique ; `frames` hors 1..=100000 ; `epic` invalide ; `region` hors auto/ntsc/pal ; `check` hors hash/ram/ports/none ; `check = ram` sans `expect_ram` (ou vide) ; `addr` hors 0..=0xFFFFFF ; `check = ports` sans `expect_ports` de longueur 4. Un fichier sans `[[rom]]` donne `Ok(vec![])`. TOML invalide -> `Err` contenant `invalid manifest`.""",
  tests=[("parse_full", "un manifeste de 3 entrees (une avec `check=\"ram\"` + `expect_ram = [ { addr = 0x7E0010, value = 0x42 } ]`, une avec `check=\"ports\"` + `expect_ports = [0xAA, 0xBB, 0, 0]`, une `optional = true` avec `region = \"pal\"`) -> 3 entrees dans l'ordre, tous les champs corrects (`addr == 0x7E0010`, `expect_ports == Some([0xAA,0xBB,0,0])`)."),
         ("defaults", "une entree avec seulement `name,path,frames,epic` : `region==Auto`, `optional==false`, `check==Check::Hash`, `expect_ram` vide, `expect_ports==None`."),
         ("empty_manifest", "texte vide -> `Ok(vec![])`."),
         ("bad_toml", "`\"[[rom\"` -> Err contenant `invalid manifest`."),
         ("duplicate_name", "deux entrees `name=\"a\"` -> Err contenant `a: duplicate`."),
         ("bad_name", "`name=\"Bad-Name\"` -> Err contenant `Bad-Name`."),
         ("bad_frames", "`frames = 0` et `frames = 100001` -> Err chacun."),
         ("bad_epic", "`epic = \"E4\"`, `\"e04\"`, `\"E100\"` -> Err chacun ; `\"E04\"` -> Ok."),
         ("bad_enums", "`region = \"jp\"` -> Err ; `check = \"crc\"` -> Err."),
         ("ram_needs_expect", "`check = \"ram\"` sans `expect_ram` -> Err contenant `expect_ram` ; avec `addr = 0x1000000` -> Err."),
         ("ports_needs_four", "`check = \"ports\"` avec 3 valeurs -> Err contenant `expect_ports`."),
         ("epic_number", "`epic_number(\"E04\")==Some(4)`, `epic_number(\"E10\")==Some(10)`, `epic_number(\"E4\")==None`, `epic_number(\"e04\")==None`, `epic_number(\"E100\")==None`.")],
  accept=["cargo test -p puce16-cli e07_04_"])

T("E07-05", "suite/baseline.rs: lecture/ecriture de baseline.json",
  size=110, kind="code", crate="puce16-cli",
  context="Fichier des hash valides par l'utilisateur. Format et ecriture deterministes (diff lisible dans git). " + CHAIN,
  embed=["06_TEST_ROMS.md#5. Manifeste"],
  write=[SU + "/baseline.rs"],
  spec="""Dans `suite/baseline.rs` (tests unitaires dans le fichier, noms `e07_05_...`):
```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub struct Entry { pub frames: u32, pub frame_hash: u64, pub state_hash: u64, pub audio_hash: u64 }
#[derive(Clone, PartialEq, Eq, Debug, Default)] pub struct Baseline { pub roms: std::collections::BTreeMap<String, Entry> }
pub fn hex64(v: u64) -> String;                 // "0x" + 16 chiffres hexa minuscules
pub fn parse_hex64(s: &str) -> Option<u64>;     // exactement "0x" + 16 hexa (majuscules ou minuscules acceptees)
impl Baseline {
    pub fn parse(text: &str) -> Result<Baseline, String>;
    pub fn to_json(&self) -> String;
    pub fn load(path: &std::path::Path) -> Result<Baseline, String>;          // fichier absent -> Ok(Baseline::default())
    pub fn save(&self, path: &std::path::Path) -> Result<(), String>;         // ecrit path.tmp puis renomme (atomique)
}
```
`parse`: JSON `{ "version": 1, "roms": { "<name>": { "frames": N, "frame_hash": "0x..", "state_hash": "0x..", "audio_hash": "0x.." } } }` via `serde_json`; `version` different de 1 -> Err contenant `unsupported baseline version`; hash mal forme -> Err contenant le nom de la ROM ; JSON invalide -> Err contenant `invalid baseline`.
`to_json`: `serde_json::to_string_pretty` d'une structure ayant `version` en premier puis `roms` (BTreeMap: cles triees), champs d'une entree dans l'ordre `frames, frame_hash, state_hash, audio_hash`, suivi d'un `\\n` final.""",
  tests=[("hex64", "`hex64(0)==\"0x0000000000000000\"` ; `hex64(0xDEADBEEF)==\"0x00000000deadbeef\"` ; `parse_hex64(\"0x00000000DEADBEEF\")==Some(0xDEADBEEF)` ; `parse_hex64(\"0xDEADBEEF\")==None` (16 chiffres exiges) ; `parse_hex64(\"deadbeef\")==None`."),
         ("roundtrip", "un `Baseline` de 2 entrees -> `to_json` -> `parse` == original."),
         ("json_exact", "`Baseline` vide -> `to_json()==\"{\\n  \\\"version\\\": 1,\\n  \\\"roms\\\": {}\\n}\\n\"`."),
         ("keys_sorted", "entrees inserees `b` puis `a` : dans `to_json`, `\"a\"` apparait avant `\"b\"`."),
         ("entry_field_order", "le JSON d'une entree contient `frames` < `frame_hash` < `state_hash` < `audio_hash` (positions croissantes dans le texte)."),
         ("bad_version", "`{\"version\":2,\"roms\":{}}` -> Err contenant `unsupported baseline version`."),
         ("bad_hash", "entree `x` avec `\"frame_hash\":\"0x12\"` -> Err contenant `x`."),
         ("bad_json", "`not json` -> Err contenant `invalid baseline`."),
         ("load_missing_is_empty", "chemin inexistant dans un dossier temporaire -> `Ok(Baseline::default())`."),
         ("save_load_atomic", "`save` dans un dossier temporaire puis `load` == original ; le fichier `.tmp` n'existe plus apres `save`.")],
  accept=["cargo test -p puce16-cli e07_05_"])

T("E07-06", "suite/report.rs: verdicts, lignes de rapport et code de sortie",
  size=90, kind="code", crate="puce16-cli",
  context="Format exact des lignes imprimees par `puce16-cli suite` (lues par l'utilisateur et par le script de rapport). " + CHAIN,
  embed=["06_TEST_ROMS.md#6. Commande"],
  write=[SU + "/report.rs"],
  spec="""Dans `suite/report.rs` (tests unitaires dans le fichier, noms `e07_06_...`):
```rust
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Verdict { Pass, Fail(String), Skip(String), New { frame_hash: u64, state_hash: u64, audio_hash: u64 } }
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)] pub struct Counts { pub pass: u32, pub fail: u32, pub skip: u32, pub new: u32 }
pub fn format_line(name: &str, v: &Verdict) -> String;
pub fn tally(vs: &[Verdict]) -> Counts;
pub fn summary(c: &Counts) -> String;
pub fn exit_code(c: &Counts) -> i32;       // 1 si fail > 0, sinon 0
```
Formats exacts: `PASS <name>` ; `FAIL <name>: <raison>` ; `SKIP <name>: <raison>` ; `NEW <name> frame_hash=0x<16 hex> state_hash=0x<16 hex> audio_hash=0x<16 hex>` (hexa minuscules). `summary`: `suite: <pass> pass, <fail> fail, <skip> skip, <new> new`. Une raison multi-lignes est conservee telle quelle (pas de reformatage).""",
  tests=[("lines", "`format_line(\"a\",&Verdict::Pass)==\"PASS a\"` ; `Fail(\"ram 0x7E0010: expected 0x42 got 0x00\".into())` -> `\"FAIL a: ram 0x7E0010: expected 0x42 got 0x00\"` ; `Skip(\"file missing\".into())` -> `\"SKIP a: file missing\"`."),
         ("new_line", "`New{frame_hash:1,state_hash:2,audio_hash:0xABC}` -> `\"NEW a frame_hash=0x0000000000000001 state_hash=0x0000000000000002 audio_hash=0x0000000000000abc\"`."),
         ("tally", "`[Pass,Pass,Fail(..),Skip(..),New{..}]` -> `Counts{pass:2,fail:1,skip:1,new:1}`."),
         ("summary", "`summary(&Counts{pass:3,fail:0,skip:1,new:2})==\"suite: 3 pass, 0 fail, 1 skip, 2 new\"`."),
         ("exit_codes", "fail>0 -> 1 ; seulement pass/skip/new -> 0 ; `Counts::default()` -> 0.")],
  accept=["cargo test -p puce16-cli e07_06_"])

T("E07-07", "core: Snes::apu_port (ports vus par le S-CPU)",
  size=40, kind="code",
  context="Le controle `ports` du manifeste lit les 4 octets que le S-CPU voit sur `$2140-$2143` (le handshake `$AA`/`$BB` de l'IPL). Petit accesseur public. " + CHAIN,
  write=[CORE + "/src/snes.rs", TST + "/e07_07_apu_port.rs"],
  spec="""Dans `snes.rs`: `pub fn apu_port(&self, i: usize) -> u8` : variant reel `self.sys.apu.port_read(i)` ; variant motif de test: 0.""",
  tests=[("handshake_visible", "ROM `tiny_rom()`-like (voir le helper `common::make_rom(&[0x80,0xFE], Map::LoRom)` de E01-08) : `Snes::new`, 3 `run_frame()` : `apu_port(0)==0xAA`, `apu_port(1)==0xBB`, `apu_port(2)==0`, `apu_port(3)==0`."),
         ("before_boot", "juste apres `Snes::new` : les quatre ports valent 0."),
         ("index_masked", "`apu_port(4)==apu_port(0)`.")],
  notes=CHAIN)

T("E07-08", "suite/runner.rs: executer une ROM du manifeste et la juger",
  size=150, kind="code", crate="puce16-cli",
  context="""Coeur du harnais: charge une ROM, execute N frames, calcule les trois hash, collecte ce que le controle demande, puis rend un verdict. Les hash sont calcules comme `puce16-cli run --hash` / `--state-hash`. """ + CHAIN,
  embed=["06_TEST_ROMS.md#6. Commande", "06_TEST_ROMS.md#5. Manifeste"],
  write=[SU + "/runner.rs"],
  spec="""Dans `suite/runner.rs` (tests unitaires dans le fichier, noms `e07_08_...`; ROM de test fabriquees en memoire avec `tiny_rom()` (voir ci-dessous) et ecrites dans un dossier temporaire):
```rust
use super::manifest::{RomEntry, Check, RegionSel};
use super::baseline::Entry;
use super::report::Verdict;
pub struct Run { pub width: usize, pub height: usize, pub pixels: Vec<u32>, pub frame_hash: u64, pub state_hash: u64, pub audio_hash: u64, pub ram: Vec<(u32, u8)>, pub ports: [u8; 4] }
pub fn frame_hash_of(width: usize, height: usize, pixels: &[u32]) -> u64;    // Fnv64: width (u32 LE), height (u32 LE), puis chaque pixel (u32 LE)
pub fn audio_hash_of(samples: &[[i16; 2]]) -> u64;                           // Fnv64: pour chaque paire, L (i16 LE) puis R (i16 LE)
pub fn run_rom(e: &RomEntry) -> Result<Run, String>;                         // Err(raison) : fichier illisible, ROM invalide, panique (avec la ligne debug)
pub fn judge(e: &RomEntry, run: &Run, base: Option<&Entry>) -> Verdict;
```
`run_rom`: lire `e.path` tel quel (relatif au dossier courant ou absolu) ; erreur -> `Err("cannot read <chemin>: <erreur>")` ; `Snes::new(&bytes, region)` avec `region = None` pour Auto, `Some(Region::Ntsc/Pal)` sinon ; erreur -> `Err("cannot load ROM: <describe_load_error>")` (fonction de `run_cmd`). Executer `e.frames` fois `run_frame()` dans `catch_unwind(AssertUnwindSafe(..))`; apres CHAQUE frame appeler `take_audio` dans un `Vec<[i16;2]>` cumule ; panique -> `Err("emulator panic: <format_debug(&snes.debug())>")`. Ensuite: `frame()` -> `width/height/pixels` (copie), les trois hash, `ram` = pour chaque `RamExpect` de `e.expect_ram` l'octet lu a l'adresse (`0x7E0000..=0x7FFFFF` -> `snes.peek_wram(addr - 0x7E0000)` ; autre adresse -> `Err("<name>: ram address 0x<addr:06X> not in WRAM")`), `ports` = `[snes.apu_port(0..4)]`.
`judge`: `Check::None` -> `Pass` ; `Check::Ram` -> `Fail("ram 0x<addr:06X>: expected 0x<v:02X> got 0x<g:02X>")` au premier octet different (les suivants ne sont pas listes), sinon `Pass` ; `Check::Ports` -> `Fail("ports: expected [AA, BB, 00, 00] got [..]")` (valeurs en hexa 2 chiffres majuscules, separees par `, `) ou `Pass` ; `Check::Hash` -> pas de baseline: `New{..}` ; `base.frames != e.frames` -> `Fail("baseline frames <b> != manifest <m>")` ; sinon comparer `frame_hash`, `state_hash`, `audio_hash` : si tous egaux `Pass`, sinon `Fail("frame_hash <actuel> != <attendu>, state_hash ...")` en ne listant que les hash differents (format `0x<16 hex>`, separes par `, `).""" + " " + TINY,
  tests=[("frame_hash_of", "`frame_hash_of(2,1,&[0xFF000000,0xFFFFFFFF])` == Fnv64 calcule dans le test sur les 16 octets `[2,0,0,0, 1,0,0,0, 0,0,0,0xFF, 0xFF,0xFF,0xFF,0xFF]` ; deux calculs identiques sont egaux, un pixel change le resultat."),
         ("audio_hash_of", "`audio_hash_of(&[])` == hash d'une entree vide (`Fnv64::new().finish()`) ; `audio_hash_of(&[[1,-1]])` == Fnv64 sur les octets `[1,0,0xFF,0xFF]`."),
         ("run_tiny_rom", "ROM `tiny_rom()` ecrite en fichier temporaire, entree `frames=3`, `check=Hash` : `run_rom` Ok, `width==256`, `height==224`, `ports==[0xAA,0xBB,0,0]`, `pixels.len()==57344`."),
         ("run_is_deterministic", "deux `run_rom` de la meme entree : memes `frame_hash`, `state_hash`, `audio_hash`."),
         ("run_ram_values", "ROM `wram_rom()` (programme ci-dessous), `expect_ram=[{addr:0x7E0010,value:0x42}]` : `run.ram==[(0x7E0010,0x42)]`."),
         ("run_ram_bad_address", "`expect_ram=[{addr:0x008000,value:1}]` -> Err contenant `not in WRAM`."),
         ("run_missing_file", "chemin inexistant -> Err commencant par `cannot read`."),
         ("run_bad_rom", "fichier de 100 octets -> Err commencant par `cannot load ROM`."),
         ("judge_none", "`Check::None` -> `Pass`."),
         ("judge_ram", "attendu 0x42 lu 0x42 -> `Pass` ; attendu 0x43 lu 0x42 -> `Fail(\"ram 0x7E0010: expected 0x43 got 0x42\")`."),
         ("judge_ports", "attendu `[0xAA,0xBB,0,0]` lu pareil -> `Pass` ; attendu `[0xAA,0xBC,0,0]` -> `Fail` contenant `expected [AA, BC, 00, 00] got [AA, BB, 00, 00]`."),
         ("judge_hash_new", "pas de baseline -> `New` avec les trois hash du run."),
         ("judge_hash_pass_and_fail", "baseline egale -> `Pass` ; `state_hash` modifie -> `Fail` contenant `state_hash` et ne contenant pas `frame_hash` ; `frames` different -> `Fail` contenant `baseline frames`.")],
  notes=CHAIN + "\n" + WPROG,
  accept=["cargo test -p puce16-cli e07_08_"])

T("E07-09", "suite.rs: la commande suite (manifeste, baseline, filtre d'epopee, PPM, mise a jour)",
  size=130, kind="ui", crate="puce16-cli",
  context="Assemble manifest + baseline + runner + report dans `suite::run` (signature fixee en E02-11). Les squelettes de `run` (chemin absent -> 2) restent valides. " + CHAIN,
  embed=["06_TEST_ROMS.md#6. Commande"],
  write=[CLI + "/src/suite.rs"],
  spec="""Dans `suite.rs` remplace le corps de `run(manifest: &Path, max_epic: Option<&str>, update_baseline: Option<&str>, dump_ppm: Option<&Path>) -> i32`:
1. Chemin absent -> stderr `path not found: <chemin>`, retour 2 (comportement existant). Lire le manifeste (erreur -> stderr `cannot read <chemin>: <err>`, 2) puis `manifest::parse` (erreur -> stderr du message, 2).
2. `max_epic`: `Some(s)` -> `manifest::epic_number(s)`; invalide -> stderr `bad --max-epic: <s>`, 2.
3. Baseline: `<dossier du manifeste>/baseline.json` via `Baseline::load` (erreur -> stderr, 2).
4. Si `update_baseline = Some(name)`: trouver l'entree `name` (absente -> stderr `unknown rom: <name>`, 2) ; `run_rom` (Err -> stderr, 1) ; `baseline.roms.insert(name, Entry{frames: e.frames, frame_hash, state_hash, audio_hash})` ; `save` ; stdout `baseline updated: <name>` ; retour 0 (aucune autre ROM n'est executee).
5. Sinon, pour chaque entree dans l'ordre du manifeste dont `epic_number(e.epic) <= max_epic` (toutes si `None`): si le fichier n'existe pas: `optional` -> `Verdict::Skip("file missing")`, sinon `Fail("file missing: <chemin>")` ; sinon `run_rom` (Err(msg) -> `Fail(msg)`) puis `judge(e, &run, baseline.roms.get(name))`. Imprimer `format_line` sur stdout immediatement. Si `dump_ppm = Some(dir)`: creer le dossier (`create_dir_all`) et ecrire `<dir>/<name>.ppm` (`ppm::encode_ppm`) pour chaque ROM executee.
6. Fin: imprimer `summary(&tally(..))` ; retourner `exit_code`.
Retirer le `#![allow(dead_code)]` de `suite.rs` seulement si plus aucun avertissement ne reste. Cette tache n'a pas de test propre: ses tests d'integration (binaire) sont dans E07-10.""",
  tests=[],
  accept=["cargo build -p puce16-cli", "cargo test -p puce16-cli e02_11_"])

T("E07-10", "Tests d'integration de `puce16-cli suite` (ROM synthetiques)",
  size=140, kind="code", crate="puce16-cli",
  context="Teste le binaire `puce16-cli suite` de bout en bout avec des ROM et manifestes fabriques dans un dossier temporaire (aucun fichier externe). " + CHAIN,
  write=[CTST + "/e07_10_suite.rs"],
  spec="""Fichier `tests/e07_10_suite.rs` du crate `puce16-cli` (lancer le binaire par `env!("CARGO_BIN_EXE_puce16-cli")` avec `std::process::Command`). Aides a recopier: `tiny_rom()` et `wram_rom()` ci-dessous ; `fn setup(dir: &Path) ` qui ecrit `dir/tiny.sfc`, `dir/wram.sfc` et un `manifest.toml` ; chemins ABSOLUS dans le TOML (utiliser `dir.display()` avec des `/`). """ + TINY + " " + WPROG + """
Manifeste de base (`<D>` = dossier temporaire):
```toml
[[rom]]
name = "tiny_hash"
path = "<D>/tiny.sfc"
frames = 3
epic = "E04"
[[rom]]
name = "wram_ram"
path = "<D>/wram.sfc"
frames = 2
epic = "E03"
check = "ram"
expect_ram = [ { addr = 0x7E0010, value = 0x42 } ]
[[rom]]
name = "tiny_ports"
path = "<D>/tiny.sfc"
frames = 3
epic = "E06"
check = "ports"
expect_ports = [0xAA, 0xBB, 0, 0]
[[rom]]
name = "absent_game"
path = "<D>/nothere.sfc"
frames = 1
epic = "E07"
optional = true
```""",
  tests=[("first_run_reports_new", "`suite <D>/manifest.toml` (pas de baseline.json) : stdout contient `NEW tiny_hash frame_hash=0x`, `PASS wram_ram`, `PASS tiny_ports`, `SKIP absent_game: file missing` et la ligne `suite: 2 pass, 0 fail, 1 skip, 1 new` ; code de sortie 0."),
         ("update_then_pass", "`suite <manifest> --update-baseline tiny_hash` : stdout `baseline updated: tiny_hash`, code 0, `<D>/baseline.json` existe et contient `\"tiny_hash\"` et `\"frames\": 3` ; relancer `suite <manifest>` : stdout contient `PASS tiny_hash`, code 0."),
         ("hash_mismatch_fails", "apres `update_then_pass`, modifier `baseline.json` (remplacer la valeur de `state_hash` par `0x0000000000000000`) : `suite` -> stdout contient `FAIL tiny_hash:` et `state_hash`, code 1."),
         ("ram_mismatch_fails", "changer `value = 0x42` en `0x43` dans le manifeste : stdout contient `FAIL wram_ram: ram 0x7E0010: expected 0x43 got 0x42`, code 1."),
         ("max_epic_filters", "`--max-epic E03` : seule `wram_ram` est executee : stdout contient `PASS wram_ram` et ne contient pas `tiny_hash` ni `tiny_ports`."),
         ("bad_max_epic", "`--max-epic X1` -> code 2 et stderr contient `bad --max-epic`."),
         ("missing_required_rom_fails", "retirer `optional = true` de `absent_game` : stdout contient `FAIL absent_game: file missing`, code 1."),
         ("unknown_update_target", "`--update-baseline nope` -> code 2 et stderr contient `unknown rom: nope`."),
         ("dump_ppm", "`--dump-ppm <D>/out` : le fichier `<D>/out/tiny_hash.ppm` existe, commence par `P6\\n256 224\\n255\\n` et a la taille 15 + 256*224*3."),
         ("invalid_manifest", "manifeste `[[rom\\n` -> code 2 et stderr contient `invalid manifest`."),
         ("deterministic_output", "deux executions successives de `suite` (avec baseline a jour) produisent exactement la meme sortie standard.")],
  accept=["cargo test -p puce16-cli e07_10_", "cargo test -p puce16-cli e02_11_"])

T("E07-11", "T-SUITE-REPORT: lancer la suite sur les vraies ROM et documenter l'etat",
  kind="tooling", size=0,
  context="""Premier passage de la suite sur les ROM homebrew reelles. Cette tache est INFORMATIVE: elle ne corrige aucun code. Les echecs seront tries par Claude (questions `spec`) avant l'etape de validation visuelle.""",
  write=["docs/qa/SUITE_REPORT.md", "tests/out/**"],
  spec="""1. `cargo build --release -p puce16-cli` puis `target/release/puce16-cli suite tests/manifest.toml --dump-ppm tests/out/suite > tests/out/suite.txt 2>&1` (le code de sortie peut etre 1: normal ici). Un `emulator panic` est un resultat, pas une erreur de la tache.
2. Ecrire `docs/qa/SUITE_REPORT.md`: la ligne `suite: ...` finale, puis un tableau `nom | epopee | verdict | detail` pour CHAQUE ROM (copie des lignes de `tests/out/suite.txt`, jamais resumees a la main), puis la liste des ROM en `FAIL` regroupees par epopee avec, pour chacune, la phrase exacte du message (panique, mismatch, fichier manquant).
3. Ne modifie aucun fichier de code ni le manifeste. Si une ROM plante, ne la retire PAS du manifeste.
4. Rappelle en tete du rapport: les verdicts `NEW` sont normaux (le baseline est vide) et sont valides a l'etape E07-12.""",
  accept=["python -c \"import sys; t=open('docs/qa/SUITE_REPORT.md',encoding='utf-8').read(); sys.exit(0 if 'suite:' in t else 1)\"",
          "python -c \"import os,sys; sys.exit(0 if os.path.isfile('tests/out/suite.txt') else 1)\""],
  notes=CHAIN)

T("E07-12", "GATE J3/J5: captures validees, baseline fige",
  kind="gate", gate=True, size=0,
  context="""Jalons J3 (ROM graphiques: modes 0 a 7, sprites, fenetres, mosaique, color math, HDMA) et J5 (sons de ROM de test). Claude trie d'abord les echecs du rapport (corrections via des taches de tuning `TUNE-nn`, une constante a la fois, ajoutees par Claude). L'utilisateur regarde les captures et fige les hash.""",
  spec="""Criteres de validation (humains):
1. Ouvrir les images `tests/out/suite/*.ppm` (viewer d'images; ou ouvrir la ROM dans le desktop): chaque ROM graphique montre l'image attendue (l'utilisateur connait le rendu attendu de ces demos; en cas de doute: question a Claude, qui compare avec la reference du depot de la ROM).
2. Pour les ROM audio: ecouter dans le desktop (E08 requis pour le son) OU accepter le hash audio stable (deux executions identiques).
3. Pour chaque ROM jugee correcte: `target/release/puce16-cli suite tests/manifest.toml --update-baseline <nom>` puis committer `tests/baseline.json`.
4. Les ROM jugees incorrectes restent en `FAIL`/`NEW`: Claude cree des taches de tuning (aucun contournement par retrait du manifeste).
5. Critere de sortie: toutes les ROM de epopees E02..E05 `PASS`; les ROM E06 `PASS` ou documentees dans `docs/qa/SUITE_REPORT.md` comme `UNKNOWN` (avec cause).""",
  user_cmds=["target/release/puce16-cli suite tests/manifest.toml", "target/release/puce16-cli suite tests/manifest.toml --update-baseline <nom>",
             "python docs/tools/task.py approve E07-12"], accept=[])

T("E07-13", "Preparer les jeux: README des ROM commerciales et verification du manifeste",
  kind="doc", size=0,
  context="Les jeux (SMW, ALttP, All-Stars) sont fournis par l'utilisateur (D-006, jamais telecharges). Cette tache verifie seulement que le montage est pret pour les GATE J6/J7.",
  write=["tests/roms/README.md", "docs/qa/GAMES_CHECKLIST.md"],
  spec="""1. `docs/qa/GAMES_CHECKLIST.md`: pour chacun des trois jeux, un tableau `Etape | Attendu | Resultat (a remplir par l'utilisateur)`:
   - SMW: logo Nintendo, ecran titre, demarrage d'une partie, niveau 1 (Yoshi's Island 2) jouable, saut/course, pieces, ennemis, musique, effets sonores, carte du monde, sauvegarde (SRAM) apres un niveau.
   - ALttP: ecran titre, choix du fichier, intro (pluie), sortie de la maison, Hyrule (scrolling, mode 7 sur la carte), combat a l'epee, donjon d'entree, musique, sauvegarde.
   - Super Mario All-Stars: menu de selection, SMB1 monde 1-1, SMB3 monde 1-1, musique, sauvegarde.
   Ajouter une colonne `Bug visible (capture/description)`.
2. Dans `tests/roms/README.md`, section jeux: confirmer les trois noms de fichier et ajouter la ligne `Les jeux sont lances avec: target/release/puce16-desktop tests/roms/games/<nom>.sfc`.
3. Verifier que `tests/manifest.toml` contient bien `game_smw`, `game_alttp`, `game_allstars` (optional, check none).""",
  accept=["python -c \"import sys; t=open('docs/qa/GAMES_CHECKLIST.md',encoding='utf-8').read(); sys.exit(0 if 'SMW' in t and 'ALttP' in t and 'All-Stars' in t else 1)\""],
  notes=CHAIN)

T("E07-14", "GATE J6: Super Mario World demarre et se joue",
  kind="gate", gate=True, size=0,
  context="Premier jeu commercial. Necessite le desktop (E03-47 ouverture de ROM); le son n'est requis qu'apres E08. Les bugs trouves deviennent des taches `TUNE-nn` creees par Claude a partir de la checklist.",
  spec="""Criteres de validation (humains), voir `docs/qa/GAMES_CHECKLIST.md`:
1. `tests/roms/games/smw.sfc` demarre jusqu'a l'ecran titre sans plantage.
2. Une partie commence; le niveau 1 est jouable (marcher, sauter, ramasser, ennemis).
3. Aucun glitch graphique majeur (sprites manquants, fond decale, palette fausse) sur ces ecrans.
4. Si un bug: remplir la colonne `Bug visible`; Claude cree les taches de correction; ne pas approuver avant correction.""",
  user_cmds=["cargo run --release -p puce16-desktop -- tests/roms/games/smw.sfc", "python docs/tools/task.py approve E07-14"], accept=[])

T("E07-15", "GATE J7: ALttP et Super Mario All-Stars jouables",
  kind="gate", gate=True, size=0,
  context="Deux jeux plus exigeants (mode 7, HDMA, IRQ, DMA, SRAM). Duree minimale de jeu: environ 5 minutes chacun, sans plantage.",
  spec="""Criteres de validation (humains), voir `docs/qa/GAMES_CHECKLIST.md`:
1. ALttP: intro, sortie de la maison, exploration d'Hyrule et premier donjon d'entree sans plantage pendant ~5 minutes; la carte (mode 7) s'affiche.
2. All-Stars: menu, SMB1 1-1 et SMB3 1-1 jouables ~5 minutes chacun.
3. Chaque jeu se sauvegarde et se recharge (SRAM, requiert E08) -- peut etre reporte au GATE J8 si E08 n'est pas termine.
4. Les bugs deviennent des taches `TUNE-nn` creees par Claude.""",
  user_cmds=["cargo run --release -p puce16-desktop -- tests/roms/games/alttp.sfc", "cargo run --release -p puce16-desktop -- tests/roms/games/smb_allstars.sfc",
             "python docs/tools/task.py approve E07-15"], accept=[])
