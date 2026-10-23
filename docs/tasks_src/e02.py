from pathlib import Path
from _dsl import T

CORE = "crates/puce16-core"
CLI = "crates/puce16-cli"
CPU = CORE + "/src/cpu"
TST = CORE + "/tests"
_TSV = Path(__file__).resolve().parents[1] / "specs" / "data" / "opcodes_65816.tsv"
_ROWS = [l.split("\t") for l in _TSV.read_text(encoding="utf-8").splitlines()[1:]]
ORACLE = "tests/oracle/65816"


def ops(*mns):
    """opcodes (2 hex digits, upper) of the given mnemonics, from opcodes_65816.tsv"""
    want = {m.upper() for m in mns}
    return [r[0].upper() for r in _ROWS if r[1] in want]


def tsv(*mns):
    return [("specs/data/opcodes_65816.tsv", ops(*mns))]


def oracle_cmd(*mns):
    return "cargo run -q -p puce16-cli -- cpu-test %s --filter %s" % (ORACLE, ",".join(ops(*mns)))


READS = ["specs/01b_CPU_DESIGN.md (sections 2, 3, 6: contrat Cpu/Bus, familles, conventions de test)"]
EMB_TEST = "01b_CPU_DESIGN.md#6. Tests d'unite"
CHAIN = ("Execution strictement sequentielle (D-020): cette tache suppose que toutes les precedentes sont terminees. "
         "Ne modifie QUE les fichiers de 'Write only in'.")

# ------------------------------------------------------------------ E02: CPU 65816 (jalon J1)

T("E02-01", "T-ORACLE: telecharger l'oracle SingleStepTests 65816",
  kind="tooling", size=0,
  context="""Les tests d'instructions 65816 (un test = une instruction, etat avant/apres, cycles) servent d'oracle a tout le CPU.
Les fichiers sont ignores par git (dossier tests/oracle/). Voir specs/06_TEST_ROMS.md section 3.""",
  embed=["06_TEST_ROMS.md#3. Oracle SingleStepTests"],
  write=["tests/oracle/**", "docs/specs/data/oracle_sources.md"],
  spec="""1. Depuis la racine du depot: `git clone --depth 1 https://github.com/SingleStepTests/65816 tests/oracle/65816` (URL a confirmer: `UNKNOWN - to confirm`; si le depot a un autre nom/organisation, le retrouver par recherche web, ne PAS inventer).
2. Si le reseau est bloque: question `tooling` (docs/02_QA_PROTOCOL.md), ne rien contourner.
3. Ne pas committer le contenu (`tests/oracle/` est dans .gitignore; verifier avec `git status`: rien de ce dossier ne doit apparaitre).
4. Ecrire `docs/specs/data/oracle_sources.md` avec: URL reelle utilisee, hash du commit clone (`git -C tests/oracle/65816 rev-parse HEAD`), date, nombre de fichiers `.json` (recursif), taille totale en Mo, et 5 exemples de noms de fichiers.""",
  accept=["python -c \"import os,sys; sys.exit(0 if os.path.isdir('tests/oracle/65816') else 1)\"",
          "python -c \"import sys; t=open('docs/specs/data/oracle_sources.md',encoding='utf-8').read(); sys.exit(0 if 'http' in t and 'json' in t else 1)\""])

T("E02-02", "T-FORMAT: documenter le format JSON exact de l'oracle",
  kind="doc", size=0, deps=["E02-01"],
  context="""Le format JSON de l'oracle n'est PAS suppose. Cette tache LIT les vrais fichiers et decrit le format. Aucun code Rust.
Le parseur (E02-12) sera ecrit uniquement a partir de ce document.""",
  embed=["06_TEST_ROMS.md#3. Oracle SingleStepTests"],
  write=["docs/specs/data/ssjson_format.md"],
  spec="""1. Lister le dossier `tests/oracle/65816` (recursif, 2 niveaux max). Ouvrir UN fichier avec `head -c 3000` (ou equivalent Python) et UN autre d'un autre mode (emulation vs natif) pour comparer.
2. Ecrire `docs/specs/data/ssjson_format.md` avec EXACTEMENT ces sections:
   - `## 1. Fichiers`: convention de nommage (comment retrouver l'opcode et le mode emulation/natif depuis le nom), nombre de tests par fichier, structure globale (tableau JSON? un test par ligne?).
   - `## 2. Cles initial et final`: la liste EXACTE des cles de l'etat (registres, drapeau emulation, RAM), leur type JSON et signification (ex: la cle de P, de DBR, de PBR, de E), forme de `ram` (liste de paires `[adresse, valeur]`?).
   - `## 3. Cycles`: forme exacte d'un cycle dans la liste `cycles` (adresse, valeur, type...), comment un cycle interne sans acces bus est represente, que signifient les lettres/drapeaux de type. Donner 1 exemple copie tel quel.
   - `## 4. Questions ouvertes`: tout ce qui est ambigu (l'adresse des cycles internes est-elle significative? les tests couvrent-ils les interruptions? etc.).
   - `## 5. Exemple complet`: UN test complet copie tel quel du fichier (assez court).
3. Ne rien deviner: si une cle n'est pas claire, la mettre en section 4.
4. Terminer en ecrivant a la fin du fichier la ligne `STATUS: awaiting review (E02-03)`.""",
  accept=["python -c \"import sys; t=open('docs/specs/data/ssjson_format.md',encoding='utf-8').read(); h=['## 1. Fichiers','## 2. Cles initial et final','## 3. Cycles','## 4. Questions ouvertes','## 5. Exemple complet']; sys.exit(0 if all(x in t for x in h) else 1)\""])

T("E02-03", "GATE: Claude relit ssjson_format.md et valide le format",
  kind="gate", gate=True, size=0, deps=["E02-02"],
  context="Point d'arret obligatoire (spec 06 section 3): le format de l'oracle est relu par Claude avant d'ecrire le moindre parseur.",
  spec="""Claude (pas Hermes) verifie `docs/specs/data/ssjson_format.md` contre 2 fichiers reels, corrige le document si besoin, et decide:
- si les cycles internes portent une adresse significative (alors Claude amende le trait `Bus` dans 01b section 2 et les taches suivantes);
- la convention de filtrage `--filter` (opcodes en hexa, ex `61,63`) et la fonction `opcode_of_file` pour E02-12;
- la valeur de `STATUS` -> `STATUS: reviewed`.""",
  user_cmds=["python docs/tools/task.py approve E02-03   # apres la relecture de Claude"],
  accept=[])

T("E02-04", "gen_tables.py: cpu/table.rs (enums Mn, Am et tables MN, AM)",
  deps=["E01-01"], size=130, kind="tooling",
  context="""Les 256 opcodes sont decodes par table generee depuis le TSV (pas de recopie a la main = pas d'erreur).
Le fichier `table.rs` est genere, jamais edite a la main.""",
  embed=["01b_CPU_DESIGN.md#1. Tables generees"],
  reads=["specs/data/opcodes_65816.tsv (colonnes op, mnemonic, mode)"],
  write=["docs/tools/gen_tables.py", CPU + "/table.rs", TST + "/e02_04_table.rs"],
  spec="""1. `docs/tools/gen_tables.py` (Python 3, stdlib seulement): lit `docs/specs/data/opcodes_65816.tsv`, ecrit `crates/puce16-core/src/cpu/table.rs` avec en tete `// GENERATED by docs/tools/gen_tables.py - do not edit` .
2. Contenu genere: enum `Mn` (92 variantes, ordre alphabetique), enum `Am` (31 variantes, ordre de la section ci-dessus), `pub static MN: [Mn; 256]`, `pub static AM: [Am; 256]`, `impl Mn { pub fn name(self) -> &'static str }` (match sur chaque variante, nom en MAJUSCULES: `Mn::Adc => "ADC"`). Derive `Clone, Copy, PartialEq, Eq, Debug` sur les deux enums.
3. Option `--check`: ne rien ecrire; code de sortie 1 si `table.rs` differe de la sortie generee (sinon 0). Le script doit echouer (exit 1, message clair) si le TSV contient un mnemonique ou un mode inconnu de la table de correspondance.
4. Lancer le script une fois pour produire `table.rs`.""",
  tests=[("known_ops", "MN/AM: 0x00 -> (Brk, Brk) ; 0x01 -> (Ora, DpXInd) ; 0x20 -> (Jsr, Abs) ; 0x22 -> (Jsl, Long) ; 0x4C -> (Jmp, Abs) ; 0x5C -> (Jml, Long) ; 0x69 -> (Adc, Imm) ; 0x6C -> (Jmp, AbsInd) ; 0x7C -> (Jmp, AbsXInd) ; 0xDC -> (Jml, AbsIndLong) ; 0xFC -> (Jsr, AbsXInd)."),
         ("more_ops", "0x0A -> (Asl, Acc) ; 0x44 -> (Mvp, Blk) ; 0x54 -> (Mvn, Blk) ; 0x62 -> (Per, Per) ; 0x80 -> (Bra, Rel) ; 0x82 -> (Brl, RelL) ; 0x83 -> (Sta, Sr) ; 0x93 -> (Sta, SrIndY) ; 0xB1 -> (Lda, DpIndY) ; 0xB7 -> (Lda, DpIndLongY) ; 0xC2 -> (Rep, Imm8) ; 0xD4 -> (Pei, Pei) ; 0xEA -> (Nop, Imp) ; 0xF4 -> (Pea, Pea) ; 0x42 -> (Wdm, Wdm) ; 0xFB -> (Xce, Imp)."),
         ("names", "`Mn::Adc.name() == \"ADC\"`, `Mn::Xce.name() == \"XCE\"`."),
         ("all_variants_used", "Parcourir MN et AM en les mettant dans un `Vec` sans doublon (les enums derivent `PartialEq`; pas besoin de Hash): MN contient exactement 92 valeurs distinctes, AM exactement 31.")],
  accept=["python docs/tools/gen_tables.py --check", "cargo test -p puce16-core e02_04_"])

T("E02-05", "Cpu: registres, trait Bus, helpers, step et squelettes de familles",
  deps=["E01-01", "E02-04"], size=150,
  context="""Coeur de l'API CPU. Cette tache cree la structure `Cpu`, le trait `Bus`, tous les helpers que les familles d'instructions utiliseront,
la boucle `step` (dispatch) et les fichiers vides de chaque famille. Apres elle, chaque tache d'instruction ne touche qu'un seul fichier.""",
  embed=["01b_CPU_DESIGN.md#2. Contrat", "01b_CPU_DESIGN.md#3. Dispatch"],
  reads=["specs/01_CPU_65816.md sections 1, 2, 5.2"],
  write=[CPU + "/mod.rs", CPU + "/interrupts.rs", CPU + "/alu.rs", CPU + "/mem.rs", CPU + "/flow.rs", CPU + "/misc.rs",
         CPU + "/alu/*.rs", CPU + "/mem/*.rs", CPU + "/flow/*.rs", CPU + "/misc/*.rs", TST + "/e02_05_cpu_core.rs"],
  spec="""1. `cpu/mod.rs`: declarations `pub mod addr; pub mod alu; pub mod mem; pub mod flow; pub mod misc; pub mod interrupts; pub mod table; pub mod testbus;` (deja la depuis E01-01; garde-les) puis les constantes FLAG_*, le trait `Bus`, la struct `Cpu` (derive `Clone, Debug`), et TOUTES les methodes du tableau de la section "Contrat" (`new`, `flag`, `set_flag`, `m8`, `x8`, `set_p`, `set_nz8/16/set_nz`, `pc24`, `fetch8/16/24`, `push8/pull8`, `push8n/pull8n`, `fix_stack_e`, `hash_into`, `step`, `reset`).
2. `step` : `if interrupts::poll(self, bus) { return; }` ; `let op = self.fetch8(bus); let mn = table::MN[op as usize]; let am = table::AM[op as usize];` puis `alu::exec(self, bus, mn, am) || mem::exec(..) || flow::exec(..) || misc::exec(..)` ; si aucun ne renvoie vrai: `panic!("unimplemented opcode {:02X} ({})", op, mn.name())`.
3. `reset`: `interrupts::reset(self, bus)`.
4. `interrupts.rs` (squelette, rempli en E02-40): `pub fn poll<B: Bus>(cpu: &mut Cpu, bus: &mut B) -> bool { false }` et `pub fn reset<B: Bus>(cpu: &mut Cpu, bus: &mut B) { cpu.e = true; }` (provisoire).
5. Familles: `alu.rs` declare `mod adc; mod sbc; mod logic; mod cmp; mod incdec; mod shift; mod tsbtrb;` et `pub(crate) fn exec<B: Bus>(cpu, bus, mn, am) -> bool { adc::exec(cpu,bus,mn,am) || sbc::exec(..) || ... }`. Idem `mem.rs` (load store push pull block), `flow.rs` (branch jump call sys), `misc.rs` (transfer flags). Chaque sous-fichier existe avec exactement:
```rust
use crate::cpu::table::{Am, Mn};
use crate::cpu::{Bus, Cpu};
pub(super) fn exec<B: Bus>(_cpu: &mut Cpu, _bus: &mut B, _mn: Mn, _am: Am) -> bool { false }
```
6. Les helpers `push8`/`pull8` ne comptent QUE les acces de pile; `step`/familles n'emettent jamais de cycle sans passer par `bus`.""",
  tests=[("new_defaults", "`Cpu::new()`: a=x=y=0, s=0x01FF, d=0, db=pb=0, pc=0, p=0x34, e=true, halted=false, waiting=false."),
         ("set_p_emulation", "e=true: `set_p(0x00)` -> p == 0x30 ; `set_p(0xCF)` -> p == 0xFF."),
         ("set_p_native_x", "e=false, x=0x1234, y=0x5678: `set_p(0x10)` -> p==0x10, x==0x0034, y==0x0078 ; ensuite `set_p(0x00)` -> p==0x00, x et y inchanges (0x0034, 0x0078)."),
         ("set_nz", "set_nz8(0x80): N=1,Z=0 ; set_nz8(0): Z=1,N=0 ; set_nz16(0x8000): N=1 ; set_nz16(0x0080): N=0,Z=0 ; set_nz(0x0100,false): Z=1 (seul l'octet bas compte) ; set_nz(0x0100,true): Z=0,N=0."),
         ("fetch_wrap", "Avec un petit bus local au test (Vec<u8> de 16 MiB ou HashMap, defini DANS le fichier de test, car FlatBus vient en E02-06): pb=0x12, pc=0xFFFF, octet en 12:FFFF = 0xAA: fetch8 -> 0xAA, pc==0x0000, pb==0x12. fetch16 en pc=0x8000 avec [0x34,0x12] -> 0x1234, pc==0x8002. fetch24 avec [0x56,0x34,0x12] -> 0x123456."),
         ("push_pull_emulation", "e=true, s=0x0100: push8(0xAB) ecrit en 0x000100, s==0x01FF. Puis pull8 renvoie 0xAB, s==0x0100."),
         ("push_pull_native", "e=false, s=0x0000: push8n(0x11) ecrit a l'adresse 0x000000, s==0xFFFF ; pull8n relit 0x11, s==0x0000. e=true, s=0x0100: push8n(5) -> ecrit en 0x000100, s==0x00FF ; apres fix_stack_e s==0x01FF."),
         ("hash_into", "Deux `Cpu::new()` donnent le meme hash (Fnv64) ; apres a=1 le hash differe ; apres e=false il differe aussi.")],
  accept=["cargo build -p puce16-core", "cargo test -p puce16-core e02_05_"],
  notes=CHAIN)

