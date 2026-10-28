from e06_a import sops, stsv, spc_cmd, CHAIN, READS, RUN, SPC, TST, CORE, CLI, ORACLE
from _dsl import T

FORMS = ("Formes (rappel de 07c section 4): `form = op & 0x1F`: `0x04` A,dp ; `0x05` A,!abs ; `0x06` A,(X) ; `0x07` A,[dp+X] ; `0x08` A,#imm ; `0x09` dp,dp ; "
         "`0x14` A,dp+X ; `0x15` A,!abs+X ; `0x16` A,!abs+Y ; `0x17` A,[dp]+Y ; `0x18` dp,#imm ; `0x19` (X),(Y). "
         "Formes A: `v = lecture (ou fetch8 pour #imm)` puis `a = op(a, v)`. `dp,dp`: `src = fetch8`, `dst = fetch8`, lit src puis dst, `dst = op(dst, src)`. "
         "`dp,#imm`: `imm = fetch8`, `dst = fetch8`, lit dst, `dst = op(dst, imm)`. `(X),(Y)`: lit `P|Y` puis `P|X`, `(X) = op((X), (Y))`.")

# ------------------------------------------------------------------ E06 part B: ALU, mots 16 bits, bits, flot (E06-15..30)

T("E06-15", "spc700/alu.rs: OR, AND, EOR (toutes les formes)",
  size=120,
  context="Premiere famille ALU: logique 8 bits. Cree le dispatch generique des 6 familles (OR AND EOR CMP ADC SBC) par `kind = op & 0xE0` et `form = op & 0x1F`; E06-16 et E06-17 ajoutent les autres `kind`. N,Z seulement (C,V,H inchanges). " + CHAIN + " " + RUN,
  embed=["07c_SPC_DESIGN.md#4. Recettes d'adressage", "07_SPC700.md#6.2 Logique et arithmetique 8 bits"],
  tsv=stsv("OR", "AND", "EOR"), reads=READS,
  write=[SPC + "/alu.rs", TST + "/e06_15_logic.rs"],
  spec="""Dans `alu.rs`:
```rust
impl Spc700 { pub fn or_op(&mut self, a: u8, b: u8) -> u8; pub fn and_op(&mut self, a: u8, b: u8) -> u8; pub fn eor_op(&mut self, a: u8, b: u8) -> u8; }  // resultat + set_nz
pub(super) fn exec<B: SmpBus>(cpu: &mut Spc700, bus: &mut B, op: u8) -> bool;
```
`exec`: `kind = op & 0xE0`, `form = op & 0x1F`; si `form` n'est pas dans la liste des 12 formes -> `false`. Ecris une fonction privee `alu_form(cpu, bus, op, f: fn(&mut Spc700, u8, u8) -> u8, write_back: bool) -> bool` qui gere les 12 formes (`write_back = false` servira a CMP en E06-17), puis ici: `kind` 0x00 -> `or_op`, 0x20 -> `and_op`, 0x40 -> `eor_op` (les autres `kind`: `false` pour l'instant). """ + FORMS,
  tests=[("or_imm", "a=0x0F, `08 F0`: a==0xFF, N=1, Z=0, cycles 2."),
         ("and_imm_zero", "a=0xF0, `28 0F`: a==0, Z=1, N=0."),
         ("eor_imm", "a=0x0F, `48 FF`: a==0xF0, N=1."),
         ("or_dp", "a=0x80, poke(0x10,&[0x01]), `04 10`: a==0x81, N=1, cycles 3."),
         ("and_abs", "a=0x3C, poke(0x3000,&[0x0F]), `25 00 30`: a==0x0C, cycles 4."),
         ("eor_x_indirect", "x=0x10, poke(0x10,&[0xFF]), a=0xFF, `46`: a==0, Z=1, cycles 3."),
         ("or_dpx_indirect", "x=2, poke(0x12,&[0x00,0x30]), poke(0x3000,&[0x40]), a=0x01, `07 10`: a==0x41, cycles 6."),
         ("and_absx", "x=1, poke(0x3000,&[0x55]), a=0xFF, `35 FF 2F`: a==0x55, cycles 5."),
         ("eor_absy", "y=2, poke(0x3002,&[0xAA]), a=0xFF, `56 00 30`: a==0x55, cycles 5."),
         ("or_dp_indirect_y", "y=1, poke(0x20,&[0xFF,0x2F]), poke(0x3000,&[0x08]), a=0x01, `17 20`: a==0x09, cycles 6."),
         ("or_dpx", "x=1, poke(0x11,&[0x02]), a=1, `14 10`: a==3, cycles 4."),
         ("or_dp_dp", "poke(0x10,&[0x0F]), poke(0x20,&[0xF0]), `09 10 20`: peek(0x20)==0xFF, N=1, cycles 6 (resultat = destination OR source)."),
         ("and_dp_dp", "poke(0x10,&[0x3C]), poke(0x20,&[0xF0]), `29 10 20`: peek(0x20)==0x30, N=0, Z=0."),
         ("eor_dp_dp_zero", "poke(0x10,&[0xFF]), poke(0x20,&[0xFF]), `49 10 20`: peek(0x20)==0, Z=1."),
         ("or_dp_imm", "poke(0x10,&[0xF0]), `18 0F 10`: peek(0x10)==0xFF, cycles 5."),
         ("and_dp_imm", "poke(0x10,&[0x3C]), `38 F0 10`: peek(0x10)==0x30 (octets `op imm dp`)."),
         ("eor_xy", "x=0x10, y=0x11, poke(0x10,&[0xF0]), poke(0x11,&[0xFF]), `59`: peek(0x10)==0x0F, N=0, cycles 5 ; `19` (OR) avec peek(0x10)=0x01, peek(0x11)=0x02 -> peek(0x10)==0x03."),
         ("other_flags_untouched", "psw=0x49 (C,H,V), a=0x01, `08 01`: psw==0x49."),
         ("p_flag", "psw=F_P, poke(0x0110,&[0x01]), a=0, `04 10`: a==1.")],
  notes=CHAIN)

