from pathlib import Path
from _dsl import T

CORE = "crates/puce16-core"
CLI = "crates/puce16-cli"
SPC = CORE + "/src/apu/spc700"
TST = CORE + "/tests"
_TSV = Path(__file__).resolve().parents[1] / "specs" / "data" / "opcodes_spc700.tsv"
_ROWS = [l.split("\t") for l in _TSV.read_text(encoding="utf-8").splitlines()[1:]]
ORACLE = "tests/oracle/spc700"


def sops(*words):
    """opcodes (2 hex digits) whose mnemonic FIRST WORD is in words (e.g. 'MOV' but not 'MOV1' / 'MOVW')"""
    want = {w.upper() for w in words}
    return [r[0].upper() for r in _ROWS if r[1].split(" ")[0] in want]


def stsv(*words):
    return [("specs/data/opcodes_spc700.tsv", sops(*words))]


def spc_cmd(*words):
    return "cargo run -q -p puce16-cli -- spc-test %s --filter %s" % (ORACLE, ",".join(sops(*words)))


CHAIN = ("Execution strictement sequentielle (D-020): cette tache suppose que toutes les precedentes sont terminees. "
         "Ne modifie QUE les fichiers de 'Write only in'.")
READS = ["specs/07_SPC700.md (semantique des instructions)", "specs/07c_SPC_DESIGN.md (contrat, recettes de cycles, tests)"]
EMB_RUN = "07c_SPC_DESIGN.md#5. Tests unitaires"
RUN = ("Aide de test (a recopier): `fn run(setup: impl FnOnce(&mut Spc700, &mut FlatSmpBus), prog: &[u8]) -> (Spc700, FlatSmpBus, u32) "
       "{ let mut cpu = Spc700::new(); let mut bus = FlatSmpBus::new(); cpu.pc = 0x0200; bus.poke(0x0200, prog); setup(&mut cpu, &mut bus); let cyc = cpu.step(&mut bus); (cpu, bus, cyc) }`. "
       "Imports: `puce16_core::apu::spc700::{Spc700, SmpBus, testbus::FlatSmpBus, F_C, F_Z, F_I, F_H, F_B, F_P, F_V, F_N}`. "
       "`psw` initial = 0x02 sauf indication. Registres non cites = 0.")

# ------------------------------------------------------------------ E06 part A: oracle SPC700, coeur, MOV (E06-01..14)

T("E06-01", "T-ORACLE: telecharger l'oracle SingleStepTests spc700",
  kind="tooling", size=0,
  context="""Les tests d'instructions SPC700 (un test = une instruction, etat avant/apres, cycles) servent d'oracle a tout le coeur SMP.
Les fichiers sont ignores par git (dossier tests/oracle/). Voir specs/06_TEST_ROMS.md section 3.""",
  embed=["06_TEST_ROMS.md#3. Oracle SingleStepTests"],
  write=["tests/oracle/**", "docs/specs/data/oracle_sources.md"],
  spec="""1. Depuis la racine du depot: `git clone --depth 1 https://github.com/SingleStepTests/spc700 tests/oracle/spc700` (URL a confirmer: `UNKNOWN - to confirm`; si le depot a un autre nom/organisation, le retrouver par recherche web, ne PAS inventer).
2. Si le reseau est bloque: question `tooling` (docs/02_QA_PROTOCOL.md), ne rien contourner.
3. Ne pas committer le contenu (`tests/oracle/` est dans .gitignore; verifier avec `git status`).
4. AJOUTER (sans effacer ce qui existe deja pour le 65816) a `docs/specs/data/oracle_sources.md` une section `## spc700` avec: URL reelle utilisee, hash du commit clone (`git -C tests/oracle/spc700 rev-parse HEAD`), date, nombre de fichiers `.json` (recursif), taille totale en Mo, et 5 exemples de noms de fichiers.""",
  accept=["python -c \"import os,sys; sys.exit(0 if os.path.isdir('tests/oracle/spc700') else 1)\"",
          "python -c \"import sys; t=open('docs/specs/data/oracle_sources.md',encoding='utf-8').read(); sys.exit(0 if '## spc700' in t and 'http' in t else 1)\""])

T("E06-02", "T-FORMAT: documenter le format JSON exact de l'oracle spc700",
  kind="doc", size=0,
  context="""Le format JSON de l'oracle SPC700 n'est PAS suppose (il peut differer de celui du 65816). Cette tache LIT les vrais fichiers et decrit le format. Aucun code Rust.
Le chargeur (E06-07) sera ecrit uniquement a partir de ce document.""",
  embed=["06_TEST_ROMS.md#3. Oracle SingleStepTests"],
  write=["docs/specs/data/spcjson_format.md"],
  spec="""1. Lister le dossier `tests/oracle/spc700` (recursif, 2 niveaux max). Ouvrir UN fichier avec `head -c 3000` (ou equivalent Python) et UN autre d'une autre famille d'instructions.
2. Ecrire `docs/specs/data/spcjson_format.md` avec EXACTEMENT ces sections:
   - `## 1. Fichiers`: convention de nommage (comment retrouver l'opcode depuis le nom), nombre de tests par fichier, structure globale (tableau JSON?).
   - `## 2. Cles initial et final`: la liste EXACTE des cles de l'etat (pc, a, x, y, sp, psw, ram...), leur type JSON et signification, forme de `ram` (liste de paires `[adresse, valeur]`?).
   - `## 3. Cycles`: forme exacte d'un cycle de la liste `cycles` (adresse, valeur, type de cycle), comment un cycle sans acces bus est represente. Donner 1 exemple copie tel quel.
   - `## 4. Questions ouvertes`: tout ce qui est ambigu (RAM plate partout, y compris $F0-$FF? l'etat final contient-il les registres modifies par l'instruction? le nombre de cycles d'une instruction correspond-il a la colonne `cycles` de `opcodes_spc700.tsv`? ecarts constates sur 10 opcodes au hasard).
   - `## 5. Exemple complet`: UN test complet copie tel quel du fichier (assez court).
3. Ne rien deviner: si une cle n'est pas claire, la mettre en section 4.
4. Terminer en ecrivant a la fin du fichier la ligne `STATUS: awaiting review (E06-03)`.""",
  accept=["python -c \"import sys; t=open('docs/specs/data/spcjson_format.md',encoding='utf-8').read(); h=['## 1. Fichiers','## 2. Cles initial et final','## 3. Cycles','## 4. Questions ouvertes','## 5. Exemple complet']; sys.exit(0 if all(x in t for x in h) else 1)\""])

