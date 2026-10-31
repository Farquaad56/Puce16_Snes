from _dsl import T

# E10: triches Pro Action Replay (PAR). Hors v1 (comme E09). Spec: docs/specs/11_CHEATS.md.
# Le Game Genie est un placeholder (E10-08): aucune specification tant que l'utilisateur ne le demande pas.

CHAIN10 = ("Execution strictement sequentielle (D-020): cette tache suppose que toutes les precedentes sont terminees. Ne modifie QUE les fichiers de 'Write only in'.")

CORE10 = "crates/puce16-core"
CLI10 = "crates/puce16-cli"
DESK10 = "crates/puce16-desktop"
SPEC11 = "specs/11_CHEATS.md"

T("E10-01", "Triches: module cheat et analyse d'un code PAR (parse_par)",
  kind="code", size=90, crate="puce16-core",
  context="Premiere tache des triches Pro Action Replay. Elle cree le module `cheat` et la fonction qui transforme un texte `AAAAAAVV` en `ParCode`. Aucune autre logique. " + CHAIN10,
  embed=[SPEC11 + "#2. Format d'un code", SPEC11 + "#5. Interface Rust"],
  reads=[CORE10 + "/src/lib.rs"],
  write=[CORE10 + "/src/lib.rs", CORE10 + "/src/cheat/mod.rs", CORE10 + "/src/cheat/par.rs", CORE10 + "/tests/e10_01_*.rs", "docs/qa/inbox/** (questions uniquement)"],
  spec="""1. `lib.rs`: ajouter `pub mod cheat;` (apres `pub mod cart;` ou a la suite des autres `pub mod`).
2. `cheat/mod.rs`: `pub mod par;` et `pub use par::{parse_par, ParCode, ParseError};` (les taches suivantes ajoutent leurs propres lignes).
3. `cheat/par.rs`: `ParCode { pub addr: u32, pub value: u8 }` et `ParseError { Length(usize), NotHex(char), LooksLikeGameGenie }`, les deux avec `#[derive(Clone, Copy, Debug, PartialEq, Eq)]` (`ParCode` implemente aussi `Display`: `{:06X}{:02X}`).
4. `parse_par(s: &str) -> Result<ParCode, ParseError>`: `trim()` du texte; si le texte contient `-` -> `LooksLikeGameGenie` (verification en premier); sinon, si le nombre de caracteres (`chars().count()`) n'est pas 8 -> `Length(n)`; sinon chaque caractere doit etre un chiffre hexadecimal ASCII, le premier invalide donne `NotHex(c)`; `addr` = les 6 premiers chiffres, `value` = les 2 derniers.
5. Pas de `unsafe` (`puce16-core` est `#![forbid(unsafe_code)]`), pas de dependance nouvelle.""",
  tests=[("valid_uppercase", "parse_par(\"7E0DBF99\") = Ok(ParCode { addr: 0x7E0DBF, value: 0x99 })"),
         ("valid_lowercase_and_spaces", "parse_par(\"  7e0dbf99 \") = Ok(ParCode { addr: 0x7E0DBF, value: 0x99 }); parse_par(\"00000000\") = Ok(ParCode { addr: 0, value: 0 }); parse_par(\"FFFFFFFF\") = Ok(ParCode { addr: 0xFFFFFF, value: 0xFF })"),
         ("wrong_length", "parse_par(\"\") = Err(Length(0)); parse_par(\"7E0DBF9\") = Err(Length(7)); parse_par(\"7E0DBF990\") = Err(Length(9))"),
         ("not_hex", "parse_par(\"7E0DBG99\") = Err(NotHex('G')); parse_par(\"ZZ0DBF99\") = Err(NotHex('Z'))"),
         ("game_genie_shape_refused", "parse_par(\"DF4B-7D6E\") = Err(LooksLikeGameGenie); parse_par(\"7E0D-BF99\") = Err(LooksLikeGameGenie)"),
         ("display_is_canonical", "format!(\"{}\", parse_par(\"7e0dbf99\").unwrap()) = \"7E0DBF99\"; format!(\"{}\", ParCode { addr: 0x12, value: 0x3 }) = \"00001203\"")],
  accept=["cargo test -p puce16-core e10_01_"],
  notes=CHAIN10)