T("E06-16", "spc700/alu.rs: ADC et SBC (toutes les formes)",
  size=120,
  context="Addition et soustraction avec retenue. Flags N V H Z C. SBC est exactement `ADC a, !b` (07 section 6.2). " + CHAIN + " " + RUN,
  embed=["07_SPC700.md#6.2 Logique et arithmetique 8 bits"],
  tsv=stsv("ADC", "SBC"), reads=READS,
  write=[SPC + "/alu.rs", TST + "/e06_16_adc_sbc.rs"],
  spec="""Dans `alu.rs`:
```rust
impl Spc700 { pub fn adc_op(&mut self, a: u8, b: u8) -> u8; pub fn sbc_op(&mut self, a: u8, b: u8) -> u8; }
```
`adc_op`: `cin = flag(C)`; `r = a as u32 + b as u32 + cin` ; `C = r > 0xFF` ; `H = ((a & 0xF) + (b & 0xF) + cin) > 0xF` ; `V = (!(a ^ b) & (a ^ (r as u8)) & 0x80) != 0` ; `set_nz(r as u8)` ; retourne `r as u8`. `sbc_op(a, b) = adc_op(a, !b)`.
Dans `exec`: `kind` 0x80 -> `adc_op`, 0xA0 -> `sbc_op`, via `alu_form(.., write_back = true)` (E06-15). """ + FORMS,
  tests=[("adc_signed_overflow", "psw=0, a=0x7F, `88 01`: a==0x80, N=1, V=1, H=1, C=0, Z=0, cycles 2."),
         ("adc_carry_out", "psw=0, a=0x01, `88 FF`: a==0x00, Z=1, C=1, H=1, V=0, N=0."),
         ("adc_carry_in", "psw=F_C, a=0x00, `88 01`: a==2, tous les flags N,V,H,C,Z a 0 (psw==0x00)."),
         ("adc_neg_neg", "psw=0, a=0x80, `88 80`: a==0, C=1, Z=1, V=1, H=0, N=0."),
         ("sbc_borrow", "psw=0 (C=0), a=0x00, `A8 01`: a==0xFE, N=1, C=0, Z=0, V=0, H=0."),
         ("sbc_no_borrow", "psw=F_C, a=0x05, `A8 01`: a==0x04, C=1, H=1, N=0, Z=0, V=0."),
         ("sbc_overflow", "psw=F_C, a=0x80, `A8 01`: a==0x7F, C=1, V=1, N=0, Z=0, H=0."),
         ("adc_dp", "psw=0, a=0x20, poke(0x10,&[0x10]), `84 10`: a==0x30, cycles 3."),
         ("adc_dp_dp", "psw=0, poke(0x10,&[0x01]), poke(0x20,&[0x02]), `89 10 20`: peek(0x20)==0x03, cycles 6."),
         ("adc_dp_imm", "psw=0, poke(0x10,&[0x0A]), `98 05 10`: peek(0x10)==0x0F, cycles 5."),
         ("adc_xy", "psw=0, x=0x10, y=0x11, poke(0x10,&[0x01]), poke(0x11,&[0x01]), `99`: peek(0x10)==0x02, cycles 5."),
         ("sbc_dp_dp", "psw=F_C, poke(0x10,&[0x01]), poke(0x20,&[0x05]), `A9 10 20`: peek(0x20)==0x04 (destination moins source), C=1."),
         ("sbc_xy", "psw=F_C, x=0x10, y=0x11, poke(0x10,&[0x03]), poke(0x11,&[0x01]), `B9`: peek(0x10)==0x02."),
         ("adc_forms_cycles", "psw=0, a=0, la valeur 0x11 placee a l'adresse effective; pour chaque (octets, cycles) : (`85 00 30`, 4), (`86` avec x=0x10 et poke(0x10), 3), (`87 10` avec x=2, poke(0x12,&[0,0x30]), poke(0x3000), 6), (`94 10` x=1 poke(0x11), 4), (`95 FF 2F` x=1 poke(0x3000), 5), (`96 00 30` y=2 poke(0x3002), 5), (`97 20` y=1 poke(0x20,&[0xFF,0x2F]) poke(0x3000), 6) : a==0x11 et le nombre de cycles indique."),
         ("sbc_forms_cycles", "memes cinq formes avec les opcodes SBC (`A5`, `A6`, `A7`, `B4`, `B5`, `B6`, `B7`), psw=F_C, a=0x20, valeur 0x11 : a==0x0F.")],
  notes=CHAIN)

T("E06-17", "spc700/alu.rs: CMP (A, X, Y et formes memoire)",
  size=110,
  context="Comparaisons: aucun resultat ecrit, flags N,Z,C seulement (V et H inchanges). " + CHAIN + " " + RUN,
  embed=["07_SPC700.md#6.2 Logique et arithmetique 8 bits"],
  tsv=stsv("CMP"), reads=READS,
  write=[SPC + "/alu.rs", TST + "/e06_17_cmp.rs"],
  spec="""Dans `alu.rs`:
```rust
impl Spc700 { pub fn cmp_op(&mut self, a: u8, b: u8) -> u8; }   // r = a.wrapping_sub(b); C = (a >= b); set_nz(r); retourne r
```
Dans `exec`: `kind` 0x60 -> `alu_form(.., cmp_op, write_back = false)` (aucune ecriture, pour dp,dp / dp,#imm / (X),(Y) la comparaison est `destination - source`). Ajoute les comparaisons X et Y (`cmp_op(x, v)`, `cmp_op(y, v)`): `C8` CMP X,#imm ; `3E` X,dp ; `1E` X,!abs ; `AD` CMP Y,#imm ; `7E` Y,dp ; `5E` Y,!abs. """ + FORMS,
  tests=[("cmp_equal", "psw=0, a=0x10, `68 10`: Z=1, C=1, N=0 ; a inchange ; cycles 2."),
         ("cmp_less", "psw=0, a=0x10, `68 11`: C=0, N=1, Z=0 (0x10-0x11 = 0xFF)."),
         ("cmp_greater", "psw=0, a=0x10, `68 0F`: C=1, N=0, Z=0."),
         ("cmp_keeps_v_h", "psw=0x48 (V,H), a=0x10, `68 10`: psw==0x4B (V,H gardes ; Z,C mis)."),
         ("cmp_a_dp", "a=0x10, poke(0x10,&[0x10]), `64 10`: Z=1, C=1, cycles 3 ; a inchange."),
         ("cmp_a_abs_x_forms", "a=0x05, valeur 0x05 aux adresses effectives : `65 00 30` (4), `66` x=0x10 (3), `67 10` x=2 poke(0x12,&[0,0x30]) (6), `74 10` x=1 (4), `75 FF 2F` x=1 (5), `76 00 30` y=2 (5), `77 20` y=1 poke(0x20,&[0xFF,0x2F]) (6) : Z=1, C=1 et les cycles indiques."),
         ("cmp_x_imm", "psw=0, x=5, `C8 05`: Z=1, C=1 ; x=5, `C8 06`: C=0, N=1."),
         ("cmp_y_imm", "psw=0, y=5, `AD 06`: C=0, N=1, Z=0."),
         ("cmp_x_dp_abs", "x=0x80, poke(0x10,&[0x01]), `3E 10`: C=1, N=0, Z=0 (0x7F) ; x=0, poke(0x3000,&[0x01]), `1E 00 30`: C=0, N=1 (0xFF), cycles 4."),
         ("cmp_y_dp_abs", "y=2, poke(0x10,&[0x02]), `7E 10`: Z=1, C=1, cycles 3 ; y=2, poke(0x3000,&[0x03]), `5E 00 30`: C=0, N=1."),
         ("cmp_dp_dp_no_write", "psw=0, poke(0x10,&[0x03]), poke(0x20,&[0x05]), `69 10 20`: C=1, Z=0, N=0 ; peek(0x20)==5 (inchange) ; le journal ne contient AUCUNE ecriture ; cycles 6."),
         ("cmp_dp_imm", "psw=0, poke(0x20,&[0x05]), `78 03 20`: C=1 ; peek(0x20)==5 ; cycles 5."),
         ("cmp_xy", "psw=0, x=0x10, y=0x11, poke(0x10,&[0x01]), poke(0x11,&[0x02]), `79`: C=0, N=1 (0x01-0x02) ; cycles 5.")],
  notes=CHAIN)