T("E06-03", "GATE: Claude relit spcjson_format.md et valide le format",
  kind="gate", gate=True, size=0,
  context="Point d'arret obligatoire (spec 06 section 3): le format de l'oracle SPC700 est relu par Claude avant d'ecrire le chargeur.",
  spec="""Claude (pas Hermes) verifie `docs/specs/data/spcjson_format.md` contre 2 fichiers reels, corrige le document si besoin, et decide:
- le niveau de comparaison des cycles (`Level::Count` par defaut: etat final + nombre de cycles; `Level::Full` seulement si le journal par cycle est fiable);
- les ecarts eventuels entre le TSV `opcodes_spc700.tsv` et les cycles de l'oracle (le TSV est corrige par Claude);
- la convention de filtrage `--filter` (opcodes en hexa, ex `E4,F4`) et la fonction `opcode_of_file` pour E06-07;
- la valeur de `STATUS` -> `STATUS: reviewed`.""",
  user_cmds=["python docs/tools/task.py approve E06-03   # apres la relecture de Claude"],
  accept=[])

T("E06-04", "gen_spc_tables.py: spc700/table.rs (BYTES, CYCLES, BRANCH_EXTRA, NAME)",
  size=100, kind="tooling",
  context="""Les cycles et tailles des 256 opcodes viennent du TSV, jamais recopies a la main. `table.rs` est genere, jamais edite a la main.""",
  embed=["07c_SPC_DESIGN.md#1. Tables generees"],
  reads=["specs/data/opcodes_spc700.tsv (colonnes op, mnemonic, bytes, cycles, branch_taken_extra)"],
  write=["docs/tools/gen_spc_tables.py", SPC + "/table.rs", TST + "/e06_04_table.rs"],
  spec="""1. `docs/tools/gen_spc_tables.py` (Python 3, stdlib seulement): lit `docs/specs/data/opcodes_spc700.tsv` (256 lignes de donnees apres l'en-tete), ecrit `crates/puce16-core/src/apu/spc700/table.rs` avec en tete `// GENERATED by docs/tools/gen_spc_tables.py - do not edit`.
2. Contenu genere: `pub static BYTES: [u8; 256]`, `pub static CYCLES: [u8; 256]`, `pub static BRANCH_EXTRA: [u8; 256]`, `pub static NAME: [&str; 256]` (indices = opcode; NAME = texte exact de la colonne `mnemonic`).
3. Option `--check`: ne rien ecrire; code de sortie 1 si `table.rs` differe de la sortie generee (sinon 0). Le script doit echouer (exit 1, message clair) si le TSV n'a pas 256 lignes ou si un opcode manque/est duplique.
4. Lancer le script une fois pour produire `table.rs`.
5. Le fichier `spc700/mod.rs` existe peut-etre deja sans `pub mod table;`: ajoute cette seule declaration si elle manque (E06-05 la reprendra).""",
  tests=[("known_rows", "NAME[0x00]==\"NOP\" ; NAME[0xE4]==\"MOV A,dp\" ; NAME[0xC7]==\"MOV [dp+X],A\" ; NAME[0xFF]==\"STOP\" ; NAME[0x3F]==\"CALL !abs\" ; NAME[0xFE]==\"DBNZ Y,rel\"."),
         ("cycles_bytes", "CYCLES[0x00]==2 ; CYCLES[0x01]==8 ; CYCLES[0x0F]==8 (BRK) ; CYCLES[0x9E]==12 (DIV) ; CYCLES[0xCF]==9 (MUL) ; CYCLES[0xE4]==3 ; CYCLES[0xC7]==7 ; BYTES[0x00]==1 ; BYTES[0x03]==3 ; BYTES[0x05]==3 ; BYTES[0x08]==2 ; BYTES[0x3F]==3."),
         ("branch_extra", "BRANCH_EXTRA[0x03]==2 (BBS) ; BRANCH_EXTRA[0x10]==2 (BPL) ; BRANCH_EXTRA[0x2E]==2 (CBNE) ; BRANCH_EXTRA[0xFE]==2 (DBNZ Y) ; BRANCH_EXTRA[0x2F]==0 (BRA, 4 cycles fixes) ; BRANCH_EXTRA[0x00]==0."),
         ("all_rows_sane", "pour les 256 opcodes: BYTES entre 1 et 3, CYCLES entre 2 et 12, NAME non vide; exactement 28 opcodes ont BRANCH_EXTRA == 2 (8 branches conditionnelles + 16 BBS/BBC + 2 CBNE + 2 DBNZ), tous les autres 0.")],
  accept=["python docs/tools/gen_spc_tables.py --check", "cargo test -p puce16-core e06_04_"])

