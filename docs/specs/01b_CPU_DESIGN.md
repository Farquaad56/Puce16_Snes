# SPEC 01b - CPU 65816: DECOUPAGE EN FICHIERS ET API INTERNE

Complete `01_CPU_65816.md` (comportement) par la structure de code. Le but: que les taches de famille d'instructions
soient **independantes** (un fichier chacune, aucune modification de fichier partage).
Les valeurs materielles viennent de `01_CPU_65816.md` ; l'oracle (tests JSON) tranche les cas `UNKNOWN - to confirm`.

## 1. Tables generees (`cpu/table.rs`)

`docs/tools/gen_tables.py` lit `docs/specs/data/opcodes_65816.tsv` et ecrit `crates/puce16-core/src/cpu/table.rs` (ne jamais l'editer a la main).
Contenu genere:
```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mn { Adc, And, Asl, Bcc, Bcs, Beq, Bit, Bmi, Bne, Bpl, Bra, Brk, Brl, Bvc, Bvs, Clc, Cld, Cli, Clv, Cmp, Cop, Cpx, Cpy,
              Dec, Dex, Dey, Eor, Inc, Inx, Iny, Jml, Jmp, Jsl, Jsr, Lda, Ldx, Ldy, Lsr, Mvn, Mvp, Nop, Ora, Pea, Pei, Per,
              Pha, Phb, Phd, Phk, Php, Phx, Phy, Pla, Plb, Pld, Plp, Plx, Ply, Rep, Rol, Ror, Rti, Rtl, Rts, Sbc, Sec, Sed,
              Sei, Sep, Sta, Stp, Stx, Sty, Stz, Tax, Tay, Tcd, Tcs, Tdc, Trb, Tsb, Tsc, Tsx, Txa, Txs, Txy, Tya, Tyx, Wai,
              Wdm, Xba, Xce }
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Am { Imp, Acc, Imm, Dp, DpX, DpY, Abs, AbsX, AbsY, Long, LongX, DpInd, DpIndLong, DpXInd, DpIndY, DpIndLongY,
              Sr, SrIndY, AbsInd, AbsXInd, AbsIndLong, Rel, RelL, Blk, Imm8, Pea, Pei, Per, Brk, Cop, Wdm }
pub static MN: [Mn; 256];     // mnemonique de chaque opcode
pub static AM: [Am; 256];     // mode de chaque opcode
impl Mn { pub fn name(self) -> &'static str }   // "ADC", "AND", ... (majuscules)
```
Correspondance colonne `mode` du TSV -> `Am`: imp=Imp, A=Acc, #=Imm, d=Dp, d,x=DpX, d,y=DpY, a=Abs, a,x=AbsX, a,y=AbsY, al=Long, al,x=LongX,
(d)=DpInd, [d]=DpIndLong, (d,x)=DpXInd, (d),y=DpIndY, [d],y=DpIndLongY, d,s=Sr, (d,s),y=SrIndY, (a)=AbsInd, (a,x)=AbsXInd, [a]=AbsIndLong,
rel=Rel, rell=RelL, blk=Blk, imm8=Imm8, pea=Pea, pei=Pei, per=Per, brk=Brk, cop=Cop, wdm=Wdm.

## 2. Contrat de `cpu/mod.rs` (API que toutes les familles utilisent)

```rust
pub const FLAG_C: u8 = 0x01; pub const FLAG_Z: u8 = 0x02; pub const FLAG_I: u8 = 0x04; pub const FLAG_D: u8 = 0x08;
pub const FLAG_X: u8 = 0x10; pub const FLAG_M: u8 = 0x20; pub const FLAG_V: u8 = 0x40; pub const FLAG_N: u8 = 0x80;

pub trait Bus {
    fn read(&mut self, addr: u32) -> u8;            // addr = 24 bits, un cycle de bus
    fn write(&mut self, addr: u32, v: u8);          // un cycle de bus
    fn idle(&mut self);                             // cycle interne (aucun acces)
    fn nmi_pending(&mut self) -> bool { false }     // true = un front NMI attend; l'appel le CONSOMME (efface le front)
    fn irq_asserted(&mut self) -> bool { false }    // niveau de la ligne IRQ (non consomme)
}

pub struct Cpu { pub a: u16, pub x: u16, pub y: u16, pub s: u16, pub d: u16, pub db: u8, pub pb: u8, pub pc: u16,
                 pub p: u8, pub e: bool, pub halted: bool, pub waiting: bool }
```
Invariant: si `e` est vrai alors `p & (FLAG_M|FLAG_X) == FLAG_M|FLAG_X`. Donc `m8()` = `p & FLAG_M != 0` suffit.

Methodes (toutes `pub`, dans `impl Cpu`; `B` = parametre generique `B: Bus`):
| Signature | Role |
|---|---|
| `new() -> Cpu` | a=x=y=0, s=0x01FF, d=0, db=pb=0, pc=0, p=0x34 (M, X, I), e=true, halted=waiting=false |
| `flag(&self, f: u8) -> bool` / `set_flag(&mut self, f: u8, v: bool)` | lecture/ecriture d'un flag de P |
| `m8(&self) -> bool` / `x8(&self) -> bool` | accumulateur / index 8 bits |
| `set_p(&mut self, v: u8)` | ecrit P: si `e` alors `p = v \| 0x30`; sinon `p = v`; puis si X=1: `x &= 0xFF; y &= 0xFF` |
| `set_nz8(&mut self, v: u8)` / `set_nz16(&mut self, v: u16)` / `set_nz(&mut self, v: u16, wide: bool)` | N = bit de signe, Z = (v==0) (seule la partie significative compte) |
| `pc24(&self) -> u32` | `(pb << 16) \| pc` |
| `fetch8<B>(&mut self, bus) -> u8` | `bus.read(pc24())` puis `pc = pc.wrapping_add(1)` (le PC ne change jamais de banque) |
| `fetch16<B>(&mut self, bus) -> u16` | deux `fetch8` (octet bas d'abord) |
| `fetch24<B>(&mut self, bus) -> u32` | trois `fetch8`, resultat `bank<<16 \| a16` |
| `push8<B>(&mut self, bus, v: u8)` | pile "ancienne": `bus.write(s as u32, v)`; puis `s -= 1` ; si `e`: `s = 0x0100 \| ((s) & 0xFF)` (boucle dans la page 1) |
| `pull8<B>(&mut self, bus) -> u8` | pile "ancienne": `s += 1` (si `e`: `s = 0x0100 \| (s & 0xFF)`) puis `bus.read(s as u32)` |
| `push8n<B>(&mut self, bus, v: u8)` / `pull8n<B>(&mut self, bus) -> u8` | pile "nouvelle" (16 bits, sans boucle de page): `write(s)`, `s = s.wrapping_sub(1)` ; `s = s.wrapping_add(1)`, `read(s)` |
| `fix_stack_e(&mut self)` | si `e`: `s = 0x0100 \| (s & 0xFF)` (a appeler en fin d'instruction "nouvelle") |
| `hash_into(&self, h: &mut Fnv64)` | ecrit a,x,y,s,d (u16), db,pb (u8), pc (u16), p (u8), e, halted, waiting (u8 0/1) dans cet ordre |
| `step<B>(&mut self, bus)` | une instruction complete ou une sequence d'interruption (section 4) |
| `reset<B>(&mut self, bus)` | delegue a `interrupts::reset` |

Les ecritures de pile 16 bits poussent l'octet HAUT d'abord puis l'octet bas (la pile descend); les lectures tirent le bas puis le haut.

## 3. Dispatch et familles

`step` execute: (1) `interrupts::poll(self, bus)` (si vrai, le pas est termine), (2) `let op = self.fetch8(bus)`, (3) `let mn = MN[op as usize]; let am = AM[op as usize];`
(4) `alu::exec(..) || mem::exec(..) || flow::exec(..) || misc::exec(..)`, avec la signature de famille:
```rust
pub(crate) fn exec<B: Bus>(cpu: &mut Cpu, bus: &mut B, mn: Mn, am: Am) -> bool;   // true = instruction traitee
```
Si aucune famille ne traite l'opcode: `panic!("unimplemented opcode {:02X} ({})", op, mn.name())` (disparait quand toutes les familles sont faites).
Chaque famille est un `match mn { ... , _ => return false }` qui renvoie `true` apres avoir execute.
Les familles sont decoupees en sous-fichiers (un fichier = une tache) declares dans le fichier de famille:
```
cpu/alu.rs   : mod adc; mod sbc; mod logic; mod cmp; mod incdec; mod shift; mod tsbtrb;      (alu/adc.rs ...)
cpu/mem.rs   : mod load; mod store; mod push; mod pull; mod block;
cpu/flow.rs  : mod branch; mod jump; mod call; mod sys;
cpu/misc.rs  : mod transfer; mod flags;
```
et chaque sous-fichier expose `pub(super) fn exec<B: Bus>(cpu: &mut Cpu, bus: &mut B, mn: Mn, am: Am) -> bool`; le fichier de famille essaie ses sous-fichiers dans l'ordre (`adc::exec(..) || sbc::exec(..) || ...`).
Mnemoniques par sous-fichier:
| Fichier | Mnemoniques |
|---|---|
| alu/adc.rs | Adc |
| alu/sbc.rs | Sbc |
| alu/logic.rs | And Ora Eor Bit |
| alu/cmp.rs | Cmp Cpx Cpy |
| alu/incdec.rs | Inc Dec (A et memoire) Inx Iny Dex Dey |
| alu/shift.rs | Asl Lsr Rol Ror (A et memoire) |
| alu/tsbtrb.rs | Tsb Trb |
| mem/load.rs | Lda Ldx Ldy |
| mem/store.rs | Sta Stx Sty Stz |
| mem/push.rs | Pha Phx Phy Php Phd Phb Phk Pea Pei Per |
| mem/pull.rs | Pla Plx Ply Plp Pld Plb |
| mem/block.rs | Mvn Mvp |
| flow/branch.rs | Bcc Bcs Beq Bne Bmi Bpl Bvc Bvs Bra Brl |
| flow/jump.rs | Jmp Jml |
| flow/call.rs | Jsr Jsl Rts Rtl |
| flow/sys.rs | Brk Cop Rti Wai Stp Wdm |
| misc/transfer.rs | Tax Tay Txa Tya Tsx Txs Txy Tyx Tcd Tdc Tcs Tsc |
| misc/flags.rs | Clc Sec Cli Sei Cld Sed Clv Rep Sep Xce Xba Nop |

## 4. Interruptions dans `step`

`interrupts::poll(cpu, bus) -> bool` (appele au debut de chaque `step`, c'est-a-dire a la frontiere d'instruction):
1. si `cpu.halted` (STP): `bus.idle()`; retourne vrai.
2. `let nmi = bus.nmi_pending(); let irq = bus.irq_asserted() && !cpu.flag(FLAG_I);`
3. si `cpu.waiting` (WAI): si `nmi` ou `bus.irq_asserted()` (meme si I=1): `waiting = false`; sinon `bus.idle()` et retourne vrai (le CPU reste a l'arret).
4. si `nmi`: sequence d'interruption NMI (vecteur natif $FFEA / emulation $FFFA), retourne vrai.
5. sinon si `irq`: sequence IRQ (natif $FFEE / emulation $FFFE), retourne vrai.
6. sinon retourne faux.
(Quand WAI est reveille par une IRQ avec I=1: `waiting=false`, puis le pas continue normalement vers l'instruction suivante: le fetch de l'opcode se fait dans le meme `step`: etape 3 ne retourne pas vrai dans ce cas.)
Un NMI consomme est perdu s'il n'est pas pris: l'appel `nmi_pending()` n'a lieu qu'a l'etape 2 et son resultat est toujours utilise (4).

## 5. Cycles des instructions implicites et de pile (complement de 01 section 7)

Apres le cycle opcode. `IO` = `bus.idle()`.
| Instruction | Cycles |
|---|---|
| PHA PHX PHY (8 bits) | `IO`; push (total 3). 16 bits: `IO`; push octet haut; push octet bas (total 4) |
| PHP PHB PHK | `IO`; push (total 3) |
| PHD | `IO`; push octet haut de D; push octet bas (total 4, pile "nouvelle") |
| PLA PLX PLY (8 bits) | `IO`; `IO`; pull (total 4). 16 bits: pull octet bas puis octet haut (total 5) |
| PLP PLB | `IO`; `IO`; pull (total 4) |
| PLD | `IO`; `IO`; pull bas; pull haut (total 5, pile "nouvelle") |
| PEA | `R(pc++)`=lo; `R(pc++)`=hi; push hi; push lo (total 5, pile "nouvelle") |
| PEI | `R(pc++)`=dp; [`IO` si DL != 0]; lit le mot en D+dp (lo puis hi, wrap bank 0); push hi; push lo (total 6, +1 si DL != 0; pile "nouvelle") |
| PER | `R(pc++)`=lo; `R(pc++)`=hi; `IO`; push de `(pc_apres_operande + off16)` octet haut puis bas (total 6; pile "nouvelle") |
| Transferts (TAX ...), CLC SEC CLI SEI CLD SED CLV, NOP, INX DEX INY DEY, formes accumulateur (INC A ...) | `IO` (total 2) |
| REP SEP | `R(pc++)`=masque; `IO` (total 3) |
| XBA | `IO`; `IO` (total 3) |
| XCE | `IO` (total 2) |
Les instructions "nouvelles" appellent `cpu.fix_stack_e()` en fin d'instruction.
PHP pousse P tel quel (en emulation: bits 5 et 4 = 1 car M=X=1: B=1 pour PHP). PLP appelle `set_p(valeur)`.

## 6. Tests d'unite des familles (convention)

Les tests utilisent `FlatBus` (`cpu/testbus.rs`): `FlatBus::new()` ; `bus.poke(addr, &[octets])` ecrit sans journal ; `bus.peek(addr) -> u8` ;
`bus.log: Vec<Cycle>` avec `Cycle { addr: u32, val: u8, kind: CycleKind }`, `CycleKind::{Read, Write, Idle}` (un cycle `Idle` a `addr = 0` et `val = 0`) ;
`bus.clear_log()`. Un test pose le programme en `00:8000` (`cpu.pb = 0; cpu.pc = 0x8000`), fixe les registres a la main, fait `cpu.step(&mut bus)`, puis verifie
registres, memoire et `bus.log.len()` (= nombre de cycles de l'instruction, opcode compris) et, quand c'est precise, la suite exacte des cycles.
Un test peut ecrire `let cycles: Vec<(char, u32, u8)>` ou 'R'/'W'/'I' est le type: a lui de choisir une forme lisible, tant que adresse, valeur, ordre sont verifies.

## 7. UNKNOWN - to confirm (laisses a l'oracle)
- Valeur de S apres RESET (octet bas) et nombre exact de cycles de la sequence de RESET (par defaut: 7 cycles comme une interruption; S=0x01FF).
- Adresse affichee sur le bus pendant les cycles `IO` (non modelisee: `idle()` sans adresse). Si l'oracle l'exige, Claude modifie le trait `Bus` (decision dans DECISIONS.md), pas Hermes.
- Cas de pile "nouvelle" en emulation (PEA PEI PER PHD PLD JSL RTL...): le comportement exact de S.h est a confirmer par l'oracle.