T("E02-06", "testbus.rs: FlatBus (bus plat journalise)",
  deps=["E02-05"], size=100,
  context="Bus de test plat 24 bits, avec journal des cycles. Utilise par tous les tests CPU et par le harnais oracle (crate cli).",
  embed=[EMB_TEST],
  write=[CPU + "/testbus.rs", TST + "/e02_06_testbus.rs"],
  spec="""```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum CycleKind { Read, Write, Idle }
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub struct Cycle { pub addr: u32, pub val: u8, pub kind: CycleKind }
pub struct FlatBus { mem: Vec<u8> /* 0x100_0000 octets */, touched: Vec<u32>, pub log: Vec<Cycle>, pub nmi: bool, pub irq: bool }
impl FlatBus {
    pub fn new() -> FlatBus;                    // memoire a zero, journal vide
    pub fn poke(&mut self, addr: u32, bytes: &[u8]);   // ecrit bytes[i] en addr+i (addr masque a 24 bits, wrap), SANS journal; memorise les adresses touchees
    pub fn peek(&self, addr: u32) -> u8;        // lecture sans journal
    pub fn clear_log(&mut self);
    pub fn reset_mem(&mut self);                // remet a zero uniquement les adresses touchees (rapide); vide aussi le journal
}
impl Default for FlatBus { ... }
impl Bus for FlatBus { ... }
```
- `read(addr)`: masque `addr & 0xFFFFFF`, ajoute `Cycle { addr, val, kind: Read }` au journal, renvoie la valeur. `write(addr, v)`: ecrit, memorise l'adresse touchee, journalise `Write`. `idle()`: journalise `Cycle { addr: 0, val: 0, kind: Idle }`.
- `nmi_pending()`: renvoie `self.nmi` et le remet a false. `irq_asserted()`: renvoie `self.irq`.
- Allouer la memoire en une fois (`vec![0u8; 1 << 24]`).""",
  tests=[("read_write_log", "write(0x7E1234, 0x5A) puis read(0x7E1234)==0x5A ; log == [Write{0x7E1234,0x5A}, Read{0x7E1234,0x5A}]."),
         ("idle_log", "idle() ajoute Cycle{addr:0,val:0,kind:Idle}."),
         ("poke_peek", "poke(0x001000, &[1,2,3]) -> peek(0x1001)==2 ; le journal reste vide."),
         ("addr_mask", "read(0x1000005) lit la meme case que 0x000005 (apres poke(5,&[9]) -> 9)."),
         ("poke_wrap", "poke(0xFFFFFF, &[7,8]) -> peek(0xFFFFFF)==7 et peek(0)==8."),
         ("reset_mem", "poke(0x10,&[5]) ; reset_mem() -> peek(0x10)==0 et journal vide."),
         ("nmi_consumed", "nmi=true: nmi_pending() renvoie true puis false ; irq=true: irq_asserted() renvoie true deux fois de suite.")],
  notes=CHAIN)

T("E02-07", "addr.rs: Ea, read_data/write_data, read_operand, rmw et squelettes de modes",
  deps=["E02-06"], size=120,
  context="""Couche d'acces aux operandes partagee par toutes les instructions. Les trois groupes de modes (absolu, direct, indirect)
sont dans des sous-fichiers remplis par les 3 taches suivantes; ici on pose l'interface et les fonctions communes.""",
  embed=["01_CPU_65816.md#3.2 Acces donnee", "01_CPU_65816.md#3.3 Instructions", EMB_TEST],
  write=[CPU + "/addr.rs", CPU + "/addr/*.rs", TST + "/e02_07_addr_core.rs"],
  spec="""Dans `cpu/addr.rs` (declare `mod absolute; mod direct; mod indirect;`):
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ea { pub lo: u32, pub hi: u32 }       // adresses 24 bits de l'octet bas et de l'octet haut
pub fn ea<B: Bus>(cpu: &mut Cpu, bus: &mut B, am: Am, store: bool) -> Ea;
pub fn read_data<B: Bus>(cpu: &mut Cpu, bus: &mut B, ea: &Ea, wide: bool) -> u16;
pub fn write_data<B: Bus>(cpu: &mut Cpu, bus: &mut B, ea: &Ea, v: u16, wide: bool);
pub fn read_operand<B: Bus>(cpu: &mut Cpu, bus: &mut B, am: Am, wide: bool) -> u16;
pub fn rmw<B: Bus, F: FnOnce(&mut Cpu, u16, bool) -> u16>(cpu: &mut Cpu, bus: &mut B, am: Am, f: F);
```
- `ea`: `store` = vrai si l'instruction ecrit (STA STX STY STZ et tous les RMW): force le cycle `IO` supplementaire des modes indexes. Dispatch: `Abs AbsX AbsY Long LongX` -> `absolute::ea(cpu,bus,am,store)` ; `Dp DpX DpY Sr` -> `direct::ea(..)` ; `DpInd DpIndLong DpXInd DpIndY DpIndLongY SrIndY` -> `indirect::ea(..)` ; tout autre mode -> `panic!("ea: mode {:?} has no memory operand", am)`.
- Sous-fichiers `addr/absolute.rs`, `addr/direct.rs`, `addr/indirect.rs`: chacun contient `pub(super) fn ea<B: Bus>(cpu: &mut Cpu, bus: &mut B, am: Am, store: bool) -> Ea { unimplemented!("E02-xx") }` (remplis plus tard).
- `read_data`: `bus.read(ea.lo)` puis, si `wide`, `bus.read(ea.hi)` -> valeur `lo | hi << 8`. `write_data`: ecrit `lo` puis (si `wide`) `hi`.
- `read_operand`: si `am == Am::Imm`: `fetch16` si `wide` sinon `fetch8` (zero-etendu); sinon `let e = ea(cpu,bus,am,false); read_data(cpu,bus,&e,wide)`.
- `rmw` (lecture-modification-ecriture, 01 section 3.3): `wide = !cpu.m8()`.
  * `Am::Acc`: `bus.idle()`; `old` = A (16 bits si wide, sinon octet bas); `new = f(cpu, old, wide)`; ecrire: wide -> `a = new`; sinon `a = (a & 0xFF00) | (new & 0xFF)` (l'octet haut B est conserve).
  * Sinon: `let e = ea(cpu,bus,am,true)`; lire `lo` (puis `hi` si wide); `bus.idle()`; `new = f(cpu, old, wide)`; si wide: ecrire d'abord l'octet HAUT (`ea.hi`), puis l'octet bas (`ea.lo`).
  * `f` recoit `wide` pour savoir s'il traite 8 ou 16 bits (et met les flags lui-meme).""",
  tests=[("read_data_8", "FlatBus: poke(0x1000,&[0x34,0x12]); Ea{lo:0x1000,hi:0x1001}; read_data(wide=false)==0x34 ; log = 1 lecture."),
         ("read_data_16", "meme Ea, wide=true -> 0x1234 ; log = [Read 0x1000, Read 0x1001] dans cet ordre."),
         ("write_data_16", "write_data(Ea{lo:0x2000,hi:0x2001}, 0xABCD, wide=true): peek(0x2000)==0xCD, peek(0x2001)==0xAB ; log = [Write 0x2000=0xCD, Write 0x2001=0xAB]."),
         ("read_operand_imm_8", "pb=0, pc=0x8000, poke(0x8000,&[0x7F,0x11]); read_operand(Imm, wide=false)==0x7F, pc==0x8001, log = 1 lecture en 0x008000."),
         ("read_operand_imm_16", "idem avec wide=true -> 0x117F, pc==0x8002, log = lectures 0x008000 puis 0x008001."),
         ("rmw_acc_8bit", "e=false, p=0x20 (M=1), a=0x12FF; rmw(Acc, |_c,v,_w| (v+1)&0xFF) -> a==0x1200 ; log = [Idle] seulement."),
         ("rmw_acc_16bit", "e=false, p=0x00, a=0x00FF; rmw(Acc, |_c,v,w| { assert!(w); v+1 }) -> a==0x0100 ; log = [Idle]."),
         ("ea_panics_for_imm", "`#[should_panic]`: ea(cpu,bus,Am::Imp,false) panique.")],
  notes=CHAIN)

T("E02-08", "addr/absolute.rs: modes Abs, AbsX, AbsY, Long, LongX",
  deps=["E02-07"], size=70,
  context="Modes d'adressage absolus. Chaque mode emet ses cycles de bus dans l'ordre exact de la spec.",
  embed=["01_CPU_65816.md#3.1 Calcul d'adresse"],
  tsv=tsv("LDA"),
  write=[CPU + "/addr/absolute.rs", TST + "/e02_08_addr_absolute.rs"],
  spec="""Implementer `pub(super) fn ea<B: Bus>(cpu, bus, am, store) -> Ea` pour (apres le cycle opcode, le PC pointe sur l'operande):
- `Am::Abs`: `aa = cpu.fetch16(bus)`; `base = (db << 16) | aa`; `lo = base`, `hi = (base + 1) & 0xFFFFFF`.
- `Am::AbsX` / `Am::AbsY`: meme base; `idx = cpu.x` (resp. `cpu.y`); `addr = (base + idx) & 0xFFFFFF`; `cross = (base >> 8) != (addr >> 8)`; cycle `bus.idle()` SI `store || !cpu.x8() || cross` (note: pour Y comme pour X c'est le flag X qui compte); `lo = addr`, `hi = (addr + 1) & 0xFFFFFF`.
- `Am::Long`: `v = cpu.fetch24(bus)`; `lo = v`, `hi = (v + 1) & 0xFFFFFF`.
- `Am::LongX`: `v = fetch24`; `addr = (v + cpu.x) & 0xFFFFFF`; aucun cycle supplementaire; `hi = (addr + 1) & 0xFFFFFF`.
- Tout autre mode: `unreachable!()`.
L'ordre des lectures d'operande est octet bas puis octet haut. Aucune lecture de la donnee elle-meme ici (c'est `read_data`).""",
  tests=[("abs", "e=false, db=0x7E, pc=0x8000, poke(0x8000,&[0x34,0x12]): ea(Abs,false) == Ea{lo:0x7E1234, hi:0x7E1235} ; log = 2 lectures (0x008000, 0x008001), pc==0x8002."),
         ("abs_bank_cross", "db=0x01, operande 0xFFFF: Ea{lo:0x01FFFF, hi:0x020000}."),
         ("absx_8bit_no_cross", "e=false, p=0x10 (X=1), x=0x05, db=0, operande 0x1210: lo==0x001215 ; log = 2 lectures (aucun Idle)."),
         ("absx_8bit_cross", "meme p, x=0x20, operande 0x12F0: lo==0x001310 ; log = 2 lectures + 1 Idle (en dernier)."),
         ("absx_store_no_cross", "X=1, x=0x05, operande 0x1210, store=true: log = 2 lectures + 1 Idle."),
         ("absx_16bit_index", "e=false, p=0x00 (X=0), x=0x0001, operande 0x1210, store=false: lo==0x001211 ; log = 2 lectures + 1 Idle."),
         ("absx_carry_to_bank", "p=0x00, db=0x01, x=0x0020, operande 0xFFF0: lo==0x020010, hi==0x020011."),
         ("absy", "p=0x10, y=0x03, operande 0x2000, db=0: lo==0x002003, log = 2 lectures."),
         ("long", "poke(0x8000,&[0x56,0x34,0x12]): ea(Long) == Ea{lo:0x123456, hi:0x123457} ; log = 3 lectures ; avec [0xFF,0xFF,0x12]: hi==0x130000."),
         ("longx", "p=0x00, x=0x0010, operande [0x56,0x34,0x12]: lo==0x123466 ; log = 3 lectures seulement."),
         ("rmw_abs_8bit", "e=false, p=0x20, db=0, poke(0x8000,&[0x34,0x12]), poke(0x1234,&[0x41]); `rmw(cpu,bus,Am::Abs,|_,v,_| v+1)`: peek(0x1234)==0x42 ; log = [R 0x8000, R 0x8001, R 0x1234, Idle, W 0x1234=0x42]."),
         ("rmw_abs_16bit", "p=0x00, poke(0x1234,&[0xFF,0x00]); rmw avec |_,v,_| v+1: peek(0x1234)==0x00, peek(0x1235)==0x01 ; les 4 derniers cycles du journal: R 0x1234, R 0x1235, Idle, W 0x1235=0x01, W 0x1234=0x00 (5 cycles apres les 2 lectures d'operande : ecriture octet HAUT d'abord).")],
  notes=CHAIN)

T("E02-09", "addr/direct.rs: modes Dp, DpX, DpY, Sr",
  deps=["E02-08"], size=80,
  context="Modes direct page et pile relative, avec les regles de wrap du mode emulation.",
  embed=["01_CPU_65816.md#3.1 Calcul d'adresse", "01_CPU_65816.md#4. Regles de wrap"],
  write=[CPU + "/addr/direct.rs", TST + "/e02_09_addr_direct.rs"],
  spec="""Implementer `ea` pour (toutes les adresses de ces modes sont en banque 0, donc < 0x10000):
- `Am::Dp`: `dl = fetch8`; si `cpu.d & 0xFF != 0` -> `bus.idle()`; `lo = (cpu.d as u32 + dl as u32) & 0xFFFF`; `hi = (lo + 1) & 0xFFFF`.
- `Am::DpX` / `Am::DpY`: `dl = fetch8`; si `cpu.d & 0xFF != 0` -> `bus.idle()`; PUIS toujours `bus.idle()`; `idx` = x (resp. y).
  * Si `cpu.e && (cpu.d & 0xFF) == 0` (page wrap): `lo = (d & 0xFF00) | ((dl + idx) & 0xFF)`, `hi = (d & 0xFF00) | ((dl + idx + 1) & 0xFF)` (arithmetique en u32 puis masque).
  * Sinon: `lo = (d + dl + idx) & 0xFFFF`, `hi = (lo + 1) & 0xFFFF`.
- `Am::Sr`: `sr = fetch8`; `bus.idle()`; `lo = (s + sr) & 0xFFFF`; `hi = (lo + 1) & 0xFFFF`.
- Autre mode: `unreachable!()`.
`store` n'a pas d'effet ici.""",
  tests=[("dp", "e=false, d=0x0000, pc=0x8000, operande 0x10: Ea{lo:0x0010, hi:0x0011} ; log = 1 lecture (pas d'Idle)."),
         ("dp_dl_nonzero", "d=0x0001, operande 0x10: lo==0x0011 ; log = [Read, Idle]."),
         ("dp_wrap_16", "e=false, d=0xFFF0, operande 0x20: lo==0x0010, hi==0x0011 (wrap sur 64 KiB, banque 0)."),
         ("dpx_native", "e=false, p=0x00 (X=0), d=0x2000, x=0x0003, operande 0x05: lo==0x2008 ; log = [Read, Idle] (un seul Idle car DL=0)."),
         ("dpx_native_dl", "d=0x2001, x=3, operande 0x05: lo==0x2009 ; log = [Read, Idle, Idle]."),
         ("dpx_native_16bit_index", "e=false, p=0x00, d=0, x=0x0120, operande 0xF0: lo==0x0210 (pas de wrap sur 8 bits en natif)."),
         ("dpx_emulation_page_wrap", "e=true, p=0x30, d=0x1200, x=0x20, operande 0xF0: lo==0x1210 ; avec x=0x00 et operande 0xFF: lo==0x12FF, hi==0x1200."),
         ("dpx_emulation_dl_nonzero", "e=true, d=0x1201, x=0x20, operande 0xF0: lo==0x1311 (pas de wrap de page car DL != 0)."),
         ("dpy", "e=false, p=0x00, d=0, y=0x0004, operande 0x10: lo==0x0014 ; log = [Read, Idle]."),
         ("sr", "e=false, s=0x01FD, operande 0x03: Ea{lo:0x0200, hi:0x0201} ; log = [Read, Idle]."),
         ("sr_wrap", "s=0xFFFF, operande 0x02: lo==0x0001.")],
  notes=CHAIN)