T("E06-05", "Spc700: registres, trait SmpBus, helpers, step et squelettes de familles",
  size=150,
  context="""Coeur de l'API SPC700. Cette tache cree la structure `Spc700`, le trait `SmpBus`, tous les helpers que les familles utiliseront, la boucle `step` (dispatch) et les fichiers de famille vides. Apres elle, chaque tache d'instruction ne touche qu'un seul fichier.
Les fichiers `addr.rs` et `testbus.rs` sont crees ici VIDES (commentaire seulement): ils sont remplis par E06-06 et E06-10. """ + CHAIN,
  embed=["07c_SPC_DESIGN.md#2. Contrat", "07c_SPC_DESIGN.md#3. Dispatch et cycles", "07_SPC700.md#1. Registres du coeur"],
  write=[SPC + "/mod.rs", SPC + "/addr.rs", SPC + "/testbus.rs", SPC + "/alu.rs", SPC + "/mov.rs", SPC + "/incdec.rs", SPC + "/word.rs",
         SPC + "/bits.rs", SPC + "/flow.rs", SPC + "/misc.rs", TST + "/e06_05_core.rs"],
  spec="""1. `spc700/mod.rs`: `pub mod table; pub mod addr; pub mod testbus; mod alu; mod mov; mod incdec; mod word; mod bits; mod flow; mod misc;` puis les constantes `F_*` (valeurs de la section 2), le trait `SmpBus`, la struct `Spc700` (derive `Clone, Debug`; champs publics `a x y sp pc psw halted`, champs prives `cyc: u32`, `taken: bool`), et TOUTES les methodes du tableau de la section 2 (`new`, `rd`, `wr`, `idle`, `fetch8`, `fetch16`, `dp`, `rd16`, `flag`, `set_flag`, `set_nz`, `ya`, `set_ya`, `push8`, `pull8`, `branch`, `step`, `reset`). `impl Default for Spc700`.
2. `step` exactement comme la section 3 (dispatch `mov::exec(self,bus,op) || alu::exec(..) || incdec::exec(..) || word::exec(..) || bits::exec(..) || flow::exec(..) || misc::exec(..)`; sinon `panic!("unimplemented opcode {:02X} ({})", op, table::NAME[op as usize])`; completion par `idle`; renvoie `max(cyc, target)`).
3. Chaque fichier de famille (`alu mov incdec word bits flow misc`) contient exactement:
```rust
use super::{SmpBus, Spc700};
pub(super) fn exec<B: SmpBus>(_cpu: &mut Spc700, _bus: &mut B, _op: u8) -> bool { false }
```
4. `addr.rs` et `testbus.rs`: un commentaire `// filled by a later task` suffit.
5. `halted`: `step` fait un `idle`, ne change pas `pc`, renvoie 1.""",
  tests=[("new_defaults", "`Spc700::new()`: a=x=y=sp=0, pc=0, psw==0x02, halted==false."),
         ("fetch_order_and_wrap", "Bus local au test (`struct TB { ram: Vec<u8> }` 65536 octets implementant `SmpBus`, idle = ne rien faire) : pc=0x0300, octets [0x34,0x12] : fetch16 -> 0x1234 (bas d'abord), pc==0x0302 ; pc=0xFFFF : fetch8 lit l'octet en 0xFFFF et pc==0x0000."),
         ("push_pull", "sp=0xEF: push8(0xAB) ecrit ram[0x01EF]==0xAB et sp==0xEE ; pull8 renvoie 0xAB et sp==0xEF. sp=0x00: push8 ecrit en 0x0100 puis sp==0xFF (wrap)."),
         ("dp_base", "psw=0x00 -> dp()==0x0000 ; psw=F_P -> dp()==0x0100."),
         ("flags_and_nz", "set_flag(F_C,true) -> psw==0x03 ; set_flag(F_C,false) -> 0x02 ; set_nz(0x80): N=1,Z=0 ; set_nz(0): Z=1,N=0 ; set_nz(0x7F): N=0,Z=0 ; flag(F_Z) reflete le bit."),
         ("ya_pair", "y=0x12, a=0x34: ya()==0x1234 ; set_ya(0xABCD): y==0xAB, a==0xCD."),
         ("rd16_order", "ram[0x10]=0x34, ram[0x11]=0x12: rd16(bus,0x10,0x11)==0x1234 ; rd16(bus,0xFF,0x00) avec ram[0xFF]=0x78, ram[0x00]=0x56 -> 0x5678."),
         ("branch_forward_backward", "pc=0x0210: branch(bus,0x05) -> pc==0x0215 ; branch(bus,0xFB) (-5) depuis 0x0215 -> 0x0210 ; pc=0x0000, branch(0xFF) -> 0xFFFF."),
         ("reset_vector", "ram[0xFFFE]=0xC0, ram[0xFFFF]=0xFF (bus local): reset(bus) -> pc==0xFFC0, a=x=y=0, sp==0, psw==0x02, halted==false."),
         ("halted_step", "halted=true: step renvoie 1 et pc inchange."),
         ("unimplemented_panics", "`#[should_panic(expected = \"unimplemented opcode 00\")]`: bus local avec ram[0x0200]=0x00 (NOP pas encore implemente a ce stade), pc=0x0200, step.")],
  notes=CHAIN)