T("E10-02", "Triches: fichier .cht (lecture, ecriture, erreurs avec numero de ligne)",
  kind="code", size=120, crate="puce16-core",
  context="Format texte d'une liste de triches (une triche par ligne, `+` active, `-` inactive). Lecture avec erreurs precises et ecriture canonique. " + CHAIN10,
  embed=[SPEC11 + "#3. Fichier de triches (.cht)", SPEC11 + "#5. Interface Rust"],
  reads=[CORE10 + "/src/cheat/par.rs", CORE10 + "/src/cheat/mod.rs"],
  write=[CORE10 + "/src/cheat/mod.rs", CORE10 + "/src/cheat/list.rs", CORE10 + "/tests/e10_02_*.rs", "docs/qa/inbox/** (questions uniquement)"],
  spec="""1. `cheat/mod.rs`: ajouter `pub mod list;` et `pub use list::{format_cheat_file, parse_cheat_file, CheatEntry, CheatFileError, CheatFileErrorKind};`.
2. `cheat/list.rs`: `CheatEntry { pub name: String, pub enabled: bool, pub codes: Vec<ParCode> }`, `CheatFileErrorKind { MissingSign, MissingEquals, EmptyName, NoCode, BadCode(ParseError) }`, `CheatFileError { pub line: usize, pub kind: CheatFileErrorKind }` (tous `Clone, Debug, PartialEq, Eq`).
3. `parse_cheat_file(text: &str) -> Result<Vec<CheatEntry>, CheatFileError>`: parcourir `text.lines()` (`lines()` gere `\\r\\n`) avec le numero de ligne a partir de 1. Ignorer les lignes vides (apres `trim()`) et celles dont le premier caractere non blanc est `#`. Sinon: le premier caractere non blanc doit etre `+` ou `-` (sinon `MissingSign`); chercher le premier `=` dans le reste (sinon `MissingEquals`); nom = texte avant `=` trim (vide -> `EmptyName`); les codes = texte apres `=` decoupe sur les espaces et les virgules (segments vides ignores); aucun segment -> `NoCode`; chaque segment passe par `parse_par`, la premiere erreur donne `BadCode(e)`. Premiere erreur rencontree = resultat.
4. `format_cheat_file(entries: &[CheatEntry]) -> String`: premiere ligne `# Puce16 cheats`, puis pour chaque entree `<+ ou -> <nom> = <code> <code>...` (codes affiches avec `Display`, separes par un espace), chaque ligne terminee par `\\n`. Un nom qui contient `=` ou un retour a la ligne n'est pas valide: la fonction ne verifie rien (c'est le role de l'appelant, tache E10-06).
5. Pas de `unsafe`, pas de dependance nouvelle.""",
  tests=[("parse_two_entries", "texte \"# note\\n+ Vies = 7E0DBE05\\n\\n- Invincible = 7E1497FF, 7E1490FF\\n\" -> 2 entrees: (\"Vies\", enabled, [7E0DBE/05]) et (\"Invincible\", disabled, [7E1497/FF, 7E1490/FF])"),
         ("parse_crlf_and_blanks", "meme contenu avec \\r\\n et des lignes d'espaces: meme resultat; un fichier vide ou fait de commentaires -> Ok(vec![])"),
         ("error_missing_sign", "\"Vies = 7E0DBE05\" -> Err(line 1, MissingSign); \"# a\\n\\n= 7E0DBE05\" -> Err(line 3, MissingSign)"),
         ("error_missing_equals_name_code", "\"+ Vies 7E0DBE05\" -> MissingEquals; \"+  = 7E0DBE05\" -> EmptyName; \"+ Vies =\" -> NoCode; \"+ Vies = , \" -> NoCode"),
         ("error_bad_code_reports_line", "\"+ A = 7E0DBE05\\n+ B = 7E0DBE0\" -> Err(line 2, BadCode(Length(7))); \"+ B = 7E0DBE0G\" -> BadCode(NotHex('G'))"),
         ("format_exact_text", "format_cheat_file de [(\"Vies\", true, [0x7E0DBE/0x05]), (\"Inv\", false, [0x7E1497/0xFF, 0x7E1490/0xFF])] = \"# Puce16 cheats\\n+ Vies = 7E0DBE05\\n- Inv = 7E1497FF 7E1490FF\\n\""),
         ("roundtrip", "parse_cheat_file(&format_cheat_file(&v)) == Ok(v) pour 3 listes (vide, une entree, trois entrees avec plusieurs codes)")],
  accept=["cargo test -p puce16-core e10_02_"],
  notes=CHAIN10)