T("E02-10", "addr/indirect.rs: modes (d) [d] (d,x) (d),y [d],y (d,s),y",
  deps=["E02-09"], size=130,
  context="Modes indirects. Les regles de wrap de page en emulation s'appliquent aux pointeurs des modes (d), (d,x), (d),y; pas aux modes longs [d] et [d],y.",
  embed=["01_CPU_65816.md#3.1 Calcul d'adresse", "01_CPU_65816.md#4. Regles de wrap"],
  write=[CPU + "/addr/indirect.rs", TST + "/e02_10_addr_indirect.rs"],
  spec="""Implementer `ea`. Notation: `D` = cpu.d, `DL` = D & 0xFF, `page_wrap` = `cpu.e && DL == 0`. Les lectures de pointeur sont des `bus.read` en banque 0 (adresse < 0x10000).
Fonction interne `fn ptr_addrs(cpu, off: u32, long: bool) -> (u32, u32, u32)` qui renvoie les adresses des octets 0, 1, 2 du pointeur pour un decalage `off` (= dl, ou dl+X):
  * `!long && page_wrap`: `p0 = (D & 0xFF00) | (off & 0xFF)`, `p1 = (D & 0xFF00) | ((off + 1) & 0xFF)`.
  * sinon: `p0 = (D + off) & 0xFFFF`, `p1 = (p0 + 1) & 0xFFFF`, `p2 = (p0 + 2) & 0xFFFF`.
Modes (apres l'opcode):
- `DpInd` (d): `dl = fetch8`; si DL != 0 -> idle; lire `ptr` (lo en p0 puis hi en p1); `base = (db << 16) | ptr`; `lo = base`, `hi = (base + 1) & 0xFFFFFF`.
- `DpIndLong` [d]: `dl = fetch8`; si DL != 0 -> idle; lire 3 octets (p0, p1, p2; `long = true`); `v = bank << 16 | ptr`; `lo = v`, `hi = (v + 1) & 0xFFFFFF`.
- `DpXInd` (d,x): `dl = fetch8`; si DL != 0 -> idle; `bus.idle()`; offset = `dl + X`; lire ptr (p0, p1 avec la regle ci-dessus appliquee a `off = dl + X`); `base = (db << 16) | ptr`; `lo/hi` comme DpInd.
- `DpIndY` (d),y: `dl = fetch8`; si DL != 0 -> idle; lire ptr comme DpInd; `base = (db << 16) | ptr`; `addr = (base + Y) & 0xFFFFFF`; `cross = (base >> 8) != (addr >> 8)`; `bus.idle()` SI `store || !cpu.x8() || cross`; `lo = addr`, `hi = (addr + 1) & 0xFFFFFF`.
- `DpIndLongY` [d],y: comme DpIndLong puis `addr = (v + Y) & 0xFFFFFF`; aucun cycle supplementaire; `lo = addr`, `hi = (addr + 1) & 0xFFFFFF`.
- `SrIndY` (d,s),y: `sr = fetch8`; `bus.idle()`; `p0 = (s + sr) & 0xFFFF`, `p1 = (p0 + 1) & 0xFFFF`; lire ptr; `bus.idle()`; `addr = ((db << 16 | ptr) + Y) & 0xFFFFFF`; `lo = addr`, `hi = (addr + 1) & 0xFFFFFF`.
- Autre mode: `unreachable!()`.""",
  tests=[("dp_ind", "e=false, d=0, db=0x7E, poke(0x10,&[0x34,0x12]), operande 0x10: Ea{lo:0x7E1234, hi:0x7E1235} ; log = [R operande, R 0x0010, R 0x0011]."),
         ("dp_ind_dl", "d=0x0101 (DL=1), poke(0x111,&[0x34,0x12]) operande 0x10: lo==db<<16|0x1234 ; log = [Read, Idle, Read, Read]."),
         ("dp_ind_emulation_wrap", "e=true, p=0x30, d=0x1200, db=0x01, operande 0xFF, poke(0x12FF,&[0x78]), poke(0x1200,&[0x56]): lo==0x015678."),
         ("dp_ind_long", "e=false, d=0, poke(0x20,&[0xFF,0xFF,0x12]), operande 0x20: Ea{lo:0x12FFFF, hi:0x130000} ; log = [Read, Read, Read, Read] (operande + 3 octets)."),
         ("dp_ind_long_bank0_wrap", "d=0xFFFF, operande 0x00 (adresse pointeur 0xFFFF+0 = 0xFFFF): poke(0xFFFF,&[0x11]), poke(0x0000,&[0x22,0x33]) -> lo==0x332211 ; log contient un Idle (DL=0xFF != 0)."),
         ("dp_x_ind", "e=false, p=0x00, d=0, x=0x0004, operande 0x10, poke(0x14,&[0x00,0x90]), db=2: lo==0x029000 ; log = [Read, Idle, Read, Read]."),
         ("dp_x_ind_emulation_wrap", "e=true, p=0x30, d=0, x=0x01, operande 0xFE, poke(0xFF,&[0x34]), poke(0x00,&[0x12]), db=0: lo==0x001234."),
         ("dp_ind_y_no_cross", "e=false, p=0x10 (X=1), d=0, db=5, poke(0x30,&[0x00,0x20]), y=0x10, operande 0x30: lo==0x052010 ; log = 3 lectures (aucun Idle)."),
         ("dp_ind_y_cross", "idem avec pointeur 0x20F8 (poke(0x30,&[0xF8,0x20])): lo==0x052108 ; log = 3 lectures + Idle en dernier."),
         ("dp_ind_y_store", "pointeur 0x2000, y=0x10, store=true: log = 3 lectures + Idle."),
         ("dp_ind_y_16bit_index", "p=0x00 (X=0), y=0x0001, pointeur 0x2000, store=false: log = 3 lectures + Idle ; lo==db<<16|0x2001."),
         ("dp_ind_long_y", "p=0x00, poke(0x40,&[0xF0,0xFF,0x01]), y=0x0020, operande 0x40: lo==0x020010, hi==0x020011 ; log = 4 lectures, aucun Idle."),
         ("sr_ind_y", "e=false, p=0x10, s=0x01F0, operande 0x04, poke(0x01F4,&[0x00,0x30]), db=0, y=0x02: lo==0x003002 ; log = [Read(operande), Idle, Read 0x01F4, Read 0x01F5, Idle].")],
  notes=CHAIN)

T("E02-11", "CLI: enregistrer les sous-commandes cpu-test, spc-test et suite (squelettes)",
  deps=["E01-07"], size=100,
  context="""Une seule tache modifie `main.rs` pour declarer toutes les futures sous-commandes de test; ensuite chaque
commande vit dans son propre fichier. Les commandes sont des squelettes qui rendent 2 (non implemente).""",
  embed=["06_TEST_ROMS.md#6. Commande"],
  write=[CLI + "/src/main.rs", CLI + "/src/cpu_test.rs", CLI + "/src/cpu_test/*.rs", CLI + "/src/spc_test.rs", CLI + "/src/spc_test/*.rs",
         CLI + "/src/suite.rs", CLI + "/src/suite/*.rs", CLI + "/tests/e02_11_subcommands.rs"],
  spec="""1. Dans `main.rs` ajouter les sous-commandes clap (en gardant `run`):
   - `cpu-test <DIR> [--filter <LISTE>]` -> `cpu_test::run(dir: &Path, filter: Option<&str>) -> i32`
   - `spc-test <DIR> [--filter <LISTE>]` -> `spc_test::run(dir: &Path, filter: Option<&str>) -> i32`
   - `suite <MANIFEST> [--max-epic <E..>] [--update-baseline <NOM>] [--dump-ppm <DIR>]` -> `suite::run(manifest: &Path, max_epic: Option<&str>, update_baseline: Option<&str>, dump_ppm: Option<&Path>) -> i32`
   Le code de sortie du processus est la valeur renvoyee (`std::process::exit`).
2. Declarer `mod cpu_test; mod spc_test; mod suite;` dans `main.rs`.
3. `cpu_test.rs` declare `mod oracle; mod compare;` ; fichiers `cpu_test/oracle.rs` et `cpu_test/compare.rs` contenant seulement `// stub: filled by a later task`. `spc_test.rs` declare `mod oracle; mod compare;` (memes stubs). `suite.rs` declare `mod manifest; mod baseline; mod runner; mod report;` avec 4 stubs.
4. Chaque `run` squelette: si le chemin donne n'existe pas -> message sur stderr `path not found: <chemin>` et renvoie 2 ; sinon message `not implemented yet` sur stderr et renvoie 2.
5. Mettre `#![allow(dead_code)]` en tete de `cpu_test.rs`, `spc_test.rs`, `suite.rs` tant que les modules internes sont vides.""",
  tests=[("help_lists_commands", "`puce16-cli --help`: stdout contient `cpu-test`, `spc-test`, `suite`, `run`."),
         ("cpu_test_missing_dir", "`cpu-test /nonexistent/dir` -> code de sortie 2 et stderr contient `path not found`."),
         ("spc_test_missing_dir", "`spc-test /nonexistent/dir` -> code 2."),
         ("suite_missing_manifest", "`suite /nonexistent/manifest.toml` -> code 2."),
         ("run_still_works", "`run --test-pattern --frames 1 --hash` -> code 0 et stdout contient `frame_hash=0x`.")],
  accept=["cargo test -p puce16-cli e02_11_", "cargo test -p puce16-cli e01_07_"], crate="puce16-cli", notes=CHAIN)

T("E02-12", "cpu-test: chargeur des fichiers JSON de l'oracle",
  deps=["E02-03", "E02-11"], size=130,
  context="""Lecture des fichiers de l'oracle 65816. Le format exact est dans `docs/specs/data/ssjson_format.md` (valide par Claude en E02-03):
ECRIRE LE PARSEUR UNIQUEMENT A PARTIR DE CE DOCUMENT, jamais de memoire.""",
  reads=["specs/data/ssjson_format.md (OBLIGATOIRE, c'est la reference du format)", "specs/06_TEST_ROMS.md section 3"],
  write=[CLI + "/src/cpu_test/oracle.rs", CLI + "/tests/data/e02_12_sample.json"],
  spec="""Dans `cpu_test/oracle.rs` (`puce16-cli` est un binaire: les tests sont des tests unitaires `#[cfg(test)] mod tests` DANS `oracle.rs`, fonctions nommees `e02_12_...`; l'exemple JSON est lu avec `include_str!("../../tests/data/e02_12_sample.json")`) :
```rust
pub struct CpuState { pub pc: u16, pub s: u16, pub p: u8, pub a: u16, pub x: u16, pub y: u16, pub dbr: u8, pub d: u16, pub pbr: u8, pub e: bool, pub ram: Vec<(u32, u8)> }
pub struct CycleRec { pub addr: Option<u32>, pub val: Option<u8>, pub kind: String }   // champs = ceux du document de format
pub struct Case { pub name: String, pub initial: CpuState, pub fin: CpuState, pub cycles: Vec<CycleRec> }
pub fn load_file(path: &std::path::Path) -> Result<Vec<Case>, String>;
pub fn list_files(dir: &std::path::Path, filter: Option<&str>) -> Result<Vec<std::path::PathBuf>, String>;
pub fn opcode_of_file(name: &str) -> Option<u8>;
```
Les noms de champs/cles JSON sont ceux du document de format (si une cle differe de ce qui est ecrit ci-dessus, le document gagne et la struct s'adapte, en gardant les memes noms Rust).
- `list_files`: tous les `.json` (recursif), tries par nom; `filter` = liste d'opcodes en hexa separes par des virgules (`61,63,E9`); un fichier est garde si `opcode_of_file(nom)` est dans la liste (ou si `filter` est None). Erreur `String` si le dossier ne se lit pas.
- `opcode_of_file`: convention de nommage du document (ex: les deux premiers chiffres hexa du nom).
- `load_file`: erreurs en `Err(String)` avec le nom du fichier; aucun `unwrap` sur des donnees externes.
- Utiliser `serde` + `serde_json` (deja dans le Cargo.toml).""",
  tests=[("load_sample", "Le fichier `tests/data/e02_12_sample.json` contient UN test copie tel quel depuis un vrai fichier de l'oracle (section 5 du document de format). `load_file` renvoie 1 cas ; verifier au moins 3 registres initiaux, 1 valeur de RAM finale et le nombre de cycles, avec les VRAIES valeurs de l'exemple."),
         ("load_bad_json", "un fichier contenant `not json` -> Err(message contenant le nom du fichier)."),
         ("opcode_of_file", "au moins 4 noms de fichiers reels (copies de la liste de `tests/oracle/65816`) -> opcode attendu ; un nom sans rapport (`readme.md`) -> None."),
         ("list_files_filter", "dossier temporaire avec 3 faux fichiers nommes selon la convention (opcodes 61, 69, E9): `filter=Some(\"61,E9\")` -> 2 fichiers tries ; `filter=None` -> 3 ; dossier absent -> Err.")],
  accept=["cargo test -p puce16-cli e02_12_"], crate="puce16-cli", notes=CHAIN)