T("E06-06", "spc700/testbus.rs: FlatSmpBus (bus plat journalise)",
  size=90,
  context="Bus de test plat 16 bits avec journal des acces. Utilise par tous les tests SPC700 et par le harnais oracle (crate cli). " + CHAIN,
  embed=[EMB_RUN],
  write=[SPC + "/testbus.rs", TST + "/e06_06_testbus.rs"],
  spec="""```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum CycleKind { Read, Write, Idle }
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub struct Cycle { pub addr: u16, pub val: u8, pub kind: CycleKind }
pub struct FlatSmpBus { mem: Vec<u8> /* 65536 */, touched: Vec<u16>, pub log: Vec<Cycle> }
impl FlatSmpBus {
    pub fn new() -> FlatSmpBus;                         // memoire a zero, journal vide
    pub fn poke(&mut self, addr: u16, bytes: &[u8]);    // ecrit bytes[i] en addr+i (wrap 16 bits), SANS journal; memorise les adresses touchees
    pub fn peek(&self, addr: u16) -> u8;                // lecture sans journal
    pub fn clear_log(&mut self);
    pub fn reset_mem(&mut self);                        // remet a zero les adresses touchees, vide le journal
}
impl Default for FlatSmpBus { ... }
impl SmpBus for FlatSmpBus { ... }
```
- `read(addr)` journalise `Cycle { addr, val, kind: Read }`; `write(addr, v)` ecrit, memorise l'adresse, journalise `Write`; `idle()` journalise `Cycle { addr: 0, val: 0, kind: Idle }`. Memoire allouee en une fois.
- Le bus est PLAT: aucun registre mappe, pas d'IPL (comme l'oracle).""",
  tests=[("read_write_log", "write(0x1234,0x5A) puis read(0x1234)==0x5A ; log == [Write{0x1234,0x5A}, Read{0x1234,0x5A}]."),
         ("idle_log", "idle() ajoute Cycle{addr:0,val:0,kind:Idle}."),
         ("poke_peek", "poke(0x0010,&[1,2,3]) -> peek(0x0011)==2 ; le journal reste vide."),
         ("poke_wrap", "poke(0xFFFF,&[7,8]) -> peek(0xFFFF)==7 et peek(0)==8."),
         ("reset_mem", "poke(0x10,&[5]) ; reset_mem() -> peek(0x10)==0 et journal vide."),
         ("flat_registers", "write(0x00F4,0x99) puis read(0x00F4)==0x99 (aucun registre mappe) ; read(0xFFC0) lit la RAM (0 apres new).")],
  notes=CHAIN)

T("E06-07", "spc-test: chargeur des fichiers JSON de l'oracle",
  size=130,
  context="""Lecture des fichiers de l'oracle SPC700. Le format exact est dans `docs/specs/data/spcjson_format.md` (valide par Claude en E06-03):
ECRIRE LE CHARGEUR UNIQUEMENT A PARTIR DE CE DOCUMENT, jamais de memoire. Meme architecture que `cpu_test/oracle.rs` (E02-12). """ + CHAIN,
  reads=["specs/data/spcjson_format.md (OBLIGATOIRE, c'est la reference du format)", "specs/06_TEST_ROMS.md section 3"],
  write=[CLI + "/src/spc_test/oracle.rs", CLI + "/tests/data/e06_07_sample.json"],
  spec="""Dans `spc_test/oracle.rs` (`puce16-cli` est un binaire: les tests sont des tests unitaires `#[cfg(test)] mod tests` DANS `oracle.rs`, fonctions nommees `e06_07_...`; l'exemple JSON est lu avec `include_str!("../../tests/data/e06_07_sample.json")`) :
```rust
pub struct SpcState { pub pc: u16, pub a: u8, pub x: u8, pub y: u8, pub sp: u8, pub psw: u8, pub ram: Vec<(u16, u8)> }
pub struct CycleRec { pub addr: Option<u16>, pub val: Option<u8>, pub kind: String }   // champs = ceux du document de format
pub struct Case { pub name: String, pub initial: SpcState, pub fin: SpcState, pub cycles: Vec<CycleRec> }
pub fn load_file(path: &std::path::Path) -> Result<Vec<Case>, String>;
pub fn list_files(dir: &std::path::Path, filter: Option<&str>) -> Result<Vec<std::path::PathBuf>, String>;
pub fn opcode_of_file(name: &str) -> Option<u8>;
```
Les noms de champs/cles JSON sont ceux du document de format (si une cle differe de ce qui est ecrit ci-dessus, le document gagne et la struct s'adapte, en gardant les noms Rust).
- `list_files`: tous les `.json` (recursif), tries par nom; `filter` = liste d'opcodes en hexa separes par des virgules (`E4,F4`); un fichier est garde si `opcode_of_file(nom)` est dans la liste (ou si `filter` est None). Erreur `String` si le dossier ne se lit pas.
- `opcode_of_file`: convention de nommage du document. `load_file`: erreurs en `Err(String)` avec le nom du fichier; aucun `unwrap` sur des donnees externes. Utiliser `serde` + `serde_json` (deja dans le Cargo.toml).""",
  tests=[("load_sample", "Le fichier `tests/data/e06_07_sample.json` contient UN test copie tel quel depuis un vrai fichier de l'oracle (section 5 du document de format). `load_file` renvoie 1 cas ; verifier au moins 3 registres initiaux, 1 valeur de RAM finale et le nombre de cycles, avec les VRAIES valeurs de l'exemple."),
         ("load_bad_json", "un fichier contenant `not json` -> Err(message contenant le nom du fichier)."),
         ("opcode_of_file", "au moins 4 noms de fichiers reels (copies de la liste de `tests/oracle/spc700`) -> opcode attendu ; un nom sans rapport (`readme.md`) -> None."),
         ("list_files_filter", "dossier temporaire avec 3 faux fichiers nommes selon la convention (opcodes E4, F4, 00): `filter=Some(\"E4,00\")` -> 2 fichiers tries ; `filter=None` -> 3 ; dossier absent -> Err.")],
  accept=["cargo test -p puce16-cli e06_07_"], crate="puce16-cli", notes=CHAIN)