T("E06-18", "VERIF oracle: OR AND EOR ADC SBC CMP",
  size=60, kind="code",
  context="Passage a l'oracle de toute l'ALU 8 bits. Les erreurs de flags H/V (ADC/SBC) et de modes memoire se corrigent ICI.",
  reads=["specs/07_SPC700.md section 6.2", "specs/data/spcjson_format.md"],
  write=[SPC + "/alu.rs", SPC + "/addr.rs", TST + "/e06_18_oracle_regressions.rs"],
  spec="""1. Lancer: `%s`
2. Pour chaque echec: lire le diff (champ fautif), localiser le bug dans les fichiers autorises, corriger. Ne pas modifier le harnais (cli), `table.rs` ni les specs.
3. Si le diff semble indiquer que la SPEC ou le TSV est faux: question `spec` (docs/02_QA_PROTOCOL.md) avec le test fautif, pas de changement a la volee.
4. Pour chaque bug corrige, ajouter dans `tests/e06_18_oracle_regressions.rs` un test unitaire (FlatSmpBus) qui le reproduit (nom `e06_18_<instruction>_<cas>`). Si aucun bug: un test `e06_18_oracle_clean` qui verifie seulement que `Spc700::new()` existe.
5. Finir quand la commande renvoie `failed: 0`.""" % spc_cmd("OR", "AND", "EOR", "ADC", "SBC", "CMP"),
  tests=[("oracle_clean_or_regressions", "voir spec etape 4")],
  accept=[spc_cmd("OR", "AND", "EOR", "ADC", "SBC", "CMP"), "cargo test -p puce16-core e06_"],
  notes=CHAIN)

T("E06-19", "spc700/incdec.rs: INC, DEC, ASL, LSR, ROL, ROR, XCN",
  size=140,
  context="Incrementations, decalages et rotations sur A, X, Y ou memoire (lecture-modification-ecriture), plus XCN. " + CHAIN + " " + RUN,
  embed=["07_SPC700.md#6.2 Logique et arithmetique 8 bits"],
  tsv=stsv("INC", "DEC", "ASL", "LSR", "ROL", "ROR", "XCN"), reads=READS,
  write=[SPC + "/incdec.rs", TST + "/e06_19_incdec.rs"],
  spec="""Dans `incdec.rs::exec`:
- INC: `BC` A ; `3D` X ; `FC` Y ; `AB` dp ; `BB` dp+X ; `AC` !abs. DEC: `9C` A ; `1D` X ; `DC` Y ; `8B` dp ; `9B` dp+X ; `8C` !abs. Flags N,Z seulement (C inchange).
- ASL: `1C` A ; `0B` dp ; `1B` dp+X ; `0C` !abs. LSR: `5C` A ; `4B` dp ; `5B` dp+X ; `4C` !abs. ROL: `3C` A ; `2B` dp ; `3B` dp+X ; `2C` !abs. ROR: `7C` A ; `6B` dp ; `7B` dp+X ; `6C` !abs. Flags N,Z,C selon 07 section 6.2.
- Forme memoire: `ea = addr::ea_dp / ea_dpx / ea_abs` ; `v = rd(ea)` ; `wr(ea, f(v))`.
- `9F` XCN A: `a = (a << 4) | (a >> 4)` ; N,Z.
Ecris des fonctions pures privees `fn asl8(cpu, v) -> u8`, `lsr8`, `rol8`, `ror8` qui mettent N,Z,C et reutilise-les pour les 4 formes de chaque instruction.""",
  tests=[("inc_a_wrap", "psw=F_C, a=0xFF, `BC`: a==0, Z=1, N=0, C=1 (inchange), cycles 2."),
         ("dec_a_wrap", "a=0, `9C`: a==0xFF, N=1, Z=0."),
         ("inc_dec_x_y", "x=0x7F `3D`: x==0x80, N=1 ; x=1 `1D`: x==0, Z=1 ; y=0xFF `FC`: y==0, Z=1 ; y=0x80 `DC`: y==0x7F, N=0."),
         ("inc_dp", "poke(0x10,&[0x7F]), `AB 10`: peek(0x10)==0x80, N=1, cycles 4 ; journal = [.., Read 0x0010, Write 0x0010=0x80]."),
         ("inc_dpx_zero", "x=1, poke(0x11,&[0xFF]), `BB 10`: peek(0x11)==0, Z=1, cycles 5."),
         ("inc_abs", "poke(0x3000,&[1]), `AC 00 30`: peek(0x3000)==2, cycles 5."),
         ("dec_memory", "poke(0x10,&[0]) `8B 10`: peek==0xFF, N=1, cycles 4 ; x=1 poke(0x11,&[1]) `9B 10`: peek(0x11)==0, Z=1, cycles 5 ; poke(0x3000,&[5]) `8C 00 30`: peek==4, cycles 5."),
         ("asl", "a=0x81 `1C`: a==0x02, C=1, N=0, cycles 2 ; poke(0x10,&[0x40]) `0B 10`: peek==0x80, N=1, C=0, cycles 4 ; x=1 poke(0x11,&[0x80]) `1B 10`: peek==0, Z=1, C=1, cycles 5 ; poke(0x3000,&[1]) `0C 00 30`: peek==2, cycles 5."),
         ("lsr", "a=0x01 `5C`: a==0, Z=1, C=1, N=0 ; a=0x80 `5C`: a==0x40, C=0 ; poke(0x10,&[0x03]) `4B 10`: peek==1, C=1 ; x=1 poke(0x11,&[2]) `5B 10`: peek==1, C=0 ; `4C 00 30` memoire 0x10 -> 0x08."),
         ("rol", "psw=0, a=0x80 `3C`: a==0, Z=1, C=1 ; psw=F_C, a=0x00 `3C`: a==1, C=0, Z=0 ; psw=F_C poke(0x10,&[0x40]) `2B 10`: peek==0x81, N=1, C=0 ; x=1 `3B 10`, abs `2C 00 30` sur des valeurs simples (resultat = (v<<1)|C)."),
         ("ror", "psw=0, a=0x01 `7C`: a==0, Z=1, C=1 ; psw=F_C, a=0 `7C`: a==0x80, N=1, C=0 ; psw=F_C poke(0x10,&[0x02]) `6B 10`: peek==0x81, C=0 ; x=1 `7B 10`, abs `6C 00 30` (resultat = (v>>1)|(C<<7))."),
         ("xcn", "a=0x1F `9F`: a==0xF1, N=1, cycles 5 ; a=0 `9F`: Z=1 ; a=0x0A: a==0xA0, N=1."),
         ("shift_cycles", "`1C` 2, `0B 10` 4, `1B 10` 5, `0C 00 30` 5, `7C` 2 (verifie par le nombre de cycles renvoye).")],
  notes=CHAIN)

