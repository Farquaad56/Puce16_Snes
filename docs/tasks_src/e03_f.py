from _dsl import T

CORE = "crates/puce16-core"
CLI = "crates/puce16-cli"
DESK = "crates/puce16-desktop"

CHAIN = ("Execution strictement sequentielle (D-020): cette tache suppose que toutes les precedentes sont terminees. "
         "Ne modifie QUE les fichiers de 'Write only in'.")

TINY = ("ROM minimale pour les tests (LoROM, 32 Kio, boucle infinie): `fn tiny_rom() -> Vec<u8> { let mut r = vec![0u8; 0x8000]; r[0] = 0x80; r[1] = 0xFE; r[0x7FFC] = 0x00; r[0x7FFD] = 0x80; r }` "
        "(BRA -2 a 0x8000, vecteur RESET 0x8000; l'unique candidat d'en-tete est LoROM). Elle reste en mode emulation.")

# ------------------------------------------------------------------ E03 part F: CLI with real ROM, desktop ROM opening, GATE J2 (E03-44..48)

T("E03-44", "CLI: puce16-cli run <rom> (vraie ROM, --state-hash, --mapper, --debug)",
  size=140, deps=["E03-43"],
  context="""La commande `run` accepte maintenant un chemin de ROM (jalon J2). Elle sert aussi de sonde pour l'utilisateur: elle imprime un hash d'etat et un `DebugSnapshot`.
Les anciens comportements (`--test-pattern`, `--hash`, `--dump-ppm`) restent valides (tests E01-07). """ + CHAIN,
  embed=["06_TEST_ROMS.md#6. Commande", "01_ARCHITECTURE.md#6. CLI"],
  write=[CLI + "/src/main.rs", CLI + "/src/run_cmd.rs", CLI + "/tests/e03_44_run_rom.rs"],
  spec="""1. `run_cmd.rs` (nouveau module, declare par `mod run_cmd;` dans `main.rs`), fonctions pures testables par des tests unitaires `#[cfg(test)]` DANS ce fichier (noms `e03_44_...`):
```rust
pub fn describe_load_error(e: &puce16_core::snes::LoadError) -> String;   // TooSmall -> "ROM too small" ; NoHeader -> "no valid ROM header found" ; UnsupportedCoprocessor(n) -> "unsupported coprocessor: <n>"
pub fn format_debug(d: &puce16_core::snes::DebugSnapshot) -> String;
```
`format_debug` produit exactement: `debug: pc=<pb:02X>:<pc:04X> a=<:04X> x=<:04X> y=<:04X> s=<:04X> d=<:04X> db=<:02X> p=<:02X> e=<0|1> v=<decimal> dot=<decimal> master=<decimal> frame=<decimal>`.
2. `main.rs`, sous-commande `run`: argument positionnel OPTIONNEL `[ROM]` ; options en plus de l'existant: `--state-hash` (imprime `state_hash=0x<16 hex minuscules>` = `Snes::state_hash()`), `--mapper lorom|hirom|exhirom` (force le mapper), `--debug` (imprime `format_debug(&snes.debug())` sur stdout, apres les autres sorties). Reste: `--frames N` (defaut 1), `--hash`, `--region ntsc|pal` (force la region; sinon deduite de l'en-tete pour une ROM, ntsc pour le motif), `--dump-ppm FICHIER`, `--test-pattern`.
3. Logique: ni `ROM` ni `--test-pattern` -> erreur d'usage (stderr `rom path or --test-pattern required`, code 2). `ROM` fourni: `std::fs::read` (erreur -> stderr `cannot read <chemin>: <erreur>`, code 2) ; `Snes::new_with(&bytes, region, mapper)` (erreur -> stderr `cannot load ROM: <describe_load_error>`, code 2). Executer `--frames` fois `run_frame()` dans `std::panic::catch_unwind(AssertUnwindSafe(..))` ; si panique: stderr `emulator panic` + `format_debug(...)` et code 1. Ordre des sorties stdout: `frame_hash=...` (si --hash), `state_hash=...` (si --state-hash), ligne `debug: ...` (si --debug).
4. Codes de sortie: 0 OK, 1 panique/echec d'emulation, 2 erreur d'usage ou de fichier.
""" + TINY,
  tests=[("format_debug", "test unitaire dans `run_cmd.rs`: `DebugSnapshot{pc:0x8000,pb:0,a:0x1234,x:0xABCD,y:0,s:0x01FF,d:0,db:0x7E,p:0x34,e:true,v:225,dot:3,master:306905,frame:1}` -> `\"debug: pc=00:8000 a=1234 x=ABCD y=0000 s=01FF d=0000 db=7E p=34 e=1 v=225 dot=3 master=306905 frame=1\"`. Avec pb=0x12, pc=0x0ABC, e=false -> `pc=12:0ABC` et `e=0`."),
         ("describe_errors", "test unitaire: `describe_load_error(&LoadError::Cart(CartError::TooSmall))==\"ROM too small\"` ; NoHeader -> `\"no valid ROM header found\"` ; `UnsupportedCoprocessor(\"GSU\")` -> `\"unsupported coprocessor: GSU\"`."),
         ("run_rom_state_hash", "ecris `tiny_rom()` dans un fichier temporaire; `run <f> --frames 3 --state-hash`: code 0 et stdout contient une ligne `state_hash=0x` suivie de 16 chiffres hexa minuscules."),
         ("run_rom_deterministic", "deux executions identiques -> sorties identiques ; `--frames 3` et `--frames 4` donnent des `state_hash` differents."),
         ("run_rom_hash_line", "`--hash` imprime `frame_hash=0x` + 16 hexa (le cadre d'un Snes reel est noir pour l'instant)."),
         ("run_rom_debug", "`run <f> --frames 1 --debug`: stdout contient `debug: pc=00:8000 ` et ` e=1 ` et ` v=225 ` et ` frame=1`."),
         ("run_rom_region", "`--region pal` et `--region ntsc` sur la meme ROM (--frames 3 --state-hash) donnent des hash differents ; `--region xyz` -> code 2 (erreur clap)."),
         ("run_missing_file", "`run /nonexistent/x.sfc` -> code 2 et stderr contient `cannot read`."),
         ("run_too_small", "fichier de 100 octets -> code 2 et stderr contient `too small`."),
         ("run_bad_mapper", "`--mapper hirom` sur `tiny_rom()` (32 Kio, en-tete HiRom absent) -> code 2 et stderr contient `no valid ROM header`."),
         ("run_dump_ppm", "`--dump-ppm <tmp>` avec une ROM: fichier de taille 15 + 256*224*3 octets commencant par `P6\\n256 224\\n255\\n`."),
         ("legacy_pattern_still_works", "`run --test-pattern --frames 5 --hash` code 0 ; `run` sans argument -> code 2 et stderr contient `rom path or --test-pattern required`.")],
  accept=["cargo test -p puce16-cli e03_44_", "cargo test -p puce16-cli e01_07_", "cargo test -p puce16-cli e02_11_"], crate="puce16-cli")