T("E06-08", "spc-test: comparaison d'etats et de cycles (fonctions pures)",
  size=120,
  context="""Compare l'etat obtenu apres `Spc700::step` avec l'etat attendu de l'oracle. Fonctions pures testables sans instruction implementee. Meme architecture que `cpu_test/compare.rs` (E02-13). """ + CHAIN,
  reads=["specs/data/spcjson_format.md", "specs/06_TEST_ROMS.md section 3 (strategie de comparaison)"],
  write=[CLI + "/src/spc_test/compare.rs"],
  spec="""Dans `spc_test/compare.rs` (tests unitaires dans le meme fichier, noms `e06_08_...`):
```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Level { Regs, Count, Full }
pub fn diff_state(expected: &SpcState, cpu: &Spc700, bus: &FlatSmpBus) -> Vec<String>;
pub fn diff_cycles(expected: &[CycleRec], got_count: u32, bus: &FlatSmpBus, level: Level) -> Vec<String>;
pub fn run_case(case: &Case, cpu: &mut Spc700, bus: &mut FlatSmpBus, level: Level) -> Vec<String>;   // liste vide = succes
```
- `diff_state`: pour chaque champ different, une ligne `a: expected 0x12 got 0x10` (hexa; pc sur 4 chiffres, autres sur 2) pour pc, a, x, y, sp, psw puis, pour chaque `(addr, val)` de `expected.ram`, `bus.peek(addr)` (ligne `ram[0x1234]: expected 0x41 got 0x42`).
- `diff_cycles`: `Level::Regs` -> toujours vide; `Level::Count` -> seulement `cycles: expected N got M` si le nombre differe (N = `expected.len()`, M = `got_count`); `Level::Full` -> nombre, puis pour chaque acces non factice du journal du bus: type/adresse/valeur doivent correspondre au cycle de l'oracle de meme index (au plus 5 lignes de diff).
- `run_case`: `bus.reset_mem()`; poke de `initial.ram`; copier l'etat initial dans `cpu` (champs un a un, `halted=false`); `bus.clear_log()`; `n = cpu.step(bus)`; puis concatener `diff_state` et `diff_cycles(.., n, bus, level)`. Un `std::panic::catch_unwind` autour du `step` convertit une panique en une ligne `panic: <message>`.""",
  tests=[("diff_state_equal", "un `SpcState` et un `Spc700` identiques + RAM identique -> liste vide."),
         ("diff_state_fields", "Cpu avec a=0x10 alors que 0x12 attendu et pc different -> 2 lignes, la premiere commence par `a: expected 0x12 got 0x10`."),
         ("diff_state_ram", "RAM attendue (0x1234, 0x41) mais bus contient 0x42 -> 1 ligne contenant `ram[0x1234]`."),
         ("diff_cycles_count", "attendu 3 cycles, obtenu 2, Level::Count -> 1 ligne contenant `expected 3 got 2` ; Level::Regs -> vide."),
         ("run_case_nop_like", "cas fabrique a la main: opcode 0xE8 (MOV A,#imm) pas encore implemente -> `run_case` renvoie une liste contenant une ligne commencant par `panic:` (la panique est capturee).")],
  accept=["cargo test -p puce16-cli e06_08_"], crate="puce16-cli", notes=CHAIN)

T("E06-09", "spc-test: commande complete (boucle sur les fichiers, rapport, code de sortie)",
  size=110,
  context="La commande `puce16-cli spc-test <DIR> [--filter ..]` utilisable par toutes les taches de verification SPC700 suivantes. Meme comportement que `cpu-test` (E02-14). " + CHAIN,
  embed=["06_TEST_ROMS.md#3. Oracle SingleStepTests"],
  reads=["specs/data/spcjson_format.md"],
  write=[CLI + "/src/spc_test.rs", CLI + "/tests/e06_09_spc_test_cmd.rs", CLI + "/tests/data/e06_09_*.json"],
  spec="""`spc_test::run(dir, filter) -> i32`:
1. `list_files(dir, filter)`; erreur de lecture du dossier ou aucun fichier retenu -> message sur stderr et code 2.
2. Pour chaque fichier (ordre trie): `load_file`; pour chaque cas: `run_case(case, &mut cpu, &mut bus, Level::Count)` (niveau choisi en E06-03; `Level::Count` par defaut) avec UN seul `Spc700` et UN seul `FlatSmpBus` reutilises. Une erreur de lecture du fichier compte comme un fichier en echec.
3. Sortie stdout: par fichier une ligne `OK <nom> (<n> tests)` ou `FAIL <nom> <k>/<n>`; apres un `FAIL`, au plus 5 echecs detailles: ligne `  <nom du test>` puis les lignes de diff indentees de 4 espaces.
4. Fin: ligne `files: <total> ok: <ok> failed: <ko>`. Code de sortie 0 si aucun echec, 1 sinon, 2 pour erreur d'usage.
5. Un cas dont `step` panique est un echec (le message `panic:` apparait dans les diffs); la commande ne s'arrete jamais sur une panique.
6. `--filter` documente dans `--help`: `liste d'opcodes en hexa separes par des virgules, ex E4,F4`.""",
  tests=[("wrong_expectation_fails", "Dossier temporaire contenant un fichier JSON (schema du document de format, nom selon la convention, opcode 00) dont UN test attend un `pc` volontairement faux (0x1234) apres NOP: la commande sort avec le code 1 (ou le test echoue par `panic:` car NOP n'est pas encore implemente: dans les deux cas stdout contient `FAIL`)."),
         ("empty_dir_usage", "dossier existant sans aucun .json -> code 2."),
         ("filter_no_match", "dossier avec un seul fichier d'opcode 00 et `--filter E4` -> code 2."),
         ("summary_line", "stdout contient une ligne commencant par `files: 1 ok: 0 failed: 1` pour le premier test."),
         ("bad_json_is_failure", "fichier invalide -> code 1 (et stdout contient `FAIL`).")],
  accept=["cargo test -p puce16-cli e06_09_", "cargo test -p puce16-cli e02_11_"], crate="puce16-cli", notes=CHAIN)