T("E02-13", "cpu-test: comparaison d'etats et de cycles (fonctions pures)",
  deps=["E02-12"], size=130,
  context="""Compare l'etat obtenu apres `Cpu::step` avec l'etat attendu de l'oracle. Fonctions pures testables sans instruction implementee.""",
  reads=["specs/data/ssjson_format.md", "specs/06_TEST_ROMS.md section 3 (strategie de comparaison)"],
  write=[CLI + "/src/cpu_test/compare.rs"],
  spec="""Dans `cpu_test/compare.rs` (tests unitaires dans le meme fichier, noms `e02_13_...`):
```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Level { Regs, Count, Full }
pub fn diff_state(expected: &CpuState, cpu: &Cpu, bus: &FlatBus) -> Vec<String>;
pub fn diff_cycles(expected: &[CycleRec], got: &[Cycle], level: Level) -> Vec<String>;
pub fn run_case(case: &Case, cpu: &mut Cpu, bus: &mut FlatBus, level: Level) -> Vec<String>;   // liste vide = succes
```
- `diff_state`: pour chaque champ different, une ligne `a: expected 0x1234 got 0x1230` (hexa; `e` en 0/1; `p` sur 2 chiffres). Compare a, x, y, s, d, db(dbr), pb(pbr), pc, p, e puis, pour chaque `(addr, val)` de `expected.ram`, `bus.peek(addr)` (ligne `ram[0x7E1234]: expected 0x41 got 0x42`).
- `diff_cycles`: `Level::Regs` -> toujours vide; `Level::Count` -> seulement la difference de nombre (`cycles: expected 7 got 6`); `Level::Full` -> nombre, puis pour chaque cycle d'index i: le type (lecture/ecriture/interne) doit correspondre, et pour les cycles NON internes l'adresse et la valeur aussi (les adresses des cycles internes ne sont pas comparees, voir document de format section 4). Ligne `cycle 3: expected W 0x001000=0x42 got R 0x001000=0x41`. Au plus 5 lignes de diff.
- `run_case`: `bus.reset_mem()`; poke de `initial.ram`; copier l'etat initial dans `cpu` (champs un a un, `halted=false`, `waiting=false`); `bus.clear_log()`; `cpu.step(bus)`; puis concatener `diff_state` et `diff_cycles(.., level)`. Un `std::panic::catch_unwind` autour du `step` convertit une panique en une ligne `panic: <message>`.""",
  tests=[("diff_state_equal", "un `CpuState` et un `Cpu` identiques (memes valeurs) + RAM identique -> liste vide."),
         ("diff_state_fields", "Cpu avec a=0x1230 attendu 0x1234 et pc different -> 2 lignes, la premiere commence par `a: expected 0x1234 got 0x1230`."),
         ("diff_state_ram", "RAM attendue (0x7E1234, 0x41) mais bus contient 0x42 -> 1 ligne contenant `ram[0x7E1234]`."),
         ("diff_cycles_count", "attendu 3 cycles, obtenu 2, Level::Count -> 1 ligne contenant `expected 3 got 2` ; Level::Regs -> vide."),
         ("diff_cycles_full_ok", "attendu [R 0x8000=0xEA, interne], obtenu [Read 0x8000=0xEA, Idle] -> vide."),
         ("diff_cycles_full_mismatch", "attendu W 0x1000=0x42, obtenu Read 0x1000=0x41 -> 1 ligne contenant `cycle 0`."),
         ("diff_cycles_max_5", "10 cycles tous faux -> exactement 5 lignes.")],
  accept=["cargo test -p puce16-cli e02_13_"], crate="puce16-cli", notes=CHAIN)

T("E02-14", "cpu-test: commande complete (boucle sur les fichiers, rapport, code de sortie)",
  deps=["E02-13", "E02-05", "E02-06"], size=120,
  context="La commande `puce16-cli cpu-test <DIR> [--filter ..]` utilisable par toutes les taches de verification CPU suivantes.",
  embed=["06_TEST_ROMS.md#3. Oracle SingleStepTests"],
  reads=["specs/data/ssjson_format.md"],
  write=[CLI + "/src/cpu_test.rs", CLI + "/tests/e02_14_cpu_test_cmd.rs", CLI + "/tests/data/e02_14_*.json"],
  spec="""`cpu_test::run(dir, filter) -> i32`:
1. `list_files(dir, filter)`; erreur de lecture du dossier ou aucun fichier retenu -> message sur stderr et code 2.
2. Pour chaque fichier (ordre trie): `load_file`; pour chaque cas: `run_case(case, &mut cpu, &mut bus, Level::Full)` avec UN seul `Cpu` et UN seul `FlatBus` reutilises (`FlatBus::reset_mem` fait la remise a zero). Une erreur de lecture du fichier compte comme un fichier en echec.
3. Sortie stdout: par fichier une ligne `OK <nom> (<n> tests)` ou `FAIL <nom> <k>/<n>`; apres un `FAIL`, au plus 5 echecs detailles: ligne `  <nom du test>` puis les lignes de diff indentees de 4 espaces.
4. Fin: ligne `files: <total> ok: <ok> failed: <ko>`. Code de sortie 0 si aucun echec, 1 sinon, 2 pour erreur d'usage.
5. Un cas dont `step` panique est un echec (le message `panic:` apparait dans les diffs); la commande ne doit jamais s'arreter sur une panique.
6. Le `--filter` est documente dans `--help`: `liste d'opcodes en hexa separes par des virgules, ex 61,63`.""",
  tests=[("wrong_expectation_fails", "Dossier temporaire contenant un fichier JSON (schema du document de format) dont UN test attend, apres un opcode NOP (0xEA) en mode emulation, un `pc` volontairement faux (0x1234): la commande sort avec le code 1, stdout contient `FAIL` et `pc: expected 0x1234`."),
         ("empty_dir_usage", "dossier existant sans aucun .json -> code 2."),
         ("filter_no_match", "dossier avec un seul fichier d'opcode EA et `--filter 61` -> code 2."),
         ("summary_line", "stdout contient une ligne commencant par `files: 1 ok: 0 failed: 1` pour le test ci-dessus."),
         ("bad_json_is_failure", "fichier invalide -> code 1 (et stdout contient `FAIL`).")],
  accept=["cargo test -p puce16-cli e02_14_", "cargo test -p puce16-cli e02_11_"], crate="puce16-cli",
  notes=CHAIN + "\nLes JSON des tests sont fabriques en suivant EXACTEMENT le document de format.")

# --- load / store + premiere verification oracle des modes d'adressage

T("E02-15", "mem/load.rs: LDA, LDX, LDY (tous modes)",
  deps=["E02-10", "E02-05"], size=60,
  context="Premiere famille d'instructions. Sert aussi de test grandeur nature de tous les modes d'adressage.",
  embed=["01_CPU_65816.md#5.1 Taille des operandes", "01_CPU_65816.md#5.2 Flags", EMB_TEST],
  tsv=tsv("LDA", "LDX", "LDY"), reads=READS,
  write=[CPU + "/mem/load.rs", TST + "/e02_15_load.rs"],
  spec="""Dans `mem/load.rs`, `exec` traite `Mn::Lda`, `Mn::Ldx`, `Mn::Ldy` (sinon `return false`):
- LDA: `wide = !cpu.m8()`; `v = addr::read_operand(cpu, bus, am, wide)`; si `wide` `cpu.a = v` sinon `cpu.a = (cpu.a & 0xFF00) | v` ; `cpu.set_nz(v, wide)`.
- LDX / LDY: `wide = !cpu.x8()`; meme lecture; `cpu.x = v` (resp. `y`) (en 8 bits la valeur est `v` zero-etendue, l'octet haut est 0); `set_nz(v, wide)`.
- Retourne `true`. Les cycles viennent des modes (aucun cycle a ajouter).""",
  tests=[("lda_imm_8_keeps_high_byte", "e=false, p=0x20, a=0x1234, `A9 80`: a==0x1280, N=1,Z=0 ; cycles = 2."),
         ("lda_imm_16_zero", "p=0x00, a=0x5555, `A9 00 00`: a==0x0000, Z=1,N=0 ; cycles = 3."),
         ("lda_abs", "p=0x20, db=0, poke(0x2000,&[0x42]), `AD 00 20`: A.l==0x42 ; cycles = 4 ; dernier cycle = Read 0x002000."),
         ("lda_long", "p=0x20, poke(0x123456,&[0x99]), `AF 56 34 12`: A.l==0x99, N=1 ; cycles = 5."),
         ("lda_dp_16", "p=0x00, d=0, poke(0x10,&[0x34,0x12]), `A5 10`: a==0x1234 ; cycles = 4."),
         ("lda_ind_y", "p=0x30 (M=1,X=1), poke(0x30,&[0x00,0x20]), db=0, poke(0x2010,&[0x77]), y=0x10, `B1 30`: A.l==0x77 ; cycles = 5."),
         ("lda_sr_ind_y", "p=0x20, s=0x01F0, poke(0x01F4,&[0x00,0x30]), poke(0x3002,&[0x5A]), y=0x0002 (p X=0 -> p=0x20), `B3 04`: A.l==0x5A ; cycles = 7."),
         ("ldx_imm_8", "p=0x30, x=0x0034, `A2 FF`: x==0x00FF, N=1 ; cycles = 2."),
         ("ldx_imm_16", "p=0x00, `A2 34 12`: x==0x1234 ; cycles = 3."),
         ("ldy_dpx", "p=0x30, d=0, x=2, poke(0x12,&[0x77]), `B4 10`: y==0x77 ; cycles = 4."),
         ("lda_emulation", "e=true, p=0x30, `A9 80`: A.l==0x80, N=1, cycles = 2.")],
  notes=CHAIN)

T("E02-16", "mem/store.rs: STA, STX, STY, STZ (tous modes)",
  deps=["E02-15"], size=70,
  context="Famille d'ecriture. Utilise `ea(.., store=true)` (cycle IO force pour les modes indexes).",
  embed=["01_CPU_65816.md#3.2 Acces donnee", EMB_TEST],
  tsv=tsv("STA", "STX", "STY", "STZ"), reads=READS,
  write=[CPU + "/mem/store.rs", TST + "/e02_16_store.rs"],
  spec="""`exec` traite `Mn::Sta`, `Mn::Stx`, `Mn::Sty`, `Mn::Stz`:
- valeur: STA -> `cpu.a`; STX -> `cpu.x`; STY -> `cpu.y`; STZ -> 0.
- largeur: STA et STZ `wide = !cpu.m8()`; STX et STY `wide = !cpu.x8()`.
- `let e = addr::ea(cpu, bus, am, true); addr::write_data(cpu, bus, &e, v, wide);` Aucun flag modifie. Retourne `true`.""",
  tests=[("sta_abs_8", "e=false, p=0x20, a=0x125A, `8D 00 20`: peek(0x2000)==0x5A ; cycles = 4 ; dernier cycle = Write 0x002000=0x5A ; P inchange."),
         ("sta_abs_16", "p=0x00, a=0x1234, `8D 00 20`: peek(0x2000)==0x34, peek(0x2001)==0x12 ; cycles = 5 ; ecritures dans l'ordre 0x2000 puis 0x2001."),
         ("sta_abs_x_8", "p=0x30, x=1, a=0x07, `9D 00 20`: peek(0x2001)==0x07 ; cycles = 5 (Idle force)."),
         ("sta_ind_y", "p=0x30, d=0, poke(0x30,&[0x00,0x20]), y=0x10, a=0x09, `91 30`: peek(0x2010)==0x09 ; cycles = 6."),
         ("sta_dp", "p=0x20, d=0, `85 10`, a=0x33: peek(0x10)==0x33 ; cycles = 3."),
         ("sta_long", "`8F 56 34 12`, a=0x44, p=0x20: peek(0x123456)==0x44 ; cycles = 5."),
         ("stx_abs_16", "p=0x00, x=0xABCD, `8E 00 20`: peek(0x2000)==0xCD, peek(0x2001)==0xAB ; cycles = 5."),
         ("sty_dp_8", "p=0x30, y=0x12, `84 10`: peek(0x10)==0x12 ; cycles = 3."),
         ("stz_dp_16", "p=0x00, poke(0x10,&[0xFF,0xFF]), `64 10`: peek(0x10)==0 et peek(0x11)==0 ; cycles = 4."),
         ("stz_abs_x", "p=0x20, x=0x0001 (X=0), poke(0x2001,&[0x55]), `9E 00 20`: peek(0x2001)==0 ; cycles = 5."),
         ("flags_untouched", "p=0xE3 -> apres STA/STX/STZ p inchange (e=false).")],
  notes=CHAIN)

T("E02-17", "VERIF oracle: modes d'adressage via LDA/LDX/LDY/STA/STX/STY/STZ",
  deps=["E02-16", "E02-14"], size=60, kind="code",
  context="""Premier passage a l'oracle: tous les tests de LDA/LDX/LDY/STA/STX/STY/STZ. Comme tous les modes d'adressage sont
exerces, les erreurs de cycles/wrap des modes se corrigent ICI (addr/*.rs). Les etapes suivantes (ALU...) reposent sur des modes valides.""",
  reads=["specs/01_CPU_65816.md sections 3 et 4", "specs/data/ssjson_format.md"],
  write=[CPU + "/addr.rs", CPU + "/addr/*.rs", CPU + "/mem/load.rs", CPU + "/mem/store.rs", TST + "/e02_17_oracle_regressions.rs"],
  spec="""1. Lancer: `%s`
2. Pour chaque echec: lire le diff (champ fautif, cycle fautif), localiser le bug dans les fichiers autorises, corriger. Ne pas modifier le harnais (cli) ni les specs.
3. Si le diff semble indiquer que la SPEC est fausse (et non le code): ne PAS adapter le code au hasard; ecrire une question `spec` (docs/02_QA_PROTOCOL.md) avec le test fautif.
4. Pour chaque bug corrige, ajouter dans `tests/e02_17_oracle_regressions.rs` un test unitaire (FlatBus) qui le reproduit (nom `e02_17_<mode>_<cas>`). Si aucun bug n'est trouve, le fichier contient un test `e02_17_oracle_clean` qui verifie seulement que `Cpu::new()` existe.
5. Finir quand la commande renvoie `failed: 0` pour tous ces opcodes.""" % oracle_cmd("LDA", "LDX", "LDY", "STA", "STX", "STY", "STZ"),
  tests=[("oracle_clean_or_regressions", "voir spec etape 4")],
  accept=[oracle_cmd("LDA", "LDX", "LDY", "STA", "STX", "STY", "STZ"), "cargo test -p puce16-core e02_"],
  notes=CHAIN + "\nSi `tests/oracle/65816` est absent (E02-01 non fait), c'est un blocage `tooling`: poser la question.")

# --- ALU

T("E02-18", "alu/adc.rs: ADC binaire (8 et 16 bits, tous modes)",
  deps=["E02-17"], size=60,
  context="ADC en binaire. Le mode decimal (flag D) est ajoute par la tache suivante dans le MEME fichier: ici, si D=1, appeler `adc_decimal(cpu, m, wide)` qui est `unimplemented!(\"E02-19\")`.",
  embed=["01_CPU_65816.md#6. ADC / SBC", EMB_TEST],
  tsv=tsv("ADC"), reads=READS,
  write=[CPU + "/alu/adc.rs", TST + "/e02_18_adc.rs"],
  spec="""`exec` traite `Mn::Adc`: `wide = !cpu.m8()`; `m = addr::read_operand(cpu, bus, am, wide)`; si `cpu.flag(FLAG_D)` -> `adc_decimal(cpu, m, wide)` sinon `adc_binary(cpu, m, wide)`; retourne `true`.
`adc_binary` (fonction privee): `a = cpu.a` (octet bas si 8 bits); `r = a + m + C` (u32); `C = r > mask` (mask 0xFF / 0xFFFF); `V = ((a ^ r) & (m ^ r) & signbit) != 0` (signbit 0x80 / 0x8000); resultat tronque au mask; ecrire dans A (8 bits: l'octet haut de A est conserve); `set_nz(result, wide)`.
Garder `adc_decimal` comme `fn adc_decimal(_cpu: &mut Cpu, _m: u16, _wide: bool) { unimplemented!("E02-19") }`.""",
  tests=[("imm_8_simple", "e=false, p=0x20, a=0x1210, C=0, `69 05`: a==0x1215, N=Z=C=V=0, cycles=2."),
         ("imm_8_carry_out", "p=0x20, a=0x12FF, `69 01`: a==0x1200, C=1, Z=1."),
         ("imm_8_overflow", "p=0x20, a=0x0050, `69 50`: A.l==0xA0, V=1, N=1, C=0."),
         ("imm_8_carry_in", "p=0x20|0x01 (C=1), a=0x0010, `69 20`: A.l==0x31."),
         ("imm_16_overflow", "p=0x00, a=0x7FFF, `69 01 00`: a==0x8000, V=1, N=1, C=0, cycles=3."),
         ("imm_16_carry_zero", "p=0x00, a=0xFFFF, `69 01 00`: a==0x0000, C=1, Z=1, V=0."),
         ("imm_16_neg_neg", "p=0x00, a=0x8000, `69 00 80`: a==0, C=1, V=1, Z=1."),
         ("abs_8", "p=0x20, db=0, poke(0x2000,&[0x03]), a=0x01, `6D 00 20`: A.l==4, cycles=4."),
         ("dp_16", "p=0x00, d=0, poke(0x10,&[0x34,0x12]), a=0x0001, `65 10`: a==0x1235, cycles=4."),
         ("emulation", "e=true, p=0x30, a=0x00FF, `69 01`: A.l==0, C=1, Z=1.")],
  notes=CHAIN)