T("E03-45", "Desktop: files.rs (extensions ROM, extraction d'un .zip, lecture d'un fichier ROM)",
  size=110,
  context="Fonctions pures de chargement de fichier ROM pour le desktop (spec 08 section 6): `.sfc/.smc/.swc/.fig` directs ou premiere entree ROM d'un `.zip`. " + CHAIN,
  embed=["08_INPUT_DESKTOP.md#6. Fichiers"],
  write=[DESK + "/src/files.rs", DESK + "/tests/e03_45_files.rs"],
  spec="""Dans `files.rs` (module deja declare par `lib.rs`; les tests importent `puce16_desktop::files::*`):
```rust
pub fn is_rom_name(name: &str) -> bool;                                  // extension (insensible a la casse) dans {sfc, smc, swc, fig}
pub fn rom_from_zip(zip_bytes: &[u8]) -> Result<(String, Vec<u8>), String>;   // (nom de l'entree, octets) de la PREMIERE entree dont `is_rom_name`; Err("no ROM file in zip") sinon; Err("invalid zip: <err>") si l'archive est illisible
pub fn load_rom_file(path: &std::path::Path) -> Result<(String, Vec<u8>), String>;
```
`load_rom_file`: extension `zip` (insensible a la casse): lire le fichier puis `rom_from_zip` ; extension ROM: `(nom du fichier, octets)` ; autre extension (ou aucune): `Err(\"unsupported file type: <ext>\")` (ext = chaine vide si absente) ; erreur de lecture: `Err(\"cannot read <chemin>: <erreur>\")`.
Crate `zip`: utilise l'API de la VERSION resolue dans `Cargo.lock` (lis sa doc locale: `cargo doc -p zip --no-deps` ou le code source dans `~/.cargo/registry`); ne devine pas l'API.""",
  tests=[("is_rom_name", "`is_rom_name`: `\"a.sfc\"`, `\"a.SMC\"`, `\"x.swc\"`, `\"x.Fig\"`, `\"dir/game.sfc\"` vrais ; `\"a.txt\"`, `\"a.zip\"`, `\"sfc\"`, `\"a.sfc.bak\"`, `\"\"` faux."),
         ("zip_first_rom", "construis en memoire un zip (crate `zip`, `ZipWriter` sur `Cursor<Vec<u8>>`, methode Stored) avec les entrees `readme.txt` (b\"hi\") puis `Game.SMC` (octets [1,2,3]) puis `other.sfc` ([9]): `rom_from_zip(&bytes)==Ok((\"Game.SMC\".to_string(), vec![1,2,3]))`."),
         ("zip_no_rom", "zip avec seulement `a.txt`: `Err` dont le message contient `no ROM`."),
         ("zip_invalid", "`rom_from_zip(&[1,2,3])`: `Err` dont le message commence par `invalid zip`."),
         ("load_plain", "dans un dossier temporaire (`std::env::temp_dir().join(format!(\"puce16_e03_45_{}\", std::process::id()))`, cree puis supprime a la fin): `game.sfc` contenant [1,2,3] -> `Ok((\"game.sfc\", [1,2,3]))`."),
         ("load_zip_file", "ecris le zip du test `zip_first_rom` dans `pack.zip`: `load_rom_file` renvoie `Ok((\"Game.SMC\", [1,2,3]))`."),
         ("load_unsupported", "`notes.txt` -> `Err` contenant `unsupported file type: txt` ; fichier sans extension -> `unsupported file type: ` (ext vide)."),
         ("load_missing", "chemin inexistant `x.sfc` -> `Err` contenant `cannot read`.")],
  accept=["cargo test -p puce16-desktop e03_45_"], crate="puce16-desktop")