T("E06-10", "spc700/addr.rs: recettes d'adressage (dp, dp+X, abs, (X), [dp+X], [dp]+Y...)",
  size=110,
  context="Fonctions d'adresse effective partagees par toutes les familles. Les acces utiles sont emis dans l'ordre de la section 4; les cycles manquants sont completes par `step`. " + CHAIN + " " + RUN,
  embed=["07c_SPC_DESIGN.md#4. Recettes d'adressage", "07_SPC700.md#5. Modes d'adressage"],
  write=[SPC + "/addr.rs", TST + "/e06_10_addr.rs"],
  spec="""Dans `spc700/addr.rs`, fonctions `pub` generiques `B: SmpBus` (retournent l'adresse effective u16):
`ea_dp`, `ea_dpx`, `ea_dpy`, `ea_abs`, `ea_absx`, `ea_absy`, `ea_x`, `ea_y`, `ea_dpx_ind`, `ea_dp_ind_y`, avec la signature `fn ea_xxx<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) -> u16` et exactement le comportement du tableau de la section 4 (consomme les octets d'operande par `cpu.fetch8/fetch16`, emet un `cpu.idle(bus)` la ou le tableau l'indique, lit les pointeurs par `cpu.rd`).
Les fonctions sont `pub` (pas `pub(super)`) pour que les tests d'integration y accedent (`puce16_core::apu::spc700::addr::ea_dp`); `mod.rs` declare deja `pub mod addr;` (E06-05).""",
  tests=[("ea_dp", "pc=0x0200, octets [0x34] : `ea_dp`==0x0034, pc==0x0201, journal = 1 lecture ; avec psw=F_P : 0x0134."),
         ("ea_dpx_wrap", "octets [0xFE], x=3 : `ea_dpx`==0x0001 (wrap page) ; journal = [Read, Idle] ; avec F_P : 0x0101."),
         ("ea_dpy_wrap", "octets [0xF8], y=0x10 : `ea_dpy`==0x0008."),
         ("ea_abs", "octets [0x34,0x12] : `ea_abs`==0x1234, pc==0x0202, journal = 2 lectures."),
         ("ea_absx_wrap", "octets [0xFF,0xFF], x=2 : `ea_absx`==0x0001 ; journal = [Read, Read, Idle]."),
         ("ea_absy", "octets [0x00,0x20], y=5 : `ea_absy`==0x2005."),
         ("ea_x_y", "x=7 : `ea_x`==0x0007, pc inchange, journal = [Idle] ; avec F_P : 0x0107 ; y=9 : `ea_y`==0x0009."),
         ("ea_dpx_ind", "octets [0x10], x=2, poke(0x12,&[0x00,0x30]) : `ea_dpx_ind`==0x3000 ; journal = [Read, Idle, Read 0x0012, Read 0x0013]."),
         ("ea_dpx_ind_wrap", "octets [0xFE], x=1, poke(0x00FF,&[0x34]), poke(0x0000,&[0x12]) : `ea_dpx_ind`==0x1234 (haut lu en 0x0000 par wrap de page)."),
         ("ea_dp_ind_y", "octets [0x20], y=0x20, poke(0x20,&[0xF0,0x10]) : `ea_dp_ind_y`==0x1110 ; journal = [Read, Read 0x0020, Read 0x0021, Idle]."),
         ("ea_dp_ind_y_wrap", "octets [0xFF], y=2, poke(0x00FF,&[0xFF]), poke(0x0000,&[0x12]) : pointeur 0x12FF + 2 -> 0x1301.")],
  notes=CHAIN)

T("E06-11", "spc700/mov.rs: MOV de chargement (A, X, Y depuis la memoire ou un immediat)",
  size=100,
  context="Premiere famille d'instructions: chargements. Flags N,Z seulement (C,V,H,I,B,P inchanges). " + CHAIN + " " + RUN,
  embed=["07_SPC700.md#6.1 MOV (41 opcodes)"],
  tsv=[("specs/data/opcodes_spc700.tsv", ["E8", "E4", "F4", "E5", "F5", "F6", "E6", "E7", "F7", "CD", "F8", "F9", "E9", "8D", "EB", "FB", "EC"])],
  reads=READS,
  write=[SPC + "/mov.rs", TST + "/e06_11_mov_load.rs"],
  spec="""Dans `mov.rs`, `exec` traite (sinon `false`) les opcodes suivants; chargement = `v` lu, registre = `v`, `set_nz(v)`:
- A: `E8` #imm (`fetch8`) ; `E4` dp ; `F4` dp+X ; `E5` !abs ; `F5` !abs+X ; `F6` !abs+Y ; `E6` (X) ; `E7` [dp+X] ; `F7` [dp]+Y.
- X: `CD` #imm ; `F8` dp ; `F9` dp+Y ; `E9` !abs.
- Y: `8D` #imm ; `EB` dp ; `FB` dp+X ; `EC` !abs.
Adresse effective par `addr::ea_*` (E06-10), lecture par `cpu.rd`. Retourne `true`.""",
  tests=[("a_imm_negative", "`E8 80`: a==0x80, N=1, Z=0, cycles==2."),
         ("a_dp_zero", "a=5, poke(0x0010,&[0]), `E4 10`: a==0, Z=1, N=0, cycles==3."),
         ("a_dpx_wrap", "x=2, poke(0x0000,&[0x7F]), `F4 FE`: a==0x7F, cycles==4."),
         ("a_abs", "poke(0x3000,&[0x42]), `E5 00 30`: a==0x42, cycles==4."),
         ("a_absx", "x=1, poke(0x3000,&[0x11]), `F5 FF 2F`: a==0x11, cycles==5."),
         ("a_absy_negative", "y=2, poke(0x3002,&[0x99]), `F6 00 30`: a==0x99, N=1, cycles==5."),
         ("a_x_indirect", "x=0x10, poke(0x0010,&[0x55]), `E6`: a==0x55, cycles==3."),
         ("a_dpx_indirect", "x=2, poke(0x0012,&[0x00,0x30]), poke(0x3000,&[0x66]), `E7 10`: a==0x66, cycles==6."),
         ("a_dp_indirect_y", "y=1, poke(0x0020,&[0xFF,0x2F]), poke(0x3000,&[0x77]), `F7 20`: a==0x77, cycles==6."),
         ("x_loads", "`CD 00`: x==0, Z=1, cycles 2 ; poke(0x10,&[0x81]) `F8 10`: x==0x81, N=1, cycles 3 ; y=3, poke(0x0001,&[0x22]), `F9 FE`: x==0x22, cycles 4 ; poke(0x3000,&[0x33]), `E9 00 30`: x==0x33, cycles 4."),
         ("y_loads", "`8D 7F`: y==0x7F, cycles 2 ; poke(0x10,&[0x80]) `EB 10`: y==0x80, N=1, cycles 3 ; x=1, poke(0x0011,&[0x44]), `FB 10`: y==0x44, cycles 4 ; poke(0x3000,&[0x55]) `EC 00 30`: y==0x55, cycles 4."),
         ("other_flags_untouched", "psw=0x49 (C,H,V), `E8 01`: psw==0x49 (ni N ni Z)."),
         ("direct_page_p_flag", "psw=F_P, poke(0x0110,&[0x5A]), `E4 10`: a==0x5A.")],
  notes=CHAIN)