T("E02-19", "alu/adc.rs: ADC decimal (BCD 8 et 16 bits)",
  deps=["E02-18"], size=70,
  context="Complete adc.rs avec le mode decimal. Utiliser l'algorithme de reference de la spec, SANS le modifier (l'oracle le valide ensuite).",
  embed=["01_CPU_65816.md#6. ADC / SBC"],
  write=[CPU + "/alu/adc.rs", TST + "/e02_19_adc_decimal.rs"],
  spec="""Remplacer le corps de `adc_decimal(cpu, m, wide)`:
- 8 bits: appliquer l'algorithme `ADC 8 bits` de la spec (u32 pour les intermediaires). Resultat dans A.l (A.h conserve), `set_nz` sur l'octet bas.
- 16 bits: meme principe sur 4 quartets en boucle: pour chaque quartet i (0..3): `r = (A & mask_i) + (m & mask_i) + carry_in` ; si `r > seuil` alors `r += correction` ; la retenue du quartet est calculee comme la spec (retenue vers le quartet suivant). V est calcule AVANT la correction du dernier quartet (quartet 3). `C` final = retenue du dernier quartet. Masques 0x000F,0x00F0,0x0F00,0xF000; seuils 0x0009,0x009F,0x09FF,0x9FFF; corrections 0x6,0x60,0x600,0x6000.
- Ecrire une fonction interne commune `adc_decimal_n(a: u32, m: u32, carry: bool, nibbles: usize) -> (u32 result, bool carry, bool overflow)` testable seule (le fichier de test l'appelle via `cpu` seulement; pas besoin de la rendre publique).""",
  tests=[("bcd_12_34", "D=1 (p=0x28 : M=1, D=1), a=0x12, C=0, `69 34`: A.l==0x46, C=0, V=0."),
         ("bcd_58_46", "D=1, a=0x58, `69 46`: A.l==0x04, C=1, V=1 (resultat de l'algorithme de reference), Z=0, N=0."),
         ("bcd_99_01", "D=1, a=0x99, `69 01`: A.l==0x00, C=1, Z=1, V=0."),
         ("bcd_carry_in", "D=1, C=1, a=0x09, `69 01`: A.l==0x11, C=0."),
         ("bcd_16_1234_4321", "D=1, M=0 (p=0x08), a=0x1234, `69 21 43`: a==0x5555, C=0 ; cycles=3."),
         ("bcd_16_0999_0001", "D=1, M=0, a=0x0999, `69 01 00`: a==0x1000, C=0, N=0."),
         ("bcd_16_9999_0001", "D=1, M=0, a=0x9999, `69 01 00`: a==0x0000, C=1, Z=1."),
         ("binary_still_works", "D=0: `69 05` avec a=0x10 donne 0x15 (non-regression de E02-18).")],
  accept=["cargo test -p puce16-core e02_19_", "cargo test -p puce16-core e02_18_"], notes=CHAIN)

T("E02-20", "alu/sbc.rs: SBC binaire (8 et 16 bits, tous modes)",
  deps=["E02-19"], size=60,
  context="SBC en binaire (A + ~m + C). Le mode decimal est ajoute par la tache suivante dans le meme fichier (squelette `sbc_decimal` = `unimplemented!(\"E02-21\")`).",
  embed=["01_CPU_65816.md#6. ADC / SBC", EMB_TEST],
  tsv=tsv("SBC"), reads=READS,
  write=[CPU + "/alu/sbc.rs", TST + "/e02_20_sbc.rs"],
  spec="""`exec` traite `Mn::Sbc` comme ADC (lecture operande selon M, branche decimale si D=1).
`sbc_binary`: `inv = !m` (masque 0xFF / 0xFFFF); `r = a + inv + C`; `C = r > mask`; `V = ((a ^ r) & (inv ^ r) & signbit) != 0`; resultat tronque; `set_nz`.""",
  tests=[("imm_8_basic", "e=false, p=0x21 (M=1, C=1), a=0x1210, `E9 05`: A.l==0x0B, C=1, N=Z=V=0, cycles=2 ; a==0x120B."),
         ("imm_8_borrow_in", "p=0x20 (C=0), a=0x10, `E9 05`: A.l==0x0A, C=1."),
         ("imm_8_underflow", "p=0x21, a=0x00, `E9 01`: A.l==0xFF, C=0, N=1."),
         ("imm_8_overflow_pos", "p=0x21, a=0x80, `E9 01`: A.l==0x7F, V=1, C=1."),
         ("imm_8_overflow_neg", "p=0x21, a=0x50, `E9 B0`: A.l==0xA0, V=1, C=0, N=1."),
         ("imm_16_basic", "p=0x01 (M=0, C=1), a=0x1000, `E9 01 00`: a==0x0FFF, C=1, cycles=3."),
         ("imm_16_underflow", "p=0x01, a=0x0000, `E9 01 00`: a==0xFFFF, C=0, N=1."),
         ("imm_16_zero", "p=0x01, a=0x1234, `E9 34 12`: a==0, Z=1, C=1."),
         ("abs_8", "p=0x21, poke(0x2000,&[0x03]), a=0x0A, `ED 00 20`: A.l==7, cycles=4.")],
  notes=CHAIN)

T("E02-21", "alu/sbc.rs: SBC decimal (BCD 8 et 16 bits)",
  deps=["E02-20"], size=70,
  context="Complete sbc.rs avec le decimal. Algorithme de reference de la spec, sans le modifier.",
  embed=["01_CPU_65816.md#6. ADC / SBC"],
  write=[CPU + "/alu/sbc.rs", TST + "/e02_21_sbc_decimal.rs"],
  spec="""Remplacer `sbc_decimal(cpu, m, wide)` par l'algorithme `SBC 8 bits` (m' = m ^ 0xFF) de la spec; 16 bits: meme principe sur 4 quartets (conditions inversees: `if r <= 0x0F { r -= 6 }` au quartet bas, `if r <= 0xFF { r -= 0x60 }`... generaliser avec les masques/seuils de la spec pour ADC; la retenue/emprunt se propage de quartet en quartet). V calcule avec `m'`. Ecrire une fonction interne `sbc_decimal_n(a, m, carry, nibbles)`.""",
  tests=[("bcd_46_12", "D=1, C=1 (p=0x29), a=0x46, `E9 12`: A.l==0x34, C=1."),
         ("bcd_12_34", "D=1, C=1, a=0x12, `E9 34`: A.l==0x78, C=0."),
         ("bcd_00_01", "D=1, C=1, a=0x00, `E9 01`: A.l==0x99, C=0."),
         ("bcd_borrow_in", "D=1, C=0, a=0x50, `E9 20`: A.l==0x29, C=1."),
         ("bcd_16_1000_0001", "D=1, M=0, C=1 (p=0x09), a=0x1000, `E9 01 00`: a==0x0999, C=1."),
         ("bcd_16_0000_0001", "D=1, M=0, C=1, a=0x0000, `E9 01 00`: a==0x9999, C=0."),
         ("binary_still_works", "D=0: non-regression de E02-20 (a=0x10, C=1, `E9 05` -> 0x0B).")],
  accept=["cargo test -p puce16-core e02_21_", "cargo test -p puce16-core e02_20_"], notes=CHAIN)

T("E02-22", "alu/logic.rs: AND, ORA, EOR, BIT",
  deps=["E02-21"], size=80,
  context="Instructions logiques et BIT.",
  embed=["01_CPU_65816.md#5.2 Flags", EMB_TEST],
  tsv=tsv("AND", "ORA", "EOR", "BIT"), reads=READS,
  write=[CPU + "/alu/logic.rs", TST + "/e02_22_logic.rs"],
  spec="""`exec` traite `Mn::And/Ora/Eor/Bit`: `wide = !cpu.m8()`; `m = read_operand(cpu,bus,am,wide)`.
- AND/ORA/EOR: `r = a op m` sur la largeur, ecriture dans A (octet haut conserve en 8 bits), `set_nz(r, wide)`.
- BIT avec `am == Am::Imm`: seulement `Z = ((a & m) == 0)` (sur la largeur); N et V inchanges.
- BIT autres modes: `Z = ((a & m) == 0)`, `N` = bit 7 de m (bit 15 si wide), `V` = bit 6 de m (bit 14 si wide). A inchange.""",
  tests=[("and_imm_8", "e=false, p=0x20, a=0xABF0, `29 3C`: a==0xAB30, N=0,Z=0 ; cycles=2."),
         ("and_zero", "p=0x20, a=0x0F, `29 F0`: A.l==0, Z=1."),
         ("ora_imm_8", "p=0x20, a=0x80, `09 01`: A.l==0x81, N=1."),
         ("eor_zero", "p=0x20, a=0xFF, `49 FF`: A.l==0, Z=1."),
         ("and_imm_16", "p=0x00, a=0xFF00, `29 F0 0F`: a==0x0F00 ; cycles=3."),
         ("ora_dp_16", "p=0x00, d=0, poke(0x10,&[0x0F,0xF0]), a=0x0101, `05 10`: a==0xF10F, N=1 ; cycles=4."),
         ("bit_imm_only_z", "p=0x20|0xC0 (N=1,V=1), a=0x0F, `89 F0`: Z=1, N=1, V=1 conserves."),
         ("bit_dp_nv", "p=0x20, d=0, poke(0x10,&[0xC0]), a=0x01, `24 10`: Z=1, N=1, V=1 ; A inchange ; cycles=3."),
         ("bit_dp_16", "p=0x00, poke(0x10,&[0x00,0x40]), a=0x4000, `24 10`: Z=0, V=1, N=0 ; cycles=4."),
         ("bit_abs_x", "p=0x30, x=1, poke(0x2001,&[0x80]), a=0x80, `3C 00 20`: N=1, Z=0, V=0 ; cycles=4.")],
  notes=CHAIN)

T("E02-23", "alu/cmp.rs: CMP, CPX, CPY",
  deps=["E02-22"], size=60,
  context="Comparaisons (soustraction sans ecriture).",
  embed=["01_CPU_65816.md#5.2 Flags", EMB_TEST],
  tsv=tsv("CMP", "CPX", "CPY"), reads=READS,
  write=[CPU + "/alu/cmp.rs", TST + "/e02_23_cmp.rs"],
  spec="""`exec` traite `Mn::Cmp/Cpx/Cpy`. Registre: A (largeur `!m8()`), X ou Y (largeur `!x8()`). `m = read_operand(..)`; `r = reg.wrapping_sub(m)` sur la largeur; `C = (reg >= m)`; `set_nz(r, wide)`; V inchange; aucun registre modifie.""",
  tests=[("cmp_equal", "e=false, p=0x20, a=0x10, `C9 10`: Z=1, C=1, N=0 ; a inchange ; cycles=2."),
         ("cmp_less", "p=0x20, a=0x10, `C9 20`: C=0, N=1, Z=0."),
         ("cmp_greater", "p=0x20, a=0x20, `C9 10`: C=1, N=0, Z=0."),
         ("cmp_v_untouched", "p=0x20|0x40 (V=1), a=0x10, `C9 20`: V reste 1."),
         ("cmp_16", "p=0x00, a=0x8000, `C9 01 00`: C=1, N=0, Z=0 (resultat 0x7FFF) ; cycles=3."),
         ("cpx_imm_16", "p=0x00, x=0x1234, `E0 34 12`: Z=1, C=1 ; cycles=3."),
         ("cpx_imm_8", "p=0x30, x=0x10, `E0 11`: C=0, N=1, Z=0 ; cycles=2."),
         ("cpy_dp", "p=0x30, d=0, poke(0x10,&[0x05]), y=0x05, `C4 10`: Z=1, C=1 ; cycles=3."),
         ("cmp_abs_x", "p=0x30, x=1, poke(0x2001,&[0x40]), a=0x40, `DD 00 20`: Z=1 ; cycles=4.")],
  notes=CHAIN)

T("E02-24", "alu/incdec.rs: INC, DEC, INX, INY, DEX, DEY",
  deps=["E02-23"], size=70,
  context="Incrementer/decrementer (accumulateur, memoire en lecture-modification-ecriture, registres d'index).",
  embed=["01_CPU_65816.md#3.3 Instructions", EMB_TEST],
  tsv=tsv("INC", "DEC", "INX", "INY", "DEX", "DEY"), reads=READS,
  write=[CPU + "/alu/incdec.rs", TST + "/e02_24_incdec.rs"],
  spec="""- `Mn::Inc` / `Mn::Dec`: `addr::rmw(cpu, bus, am, |c, v, wide| { let r = (v +/- 1) tronque a la largeur; c.set_nz(r, wide); r })` (la forme accumulateur `Am::Acc` est geree par `rmw`).
- `Mn::Inx/Iny/Dex/Dey`: `bus.idle()`; `wide = !cpu.x8()`; `r = reg +/- 1` tronque (8 bits si x8: l'octet haut reste 0); `set_nz(r, wide)`.
- C n'est jamais modifie.""",
  tests=[("inc_a_8", "e=false, p=0x20, a=0x12FF, `1A`: a==0x1200, Z=1 ; cycles=2."),
         ("inc_a_16", "p=0x00, a=0x7FFF, `1A`: a==0x8000, N=1 ; avec a=0xFFFF: a==0, Z=1."),
         ("dec_a_16", "p=0x00, a=0x0000, `3A`: a==0xFFFF, N=1."),
         ("dec_a_8", "p=0x20, a=0x01, `3A`: A.l==0, Z=1."),
         ("inc_dp_8", "p=0x20, d=0, poke(0x10,&[0x7F]), `E6 10`: peek(0x10)==0x80, N=1 ; cycles=5 ; log = [R,R,R data,Idle,W data]."),
         ("inc_abs_16", "p=0x00, poke(0x2000,&[0xFF,0x00]), `EE 00 20`: peek(0x2000)==0x00, peek(0x2001)==0x01 ; cycles=8 ; les deux ecritures: 0x2001 puis 0x2000."),
         ("dec_abs_x", "p=0x20 (M=1) ; X=0 donc index 16 bits, x=1, poke(0x2001,&[0x01]), `DE 00 20`: peek(0x2001)==0, Z=1 ; cycles=7."),
         ("inx_8", "p=0x30, x=0x00FF, `E8`: x==0, Z=1 ; cycles=2."),
         ("inx_16", "p=0x00, x=0xFFFF, `E8`: x==0, Z=1."),
         ("dey_16", "p=0x00, y=0, `88`: y==0xFFFF, N=1."),
         ("carry_untouched", "C=1 avant INC/DEC/INX: C reste 1.")],
  notes=CHAIN)