T("E06-20", "spc700/misc.rs: DAA, DAS, MUL, DIV",
  size=110,
  context="Arithmetique speciale du SPC700. `DIV` ne doit JAMAIS paniquer (X = 0 compris): calculs en u32. " + CHAIN + " " + RUN,
  embed=["07_SPC700.md#6.2 Logique et arithmetique 8 bits"],
  tsv=stsv("DAA", "DAS", "MUL", "DIV"), reads=READS,
  write=[SPC + "/misc.rs", TST + "/e06_20_arith.rs"],
  spec="""Dans `misc.rs::exec`:
- `DF` DAA A: `if C || A > 0x99 { A = A.wrapping_add(0x60); C = 1 }` puis `if H || (A & 0xF) > 9 { A = A.wrapping_add(6) }` ; `set_nz(A)`.
- `BE` DAS A: `if !C || A > 0x99 { A = A.wrapping_sub(0x60); C = 0 }` puis `if !H || (A & 0xF) > 9 { A = A.wrapping_sub(6) }` ; `set_nz(A)`.
- `CF` MUL YA: `r = y as u16 * a as u16` ; `a = r & 0xFF` ; `y = r >> 8` ; N,Z selon **Y**.
- `9E` DIV YA,X: `ya = (y << 8) | a` (u32) ; `V = (y >= x)` ; `H = ((y & 15) >= (x & 15))` ; `if (y as u32) < ((x as u32) << 1) { a = ya / x ; y = ya % x }` sinon `a = 255 - (ya - (x << 9)) / (256 - x)` et `y = x + (ya - (x << 9)) % (256 - x)` ; resultats tronques a 8 bits ; N,Z selon A. Si `x == 0` la premiere branche n'est jamais prise (`y < 0` faux), donc pas de division par zero ; utilise des `u32` et `wrapping_sub` pour ne jamais paniquer.""",
  tests=[("daa_low_nibble", "psw=0, a=0x0A, `DF`: a==0x10, N=0, Z=0, cycles 3."),
         ("daa_carry_out", "psw=0, a=0x9A, `DF`: a==0x00, C=1, Z=1."),
         ("daa_half_carry", "psw=F_H, a=0x05, `DF`: a==0x0B, H inchange."),
         ("daa_carry_in", "psw=F_C, a=0x12, `DF`: a==0x72, C=1."),
         ("das_no_adjust", "psw=F_C|F_H, a=0x25, `BE`: a==0x25."),
         ("das_borrow", "psw=F_H (C=0), a=0x25, `BE`: a==0xC5, N=1, C=0."),
         ("das_low", "psw=F_C (H=0), a=0x00, `BE`: a==0xFA, N=1."),
         ("mul", "y=0x10, a=0x10, `CF`: a==0x00, y==0x01, N=0, Z=0, cycles 9 ; y=0xFF, a=0xFF: a==0x01, y==0xFE, N=1 ; y=0, a=5: a==0, y==0, Z=1."),
         ("div_normal", "y=0, a=0x64, x=0x0A, `9E`: a==10, y==0, V=0, H=0, N=0, Z=0, cycles 12."),
         ("div_remainder", "y=1, a=0x05, x=2, `9E`: a==0x82, y==1, V=0, H=0, N=1."),
         ("div_overflow_branch", "y=2, a=0, x=1, `9E`: a==0xFF, y==1, V=1, H=1, N=1."),
         ("div_by_zero_no_panic", "y=0x12, a=0x34, x=0, `9E`: pas de panique ; a==0xED, y==0x34, V=1, H=1, N=1.")],
  notes=CHAIN)

T("E06-21", "spc700/word.rs: ADDW, SUBW, CMPW, INCW, DECW",
  size=120,
  context="Arithmetique 16 bits sur un mot de la page directe (octet haut en `P|((b+1)&0xFF)`). Reutilise `adc_op`/`sbc_op` (E06-16). " + CHAIN + " " + RUN,
  embed=["07_SPC700.md#6.3 16 bits (operande `dp`, octet haut en `P|((b+1)&0xFF)`)"],
  tsv=stsv("ADDW", "SUBW", "CMPW", "INCW", "DECW"), reads=READS,
  write=[SPC + "/word.rs", TST + "/e06_21_word.rs"],
  spec="""Dans `word.rs::exec` (`b = fetch8`, `a0 = P | b`, `a1 = P | ((b + 1) & 0xFF)`):
- `7A` ADDW YA,dp: `lo = rd(a0)`, `hi = rd(a1)` ; `set_flag(C, false)` ; `a = adc_op(a, lo)` ; `y = adc_op(y, hi)` ; puis `set_flag(Z, ya() == 0)` (N,V,H,C viennent du 2e `adc_op`).
- `9A` SUBW YA,dp: `lo`, `hi` lus ; `set_flag(C, true)` ; `a = sbc_op(a, lo)` ; `y = sbc_op(y, hi)` ; `set_flag(Z, ya() == 0)`.
- `5A` CMPW YA,dp: `w = (hi << 8) | lo` ; `r = ya() as u32 - w as u32` (wrapping 16 bits) ; `C = ya() >= w` ; `N` = bit 15 de `r & 0xFFFF` ; `Z = (r & 0xFFFF) == 0` ; V,H inchanges.
- `3A` INCW dp / `1A` DECW dp, dans CET ordre d'acces: `lo = rd(a0)` ; `new_lo = lo +/- 1` (wrapping) ; `wr(a0, new_lo)` ; `hi = rd(a1)` ; `new_hi = hi` plus 1 si INCW et `new_lo == 0`, moins 1 si DECW et `lo == 0` (wrapping) ; `wr(a1, new_hi)` ; `N` = bit 7 de `new_hi`, `Z = (new_lo == 0 && new_hi == 0)`.""",
  tests=[("addw_basic", "psw=0, y=0x01, a=0xFF, poke(0x10,&[0x01,0x00]), `7A 10`: y==0x02, a==0x00, N=0, Z=0, C=0, V=0, H=0, cycles 5."),
         ("addw_signed_overflow", "psw=0, y=0x7F, a=0xFF, poke(0x10,&[0x01,0x00]), `7A 10`: ya==0x8000, N=1, V=1, H=1, C=0, Z=0."),
         ("addw_carry_zero", "psw=0, y=0xFF, a=0xFF, poke(0x10,&[0x01,0x00]), `7A 10`: ya==0x0000, Z=1, C=1, H=1, N=0, V=0 (psw==0x0B)."),
         ("addw_ignores_carry_in", "psw=F_C, y=0, a=0, poke(0x10,&[0,0]), `7A 10`: ya==0, Z=1, C=0."),
         ("subw_basic", "psw=0, y=0x02, a=0x00, poke(0x10,&[0x01,0x00]), `9A 10`: ya==0x01FF, C=1, H=1, N=0, Z=0, V=0 (psw==0x09), cycles 5."),
         ("subw_zero", "psw=0, y=0x12, a=0x34, poke(0x10,&[0x34,0x12]), `9A 10`: ya==0, Z=1, C=1, H=1, N=0, V=0 (psw==0x0B)."),
         ("cmpw_equal", "psw=0, y=0x12, a=0x34, poke(0x10,&[0x34,0x12]), `5A 10`: Z=1, C=1, N=0 ; ya inchange ; cycles 4."),
         ("cmpw_less", "poke(0x10,&[0x35,0x12]): C=0, N=1, Z=0 ; poke(0x10,&[0x33,0x12]): C=1, N=0, Z=0."),
         ("cmpw_keeps_v_h", "psw=0x48, ya=0x1234 vs 0x1234, `5A 10`: V et H toujours a 1."),
         ("incw", "poke(0x10,&[0xFF,0x00]), `3A 10`: peek(0x10)==0x00, peek(0x11)==0x01, N=0, Z=0, cycles 6 ; journal = [.., Read 0x0010, Write 0x0010, Read 0x0011, Write 0x0011] ; mot 0xFFFF -> 0: Z=1 ; mot 0x7FFF -> 0x8000: N=1."),
         ("decw", "poke(0x10,&[0x00,0x01]), `1A 10`: peek(0x10)==0xFF, peek(0x11)==0x00, cycles 6 ; mot 0x0000 -> 0xFFFF: N=1 ; mot 0x0001 -> 0: Z=1."),
         ("page_wrap", "poke(0x00FF,&[0xFF]), poke(0x0000,&[0x00]), `3A FF`: peek(0x00FF)==0x00, peek(0x0000)==0x01 (le haut est lu et ecrit en 0x0000)."),
         ("p_flag", "psw=F_P, poke(0x0110,&[0x01,0x00]), `3A 10`: peek(0x0110)==0x02.")],
  notes=CHAIN)