T("E10-03", "Triches: application sur la console (Snes::set_cheats, ecriture silencieuse a chaque image)",
  kind="code", size=110, crate="puce16-core",
  context="La console garde une liste de `ParCode` et les ecrit une fois par image, a la fin de `run_frame` (debut du VBlank). Ecriture routee comme dans la table de la spec: WRAM, SRAM par la cartouche, le reste ignore. Rien n'entre dans l'etat sauvegarde ni dans le hash. " + CHAIN10,
  embed=[SPEC11 + "#4. Application sur la console", SPEC11 + "#5. Interface Rust"],
  reads=[CORE10 + "/src/snes.rs (struct Snes, new_with, run_frame, reset, load_state, peek_wram)", CORE10 + "/src/bus/route.rs", CORE10 + "/src/bus/wram.rs", CORE10 + "/src/cart/mod.rs (write, sram)",
         CORE10 + "/tests/common/prog.rs (build_rom_with)", CORE10 + "/tests/e09_10_*.rs (exemple d'usage de Snes::new, run_frame, peek_wram)"],
  write=[CORE10 + "/src/snes.rs", CORE10 + "/src/bus/route.rs", CORE10 + "/tests/e10_03_*.rs", "docs/qa/inbox/** (questions uniquement)"],
  spec="""1. `route.rs`: ajouter a `impl SysBus` la methode `pub fn cheat_write(&mut self, addr: u32, v: u8)` qui decode `bank = (addr >> 16) as u8`, `a = (addr & 0xFFFF) as u16` selon la table de la spec section 4: banques `0x00..=0x3F` et `0x80..=0xBF` avec `a < 0x2000` -> `self.wram.write(a as u32, v)`; banque `0x7E` -> `self.wram.write(a as u32, v)`; banque `0x7F` -> `self.wram.write(0x10000 + a as u32, v)`; banques `0x00..=0x3F`/`0x80..=0xBF` avec `a >= 0x6000`, banques `0x40..=0x7D` et `0xC0..=0xFF` -> `let _ = self.cart.write(bank, a, v);`; tout le reste est ignore. Aucun effet sur `master`, `mdr` ni les registres.
2. `snes.rs`: nouveau champ `cheats: Vec<ParCode>` dans `Snes` (initialise vide dans les trois constructeurs: `new_test_pattern`, `new_with`, et tout autre endroit qui construit `Snes {..}`); `pub fn set_cheats(&mut self, codes: &[ParCode])` (remplace la liste) et `pub fn cheats(&self) -> &[ParCode]`. Le champ n'est PAS dans `save_state`, `load_state` ni `state_hash`; `reset()` et `load_state()` ne le vident pas.
3. `run_frame`, variante `Inner::Real`: apres la boucle (et apres `self.frame = sys.vblank_count();`), `for c in &self.cheats { sys.cheat_write(c.addr, c.value); }`. Attention a l'emprunt: la boucle `match &mut self.inner` emprunte `self.inner` et `self.cheats` est un autre champ, mais ecrire `let cheats = &self.cheats;` avant le `match`, ou restructurer legerement si le compilateur refuse. La variante `Pattern` n'applique rien.
4. Tests (integration, fichier `e10_03_*.rs`): ROM LoRom construite par `build_rom_with` avec le programme `E6 00 80 FC` en tete (`INC $00` puis `BRA -4`, vecteur reset `0x8000`), 2 images au minimum. Pour la SRAM: ROM avec SRAM de 8 Kio (voir `build_rom_with`: parametres SRAM et chipset; chipset `0x02` = ROM + RAM + batterie) et `Snes::sram()` pour relire.""",
  tests=[("wram_bank7e_written_each_frame", "programme INC $00 en boucle; set_cheats([7E0000/0x42]); apres chaque run_frame (3 images de suite) peek_wram(0) = 0x42 (la console incremente puis la triche reecrit)"),
         ("wram_low_mirror_and_bank7f", "set_cheats([000100/0x11, 7F0010/0x22, 7E0200/0x33]) puis 1 image: peek_wram(0x100) = 0x11, peek_wram(0x10010) = 0x22, peek_wram(0x200) = 0x33"),
         ("sram_written_through_cart", "ROM avec SRAM 8 Kio: set_cheats([700001/0x5A]) puis 1 image: sram().unwrap()[1] = 0x5A"),
         ("rom_and_io_ignored", "set_cheats([008000/0xEE, 002100/0x0F, 004200/0x81, 7E0300/0x01]): apres 1 image peek_wram(0x300) = 0x01 et le programme tourne toujours (la ROM n'est pas modifiee: relire par Snes ne plante pas, state_hash calcule sans panique)"),
         ("order_last_wins", "set_cheats([7E0400/0x01, 7E0400/0x02]) puis 1 image: peek_wram(0x400) = 0x02"),
         ("no_cheat_hash_unchanged", "deux Snes identiques: l'un fait set_cheats(&[]) avant 2 images, l'autre non: state_hash egaux; cheats() = [] ; save_state de chacun identique"),
         ("cheats_survive_reset_and_load_state", "set_cheats([7E0500/0x07]); reset(); cheats().len() = 1; save_state; load_state: cheats().len() = 1 et l'effet est applique a l'image suivante")],
  accept=["cargo test -p puce16-core e10_03_", "cargo test -p puce16-core e03_34_", "cargo test -p puce16-core e08_"],
  notes=CHAIN10)