T("E06-12", "spc700/mov.rs: MOV de stockage (avec lecture factice de la destination)",
  size=110,
  context="""Stockages: A, X, Y vers la memoire. Chaque stockage fait une LECTURE FACTICE de l'adresse de destination avant l'ecriture (07 section 6.1; important pour $FD-$FF dont la lecture efface le compteur). Aucun flag modifie. """ + CHAIN + " " + RUN,
  embed=["07_SPC700.md#6.1 MOV (41 opcodes)"],
  tsv=[("specs/data/opcodes_spc700.tsv", ["C4", "C5", "C6", "C7", "D4", "D5", "D6", "D7", "C9", "D8", "D9", "CB", "CC", "DB"])],
  reads=READS,
  write=[SPC + "/mov.rs", TST + "/e06_12_mov_store.rs"],
  spec="""Ajoute a `mov.rs::exec` (`ea` = adresse effective par `addr::ea_*`; puis `let _ = cpu.rd(bus, ea); cpu.wr(bus, ea, v);`):
- depuis A: `C4` dp ; `C5` !abs ; `C6` (X) ; `C7` [dp+X] ; `D4` dp+X ; `D5` !abs+X ; `D6` !abs+Y ; `D7` [dp]+Y.
- depuis X: `C9` !abs ; `D8` dp ; `D9` dp+Y.
- depuis Y: `CB` dp ; `CC` !abs ; `DB` dp+X.
Retourne `true`; aucun flag modifie.""",
  tests=[("dp_with_dummy_read", "a=0x5A, `C4 10`: peek(0x0010)==0x5A, cycles==4 ; journal complet = [Read 0x0200, Read 0x0201, Read 0x0010 (factice), Write 0x0010=0x5A]."),
         ("abs", "a=0x11, `C5 00 30`: peek(0x3000)==0x11, cycles==5."),
         ("x_indirect", "x=0x20, a=0x22, `C6`: peek(0x0020)==0x22, cycles==4."),
         ("dpx_indirect", "x=2, a=0x33, poke(0x0012,&[0x00,0x30]), `C7 10`: peek(0x3000)==0x33, cycles==7."),
         ("dpx", "x=1, a=0x44, `D4 10`: peek(0x0011)==0x44, cycles==5."),
         ("absx_absy", "x=1, a=0x55, `D5 FF 2F`: peek(0x3000)==0x55, cycles==6 ; y=2, a=0x66, `D6 00 30`: peek(0x3002)==0x66, cycles==6."),
         ("dp_indirect_y", "y=1, a=0x77, poke(0x0020,&[0xFF,0x2F]), `D7 20`: peek(0x3000)==0x77, cycles==7."),
         ("from_x", "x=0x88, `C9 00 30`: peek(0x3000)==0x88, cycles 5 ; `D8 10`: peek(0x0010)==0x88, cycles 4 ; y=3, `D9 FE`: peek(0x0001)==0x88, cycles 5."),
         ("from_y", "y=0x99, `CB 10`: peek(0x0010)==0x99, cycles 4 ; `CC 00 30`: peek(0x3000)==0x99, cycles 5 ; x=1, `DB 10`: peek(0x0011)==0x99, cycles 5."),
         ("no_flags", "psw=0xC3, a=0x00, `C4 10`: psw==0xC3 (inchange)."),
         ("dummy_read_logged", "x=0x20, a=0x22, `C6`: le journal contient une lecture de 0x0020 AVANT l'ecriture de 0x0020.")],
  notes=CHAIN)