T("E03-46", "Desktop: emu_thread LoadRom, Reset reel, Event::Error",
  size=140,
  context="""Le thread d'emulation sait maintenant remplacer son `Snes` par un `Snes` reel (commande `LoadRom`). Une erreur de chargement ou une panique d'emulation est rapportee par `Event::Error` sans tuer le thread.
Comme un variant est ajoute a `Event`, `app.rs` doit rester compilable (traite `Event::Error`). """ + CHAIN,
  embed=["08_INPUT_DESKTOP.md#7. Architecture desktop"],
  write=[DESK + "/src/emu_thread.rs", DESK + "/src/app.rs", DESK + "/tests/e03_46_emu_load.rs"],
  spec="""Modifie `emu_thread.rs` (en gardant le comportement de E01-09):
- `Command` gagne `LoadRom(Vec<u8>, String)` (octets, nom affiche). `Event` gagne `Error(String)`.
- `LoadRom(bytes, name)`: `Snes::new(&bytes, None)`. `Ok(s)`: remplace le `Snes` ; re-applique les derniers `SetJoypad` recus (le thread garde `joy: [u16;2]`) ; `frame_no` repart de 0 ; `paused = false` ; envoie `Event::Status(format!(\"Loaded {} ({:?}, {:?})\", name, info.map, info.region))` avec `info = snes.cart_info().unwrap()`. `Err(e)`: envoie `Event::Error(format!(\"cannot load {}: {}\", name, <message>))` ou `<message>` = "ROM too small" / "no valid ROM header found" / "unsupported coprocessor: <n>" ; l'ancien `Snes` est conserve et l'etat de pause inchange.
- `Reset` appelle desormais `snes.reset()` (motif ou reel) puis remet `frame_no` a 0.
- Chaque `run_frame()` est protege par `std::panic::catch_unwind(AssertUnwindSafe(..))` : en cas de panique, envoyer `Event::Error(\"emulator panic: <message>\")`, passer en pause, et ne plus appeler `run_frame` tant que l'utilisateur n'a pas fait `Reset` ou `LoadRom`.
Dans `app.rs`: ajouter le bras `Event::Error(e) => self.status = format!(\"ERROR: {}\", e)` (match exhaustif). Aucune autre modification de l'UI dans cette tache.
""" + TINY + " `Event` n'implemente pas `PartialEq` : utilise `matches!`.",
  tests=[("load_ok_status", "`spawn_test_pattern(Ntsc, true)` ; envoie `LoadRom(tiny_rom(), \"tiny.sfc\".into())` : on recoit (timeout 2 s) un `Event::Status` contenant `Loaded tiny.sfc` et `LoRom`."),
         ("load_ok_black_frame", "suite: `FrameAdvance` -> `Event::Frame` avec `frame_no==1`, `width==256`, `height==224` et le pixel (0,0) == 0xFF000000 (le motif de test avait un pixel colore)."),
         ("load_resumes_running", "`spawn_test_pattern(Ntsc, true)` (en pause), `LoadRom(tiny_rom(), ..)` : au moins 2 `Event::Frame` sont recus sans envoyer `FrameAdvance` (le chargement leve la pause)."),
         ("load_error_keeps_old", "`LoadRom(vec![0u8;10], \"bad.sfc\".into())` : `Event::Error` contenant `cannot load bad.sfc` et `ROM too small` ; puis `FrameAdvance` produit toujours une frame (le motif de test d'avant continue)."),
         ("reset_real", "apres `LoadRom` valide et deux `FrameAdvance` (frame_no 1 puis 2) : `Reset` puis `FrameAdvance` donne `frame_no==1`."),
         ("joypad_survives_load", "`SetJoypad(0, 0x8000)` AVANT `LoadRom`: le chargement ne plante pas (pas de verification visuelle: la ROM minimale ne lit pas la manette)."),
         ("quit_after_load", "`quit()` apres `LoadRom` retourne en < 1 s sans panique.")],
  accept=["cargo test -p puce16-desktop e03_46_", "cargo test -p puce16-desktop e01_09_", "cargo build -p puce16-desktop"], crate="puce16-desktop")