T("E10-04", "Triches: options CLI --cheat et --cheat-code",
  kind="code", size=90, crate="puce16-cli",
  context="Le CLI peut charger des triches: `run <rom> --cheat fichier.cht --cheat-code 7E0DBF99`. Une fonction pure collecte les codes; la commande les passe a `Snes::set_cheats` avant la premiere image. " + CHAIN10,
  embed=[SPEC11 + "#6. Ligne de commande"],
  reads=[CLI10 + "/src/main.rs", CLI10 + "/src/run_cmd.rs", CLI10 + "/tests/e03_44_run_rom.rs (style des tests du binaire)", CORE10 + "/src/cheat/mod.rs"],
  write=[CLI10 + "/src/main.rs", CLI10 + "/src/cheats.rs", CLI10 + "/src/run_cmd.rs", CLI10 + "/tests/e10_04_*.rs", "docs/qa/inbox/** (questions uniquement)"],
  spec="""1. `cheats.rs` (module declare dans `main.rs`): `pub fn collect_cheats(file_text: Option<&str>, codes: &[String]) -> Result<Vec<ParCode>, String>`. Le texte du fichier passe par `parse_cheat_file`; seules les entrees `enabled` contribuent, dans l'ordre, tous leurs codes; puis chaque element de `codes` passe par `parse_par` dans l'ordre. Erreurs de fichier: `format!(\"fichier .cht ligne {}: {:?}\", e.line, e.kind)`; erreurs de code: `format!(\"code invalide '{}': {:?}\", texte, erreur)`.
2. Options de `run`: `--cheat <FICHIER>` (une fois) et `--cheat-code <CODE>` (repetable), dans la meme structure d'arguments que `--input`. Dans la commande: lire le fichier (erreur de lecture -> message), appeler `collect_cheats`; en cas d'erreur afficher `erreur cheat: <message>` sur stderr et sortir avec un code non nul sans lancer la console. Sinon `snes.set_cheats(&codes)` avant la premiere image.
3. Sans option, le comportement de `run` est exactement celui d'avant (aucun test existant ne change).
4. Les tests de `collect_cheats` sont dans `cheats.rs` (`#[cfg(test)] mod tests`, noms prefixes `e10_04_`); les tests du binaire sont dans `tests/e10_04_*.rs` (copier le style de `e03_44_run_rom.rs`: ROM synthetique ecrite dans un dossier temporaire, appel du binaire compile).""",
  tests=[("collect_only_enabled_in_order", "fichier \"+ A = 7E0001AA\\n- B = 7E0002BB\\n+ C = 7E0003CC 7E0004DD\" + codes [\"7E0005EE\"] -> [0x7E0001/AA, 0x7E0003/CC, 0x7E0004/DD, 0x7E0005/EE]"),
         ("collect_without_file", "None + [\"7E0010FF\"] -> [0x7E0010/FF]; None + [] -> []"),
         ("collect_bad_code_message", "[\"7E0010F\"] -> Err contenant \"code invalide '7E0010F'\" et \"Length(7)\""),
         ("collect_bad_file_message", "fichier \"oops\" -> Err contenant \"ligne 1\" et \"MissingSign\""),
         ("binary_applies_cheat_code", "ROM synthetique (programme E6 00 80 FC, vecteur reset 0x8000), `run <rom> --frames 3 --cheat-code 7E000042 --state-hash` comparee a la meme commande sans option: les sorties different; code de sortie 0"),
         ("binary_rejects_bad_code", "`run <rom> --frames 1 --cheat-code ZZ` -> code de sortie non nul et stderr contient \"erreur cheat\"")],
  accept=["cargo test -p puce16-cli e10_04_", "cargo test -p puce16-cli e03_44_", "cargo build --release -p puce16-cli"],
  notes=CHAIN10)