T("E06-13", "spc700/mov.rs: transferts, MOV dp,#imm, MOV dp,dp, (X)+ et MOVW",
  size=130,
  context="Fin de la famille MOV: transferts entre registres, immediats vers memoire, copie memoire a memoire, auto-increment de X, mots 16 bits. " + CHAIN + " " + RUN,
  embed=["07_SPC700.md#6.1 MOV (41 opcodes)"],
  tsv=[("specs/data/opcodes_spc700.tsv", ["5D", "7D", "DD", "FD", "9D", "BD", "8F", "FA", "BF", "AF", "BA", "DA"])],
  reads=READS,
  write=[SPC + "/mov.rs", TST + "/e06_13_mov_misc.rs"],
  spec="""Ajoute a `mov.rs::exec`:
- Transferts avec `set_nz`: `5D` MOV X,A ; `7D` MOV A,X ; `DD` MOV A,Y ; `FD` MOV Y,A ; `9D` MOV X,SP. Sans flag: `BD` MOV SP,X.
- `8F` MOV dp,#imm: octets `op imm dp` (`imm = fetch8` PUIS `dp = fetch8`) ; `ea = P | dp` ; lecture factice de `ea` puis ecriture.
- `FA` MOV dp,dp: octets `op src dst` ; lit la source, ecrit la destination (PAS de lecture factice de la destination).
- `BF` MOV A,(X)+: `a = rd(P|X)` ; `x += 1` ; `set_nz(a)`. `AF` MOV (X)+,A: `ea = P|X` ; lecture factice ; ecriture de A ; `x += 1`.
- `BA` MOVW YA,dp: `b = fetch8` ; `lo = rd(P|b)` ; `hi = rd(P|((b+1)&0xFF))` ; `ya = (hi<<8)|lo` ; N = bit 15, Z = (ya == 0).
- `DA` MOVW dp,YA: `b = fetch8` ; lecture factice de `P|b` ; ecrit A en `P|b` puis Y en `P|((b+1)&0xFF)` ; aucun flag. (`UNKNOWN - to confirm`: ce sont 5 acces pour 4 cycles dans le TSV; `step` renverra 5; l'oracle tranche en E06-14.)""",
  tests=[("reg_transfers", "a=0x80 `5D`: x==0x80, N=1, cycles 2 ; x=0 `7D`: a==0, Z=1 ; y=0x7F `DD`: a==0x7F ; a=0x01 `FD`: y==0x01 ; sp=0xEF `9D`: x==0xEF, N=1."),
         ("mov_sp_x_no_flags", "x=0x30, psw=0x00 `BD`: sp==0x30, psw==0x00, cycles 2."),
         ("dp_imm", "`8F 5A 10`: peek(0x0010)==0x5A, cycles==5 ; le journal contient une lecture factice de 0x0010 avant l'ecriture."),
         ("dp_dp", "poke(0x0010,&[0x77]), `FA 10 20`: peek(0x0020)==0x77, cycles==5 ; journal: lecture de 0x0010, ecriture de 0x0020, AUCUNE lecture de 0x0020."),
         ("a_x_postinc", "x=0x10, poke(0x0010,&[0x81]), `BF`: a==0x81, N=1, x==0x11, cycles==4."),
         ("x_postinc_store", "x=0x10, a=0x33, `AF`: peek(0x0010)==0x33, x==0x11, cycles==4."),
         ("movw_load", "poke(0x0010,&[0x34,0x12]), `BA 10`: y==0x12, a==0x34, N=0, Z=0, cycles==5 ; poke(0x10,&[0,0]) -> Z=1 ; poke(0x10,&[0x00,0x80]) -> N=1."),
         ("movw_load_wrap", "poke(0x00FF,&[0x11]), poke(0x0000,&[0x22]), `BA FF`: a==0x11, y==0x22."),
         ("movw_store", "y=0xAB, a=0xCD, `DA 10`: peek(0x0010)==0xCD, peek(0x0011)==0xAB ; psw inchange ; journal: lecture factice 0x0010, ecriture 0x0010 (A), ecriture 0x0011 (Y)."),
         ("movw_store_wrap", "y=0x12, a=0x34, `DA FF`: peek(0x00FF)==0x34, peek(0x0000)==0x12.")],
  notes=CHAIN)

T("E06-14", "VERIF oracle: toutes les instructions MOV / MOVW",
  size=60, kind="code",
  context="""Premier passage a l'oracle SPC700: tous les tests de MOV et MOVW. Comme tous les modes d'adressage sont exerces, les erreurs de cycles/wrap des modes se corrigent ICI (addr.rs). Les etapes suivantes reposent sur des modes valides.
`MOVW dp,YA` (DA): le TSV dit 4 cycles mais la description fait 5 acces: si l'oracle dit 5, c'est une question `spec` (Claude corrige le TSV et regenere table.rs), pas un changement de code a la volee.""",
  reads=["specs/07_SPC700.md sections 5 et 6.1", "specs/data/spcjson_format.md"],
  write=[SPC + "/addr.rs", SPC + "/mov.rs", TST + "/e06_14_oracle_regressions.rs"],
  spec="""1. Lancer: `%s`
2. Pour chaque echec: lire le diff (champ fautif, nombre de cycles), localiser le bug dans les fichiers autorises, corriger. Ne pas modifier le harnais (cli), `table.rs` ni les specs.
3. Si le diff semble indiquer que la SPEC ou le TSV est faux (et non le code): ne PAS adapter le code au hasard; ecrire une question `spec` (docs/02_QA_PROTOCOL.md) avec le test fautif.
4. Pour chaque bug corrige, ajouter dans `tests/e06_14_oracle_regressions.rs` un test unitaire (FlatSmpBus) qui le reproduit (nom `e06_14_<mode>_<cas>`). Si aucun bug n'est trouve, le fichier contient un test `e06_14_oracle_clean` qui verifie seulement que `Spc700::new()` existe.
5. Finir quand la commande renvoie `failed: 0` pour tous ces opcodes.""" % spc_cmd("MOV", "MOVW"),
  tests=[("oracle_clean_or_regressions", "voir spec etape 4")],
  accept=[spc_cmd("MOV", "MOVW"), "cargo test -p puce16-core e06_"],
  notes=CHAIN + "\nSi `tests/oracle/spc700` est absent (E06-01 non fait), c'est un blocage `tooling`: poser la question.")