T("E02-25", "alu/shift.rs: ASL, LSR, ROL, ROR",
  deps=["E02-24"], size=80,
  context="Decalages et rotations (accumulateur et memoire).",
  embed=["01_CPU_65816.md#5.2 Flags", EMB_TEST],
  tsv=tsv("ASL", "LSR", "ROL", "ROR"), reads=READS,
  write=[CPU + "/alu/shift.rs", TST + "/e02_25_shift.rs"],
  spec="""Chaque instruction = `addr::rmw(cpu, bus, am, |c, v, wide| {...})` (le bit de signe = 0x80 ou 0x8000, le masque = 0xFF ou 0xFFFF):
- ASL: `C = v & signbit != 0`; `r = (v << 1) & mask`.
- LSR: `C = v & 1 != 0`; `r = v >> 1`.
- ROL: `cin = C`; `C = v & signbit != 0`; `r = ((v << 1) | cin) & mask`.
- ROR: `cin = C`; `C = v & 1 != 0`; `r = (v >> 1) | (cin ? signbit : 0)`.
- `c.set_nz(r, wide)`. (Pour LSR N est toujours 0.)""",
  tests=[("asl_a_8", "e=false, p=0x20, a=0x1281, `0A`: A.l==0x02, C=1, N=0, Z=0, a==0x1202 ; cycles=2."),
         ("asl_a_16", "p=0x00, a=0x4000, `0A`: a==0x8000, N=1, C=0."),
         ("lsr_a_8", "p=0x20, a=0x01, `4A`: A.l==0, C=1, Z=1, N=0."),
         ("lsr_a_16", "p=0x00, a=0x0003, `4A`: a==1, C=1."),
         ("rol_a_8", "p=0x21 (C=1), a=0x80, `2A`: A.l==0x01, C=1."),
         ("rol_a_16", "p=0x00 (C=0), a=0x8001, `2A`: a==0x0002, C=1."),
         ("ror_a_8", "p=0x21, a=0x01, `6A`: A.l==0x80, C=1, N=1."),
         ("ror_a_16", "p=0x01 (M=0,C=1), a=0, `6A`: a==0x8000, C=0, N=1."),
         ("asl_dp", "p=0x20, d=0, poke(0x10,&[0x40]), `06 10`: peek(0x10)==0x80, N=1 ; cycles=5."),
         ("ror_abs_x", "p=0x20 (index 16 bits), x=1, poke(0x2001,&[0x02]), C=0, `7E 00 20`: peek(0x2001)==0x01 ; cycles=7."),
         ("rol_abs_16", "p=0x00, C=1, poke(0x2000,&[0x00,0x80]), `2E 00 20`: peek(0x2000)==0x01, peek(0x2001)==0x00, C=1 ; cycles=8.")],
  notes=CHAIN)

T("E02-26", "alu/tsbtrb.rs: TSB, TRB",
  deps=["E02-25"], size=50,
  context="Test-and-set / test-and-reset de bits.",
  embed=["01_CPU_65816.md#5.2 Flags", EMB_TEST],
  tsv=tsv("TSB", "TRB"), reads=READS,
  write=[CPU + "/alu/tsbtrb.rs", TST + "/e02_26_tsbtrb.rs"],
  spec="""`addr::rmw(cpu, bus, am, |c, v, wide| {...})` avec `a` = A (largeur): `c.set_flag(FLAG_Z, (a & v) == 0)`; TSB renvoie `v | a`; TRB renvoie `v & !a`. N, V, C inchanges.""",
  tests=[("tsb_dp_8", "e=false, p=0x20, d=0, poke(0x10,&[0xF0]), a=0x0F, `04 10`: Z=1, peek(0x10)==0xFF ; cycles=5."),
         ("trb_dp_8", "p=0x20, poke(0x10,&[0xFF]), a=0x0F, `14 10`: Z=0, peek(0x10)==0xF0."),
         ("tsb_abs_16", "p=0x00, poke(0x2000,&[0xF0,0x00]), a=0x1234, `0C 00 20`: Z=0, peek(0x2000)==0xF4, peek(0x2001)==0x12 ; cycles=8."),
         ("trb_abs_16", "p=0x00, poke(0x2000,&[0xFF,0xFF]), a=0x00FF, `1C 00 20`: peek(0x2000)==0x00, peek(0x2001)==0xFF ; Z=0."),
         ("nvc_untouched", "p=0x20|0xC1: apres TSB, N,V,C inchanges.")],
  notes=CHAIN)

T("E02-27", "VERIF oracle: toutes les instructions ALU",
  deps=["E02-26", "E02-14"], size=60,
  context="Passage a l'oracle de toutes les instructions de la famille ALU (ADC SBC AND ORA EOR BIT CMP CPX CPY INC DEC INX INY DEX DEY ASL LSR ROL ROR TSB TRB), y compris les cas decimaux.",
  reads=["specs/01_CPU_65816.md sections 5 et 6", "specs/data/ssjson_format.md"],
  write=[CPU + "/alu/*.rs", TST + "/e02_27_oracle_regressions.rs"],
  spec="""1. Lancer: `%s`
2. Corriger chaque echec dans `cpu/alu/*.rs` uniquement (diff = champ ou cycle fautif). Les modes d'adressage sont valides (E02-17): si un echec vient clairement d'un mode, poser une question `spec`/`bug` au lieu de modifier addr/.
3. Pour les cas decimaux: l'algorithme de la spec est un point de depart; si l'oracle le contredit, poser une question `spec` avec le premier test en echec (valeurs A, m, C, D attendues/obtenues) AVANT de changer l'algorithme.
4. Un test de regression par bug dans `tests/e02_27_oracle_regressions.rs` (nom `e02_27_<instruction>_<cas>`), ou le test `e02_27_oracle_clean` si rien a corriger (verifie `Cpu::new().p == 0x34`).
5. Fin: `failed: 0`.""" % oracle_cmd("ADC", "SBC", "AND", "ORA", "EOR", "BIT", "CMP", "CPX", "CPY", "INC", "DEC", "INX", "INY", "DEX", "DEY", "ASL", "LSR", "ROL", "ROR", "TSB", "TRB"),
  tests=[("oracle_clean_or_regressions", "voir spec etape 4")],
  accept=[oracle_cmd("ADC", "SBC", "AND", "ORA", "EOR", "BIT", "CMP", "CPX", "CPY", "INC", "DEC", "INX", "INY", "DEX", "DEY", "ASL", "LSR", "ROL", "ROR", "TSB", "TRB"),
          "cargo test -p puce16-core e02_"],
  notes=CHAIN)

# --- pile et blocs

T("E02-28", "mem/push.rs: PHA PHX PHY PHP PHD PHB PHK PEA PEI PER",
  deps=["E02-27"], size=110,
  context="Instructions qui poussent sur la pile. Cycles et ordre exacts dans 01b section 5.",
  embed=["01b_CPU_DESIGN.md#5. Cycles des instructions implicites", EMB_TEST],
  tsv=tsv("PHA", "PHX", "PHY", "PHP", "PHD", "PHB", "PHK", "PEA", "PEI", "PER"), reads=READS,
  write=[CPU + "/mem/push.rs", TST + "/e02_28_push.rs"],
  spec="""`exec` traite les 10 mnemoniques:
- PHA/PHX/PHY: `bus.idle()`; `wide` = `!m8()` (PHA) ou `!x8()` (PHX, PHY); si wide: `push8(hi)` puis `push8(lo)`; sinon `push8(lo)` (pile "ancienne": `push8`).
- PHP: `idle`; `push8(cpu.p)`. PHB: `idle`; `push8(db)`. PHK: `idle`; `push8(pb)`.
- PHD: `idle`; `push8n(d >> 8)`; `push8n(d & 0xFF)`; `fix_stack_e()` (pile "nouvelle").
- PEA: `lo = fetch8`, `hi = fetch8`; `push8n(hi)`; `push8n(lo)`; `fix_stack_e()`.
- PEI: `e = addr::ea(cpu, bus, Am::Dp, false)` (ses cycles: operande + Idle si DL != 0); `v = addr::read_data(.., wide=true)`; `push8n(v >> 8)`; `push8n(v & 0xFF)`; `fix_stack_e()`.
- PER: `lo = fetch8`, `hi = fetch8`; `bus.idle()`; `target = cpu.pc.wrapping_add(off16)` (pc deja apres l'operande); `push8n(target >> 8)`; `push8n(target & 0xFF)`; `fix_stack_e()`.
Les cas de pile "nouvelle" en emulation sont laisses a l'oracle (01b section 7): ne pas ecrire de test dessus.""",
  tests=[("pha_8", "e=false, p=0x20, s=0x01FF, a=0x12AB, `48`: peek(0x01FF)==0xAB, s==0x01FE ; cycles=3 ; log = [R opcode?..]: les 2 derniers cycles = [Idle, Write 0x01FF=0xAB]."),
         ("pha_16", "p=0x00, s=0x01FF, a=0x1234, `48`: peek(0x01FF)==0x12, peek(0x01FE)==0x34, s==0x01FD ; cycles=4."),
         ("phx_8_phy_16", "p=0x30, s=0x01FF, x=0x00AA, `DA`: peek(0x01FF)==0xAA. Puis p=0x00, y=0xBEEF, `5A`: deux octets BE puis EF-en-dessous."),
         ("php", "e=false, p=0x65, s=0x01FF, `08`: peek(0x01FF)==0x65, s==0x01FE ; cycles=3."),
         ("php_emulation", "e=true, p=0x34, s=0x01FF, `08`: peek(0x01FF)==0x34."),
         ("phd", "e=false, d=0xBEEF, s=0x01FF, `0B`: peek(0x01FF)==0xBE, peek(0x01FE)==0xEF, s==0x01FD ; cycles=4."),
         ("phb_phk", "db=0x7E, s=0x01FF, `8B`: peek(0x01FF)==0x7E ; pb=0, `4B` ensuite: peek(0x01FE)==0x00 (pb)."),
         ("pha_emulation_wrap", "e=true, p=0x30, s=0x0100, a=0x77, `48`: peek(0x0100)==0x77, s==0x01FF."),
         ("pea", "e=false, s=0x01FF, `F4 34 12`: peek(0x01FF)==0x12, peek(0x01FE)==0x34, s==0x01FD ; cycles=5."),
         ("pei", "e=false, d=0, poke(0x10,&[0x78,0x56]), s=0x01FF, `D4 10`: peek(0x01FF)==0x56, peek(0x01FE)==0x78 ; cycles=6 ; avec d=0x0001 (et poke(0x11,..)) cycles=7."),
         ("per_positive", "pc=0x8000, `62 10 00`: pousse 0x80 puis 0x13 (cible = 0x8003+0x0010) ; cycles=6."),
         ("per_negative", "`62 FD FF` en 0x8000: pousse 0x80 puis 0x00.")],
  notes=CHAIN)

T("E02-29", "mem/pull.rs: PLA PLX PLY PLP PLD PLB",
  deps=["E02-28"], size=80,
  context="Instructions qui depilent. Cycles et ordre exacts dans 01b section 5.",
  embed=["01b_CPU_DESIGN.md#5. Cycles des instructions implicites", EMB_TEST],
  tsv=tsv("PLA", "PLX", "PLY", "PLP", "PLD", "PLB"), reads=READS,
  write=[CPU + "/mem/pull.rs", TST + "/e02_29_pull.rs"],
  spec="""- PLA/PLX/PLY: `idle`; `idle`; `wide` selon M (PLA) ou X; `lo = pull8()`; si wide `hi = pull8()`; ecrire le registre (PLA 8 bits: octet haut conserve) ; `set_nz`.
- PLP: `idle; idle; v = pull8(); cpu.set_p(v)`.
- PLB: `idle; idle; v = pull8(); cpu.db = v; set_nz8(v)`.
- PLD: `idle; idle; lo = pull8n(); hi = pull8n(); cpu.d = hi<<8|lo; set_nz16(d); fix_stack_e()`.""",
  tests=[("pla_8", "e=false, p=0x20, s=0x01FE, poke(0x01FF,&[0x80]), a=0x1200, `68`: a==0x1280, N=1, s==0x01FF ; cycles=4."),
         ("pla_16", "p=0x00, s=0x01FD, poke(0x01FE,&[0x34,0x12]), `68`: a==0x1234, s==0x01FF ; cycles=5."),
         ("plx_8", "p=0x30, s=0x01FE, poke(0x01FF,&[0x00]), x=0x55, `FA`: x==0, Z=1."),
         ("ply_16", "p=0x00, s=0x01FD, poke(0x01FE,&[0xEF,0xBE]), `7A`: y==0xBEEF, N=1."),
         ("plp_native", "e=false, p=0x00, s=0x01FE, poke(0x01FF,&[0x30]), `28`: p==0x30."),
         ("plp_x_clears_high", "e=false, p=0x00, x=0x1234, y=0x5678, poke(0x01FF,&[0x10]), s=0x01FE, `28`: x==0x0034, y==0x0078."),
         ("plp_emulation", "e=true, p=0x30, poke(0x01FF,&[0x00]), s=0x01FE, `28`: p==0x30."),
         ("plb", "p=0x30, s=0x01FE, poke(0x01FF,&[0x7E]), `AB`: db==0x7E, N=0 ; avec 0x80: N=1 ; cycles=4."),
         ("pld", "e=false, s=0x01FD, poke(0x01FE,&[0x00,0x00]), d=0x1234, `2B`: d==0, Z=1, s==0x01FF ; cycles=5."),
         ("pla_emulation_wrap", "e=true, p=0x30, s=0x01FF, poke(0x0100,&[0x42]), `68`: A.l==0x42, s==0x0100.")],
  notes=CHAIN)

T("E02-30", "mem/block.rs: MVN, MVP",
  deps=["E02-29"], size=60,
  context="Copie de blocs. Un octet par execution; l'instruction se rejoue tant que A != 0xFFFF.",
  embed=["01_CPU_65816.md#7. Controle de flux", EMB_TEST],
  tsv=tsv("MVN", "MVP"), reads=READS,
  write=[CPU + "/mem/block.rs", TST + "/e02_30_block.rs"],
  spec="""`exec` traite `Mn::Mvn`/`Mn::Mvp` (sequence de la spec, ligne MVN/MVP):
`dst = fetch8`; `src = fetch8`; `cpu.db = dst`; `v = bus.read((src << 16) | x)`; `bus.write((dst << 16) | y, v)`; `bus.idle()`; `bus.idle()`; `x`, `y` +1 (MVN) ou -1 (MVP) en wrapping sur 16 bits, ou sur 8 bits si `x8()`; `a = a.wrapping_sub(1)` (16 bits toujours); si `a != 0xFFFF`: `pc = pc.wrapping_sub(3)`. Total 7 cycles (opcode compris). x/y a 8 bits: la valeur reste dans 0..255. NB: l'ordre des octets operande est (destination, source).""",
  tests=[("mvn_one_byte", "e=false, p=0x00, a=0x0001 (2 octets), x=0x1000, y=0x2000, poke(0x021000,&[0xAA,0xBB]), `54 01 02` en 0x8000: apres 1 step: peek(0x012000)==0xAA, a==0x0000, x==0x1001, y==0x2001, db==0x01, pc==0x8000 (rejoue) ; cycles=7."),
         ("mvn_finishes", "puis un 2e step: peek(0x012001)==0xBB, a==0xFFFF, pc==0x8003."),
         ("mvn_log", "cycles du 1er step: R 0x008000, R 0x008001, R 0x008002, R 0x021000, W 0x012000=0xAA, Idle, Idle."),
         ("mvp_decrements", "a=0x0000, x=0x1001, y=0x2001, poke(0x021001,&[0x5A]), `44 01 02`: peek(0x012001)==0x5A, x==0x1000, y==0x2000, pc==0x8003, a==0xFFFF."),
         ("mvn_8bit_index", "p=0x10 (X=1), x=0x00FF, y=0x0010, a=0, `54 00 00`: x==0x0000 (wrap 8 bits), y==0x0011."),
         ("mvn_a_wrap", "a=0x0000: un seul octet copie puis a==0xFFFF et pc==0x8003.")],
  notes=CHAIN)