T("E10-05", "Triches: fichiers .cht du desktop et commande SetCheats de la console",
  kind="code", size=110, crate="puce16-desktop",
  context="Plomberie du desktop, sans interface: chemin du `.cht` d'une ROM, lecture/ecriture, commande vers la console, et conservation de la liste de codes par le thread console apres un `Reset` ou un rechargement. " + CHAIN10,
  embed=[SPEC11 + "#7. Desktop"],
  reads=[DESK10 + "/src/files.rs (srm_path, write_atomic)", DESK10 + "/src/emu_thread.rs (Command, emu_loop, reload_console, handle_load_state)", DESK10 + "/tests/e08_20_files.rs (style des tests de fichiers)", CORE10 + "/src/cheat/mod.rs"],
  write=[DESK10 + "/src/files.rs", DESK10 + "/src/emu_thread.rs", DESK10 + "/tests/e10_05_*.rs", "docs/qa/inbox/** (questions uniquement)"],
  spec="""1. `files.rs`: `pub fn cheat_path(rom: &Path) -> PathBuf` (meme dossier et meme nom que la ROM, extension remplacee par `cht`, comme `srm_path` le fait pour `srm`); `pub fn load_cheat_entries(path: &Path) -> Result<Vec<CheatEntry>, String>` (fichier absent -> `Ok(vec![])`; erreur de lecture ou de syntaxe -> `Err(message en francais avec le numero de ligne)`); `pub fn save_cheat_entries(path: &Path, entries: &[CheatEntry]) -> Result<(), String>` (utilise `format_cheat_file` et `write_atomic`).
2. `emu_thread.rs`: nouvelle variante `Command::SetCheats(Vec<ParCode>)`. Le thread console garde une variable `cheats: Vec<ParCode>`: a la reception, `cheats = codes; snes.set_cheats(&cheats)`. Partout ou la console est recreee ou remplacee (rechargement de ROM, `reload_console`, chargement d'etat si `Snes` est reconstruit), reappliquer `snes.set_cheats(&cheats)` sur la nouvelle console, SAUF au chargement d'une nouvelle ROM par `Command::LoadRom` ou la liste est videe (`cheats.clear()`) puis reappliquee vide (la fenetre de triches renverra les codes du nouveau `.cht`).
3. Pas d'interface ici (tache suivante). Si `Command` est utilise dans un `match` exhaustif ailleurs, ajouter la branche manquante.""",
  tests=[("cheat_path_replaces_extension", "cheat_path(\"C:/jeux/smw.sfc\") se termine par smw.cht dans le meme dossier; cheat_path(\"/x/y.z.smc\") donne /x/y.z.cht; un nom sans extension recoit .cht"),
         ("load_missing_file_is_empty", "load_cheat_entries sur un chemin inexistant = Ok(vec![])"),
         ("save_then_load_roundtrip", "dossier temporaire unique (std::env::temp_dir + nom unique): save_cheat_entries de 2 entrees puis load_cheat_entries = les memes entrees; le fichier commence par \"# Puce16 cheats\""),
         ("load_bad_file_reports_line", "fichier contenant \"+ A = 7E0DBE05\\nfaux\" -> Err dont le message contient \"2\""),
         ("emu_thread_applies_cheats_after_reset", "demarrer le thread console avec une ROM synthetique (programme E6 00 80 FC, voir e03_46_emu_load.rs pour le style), envoyer SetCheats([7E000042]), Reset, attendre une image, lire la WRAM par les donnees de debug (ou une image publiee): la valeur 0x42 est presente; si lire la WRAM par le thread n'est pas possible, tester la fonction pure extraite qui rejoue la liste apres reload")],
  accept=["cargo test -p puce16-desktop e10_05_", "cargo test -p puce16-desktop e08_", "cargo build -p puce16-desktop"],
  notes=CHAIN10)