T("E03-47", "Desktop: ouvrir une ROM (menu, Ctrl+O, glisser-deposer, argument de ligne de commande)",
  size=130, kind="ui",
  context="""Dernier maillon de l'ouverture de ROM (jalon D0 complet). Code UI: pas de test automatique, sauf la petite fonction pure `rom_arg`.
Cette tache utilise `files::load_rom_file` (E03-45) et `Command::LoadRom` (E03-46). """ + CHAIN,
  embed=["08_INPUT_DESKTOP.md#6. Fichiers", "08_INPUT_DESKTOP.md#10. Jalons desktop (lies a D-015)"],
  write=[DESK + "/src/app.rs", DESK + "/src/main.rs", DESK + "/tests/e03_47_args.rs"],
  spec="""1. `app.rs`: `pub fn rom_arg(args: &[String]) -> Option<&str>` : premier element de `args[1..]` qui ne commence pas par `-`.
2. `App` gagne `pub fn open_path(&mut self, path: &std::path::Path)`: `files::load_rom_file(path)` ; `Ok((name, bytes))` -> envoie `Command::LoadRom(bytes, name)` ; `Err(e)` -> `self.status = format!(\"ERROR: {}\", e)`.
3. Menu Fichier: entree "Ouvrir une ROM..." (raccourci affiche `Ctrl+O`) : `rfd::FileDialog::new().add_filter("ROM SNES", &["sfc","smc","swc","fig","zip"]).pick_file()` puis `open_path`. Le raccourci `Hotkey::OpenRom` (E01-13) declenche la meme action.
4. Glisser-deposer: pour chaque fichier de `ctx.input(|i| i.raw.dropped_files.clone())` avec un `path`, appeler `open_path` sur le premier seulement.
5. `main.rs`: `std::env::args()`; si `rom_arg` donne un chemin: `app.open_path(..)` juste apres la creation de `App` (avant la boucle UI).
6. Affichage: la barre d'etat montre le dernier `Event::Status` (titre, mapper, region) ; le titre de fenetre n'est pas modifie ici.
Ne PAS ajouter SRAM/save states (E08).""",
  tests=[("rom_arg", "`rom_arg(&[\"puce16-desktop\".into()])==None` ; avec `[\"puce16-desktop\",\"game.sfc\"]` -> `Some(\"game.sfc\")` ; avec `[\"x\",\"--foo\",\"a.smc\"]` -> `Some(\"a.smc\")` ; avec `[\"x\",\"-v\"]` -> `None`."),
         ("rom_arg_first_wins", "`[\"x\",\"a.sfc\",\"b.sfc\"]` -> `Some(\"a.sfc\")`.")],
  accept=["cargo test -p puce16-desktop e03_47_", "cargo build -p puce16-desktop"], crate="puce16-desktop",
  notes="Verification visuelle faite par l'utilisateur au GATE E03-48.")