T("E02-31", "VERIF oracle: pile et blocs (PH*/PL*/PEA/PEI/PER/MVN/MVP)",
  deps=["E02-30", "E02-14"], size=50,
  context="Passage a l'oracle des instructions de pile et de copie de blocs. Les cas de pile 'nouvelle' en mode emulation se tranchent ici.",
  reads=["specs/01b_CPU_DESIGN.md section 7", "specs/data/ssjson_format.md"],
  write=[CPU + "/mem/push.rs", CPU + "/mem/pull.rs", CPU + "/mem/block.rs", TST + "/e02_31_oracle_regressions.rs"],
  spec="""1. Lancer: `%s`
2. Corriger chaque echec (fichiers autorises seulement). Si un comportement de pile en emulation (PEA PEI PER PHD PLD) differe du plan de 01b, c'est l'oracle qui a raison: adapter le code ET noter dans `docs/qa/inbox/` une question `spec` pour que Claude mette a jour 01b section 7.
3. Un test de regression par bug (`e02_31_<instruction>_<cas>`), sinon `e02_31_oracle_clean` (verifie `Cpu::new().s == 0x01FF`).
4. Fin: `failed: 0`.""" % oracle_cmd("PHA", "PHX", "PHY", "PHP", "PHD", "PHB", "PHK", "PEA", "PEI", "PER", "PLA", "PLX", "PLY", "PLP", "PLD", "PLB", "MVN", "MVP"),
  tests=[("oracle_clean_or_regressions", "voir spec etape 3")],
  accept=[oracle_cmd("PHA", "PHX", "PHY", "PHP", "PHD", "PHB", "PHK", "PEA", "PEI", "PER", "PLA", "PLX", "PLY", "PLP", "PLD", "PLB", "MVN", "MVP"),
          "cargo test -p puce16-core e02_"],
  notes=CHAIN)

# --- controle de flux

T("E02-32", "flow/branch.rs: Bxx, BRA, BRL",
  deps=["E02-31"], size=60,
  context="Branchements relatifs. Cycles: voir spec 01 section 7.",
  embed=["01_CPU_65816.md#7. Controle de flux", EMB_TEST],
  tsv=tsv("BCC", "BCS", "BEQ", "BNE", "BMI", "BPL", "BVC", "BVS", "BRA", "BRL"), reads=READS,
  write=[CPU + "/flow/branch.rs", TST + "/e02_32_branch.rs"],
  spec="""- Bxx (`Am::Rel`): `off = fetch8 as i8`; condition: BCC C=0, BCS C=1, BNE Z=0, BEQ Z=1, BPL N=0, BMI N=1, BVC V=0, BVS V=1, BRA toujours. Si condition vraie: `target = cpu.pc.wrapping_add(off as i16 as u16)`; `bus.idle()`; si `cpu.e` et `(cpu.pc & 0xFF00) != (target & 0xFF00)`: un 2e `bus.idle()`; `cpu.pc = target`. (`cpu.pc` = adresse apres l'operande.)
- BRL (`Am::RelL`): `lo = fetch8`, `hi = fetch8`; `bus.idle()`; `pc = pc.wrapping_add(off16)`. 4 cycles.
- Le PC reste dans la banque `pb` (wrap 16 bits).""",
  tests=[("bne_not_taken", "e=false, p=0x22 (Z=1), pc=0x8000, `D0 10`: pc==0x8002, cycles=2."),
         ("bne_taken", "p=0x20 (Z=0), `D0 10`: pc==0x8012, cycles=3 (log: R, R, Idle)."),
         ("bra_negative", "`80 FE` en 0x8000: pc==0x8000, cycles=3."),
         ("emulation_page_cross", "e=true, p=0x30, pc=0x80F0, `D0 20` (Z=0): apres operande pc=0x80F2, cible 0x8112 -> pc==0x8112, cycles=4."),
         ("native_page_cross_no_penalty", "e=false, meme programme: cycles=3."),
         ("emulation_same_page", "e=true, pc=0x8000, `D0 10`: cycles=3."),
         ("all_conditions", "table de verite: BCC (C=0), BCS (C=1), BEQ (Z=1), BNE (Z=0), BMI (N=1), BPL (N=0), BVC (V=0), BVS (V=1): pour chaque opcode (90 B0 F0 D0 30 10 50 70), pris/non pris selon le flag, offset +4."),
         ("brl", "`82 00 01` en 0x8000: pc==0x8103, cycles=4. `82 FD FF`: pc==0x8000."),
         ("wrap_in_bank", "pb=0x05, pc=0xFFF0, `80 20`: apres operande 0xFFF2 + 0x20 -> pc==0x0012, pb==0x05.")],
  notes=CHAIN)

T("E02-33", "flow/jump.rs: JMP et JML (tous modes)",
  deps=["E02-32"], size=70,
  context="Sauts sans pile.",
  embed=["01_CPU_65816.md#7. Controle de flux", EMB_TEST],
  tsv=tsv("JMP", "JML"), reads=READS,
  write=[CPU + "/flow/jump.rs", TST + "/e02_33_jump.rs"],
  spec="""- `Jmp` + `Am::Abs`: `cpu.pc = fetch16`. (3 cycles)
- `Jml` + `Am::Long`: `v = fetch24`; `pb = v >> 16`; `pc = v & 0xFFFF`. (4 cycles)
- `Jmp` + `Am::AbsInd`: `a = fetch16`; `lo = bus.read(a)`; `hi = bus.read((a + 1) & 0xFFFF)` (banque 0, wrap 16 bits); `pc = hi<<8|lo`. (5 cycles)
- `Jml` + `Am::AbsIndLong`: `a = fetch16`; lit 3 octets en banque 0 a `a`, `a+1`, `a+2` (chaque adresse masquee 16 bits); `pc = ..`; `pb = bank`. (6 cycles)
- `Jmp` + `Am::AbsXInd`: `a = fetch16`; `bus.idle()`; `p = (a + x) & 0xFFFF`; lit `lo` en `PB:p`, `hi` en `PB:(p+1)&0xFFFF`; `pc = ..`. (6 cycles)""",
  tests=[("jmp_abs", "pb=0x03, pc=0x8000, `4C 34 12`: pc==0x1234, pb==0x03, cycles=3."),
         ("jml_long", "`5C 56 34 12`: pb==0x12, pc==0x3456, cycles=4."),
         ("jmp_ind", "poke(0x2000,&[0x78,0x56]), `6C 00 20`: pc==0x5678, cycles=5 ; pointeur lu en banque 0 meme si pb=0x03."),
         ("jmp_ind_wrap", "poke(0xFFFF,&[0x34]), poke(0x0000,&[0x12]), `6C FF FF`: pc==0x1234."),
         ("jml_ind_long", "poke(0x2000,&[0x56,0x34,0x12]), `DC 00 20`: pb==0x12, pc==0x3456, cycles=6."),
         ("jmp_abs_x_ind", "pb=0x03, p=0x00, x=0x0004, poke(0x032004,&[0x00,0x90]), `7C 00 20`: pc==0x9000, pb==0x03, cycles=6 ; log = [R,R,R,Idle,R 0x032004,R 0x032005]."),
         ("jmp_abs_x_ind_wrap", "pb=0x03, x=1, `7C FF FF`: octets lus en 0x03:0000 et 0x03:0001.")],
  notes=CHAIN)

T("E02-34", "flow/call.rs: JSR, JSL, RTS, RTL",
  deps=["E02-33"], size=100,
  context="Appels et retours de sous-programme. Attention: PC pousse = adresse du DERNIER octet de l'instruction.",
  embed=["01_CPU_65816.md#7. Controle de flux", EMB_TEST],
  tsv=tsv("JSR", "JSL", "RTS", "RTL"), reads=READS,
  write=[CPU + "/flow/call.rs", TST + "/e02_34_call.rs"],
  spec="""- JSR `Am::Abs` (pile "ancienne"): `aal = fetch8`, `aah = fetch8`; `bus.idle()`; `push8(pc_apres_operande - 1 >> 8)`; `push8(.. & 0xFF)` (PC pousse = `pc.wrapping_sub(1)` ou `pc` = adresse apres l'operande); `pc = aah<<8|aal`. 6 cycles.
- JSR `Am::AbsXInd` (pile "nouvelle"): `aal = fetch8`; `push8n(pc >> 8)`; `push8n(pc & 0xFF)` ou `pc` pointe sur l'octet `aah` (non encore lu); `aah = fetch8`; `bus.idle()`; `p = (aah<<8|aal) + x` (16 bits); `lo = read(PB:p)`, `hi = read(PB:p+1)`; `pc = ..`; `fix_stack_e()`. 8 cycles.
- JSL (`Am::Long`, pile "nouvelle"): `aal = fetch8`; `aah = fetch8`; `push8n(pb)`; `bus.idle()`; `bank = fetch8`; `push8n(pc_hi)`; `push8n(pc_lo)` ou `pc` = adresse du dernier octet (`pc - 1` apres le fetch du bank); `pb = bank`; `pc = aah<<8|aal`; `fix_stack_e()`. 8 cycles.
- RTS (pile "ancienne"): `idle; idle; lo = pull8; hi = pull8; idle; pc = (hi<<8|lo) + 1` (wrap 16 bits). 6 cycles.
- RTL (pile "nouvelle"): `idle; idle; lo = pull8n; hi = pull8n; bank = pull8n; pb = bank; pc = (hi<<8|lo) + 1`; `fix_stack_e()`. 6 cycles.
(Convention d'ecriture: l'octet de pile le plus profond = le plus haut.)""",
  tests=[("jsr_abs", "e=false, s=0x01FF, pc=0x8000, `20 34 12`: pc==0x1234, s==0x01FD, peek(0x01FF)==0x80, peek(0x01FE)==0x02 ; cycles=6 ; log = [R,R,R,Idle,W 0x01FF=0x80,W 0x01FE=0x02]."),
         ("jsl", "s=0x01FF, pb=0, `22 56 34 12`: pb==0x12, pc==0x3456, s==0x01FC ; peek(0x01FF)==0x00 (PB), peek(0x01FE)==0x80, peek(0x01FD)==0x03 ; cycles=8 ; log = [R,R,R,W 0x01FF,Idle,R,W 0x01FE,W 0x01FD]."),
         ("jsr_abs_x_ind", "e=false, p=0x00, s=0x01FF, x=2, pb=0, poke(0x2002,&[0x00,0x90]), `FC 00 20`: pc==0x9000, s==0x01FD, peek(0x01FF)==0x80, peek(0x01FE)==0x02 ; cycles=8."),
         ("rts", "s=0x01FD, poke(0x01FE,&[0x02,0x80]), `60`: pc==0x8003, s==0x01FF ; cycles=6 ; log = [R,Idle,Idle,R 0x01FE,R 0x01FF,Idle]."),
         ("rtl", "s=0x01FC, poke(0x01FD,&[0x03,0x80,0x05]), `6B`: pc==0x8004, pb==0x05, s==0x01FF ; cycles=6."),
         ("jsr_rts_roundtrip", "poke(0x9000,&[0x60]) ; `20 00 90` en 0x8000 puis step: pc==0x8003, s==0x01FF."),
         ("jsl_rtl_roundtrip", "poke(0x129000,&[0x6B]) ; `22 00 90 12` en 0x8000 puis step: pb==0, pc==0x8004, s==0x01FF."),
         ("jsr_emulation_wrap", "e=true, p=0x30, s=0x0100, `20 00 90`: peek(0x0100)==0x80, peek(0x01FF)==0x02, s==0x01FE.")],
  notes=CHAIN)

T("E02-35", "flow/sys.rs: BRK, COP, RTI, WAI, STP, WDM",
  deps=["E02-34"], size=110,
  context="Instructions systeme et retour d'interruption logicielle.",
  embed=["01_CPU_65816.md#7. Controle de flux", EMB_TEST],
  tsv=tsv("BRK", "COP", "RTI", "WAI", "STP", "WDM"), reads=READS,
  write=[CPU + "/flow/sys.rs", TST + "/e02_35_sys.rs"],
  spec="""- BRK/COP: `sig = fetch8` (ignore); si `!cpu.e`: `push8(pb)`; `push8(pc >> 8)`; `push8(pc & 0xFF)` (pc = adresse apres la signature); `push8(cpu.p)`; vecteur: BRK natif 0xFFE6, COP natif 0xFFE4, emulation BRK 0xFFFE, COP 0xFFF4; `lo = bus.read(vec)`, `hi = bus.read(vec+1)`; `set_flag(I,true)`; `set_flag(D,false)`; `pb = 0`; `pc = hi<<8|lo`. 8 cycles en natif, 7 en emulation. (`p` pousse tel quel: en emulation les bits 5 et 4 sont 1, donc B=1.)
- RTI: `idle; idle; v = pull8(); cpu.set_p(v); lo = pull8; hi = pull8; pc = ..; si !e: pb = pull8`. 7 cycles natif, 6 emulation. PC non incremente.
- WAI: `idle; idle; cpu.waiting = true`.
- STP: `idle; idle; cpu.halted = true`.
- WDM (`Am::Wdm`): `fetch8` (signature ignoree). 2 cycles.""",
  tests=[("brk_native", "e=false, p=0x01, pb=0x03, pc=0x8000, s=0x01FF, poke(0xFFE6,&[0x00,0x90]), `BRK 00` (octets `00 00` en 03:8000): peek(0x01FF)==0x03, peek(0x01FE)==0x80, peek(0x01FD)==0x02, peek(0x01FC)==0x01, s==0x01FB, pc==0x9000, pb==0, I=1, D=0 ; cycles=8."),
         ("brk_emulation", "e=true, p=0x30, s=0x01FF, poke(0xFFFE,&[0x00,0x88]), `00 00`: peek(0x01FF)==0x80, peek(0x01FE)==0x02, peek(0x01FD)==0x30, s==0x01FC, pc==0x8800, pb==0 ; cycles=7."),
         ("cop_native_vector", "e=false, poke(0xFFE4,&[0x00,0xA0]), `02 00`: pc==0xA000 ; cycles=8."),
         ("brk_clears_decimal", "p=0x08 (D=1), e=false: apres BRK D=0 et I=1, mais l'octet P empile contient D=1 (0x08)."),
         ("rti_native", "e=false, s=0x01FB, poke(0x01FC,&[0x24,0x00,0x90,0x05]), `40`: p==0x24, pc==0x9000, pb==0x05, s==0x01FF ; cycles=7."),
         ("rti_emulation", "e=true, s=0x01FC, poke(0x01FD,&[0x00,0x00,0x90]), `40`: p==0x30, pc==0x9000, pb inchange, s==0x01FF ; cycles=6."),
         ("wai", "`CB`: waiting==true, pc==0x8001, cycles=3."),
         ("stp", "`DB`: halted==true, pc==0x8001, cycles=3."),
         ("wdm", "`42 99` en 0x8000: pc==0x8002, cycles=2, aucun autre changement.")],
  notes=CHAIN)