T("E10-06", "Triches: modele CheatPanel, menu Triches a cases a cocher et fenetre d'ajout",
  kind="code", size=140, crate="puce16-desktop",
  context="Interface desktop des triches, pensee pour aller vite: le menu `Triches` contient directement une case a cocher par triche (activer/desactiver en un clic, le menu reste ouvert), plus `Ajouter une triche...` (petite fenetre, un champ de codes que l'on colle, nom facultatif). Un modele pur et teste (`CheatPanel`) porte toute la logique. " + CHAIN10,
  embed=[SPEC11 + "#7. Desktop"],
  reads=[DESK10 + "/src/app.rs (menu_bar, open_path, struct App, comment le debogueur est ouvert)", DESK10 + "/src/debugger.rs (modele d'une fenetre egui)", DESK10 + "/src/files.rs", CORE10 + "/src/cheat/mod.rs", DESK10 + "/src/lib.rs"],
  write=[DESK10 + "/src/lib.rs", DESK10 + "/src/cheat_panel.rs", DESK10 + "/src/app.rs", DESK10 + "/tests/e10_06_*.rs", "docs/qa/inbox/** (questions uniquement)"],
  spec="""1. `cheat_panel.rs` (module declare dans `lib.rs`): `pub struct CheatPanel { entries: Vec<CheatEntry>, pub add_open: bool, pub codes_input: String, pub name_input: String }` avec: `new(entries: Vec<CheatEntry>) -> CheatPanel`; `entries(&self) -> &[CheatEntry]`; `toggle(&mut self, i: usize)`; `remove(&mut self, i: usize)` et `disable_all(&mut self)`; `active_codes(&self) -> Vec<ParCode>` (entrees actives, dans l'ordre, tous leurs codes). Indices hors limites: aucun effet, pas de panique.
2. Validation en direct: `pub fn check_codes(text: &str) -> Result<Vec<ParCode>, String>`: le texte est decoupe sur espaces, virgules ET retours a la ligne (segments vides ignores); aucun code -> `Err(\"saisir au moins un code\")`; chaque segment passe par `parse_par`; la premiere erreur donne `Err(format!(\"code '{}': {}\", segment, raison))` ou `raison` est en francais (`Length(n)` -> `\"{n} chiffres au lieu de 8\"`, `NotHex(c)` -> `\"caractere '{c}' non hexadecimal\"`, `LooksLikeGameGenie` -> `\"code Game Genie non gere (Pro Action Replay brut seulement)\"`).
3. `add(&mut self, name: &str, codes_text: &str) -> Result<(), String>`: utilise `check_codes`; le nom est trim; vide -> le texte du premier code (en majuscules) sert de nom; un nom contenant `=` ou un retour a la ligne est refuse (`Err(\"nom invalide\")`). Succes: entree ajoutee ACTIVE a la fin.
4. Dessin (non teste, seulement compile), dans le meme fichier: `pub fn menu_ui(&mut self, ui: &mut egui::Ui, rom_loaded: bool) -> bool` qui dessine le contenu du menu `Triches` (renvoie `true` si la liste a change): grise si `!rom_loaded`; si aucune triche, une ligne grisee `Aucune triche pour ce jeu`; sinon une `ui.checkbox` par triche (texte = nom, `on_hover_text` = codes), et le menu NE SE FERME PAS au clic sur une case (ne pas appeler `ui.close_menu()`); puis un separateur; `Ajouter une triche...` (met `add_open = true` et ferme le menu); sous-menu `Supprimer` (un bouton par triche); `Tout desactiver`. Et `pub fn add_window(&mut self, ctx: &egui::Context) -> bool`: petite fenetre `Ajouter une triche` si `add_open`: champ multiligne `Codes` (focus automatique a l'ouverture), champ `Nom (facultatif)`, message d'erreur rouge en direct sous les codes (via `check_codes`, rien d'affiche si le champ est vide), bouton `Ajouter` grise tant que `check_codes` est en erreur, Entree valide (Maj+Entree = retour a la ligne dans le champ), Echap ferme. Succes: `add`, champs vides, `add_open = false`, renvoie `true`.
5. `app.rs`: champ `cheats: CheatPanel` dans `App`; dans `open_path`, apres une lecture de ROM reussie: `cheat_path`, `load_cheat_entries` (en cas d'erreur: message dans la barre d'etat et liste vide), `self.cheats = CheatPanel::new(entries)`, puis `Command::SetCheats(self.cheats.active_codes())`. Dans `menu_bar`: `ui.menu_button(\"Triches\", |ui| { changed |= self.cheats.menu_ui(ui, rom_chargee); })`. Dans la mise a jour de l'interface: `changed |= self.cheats.add_window(ctx);` puis si `changed`: envoyer `Command::SetCheats(active_codes())` et `save_cheat_entries(cheat_path(rom_path), entries)` en signalant une erreur d'ecriture dans la barre d'etat.""",
  tests=[("check_codes_valid_separators", "check_codes(\"7E0001AA, 7E0002BB 7E0003CC\\n7E0004DD\") = Ok de 4 codes dans l'ordre"),
         ("check_codes_errors_in_french", "check_codes(\"\") et check_codes(\"  \\n \") = Err contenant \"au moins un code\"; check_codes(\"7E0001A\") = Err contenant \"7E0001A\" et \"7 chiffres au lieu de 8\"; check_codes(\"7E0001GA\") = Err contenant \"'G'\"; check_codes(\"DF4B-7D6E\") = Err contenant \"Game Genie\""),
         ("add_valid_entry_is_active", "add(\"Vies\", \"7E0DBE05\") = Ok; entries().len() = 1; entree enabled = true avec 1 code 7E0DBE/05"),
         ("add_empty_name_uses_first_code", "add(\"\", \"7e0dbe05 7E0001AA\") = Ok; nom = \"7E0DBE05\"; 2 codes"),
         ("add_rejects_bad_name_and_code", "add(\"a=b\", \"7E0001AA\") = Err; add(\"A\", \"\") = Err; add(\"A\", \"7E0001A\") = Err; entries() reste vide"),
         ("toggle_and_active_codes", "3 entrees dont la 2e desactivee par toggle(1): active_codes() ne contient que les codes des entrees 1 et 3 dans l'ordre; toggle(1) de nouveau les active toutes"),
         ("disable_all_and_remove", "disable_all(): active_codes() = [] et les entrees restent; remove(0) retire la premiere; remove(99) et toggle(99) ne changent rien et ne paniquent pas"),
         ("new_keeps_loaded_entries", "CheatPanel::new(parse_cheat_file(\"+ A = 7E0001AA\\n- B = 7E0002BB\\n\").unwrap()): entries().len() = 2, active_codes() = [7E0001/AA]")],
  accept=["cargo test -p puce16-desktop e10_06_", "cargo test -p puce16-desktop e08_", "cargo build --release -p puce16-desktop"],
  notes=CHAIN10)