T("E03-48", "GATE J2: le systeme assemble demarre une ROM (CLI + desktop)",
  size=0, gate=True, kind="gate",
  context="""Fin de l'epopee E03. Le CPU, le bus, le DMA/HDMA, les manettes, les interruptions et le chargement de ROM sont assembles. Il n'y a pas encore de rendu (E04/E05) ni de son (E06): l'image reste NOIRE.
Un jeu commercial va probablement boucler en attendant l'APU: c'est NORMAL a ce stade. Le but est de verifier l'absence de plantage et la stabilite. Claude et l'utilisateur relisent les points UNKNOWN restants.""",
  spec="""Criteres de validation (humains):
1. `cargo test --workspace` est vert (aucun test ignore).
2. Avec une vraie ROM de l'utilisateur (ex: Super Mario World): `cargo run -p puce16-cli -- run <rom> --frames 300 --state-hash --debug` se termine avec le code 0 (pas de panique), imprime `state_hash=0x...` et une ligne `debug: ...` (pc dans la ROM). Deux executions donnent le MEME `state_hash`.
3. Le meme avec `--region pal` ne plante pas non plus.
4. Le desktop: `cargo run -p puce16-desktop -- <rom>` ouvre une fenetre noire sans plantage, la barre d'etat affiche `Loaded <nom> (<mapper>, <region>)` ; Fichier > Ouvrir et le glisser-deposer fonctionnent ; Pause, Image suivante, Reset fonctionnent ; un `.zip` contenant une ROM s'ouvre.
5. Claude relit `docs/qa/QA_LOG.md` pour les questions `UNKNOWN` posees en E03 (positions NMI/IRQ/auto-read/HDMA, refresh DRAM, delais math) et decide lesquelles ajuster avant E04.""",
  user_cmds=["cargo test --workspace", "cargo run -p puce16-cli -- run <ROM> --frames 300 --state-hash --debug", "cargo run -p puce16-cli -- run <ROM> --frames 300 --state-hash --region pal", "cargo run -p puce16-desktop -- <ROM>"],
  accept=[])