T("E06-22", "VERIF oracle: INC DEC ASL LSR ROL ROR XCN DAA DAS MUL DIV et mots 16 bits",
  size=60, kind="code",
  context="Passage a l'oracle des decalages, de l'arithmetique speciale et des instructions 16 bits.",
  reads=["specs/07_SPC700.md sections 6.2 et 6.3", "specs/data/spcjson_format.md"],
  write=[SPC + "/incdec.rs", SPC + "/misc.rs", SPC + "/word.rs", SPC + "/alu.rs", TST + "/e06_22_oracle_regressions.rs"],
  spec="""1. Lancer: `%s`
2. Pour chaque echec: lire le diff, localiser le bug dans les fichiers autorises, corriger. Ne pas modifier le harnais, `table.rs` ni les specs.
3. Si le diff indique une SPEC ou un TSV faux (ex: flags de DIV/DAA, cycles de CMPW): question `spec` (docs/02_QA_PROTOCOL.md), pas de changement a la volee.
4. Pour chaque bug corrige, ajouter dans `tests/e06_22_oracle_regressions.rs` un test unitaire (FlatSmpBus) nomme `e06_22_<instruction>_<cas>`. Si aucun bug: un test `e06_22_oracle_clean`.
5. Finir quand la commande renvoie `failed: 0`.""" % spc_cmd("INC", "DEC", "ASL", "LSR", "ROL", "ROR", "XCN", "DAA", "DAS", "MUL", "DIV", "ADDW", "SUBW", "CMPW", "INCW", "DECW"),
  tests=[("oracle_clean_or_regressions", "voir spec etape 4")],
  accept=[spc_cmd("INC", "DEC", "ASL", "LSR", "ROL", "ROR", "XCN", "DAA", "DAS", "MUL", "DIV", "ADDW", "SUBW", "CMPW", "INCW", "DECW"), "cargo test -p puce16-core e06_"],
  notes=CHAIN)

T("E06-23", "spc700/bits.rs: SET1, CLR1, BBS, BBC, TSET1, TCLR1",
  size=120,
  context="Manipulation de bits de la page directe et test-et-positionne. " + CHAIN + " " + RUN,
  embed=["07_SPC700.md#6.4 Bits"],
  tsv=stsv("SET1", "CLR1", "BBS", "BBC", "TSET1", "TCLR1"), reads=READS,
  write=[SPC + "/bits.rs", TST + "/e06_23_bits.rs"],
  spec="""Dans `bits.rs::exec` (`n` = numero de bit 0..=7):
- `SET1 dp.n` = `n*0x20 + 0x02` (02 22 42 62 82 A2 C2 E2) ; `CLR1 dp.n` = `n*0x20 + 0x12` : `b = fetch8` ; `ea = P|b` ; `v = rd(ea)` ; `wr(ea, v | bit)` (resp. `v & !bit`). Aucun flag.
- `BBS dp.n,rel` = `n*0x20 + 0x03` ; `BBC dp.n,rel` = `n*0x20 + 0x13` : octets `op dp rel` (`b = fetch8`, `rel = fetch8`) ; `v = rd(P|b)` ; saute (`cpu.branch(bus, rel)`) si le bit vaut 1 (BBS) / 0 (BBC). Aucun flag.
- `0E` TSET1 !abs: `ea = fetch16` ; `t = rd(ea)` ; `r = a.wrapping_sub(t)` ; `set_nz(r)` (N = bit 7 de r, Z = (r == 0)) ; `wr(ea, t | a)`. `4E` TCLR1 !abs: idem avec `wr(ea, t & !a)`.""",
  tests=[("set1", "poke(0x10,&[0x00]), `02 10` (SET1 dp.0): peek(0x10)==0x01, cycles 4 ; `E2 10` (dp.7) sur 0: peek==0x80 ; `42 10` (dp.2): 0x04."),
         ("clr1", "poke(0x10,&[0xFF]), `72 10` (CLR1 dp.3): peek(0x10)==0xF7, cycles 4 ; `12 10` (dp.0) sur 0xFF -> 0xFE."),
         ("set_clr_no_flags", "psw=0xC3, `02 10`: psw==0xC3."),
         ("bbs_taken", "poke(0x10,&[0x04]), `43 10 05` (BBS dp.2,+5): pc==0x0208 (0x0203 + 5), cycles 7."),
         ("bbs_not_taken", "poke(0x10,&[0x00]), `43 10 05`: pc==0x0203, cycles 5."),
         ("bbc_taken_backward", "poke(0x10,&[0x00]), `33 10 FC` (BBC dp.1,-4): pc==0x01FF, cycles 7."),
         ("bbc_not_taken", "poke(0x10,&[0x02]), `33 10 FC`: pc==0x0203, cycles 5."),
         ("bbs_dp_n_opcodes", "BBS dp.7 = `E3`, BBC dp.7 = `F3` : poke(0x10,&[0x80]) `E3 10 02`: pc==0x0205 ; `F3 10 02`: pc==0x0203."),
         ("tset1", "a=0x0F, poke(0x3000,&[0xF0]), `0E 00 30`: peek(0x3000)==0xFF, N=0, Z=0 (0x0F-0xF0 = 0x1F), cycles 6 ; a=0x80, mem 0x80: Z=1, mem 0x80 ; a=0x01, mem 0x02: N=1 (0xFF), mem 0x03."),
         ("tclr1", "a=0x0F, poke(0x3000,&[0xFF]), `4E 00 30`: peek(0x3000)==0xF0, N=0, Z=0 (0x0F-0xFF = 0x10), cycles 6."),
         ("tset1_keeps_c_v", "psw=F_C|F_V, a=0, mem 0, `0E 00 30`: C et V toujours a 1, Z=1.")],
  notes=CHAIN)