T("E10-07", "GATE Triches: Pro Action Replay valide par l'utilisateur avec de vrais codes",
  kind="gate", gate=True, size=0,
  context="Fin des triches PAR. Les tests n'utilisent que des valeurs arbitraires: seuls de vrais codes de gamehacking.org sur un vrai jeu prouvent que l'ensemble fonctionne. Aucun code a ecrire.",
  spec="""Criteres de validation (humains):
1. `cargo test --workspace` est vert.
2. Prendre sur gamehacking.org (Super Nintendo) un code PAR decode `AAAAAAVV` pour un jeu que tu possedes (par exemple des vies infinies), au format a 8 chiffres sans tiret.
3. Desktop: ouvrir le jeu, menu Triches > Ajouter une triche..., coller le code (le nom est facultatif) et valider avec Entree; la triche apparait cochee dans le menu. L'effet attendu apparait en jeu (par exemple le compteur de vies ne baisse plus). Decocher la case dans le menu: l'effet disparait. Recocher: il revient (le menu reste ouvert apres un clic).
4. Fermer le desktop puis le relancer avec le meme jeu: la triche est toujours dans la liste (fichier `.cht` a cote de la ROM) avec le meme etat coche/decoche.
5. Un code invalide (par exemple 7 chiffres, ou un code Game Genie avec tiret) affiche une erreur dans la fenetre et n'ajoute rien.
6. Reset (touche du desktop) puis charger un etat sauvegarde: la triche reste active.
7. CLI: la commande `run` avec `--cheat-code` donne une image differente de la meme commande sans option.
Si l'effet est absent: noter le jeu, le code exact et ce qui est observe; Claude verifie d'abord que l'adresse est bien en WRAM ($7E/$7F ou miroir bas) avant tout soupcon sur le code.""",
  user_cmds=["cargo test --workspace",
             "cargo run --release -p puce16-desktop",
             "cargo run --release -p puce16-cli -- run tests/roms/games/smw.sfc --frames 600 --cheat-code 7E0DBE05 --dump-ppm tests/out/cheat_smw.ppm",
             "python docs/tools/task.py approve E10-07"],
  accept=[])

T("E10-08", "Game Genie (placeholder, hors perimetre)",
  kind="doc", deferred=True, size=0,
  context="HORS perimetre de l'epic E10 (decision de l'utilisateur: Pro Action Replay d'abord). Placeholder: aucune specification n'existe. Ne pas assigner a Hermes. Quand l'utilisateur lance le Game Genie, Claude ecrit d'abord la specification (table de decodage `XXXX-XXXX` -> adresse + valeur, verifiee sur des codes reels de gamehacking.org avec leur forme decodee), puis decoupe la tache.",
  spec="Aucune action pour l'instant. Plus tard: (1) specification du decodage, (2) decodeur dans `crates/puce16-core/src/cheat/gg.rs`, (3) filtre de lecture dans `Cart::read_bus` qui remplace l'octet lu a l'adresse ciblee (le Game Genie intercepte les lectures de ROM, il n'ecrit pas), (4) integration au fichier .cht et a l'interface.",
  write=[], tests=[], accept=[])