T("E02-36", "VERIF oracle: controle de flux (Bxx, JMP, JSR, RTS, BRK, RTI...)",
  deps=["E02-35", "E02-14"], size=50,
  context="Passage a l'oracle de toutes les instructions de saut, appel et systeme.",
  reads=["specs/01_CPU_65816.md section 7", "specs/data/ssjson_format.md"],
  write=[CPU + "/flow/*.rs", TST + "/e02_36_oracle_regressions.rs"],
  spec="""1. Lancer: `%s`
2. Corriger chaque echec dans `cpu/flow/*.rs` uniquement.
3. WAI et STP: l'oracle ne teste que l'instruction elle-meme (les cycles d'attente ne sont pas observables): si l'oracle les ecarte, ignorer.
4. Un test de regression par bug (`e02_36_<instruction>_<cas>`), sinon `e02_36_oracle_clean` (verifie `Cpu::new().e`).
5. Fin: `failed: 0`.""" % oracle_cmd("BCC", "BCS", "BEQ", "BNE", "BMI", "BPL", "BVC", "BVS", "BRA", "BRL", "JMP", "JML", "JSR", "JSL", "RTS", "RTL", "BRK", "COP", "RTI", "WAI", "STP", "WDM"),
  tests=[("oracle_clean_or_regressions", "voir spec etape 4")],
  accept=[oracle_cmd("BCC", "BCS", "BEQ", "BNE", "BMI", "BPL", "BVC", "BVS", "BRA", "BRL", "JMP", "JML", "JSR", "JSL", "RTS", "RTL", "BRK", "COP", "RTI", "WAI", "STP", "WDM"),
          "cargo test -p puce16-core e02_"],
  notes=CHAIN)

# --- divers

T("E02-37", "misc/transfer.rs: TAX TAY TXA TYA TSX TXS TXY TYX TCD TDC TCS TSC",
  deps=["E02-36"], size=90,
  context="Transferts de registres. Regle de largeur = taille de la DESTINATION (voir spec 5.2).",
  embed=["01_CPU_65816.md#5.2 Flags", EMB_TEST],
  tsv=tsv("TAX", "TAY", "TXA", "TYA", "TSX", "TXS", "TXY", "TYX", "TCD", "TDC", "TCS", "TSC"), reads=READS,
  write=[CPU + "/misc/transfer.rs", TST + "/e02_37_transfer.rs"],
  spec="""Tous: `bus.idle()` puis le transfert (2 cycles au total).
- TAX/TAY: dest X/Y de largeur `!x8()`: wide -> `x = a`; sinon `x = a & 0xFF`; `set_nz(x, wide)`.
- TXA/TYA: dest A de largeur `!m8()`: wide -> `a = x`; sinon `a = (a & 0xFF00) | (x & 0xFF)`; `set_nz`.
- TSX: dest X, largeur `!x8()`: wide -> `x = s` sinon `x = s & 0xFF`; `set_nz`.
- TXS: aucun flag. `if e { s = 0x0100 | (x & 0xFF) } else { s = x }`.
- TXY/TYX: largeur `!x8()`; copie (8 bits: octet bas, haut a 0); `set_nz`.
- TCD: `d = a` (16 bits toujours); `set_nz16(d)`. TDC: `a = d`; `set_nz16`.
- TCS: aucun flag. `s = a`; si `e`: `s = 0x0100 | (a & 0xFF)`.
- TSC: `a = s` (16 bits); `set_nz16`.""",
  tests=[("tax_16", "e=false, p=0x20 (M=1, X=0), a=0x1234, `AA`: x==0x1234 (dest 16 bits), cycles=2."),
         ("tax_8", "p=0x10 (X=1, M=0), a=0x1280, x=0x0099, `AA`: x==0x0080, N=1."),
         ("txa_m8", "p=0x20 (M=1, X=0), x=0xABCD, a=0x1200, `8A`: a==0x12CD."),
         ("txa_m16", "p=0x00, x=0x1234, `8A`: a==0x1234."),
         ("tay_tya", "p=0x00: a=0x8000 `A8` -> y==0x8000, N=1 ; y=0x0000 `98` -> a==0, Z=1."),
         ("tsx_8", "p=0x10, s=0x01FF, `BA`: x==0x00FF, N=1."),
         ("txs_native", "e=false, x=0x1234, `9A`: s==0x1234, flags inchanges (p identique)."),
         ("txs_emulation", "e=true, p=0x30, x=0x56, `9A`: s==0x0156."),
         ("txy_tyx", "p=0x00, x=0x1234, `9B`: y==0x1234 ; p=0x10, y=0x0080, `BB`: x==0x0080, N=1."),
         ("tcd_tdc", "a=0x8000, `5B`: d==0x8000, N=1 ; d=0, `7B`: a==0, Z=1."),
         ("tcs_native_emulation", "e=false, a=0x2345, `1B`: s==0x2345 ; e=true, p=0x30: s==0x0145 ; flags jamais modifies."),
         ("tsc", "s=0x01FF, `3B`: a==0x01FF, N=0, Z=0.")],
  notes=CHAIN)

T("E02-38", "misc/flags.rs: CLC SEC CLI SEI CLD SED CLV REP SEP XCE XBA NOP",
  deps=["E02-37"], size=90,
  context="Drapeaux, changement de mode emulation, echange d'octets et NOP.",
  embed=["01_CPU_65816.md#5.2 Flags", "01b_CPU_DESIGN.md#5. Cycles des instructions implicites", EMB_TEST],
  tsv=tsv("CLC", "SEC", "CLI", "SEI", "CLD", "SED", "CLV", "REP", "SEP", "XCE", "XBA", "NOP"), reads=READS,
  write=[CPU + "/misc/flags.rs", TST + "/e02_38_flags.rs"],
  spec="""- CLC/SEC/CLI/SEI/CLD/SED/CLV: `bus.idle()`; `set_flag(..)`. 2 cycles.
- REP: `m = fetch8`; `bus.idle()`; `cpu.set_p(cpu.p & !m)`. SEP: idem avec `cpu.p | m`. 3 cycles. (set_p gere l'emulation et la mise a zero des octets hauts.)
- XCE: `bus.idle()`; `old_e = cpu.e`; `cpu.e = cpu.flag(FLAG_C)`; `set_flag(C, old_e)`; si `cpu.e`: `set_p(cpu.p)` (force M=X=1 et efface les octets hauts de X/Y) et `s = 0x0100 | (s & 0xFF)`. 2 cycles.
- XBA: `bus.idle(); bus.idle()`; `a = a.rotate_left(8)` (echange des octets); `set_nz8(a & 0xFF)`. 3 cycles.
- NOP: `bus.idle()`. 2 cycles.""",
  tests=[("flag_ops", "pour chacun de CLC SEC CLI SEI CLD SED CLV (18 38 58 78 D8 F8 B8): depart p=0xFF (e=false) pour les Clx, p=0x00 pour les Sxx: le bit correspondant (C,C,I,I,D,D,V) vaut la valeur attendue, les autres bits inchanges ; cycles=2."),
         ("rep_native", "e=false, p=0xFF, `C2 30`: p==0xCF ; cycles=3."),
         ("rep_keeps_emulation", "e=true, p=0x30, `C2 30`: p==0x30."),
         ("sep_clears_index_high", "e=false, p=0x00, x=0x1234, y=0x5678, `E2 10`: p==0x10, x==0x0034, y==0x0078 ; cycles=3."),
         ("xce_to_native", "e=true, p=0x30 (C=0), `FB`: e==false, C=1, p==0x31 ; cycles=2."),
         ("xce_to_emulation", "e=false, p=0x01 (M=0,X=0,C=1), x=0x1234, y=0x5678, s=0x1234, `FB`: e==true, C=0, p==0x30, x==0x0034, y==0x0078, s==0x0134."),
         ("xba", "a=0x1280, `EB`: a==0x8012, N=0 (octet bas 0x12), Z=0 ; cycles=3."),
         ("xba_zero", "a=0x0012, `EB`: a==0x1200, Z=1."),
         ("nop", "`EA`: pc==0x8001, cycles=2, registres inchanges.")],
  notes=CHAIN)

T("E02-39", "VERIF oracle: transferts, drapeaux, XCE/XBA/NOP",
  deps=["E02-38", "E02-14"], size=50,
  context="Passage a l'oracle des instructions restantes (famille misc).",
  reads=["specs/01_CPU_65816.md section 5.2", "specs/data/ssjson_format.md"],
  write=[CPU + "/misc/*.rs", TST + "/e02_39_oracle_regressions.rs"],
  spec="""1. Lancer: `%s`
2. Corriger les echecs dans `cpu/misc/*.rs` uniquement.
3. Regressions: `e02_39_<instruction>_<cas>`, sinon `e02_39_oracle_clean` (verifie `Cpu::new().p == 0x34`).
4. Fin: `failed: 0`.""" % oracle_cmd("TAX", "TAY", "TXA", "TYA", "TSX", "TXS", "TXY", "TYX", "TCD", "TDC", "TCS", "TSC", "CLC", "SEC", "CLI", "SEI", "CLD", "SED", "CLV", "REP", "SEP", "XCE", "XBA", "NOP"),
  tests=[("oracle_clean_or_regressions", "voir spec etape 3")],
  accept=[oracle_cmd("TAX", "TAY", "TXA", "TYA", "TSX", "TXS", "TXY", "TYX", "TCD", "TDC", "TCS", "TSC", "CLC", "SEC", "CLI", "SEI", "CLD", "SED", "CLV", "REP", "SEP", "XCE", "XBA", "NOP"),
          "cargo test -p puce16-core e02_"],
  notes=CHAIN)

T("E02-40", "interrupts.rs: poll (NMI/IRQ/WAI/STP), sequence d'interruption et RESET",
  deps=["E02-39"], size=130,
  context="Dernier morceau du CPU: prise en compte des interruptions a la frontiere d'instruction. Remplace le squelette de E02-05.",
  embed=["01b_CPU_DESIGN.md#4. Interruptions", "01_CPU_65816.md#8. Interruptions", EMB_TEST],
  write=[CPU + "/interrupts.rs", TST + "/e02_40_interrupts.rs"],
  spec="""```rust
pub fn poll<B: Bus>(cpu: &mut Cpu, bus: &mut B) -> bool;      // algorithme de 01b section 4
pub fn service<B: Bus>(cpu: &mut Cpu, bus: &mut B, nmi: bool);   // sequence materielle
pub fn reset<B: Bus>(cpu: &mut Cpu, bus: &mut B);
```
- `service`: `bus.read(cpu.pc24())` (opcode fantome, resultat ignore); `bus.idle()`; si `!cpu.e`: `push8(pb)`; `push8(pc >> 8)`; `push8(pc & 0xFF)` (pc = adresse de la PROCHAINE instruction, non incrementee); P pousse: en natif `cpu.p`; en emulation `(cpu.p | 0x20) & !0x10` (B=0); vecteur: NMI natif 0xFFEA / emulation 0xFFFA ; IRQ natif 0xFFEE / emulation 0xFFFE; `lo = read(vec)`, `hi = read(vec+1)`; `I=1`, `D=0`, `pb=0`, `pc = hi<<8|lo`. 8 cycles natif, 7 emulation.
- `poll`: exactement l'algorithme de 01b section 4 (ordre: halted; waiting; NMI avant IRQ).
- `reset`: `e = true`; `p = 0x34` (M, X, I; D=0 et le reste 0); `s = 0x01FF`; `db = pb = 0`; `d = 0`; `x &= 0xFF; y &= 0xFF`; `halted = waiting = false`; 5 appels `bus.idle()` (placeholder: UNKNOWN - to confirm) puis `lo = read(0xFFFC)`, `hi = read(0xFFFD)`; `pc = hi<<8|lo`.""",
  tests=[("nmi_native", "e=false, p=0x00, pb=0x02, pc=0x8123, s=0x01FF, poke(0xFFEA,&[0x00,0x90]), bus.nmi=true; `poll` renvoie true: pc==0x9000, pb==0, I=1, s==0x01FB, peek(0x01FF)==0x02, peek(0x01FE)==0x81, peek(0x01FD)==0x23, peek(0x01FC)==0x00 ; cycles=8 ; log = [R 0x028123, Idle, W, W, W, W, R 0xFFEA, R 0xFFEB] ; bus.nmi == false ensuite."),
         ("irq_emulation", "e=true, p=0x30 (I=0), pc=0x8123, s=0x01FF, poke(0xFFFE,&[0x00,0x88]), bus.irq=true: poll -> true ; pc==0x8800 ; peek(0x01FF)==0x81, peek(0x01FE)==0x23, peek(0x01FD)==0x20 (B=0) ; cycles=7 ; I=1."),
         ("irq_masked", "p avec I=1, bus.irq=true: poll -> false, journal vide, pc inchange."),
         ("nmi_priority_over_irq", "bus.nmi et bus.irq tous deux vrais, I=0: le vecteur pris est celui de NMI."),
         ("wai_waits", "waiting=true, rien en attente: poll -> true avec exactement 1 cycle Idle ; waiting reste true."),
         ("wai_wakes_on_masked_irq", "waiting=true, I=1, bus.irq=true: poll -> false, waiting==false, aucun cycle."),
         ("wai_nmi", "waiting=true, bus.nmi=true: poll -> true (sequence NMI), waiting==false."),
         ("stp", "halted=true: poll -> true avec 1 Idle ; halted reste true ; irq/nmi ignores."),
         ("reset", "poke(0xFFFC,&[0x00,0x80]); cpu avec e=false, p=0x00, x=0x1234, y=0x5678, s=0x0000, d=0x1111, db=5, pb=6: `reset`: e, p==0x34, pc==0x8000, pb==0, db==0, d==0, s==0x01FF, x==0x34, y==0x78 ; cycles=7."),
         ("step_takes_interrupt", "bus.nmi=true: `cpu.step(&mut bus)` execute la sequence NMI (pas d'instruction) -> pc == vecteur.")],
  notes=CHAIN)

T("E02-41", "GATE J1: tous les opcodes 65816 passent l'oracle",
  kind="gate", gate=True, size=0, deps=["E02-40"],
  context="Fin de l'epopee E02. Jalon J1 (specs/06_TEST_ROMS.md section 8): le CPU est valide par l'oracle, instruction par instruction.",
  spec="""Criteres de validation:
1. `cargo run --release -p puce16-cli -- cpu-test tests/oracle/65816` termine avec le code 0 (aucun FAIL).
2. `cargo test --workspace` est vert.
3. Aucun `unimplemented!` / `todo!` restant dans `crates/puce16-core/src/cpu/` (`grep -rn "unimplemented!\\|todo!" crates/puce16-core/src/cpu`).
4. Les questions `spec` ouvertes de la famille CPU dans `docs/qa/inbox/` sont toutes resolues (Claude).""",
  user_cmds=["cargo run --release -p puce16-cli -- cpu-test tests/oracle/65816", "cargo test --workspace",
             "python docs/tools/task.py approve E02-41"],
  accept=[])