T("E06-24", "spc700/bits.rs: OR1, AND1, EOR1, MOV1, NOT1 (bits de la memoire absolue)",
  size=110,
  context="Operations carry/bit avec un operande de 16 bits `v`: adresse `v & 0x1FFF` (ABSOLUE, la page directe P est ignoree), numero de bit `v >> 13`. " + CHAIN + " " + RUN,
  embed=["07_SPC700.md#6.4 Bits", "07_SPC700.md#5. Modes d'adressage (notation du TSV)"],
  tsv=stsv("OR1", "AND1", "EOR1", "MOV1", "NOT1"), reads=READS,
  write=[SPC + "/bits.rs", TST + "/e06_24_carry_bits.rs"],
  spec="""Dans `bits.rs::exec` (`v = fetch16` ; `ea = v & 0x1FFF` ; `bit = (v >> 13) as u8` ; `m = rd(ea) >> bit & 1 != 0`):
- `0A` OR1 C,m.b: `C |= m` ; `2A` OR1 C,/m.b: `C |= !m`. `4A` AND1 C,m.b: `C &= m` ; `6A` AND1 C,/m.b: `C &= !m`. `8A` EOR1 C,m.b: `C ^= m`. `AA` MOV1 C,m.b: `C = m`.
- `CA` MOV1 m.b,C: `t = rd(ea)` ; ecrit le bit a la valeur de C ; `wr(ea, t')`. `EA` NOT1 m.b: `t = rd(ea)` ; inverse le bit ; `wr`.
Seul C change pour les six premiers; MOV1 m.b,C et NOT1 ne changent aucun flag.""",
  tests=[("or1", "psw=0, poke(0x0005,&[0x02]), `0A 05 20` (v=0x2005 : adresse 0x0005, bit 1): C=1, cycles 5 ; avec poke 0x00: C=0."),
         ("or1_inverted", "psw=0, poke(0x0005,&[0x00]), `2A 05 20`: C=1 ; poke(0x0005,&[0x02]): C=0."),
         ("and1", "psw=F_C, poke(0x0005,&[0x02]), `4A 05 20`: C=1, cycles 4 ; poke 0x00: C=0."),
         ("and1_inverted", "psw=F_C, poke(0x0005,&[0x02]), `6A 05 20`: C=0 ; poke(0x0005,&[0x00]): C=1."),
         ("eor1", "psw=F_C, poke(0x0005,&[0x02]), `8A 05 20`: C=0, cycles 5 ; psw=0: C=1."),
         ("mov1_to_c", "psw=0, poke(0x0005,&[0x02]), `AA 05 20`: C=1, cycles 4 ; psw=F_C, poke 0x00: C=0."),
         ("mov1_to_mem", "psw=F_C, poke(0x0005,&[0x00]), `CA 05 20`: peek(0x0005)==0x02, cycles 6 ; psw=0, poke(0x0005,&[0xFF]): peek==0xFD."),
         ("not1", "poke(0x0005,&[0x00]), `EA 05 20`: peek==0x02, cycles 5 ; deuxieme execution -> 0x00."),
         ("bit_seven_and_address", "`v=0xE010`: adresse 0x0010, bit 7 : poke(0x0010,&[0x80]), psw=0, `AA 10 E0`: C=1 ; `v=0x3FFF`: adresse 0x1FFF, bit 1."),
         ("direct_page_ignored", "psw=F_P, poke(0x0005,&[0x02]), poke(0x0105,&[0x00]), psw bit C=0, `AA 05 20`: C=1 (lit 0x0005, pas 0x0105)."),
         ("no_flag_change_mem_ops", "psw=0xC3, `EA 05 20`: psw==0xC3.")],
  notes=CHAIN)

T("E06-25", "VERIF oracle: instructions de bits",
  size=50, kind="code",
  context="Passage a l'oracle de SET1 CLR1 BBS BBC TSET1 TCLR1 OR1 AND1 EOR1 MOV1 NOT1.",
  reads=["specs/07_SPC700.md section 6.4", "specs/data/spcjson_format.md"],
  write=[SPC + "/bits.rs", SPC + "/addr.rs", TST + "/e06_25_oracle_regressions.rs"],
  spec="""1. Lancer: `%s`
2. Pour chaque echec: lire le diff, localiser le bug dans les fichiers autorises, corriger. Ne pas modifier le harnais, `table.rs` ni les specs.
3. Si le diff indique une SPEC ou un TSV faux: question `spec` (docs/02_QA_PROTOCOL.md).
4. Pour chaque bug corrige, un test unitaire dans `tests/e06_25_oracle_regressions.rs` nomme `e06_25_<instruction>_<cas>`. Si aucun bug: un test `e06_25_oracle_clean`.
5. Finir quand la commande renvoie `failed: 0`.""" % spc_cmd("SET1", "CLR1", "BBS", "BBC", "TSET1", "TCLR1", "OR1", "AND1", "EOR1", "MOV1", "NOT1"),
  tests=[("oracle_clean_or_regressions", "voir spec etape 4")],
  accept=[spc_cmd("SET1", "CLR1", "BBS", "BBC", "TSET1", "TCLR1", "OR1", "AND1", "EOR1", "MOV1", "NOT1"), "cargo test -p puce16-core e06_"],
  notes=CHAIN)

T("E06-26", "spc700/flow.rs: branches (Bxx, BRA, CBNE, DBNZ)",
  size=120,
  context="Branches relatives. Le cycle supplementaire d'une branche prise est ajoute par `step` (`BRANCH_EXTRA`): la famille appelle seulement `cpu.branch(bus, rel)`. " + CHAIN + " " + RUN,
  embed=["07_SPC700.md#6.6 Controle de flux (cycles dans le TSV; `branch_taken_extra` s'ajoute si la branche est prise)"],
  tsv=stsv("BPL", "BMI", "BVC", "BVS", "BCC", "BCS", "BNE", "BEQ", "BRA", "CBNE", "DBNZ"), reads=READS,
  write=[SPC + "/flow.rs", TST + "/e06_26_branches.rs"],
  spec="""Dans `flow.rs::exec`:
- Conditionnels (`rel = fetch8`; si la condition: `cpu.branch(bus, rel)`): `10` BPL (!N) ; `30` BMI (N) ; `50` BVC (!V) ; `70` BVS (V) ; `90` BCC (!C) ; `B0` BCS (C) ; `D0` BNE (!Z) ; `F0` BEQ (Z). `2F` BRA: toujours.
- `2E` CBNE dp,rel: octets `op dp rel` ; `v = rd(P|dp)` ; saute si `a != v`. `DE` CBNE dp+X,rel: `dp = fetch8`, `rel = fetch8`, un `idle` pour l'index, `ea = P | ((dp + x) & 0xFF)`, `v = rd(ea)` ; saute si `a != v`. Aucun flag.
- `FE` DBNZ Y,rel: `rel = fetch8` ; `y -= 1` (wrapping) ; saute si `y != 0`. `6E` DBNZ dp,rel: octets `op dp rel` ; `ea = P|dp` ; `v = rd(ea) - 1` ; `wr(ea, v)` ; saute si `v != 0`. Aucun flag.""",
  tests=[("beq_taken", "psw=F_Z, `F0 05`: pc==0x0207 (0x0202+5), cycles 4."),
         ("beq_not_taken", "psw=0, `F0 05`: pc==0x0202, cycles 2."),
         ("bne_backward", "psw=0, `D0 FE`: pc==0x0200 (0x0202-2), cycles 4."),
         ("bpl_bmi", "psw=0 `10 10`: pc==0x0212 ; psw=F_N `10 10`: pc==0x0202 ; psw=F_N `30 10`: pc==0x0212 ; psw=0 `30 10`: pc==0x0202."),
         ("bvc_bvs", "psw=0 `50 04`: pc==0x0206 ; psw=F_V `50 04`: pc==0x0202 ; psw=F_V `70 04`: pc==0x0206 ; psw=0 `70 04`: pc==0x0202."),
         ("bcc_bcs", "psw=0 `90 04`: pc==0x0206 ; psw=F_C `90 04`: pc==0x0202 ; psw=F_C `B0 04`: pc==0x0206 ; psw=0 `B0 04`: pc==0x0202."),
         ("bra", "`2F 7F`: pc==0x0281, cycles 4 ; `2F 80` (-128): pc==0x0182, cycles 4."),
         ("branch_keeps_flags", "psw=0xC3 `F0 05` ou `D0 05` : psw==0xC3."),
         ("cbne_dp", "poke(0x10,&[0x07]), a=0x07, `2E 10 05`: pc==0x0203, cycles 5 ; a=0x08: pc==0x0208, cycles 7."),
         ("cbne_dpx", "x=1, poke(0x11,&[0x01]), a=0, `DE 10 05`: pc==0x0208, cycles 8 ; a=1: pc==0x0203, cycles 6."),
         ("dbnz_y", "y=2, `FE 05`: y==1, pc==0x0207, cycles 6 ; y=1: y==0, pc==0x0202, cycles 4 ; y=0: y==0xFF, pc==0x0207."),
         ("dbnz_dp", "poke(0x10,&[2]), `6E 10 05`: peek(0x10)==1, pc==0x0208, cycles 7 ; poke(0x10,&[1]): peek==0, pc==0x0203, cycles 5."),
         ("dbnz_keeps_flags", "psw=0xC3, y=2, `FE 05`: psw==0xC3.")],
  notes=CHAIN)

T("E06-27", "spc700/flow.rs: JMP, CALL, PCALL, TCALL, BRK, RET, RETI",
  size=130,
  context="Sauts, appels et retours. La pile est en page 1 (`push8`/`pull8`). Adresse de retour = adresse de l'instruction SUIVANTE. " + CHAIN + " " + RUN,
  embed=["07_SPC700.md#6.6 Controle de flux (cycles dans le TSV; `branch_taken_extra` s'ajoute si la branche est prise)"],
  tsv=stsv("JMP", "CALL", "PCALL", "TCALL", "BRK", "RET", "RETI"), reads=READS,
  write=[SPC + "/flow.rs", TST + "/e06_27_calls.rs"],
  spec="""Dans `flow.rs::exec`:
- `5F` JMP !abs: `pc = fetch16`. `1F` JMP [!abs+X]: `abs = fetch16` ; idle ; `ea = (abs + x) & 0xFFFF` ; `pc = rd16(ea, ea+1)` (adresses modulo 65536).
- `3F` CALL !abs: `t = fetch16` ; idle ; `push8(pc >> 8)` puis `push8(pc & 0xFF)` ; `pc = t`. `4F` PCALL up: `n = fetch8` ; empile `pc` de la meme facon ; `pc = 0xFF00 | n`.
- TCALL n (n = 0..=15), opcode `n*0x10 + 0x01` : empile `pc` (haut puis bas) ; `pc = rd16(0xFFDE - 2*n, 0xFFDF - 2*n)`.
- `0F` BRK: empile `pc` (haut, bas) puis `psw` ; `B = 1`, `I = 0` ; `pc = rd16(0xFFDE, 0xFFDF)`.
- `6F` RET: `lo = pull8` ; `hi = pull8` ; `pc = (hi << 8) | lo`. `7F` RETI: `psw = pull8` ; puis `lo`, `hi` comme RET.""",
  tests=[("jmp_abs", "`5F 00 30`: pc==0x3000, cycles 3."),
         ("jmp_abs_x_indirect", "x=2, poke(0x3002,&[0x34,0x12]), `1F 00 30`: pc==0x1234, cycles 6 ; x=1, `1F FF FF`: pointeur lu en 0x0000 (wrap) : poke(0x0000,&[0x78,0x56]) -> pc==0x5678."),
         ("call", "sp=0xEF, `3F 00 30`: pc==0x3000, sp==0xED, peek(0x01EF)==0x02, peek(0x01EE)==0x03 (adresse de retour 0x0203), cycles 8."),
         ("pcall", "sp=0xEF, `4F 20`: pc==0xFF20, sp==0xED, retour 0x0202 (peek(0x01EF)==0x02, peek(0x01EE)==0x02), cycles 6."),
         ("tcall_1", "sp=0xEF, poke(0xFFDC,&[0x78,0x56]), `11` (TCALL 1): pc==0x5678, retour 0x0201 (peek(0x01EF)==0x02, peek(0x01EE)==0x01), sp==0xED, cycles 8."),
         ("tcall_0_and_15", "poke(0xFFDE,&[0x34,0x12]) `01`: pc==0x1234 ; poke(0xFFC0,&[0x21,0x43]) `F1`: pc==0x4321."),
         ("brk", "sp=0xEF, psw=0x86, poke(0xFFDE,&[0x00,0x40]), `0F`: pc==0x4000, sp==0xEC, peek(0x01EF)==0x02, peek(0x01EE)==0x01, peek(0x01ED)==0x86, psw==0x92 (B=1, I=0), cycles 8."),
         ("ret", "sp=0xED, poke(0x01EE,&[0x34,0x12]), `6F`: pc==0x1234, sp==0xEF, cycles 5."),
         ("reti", "sp=0xEC, poke(0x01ED,&[0xA5,0x78,0x56]), `7F`: psw==0xA5, pc==0x5678, sp==0xEF, cycles 6."),
         ("call_ret_roundtrip", "`3F 00 30` puis, avec `6F` en 0x3000 : apres deux `step`, pc==0x0203 et sp d'origine.")],
  notes=CHAIN)

T("E06-28", "spc700/misc.rs: PUSH, POP, flags, NOP, SLEEP, STOP",
  size=110,
  context="Dernieres instructions du SPC700: pile, indicateurs, NOP et l'arret du coeur. Apres cette tache, les 256 opcodes sont implementes. " + CHAIN + " " + RUN,
  embed=["07_SPC700.md#6.5 Flags et pile", "07_SPC700.md#6.6 Controle de flux (cycles dans le TSV; `branch_taken_extra` s'ajoute si la branche est prise)"],
  tsv=stsv("PUSH", "POP", "CLRC", "SETC", "NOTC", "EI", "DI", "CLRV", "CLRP", "SETP", "NOP", "SLEEP", "STOP"), reads=READS,
  write=[SPC + "/misc.rs", TST + "/e06_28_misc.rs"],
  spec="""Dans `misc.rs::exec`:
- PUSH: `2D` A ; `4D` X ; `6D` Y ; `0D` PSW (`push8`). POP: `AE` A ; `CE` X ; `EE` Y ; `8E` PSW (`pull8`, aucun effet de flag sauf POP PSW qui charge tout `psw`).
- `60` CLRC (C=0) ; `80` SETC ; `ED` NOTC ; `A0` EI (I=1) ; `C0` DI (I=0) ; `E0` CLRV (V=0 ET H=0) ; `20` CLRP (P=0) ; `40` SETP (P=1) ; `00` NOP.
- `EF` SLEEP et `FF` STOP: `halted = true` (`pc` reste a l'adresse suivante); les `step` suivants renvoient 1 (E06-05).""",
  tests=[("push_a", "sp=0xEF, a=0x5A, `2D`: peek(0x01EF)==0x5A, sp==0xEE, cycles 4."),
         ("push_x_y_psw", "x=0x11 `4D`: peek(0x01EF)==0x11 ; y=0x22 `6D`: peek(0x01EF)==0x22 ; psw=0x86 `0D`: peek(0x01EF)==0x86 ; sp==0xEE ; psw inchange."),
         ("pop_a_no_flags", "sp=0xEE, poke(0x01EF,&[0x9C]), psw=0x00, `AE`: a==0x9C, sp==0xEF, psw==0x00 (pas de N/Z), cycles 4."),
         ("pop_x_y", "`CE` et `EE` avec poke(0x01EF,&[0x33]): x==0x33 resp. y==0x33."),
         ("pop_psw", "sp=0xEE, poke(0x01EF,&[0xA5]), `8E`: psw==0xA5."),
         ("carry_ops", "psw=0 `80`: psw==0x01 ; psw=1 `60`: psw==0 ; psw=0 `ED`: psw==1 ; psw=1 `ED`: psw==0 ; cycles: SETC 2, CLRC 2, NOTC 3."),
         ("i_flag", "psw=0 `A0`: psw==0x04, cycles 3 ; psw=0x04 `C0`: psw==0, cycles 3."),
         ("clrv_clears_h", "psw=0x48 (V,H) `E0`: psw==0x00, cycles 2 ; psw=0xC9 `E0`: psw==0x81."),
         ("p_flag", "psw=0 `40`: psw==0x20 ; `20`: psw==0 ; cycles 2."),
         ("nop", "psw=0x5A, `00`: tous les registres inchanges, pc==0x0201, cycles 2."),
         ("sleep_halts", "`EF`: halted==true, cycles 7, pc==0x0201 ; un `step` de plus renvoie 1 et pc==0x0201 ; idem `FF`."),
         ("all_opcodes_implemented", "Pour chacun des 256 opcodes: bus plat avec l'opcode en 0x0200 et des octets d'operande 0x00; `catch_unwind` autour d'un `step`: AUCUN ne doit paniquer avec le message `unimplemented opcode` (d'autres paniques ne sont pas attendues non plus ; la valeur de retour est ignoree).")],
  notes=CHAIN)

T("E06-29", "VERIF oracle: flux de controle, pile, indicateurs",
  size=50, kind="code",
  context="Passage a l'oracle des branches, appels, retours, PUSH/POP, flags, NOP/SLEEP/STOP.",
  reads=["specs/07_SPC700.md sections 6.5 et 6.6", "specs/data/spcjson_format.md"],
  write=[SPC + "/flow.rs", SPC + "/misc.rs", TST + "/e06_29_oracle_regressions.rs"],
  spec="""1. Lancer: `%s`
2. Pour chaque echec: lire le diff, localiser le bug dans les fichiers autorises, corriger. Ne pas modifier le harnais, `table.rs` ni les specs.
3. Si le diff indique une SPEC ou un TSV faux (ex: BRK, RETI, SLEEP/STOP): question `spec` (docs/02_QA_PROTOCOL.md).
4. Pour chaque bug corrige, un test unitaire dans `tests/e06_29_oracle_regressions.rs` nomme `e06_29_<instruction>_<cas>`. Si aucun bug: un test `e06_29_oracle_clean`.
5. Finir quand la commande renvoie `failed: 0`.""" % spc_cmd("BPL", "BMI", "BVC", "BVS", "BCC", "BCS", "BNE", "BEQ", "BRA", "CBNE", "DBNZ", "JMP", "CALL", "PCALL", "TCALL", "BRK", "RET", "RETI", "PUSH", "POP", "CLRC", "SETC", "NOTC", "EI", "DI", "CLRV", "CLRP", "SETP", "NOP", "SLEEP", "STOP"),
  tests=[("oracle_clean_or_regressions", "voir spec etape 4")],
  accept=[spc_cmd("BPL", "BMI", "BVC", "BVS", "BCC", "BCS", "BNE", "BEQ", "BRA", "CBNE", "DBNZ", "JMP", "CALL", "PCALL", "TCALL", "BRK", "RET", "RETI", "PUSH", "POP", "CLRC", "SETC", "NOTC", "EI", "DI", "CLRV", "CLRP", "SETP", "NOP", "SLEEP", "STOP"), "cargo test -p puce16-core e06_"],
  notes=CHAIN)

T("E06-30", "GATE J4a: tous les opcodes SPC700 passent l'oracle",
  kind="gate", gate=True, size=0,
  context="Fin du coeur SPC700 (jalon J4, premiere moitie, specs/06_TEST_ROMS.md section 8): le SMP est valide par l'oracle, instruction par instruction.",
  spec="""Criteres de validation:
1. `cargo run --release -p puce16-cli -- spc-test tests/oracle/spc700` termine avec le code 0 (aucun FAIL).
2. `cargo test --workspace` est vert.
3. Aucun `unimplemented!` / `todo!` dans `crates/puce16-core/src/apu/spc700/` (`grep -rn "unimplemented!\\|todo!" crates/puce16-core/src/apu/spc700`).
4. Les questions `spec` ouvertes de la famille SPC700 dans `docs/qa/inbox/` sont toutes resolues (Claude), et le TSV `opcodes_spc700.tsv` corrige si besoin.""",
  user_cmds=["cargo run --release -p puce16-cli -- spc-test tests/oracle/spc700", "cargo test --workspace",
             "python docs/tools/task.py approve E06-30"],
  accept=[])
