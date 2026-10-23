from _dsl import T

CORE = "crates/puce16-core"
SRC = CORE + "/src"
TST = CORE + "/tests"

CHAIN = ("Execution strictement sequentielle (D-020): cette tache suppose que toutes les precedentes sont terminees. "
         "Ne modifie QUE les fichiers de 'Write only in'.")

# ------------------------------------------------------------------ E04 part A: memoires PPU et registres a latch (E04-01..05)
# Principe: chaque memoire est une petite structure PURE (sans connaissance du temps). Le Ppu (E04-08) l'appelle et lui passe `ok`
# (acces autorise ou non, spec 05 section 3).

T("E04-01", "ppu/vram.rs: VRAM, VMAIN, VMADD, prefetch, lecture/ecriture",
  deps=["E03-48"], size=140,
  context="""Debut de l'epic E04 (registres et memoires du PPU). Le squelette de timing (compteurs, dots, evenements) existe deja (E03-07..09): ne le refais pas.
La VRAM est une structure pure: elle ne sait pas si l'acces est autorise, l'appelant (le Ppu) lui passe `ok: bool`. """ + CHAIN,
  embed=["05_PPU.md#1. Memoires du PPU", "05_PPU.md#2.3 VRAM"],
  write=[SRC + "/ppu/vram.rs", TST + "/e04_01_vram.rs"],
  spec="""Dans `ppu/vram.rs`:
```rust
pub fn remap(mode: u8, a: u16) -> u16;            // mode 0..=3 (VMAIN bits 3-2), formules de la section 2.3 ; mode > 3: comme 0
pub struct Vram { mem: Vec<u16> /* 0x8000 mots a 0 */, vmain: u8, vmadd: u16, latch: u16 }
impl Vram {
    pub fn new() -> Vram;                          // tout a 0
    pub fn reset_regs(&mut self);                  // vmain = 0, vmadd = 0, latch = 0 ; la memoire est CONSERVEE
    pub fn set_vmain(&mut self, v: u8);  pub fn vmain(&self) -> u8;  pub fn vmadd(&self) -> u16;
    pub fn step(&self) -> u16;                     // vmain bits 1-0: 0 -> 1, 1 -> 32, 2 ou 3 -> 128
    pub fn inc_on_high(&self) -> bool;             // vmain bit 7
    pub fn set_addr_l(&mut self, v: u8);           // (vmadd & 0xFF00) | v, puis prefetch
    pub fn set_addr_h(&mut self, v: u8);           // (vmadd & 0x00FF) | ((v & 0x7F) << 8), puis prefetch
    pub fn write_low(&mut self, v: u8, ok: bool);  pub fn write_high(&mut self, v: u8, ok: bool);
    pub fn read_low(&mut self, ok: bool) -> u8;    pub fn read_high(&mut self, ok: bool) -> u8;
    pub fn word(&self, a: u16) -> u16;             // mem[a & 0x7FFF]
    pub fn set_word(&mut self, a: u16, w: u16);    // debug/test: mem[a & 0x7FFF] = w
    pub fn mem(&self) -> &[u16];
    pub fn hash_into(&self, h: &mut crate::hash::Fnv64);   // toute la memoire (write_u16_slice), puis vmain (u8), vmadd (u16), latch (u16)
}
```
Regles (section 2.3, a suivre EXACTEMENT):
- Adresse d'acces: `ra = remap((vmain >> 2) & 3, vmadd) & 0x7FFF`. `vmadd` lui-meme n'est jamais remappe.
- prefetch: `latch = mem[ra]`.
- `write_low(v, ok)`: si `ok`, octet bas de `mem[ra]` = v. Puis, si `!inc_on_high()`: `vmadd = (vmadd + step()) & 0x7FFF` (meme si `!ok`). `write_high`: idem sur l'octet haut, increment si `inc_on_high()`.
- `read_low(ok)`: `byte = if ok { latch & 0xFF } else { 0 }`; puis si `!inc_on_high()`: `latch = mem[ra]` (adresse AVANT increment) et `vmadd += step` (masque 0x7FFF) -- le rechargement et l'increment ont lieu MEME si `!ok`. `read_high(ok)`: `byte = latch >> 8` (ou 0), rechargement et increment si `inc_on_high()`.""",
  tests=[("remap_modes", "`remap(0,0x1234)==0x1234` ; `remap(1,0x1234)==0x12A1` ; `remap(2,0x1234)==0x13A0` ; `remap(3,0x1234)==0x11A4` ; `remap(1,0x0001)==0x0008` ; `remap(1,0x0020)==0x0001` ; `remap(2,0x0040)==0x0001` ; `remap(3,0x0080)==0x0001` ; `remap(2,0x0001)==0x0008` ; `remap(3,0x0001)==0x0008` ; `remap(1,0x00FF)==0x00FF` ; `remap(2,0x01FF)==0x01FF`."),
         ("power_on", "`Vram::new()`: `vmain()==0`, `vmadd()==0`, `word(0)==0`, `word(0x7FFF)==0`, `mem().len()==0x8000`."),
         ("step_values", "`set_vmain(0x00)`: step()==1, inc_on_high()==false ; `0x01`: 32 ; `0x02`: 128 ; `0x03`: 128 ; `0x80`: step 1 et inc_on_high()==true ; `0x8D`: step 32 et inc_on_high()==true."),
         ("write_low_inc1", "vmain 0x00, adresse 0 (`set_addr_l(0); set_addr_h(0)`) : `write_low(0x11,true)`, `write_low(0x22,true)` -> `word(0)==0x0011`, `word(1)==0x0022`, `vmadd()==2` ; ensuite `write_high(0x33,true)` -> `word(2)==0x3300` et `vmadd()` reste 2 (increment sur l'octet bas seulement)."),
         ("write_word_inc_high", "vmain 0x80, adresse 0x0100 (`set_addr_l(0); set_addr_h(1)`) : `write_low(0xCD,true)` (vmadd reste 0x100), `write_high(0xAB,true)` -> `word(0x100)==0xABCD`, `vmadd()==0x101` ; puis `write_low(0x01,true); write_high(0x02,true)` -> `word(0x101)==0x0201`, `vmadd()==0x102`."),
         ("step32_step128", "vmain 0x81, adresse 0 : `write_low(1,true); write_high(2,true)` -> `word(0)==0x0201`, `vmadd()==32` ; `write_low(3,true); write_high(4,true)` -> `word(32)==0x0403`, `vmadd()==64`. Puis vmain 0x82, adresse 0 : trois paires d'ecritures -> `vmadd()==384`, `word(128)!=0` et `word(256)!=0` (ecrites avec des valeurs non nulles)."),
         ("addr_wrap_mask", "vmain 0x00, `set_addr_l(0xFF); set_addr_h(0x7F)` -> `vmadd()==0x7FFF` ; `write_low(0x5A,true)` -> `word(0x7FFF)==0x005A` et `vmadd()==0` (masque 0x7FFF). `set_addr_l(0); set_addr_h(0xFF)` -> `vmadd()==0x7F00` (bit 7 de VMADDH ignore)."),
         ("blocked_write", "vmain 0x00, adresse 5 : `write_low(0x77,false)` -> `word(5)==0` et `vmadd()==6`. vmain 0x80, adresse 9 : `write_high(0x88,false)` -> `word(9)==0` et `vmadd()==10` ; `write_low(0x99,false)` ne change pas vmadd (reste 10)."),
         ("remap_applied", "vmain 0x04 (remap mode 1, increment 1, sur octet bas), adresse 1 : `write_low(0x99,true)` -> `word(8)==0x0099`, `word(1)==0`, `vmadd()==2` (vmadd n'est pas remappe)."),
         ("prefetch_on_addr", "`set_word(0x0100,0xCAFE)` ; vmain 0 ; `set_addr_l(0); set_addr_h(1)` -> `read_low(true)==0xFE`. Avec vmain 0x04 : `set_word(8,0xABCD)`, `set_addr_l(1); set_addr_h(0)` -> `read_low(true)==0xCD` (le prefetch utilise le remap)."),
         ("read_low_sequence", "`set_word(5,0xBEEF); set_word(6,0x1234); set_word(7,0x5678)`, vmain 0, `set_addr_l(5); set_addr_h(0)` : quatre `read_low(true)` renvoient 0xEF, 0xEF, 0x34, 0x78 (les deux premieres lectures renvoient le meme mot) et `vmadd()==9`."),
         ("read_high_sequence", "`set_word(10,0xA1B2); set_word(11,0xC3D4)`, vmain 0x80, adresse 10 : `read_high(true)==0xA1` (vmadd 11) ; `read_low(true)==0xB2` (ne change ni le latch ni vmadd) ; `read_high(true)==0xA1` (vmadd 12) ; `read_high(true)==0xC3` (vmadd 13)."),
         ("read_blocked", "`set_word(20,0x1111)`, vmain 0, adresse 20 : `read_low(false)==0` et `vmadd()==21` ; ensuite `read_low(true)==0x11` (le latch a ete recharge par la lecture bloquee)."),
         ("hash", "hash identique pour deux `Vram::new()` ; differe apres `set_word(3,1)` ; differe apres `set_vmain(0x80)` ; differe apres `set_addr_l(1)` ; `reset_regs()` ne modifie pas la memoire (`word(3)` conserve).")])

T("E04-02", "ppu/cgram.rs: CGRAM (256 mots, ecriture/lecture par paires)",
  size=100,
  context="La CGRAM est une structure pure. Les acces sont toujours autorises en v1 (spec 05 section 2.4). " + CHAIN,
  embed=["05_PPU.md#2.4 CGRAM"],
  write=[SRC + "/ppu/cgram.rs", TST + "/e04_02_cgram.rs"],
  spec="""Dans `ppu/cgram.rs`:
```rust
pub struct Cgram { mem: [u16; 256], addr: u8, toggle: bool, latch: u8 }
impl Cgram {
    pub fn new() -> Cgram;                         // tout a 0 / false
    pub fn reset_regs(&mut self);                  // addr = 0, toggle = false, latch = 0 ; memoire CONSERVEE
    pub fn set_addr(&mut self, v: u8);             // CGADD: addr = v ; toggle = false
    pub fn write(&mut self, v: u8);                // CGDATA, section 2.4
    pub fn read(&mut self, ppu2_mdr: u8) -> u8;    // CGDATAREAD, section 2.4 ; renvoie l'octet (l'appelant le memorise comme ppu2_mdr)
    pub fn addr(&self) -> u8;  pub fn toggle(&self) -> bool;
    pub fn word(&self, i: u8) -> u16;              // mem[i]
    pub fn set_word(&mut self, i: u8, w: u16);     // debug/test: mem[i] = w & 0x7FFF
    pub fn mem(&self) -> &[u16; 256];
    pub fn hash_into(&self, h: &mut crate::hash::Fnv64);   // mem (write_u16_slice), addr (u8), toggle (u8), latch (u8)
}
```
`write` : si `!toggle` { `latch = v` } sinon { `mem[addr] = ((v & 0x7F) as u16) << 8 | latch as u16 ; addr = addr.wrapping_add(1)` } ; puis `toggle = !toggle`.
`read(m)` : `w = mem[addr]` ; si `!toggle` -> `byte = (w & 0xFF) as u8` ; sinon `byte = ((w >> 8) as u8 & 0x7F) | (m & 0x80)` et `addr = addr.wrapping_add(1)` ; puis `toggle = !toggle` ; renvoie `byte`.""",
  tests=[("set_addr_resets_toggle", "`write(0x12)` -> toggle()==true ; `set_addr(5)` -> addr()==5, toggle()==false."),
         ("pair_write", "`set_addr(0)`; `write(0x1F); write(0x03)` -> `word(0)==0x031F`, `addr()==1`, `toggle()==false` ; `write(0xFF); write(0xFF)` -> `word(1)==0x7FFF` (bit 15 efface), `addr()==2`."),
         ("latch_not_committed", "`set_addr(3); write(0xAA)` : `word(3)==0`, `toggle()==true`, `addr()==3`. Le 2eme octet `write(0x01)` -> `word(3)==0x01AA`, `addr()==4`."),
         ("addr_wrap", "`set_addr(255); write(0x01); write(0x02)` -> `word(255)==0x0201`, `addr()==0`."),
         ("read_pair", "`set_word(2,0x7ABC)` ; `set_addr(2)` ; `read(0x00)==0xBC` (toggle vrai, addr reste 2) ; `read(0x00)==0x7A` (addr 3, toggle faux). Meme sequence avec `m=0x80` sur le 2eme octet : `read(0x80)==0xFA`."),
         ("set_word_masks_bit15", "`set_word(0,0xFFFF)` -> `word(0)==0x7FFF`."),
         ("shared_toggle", "`set_addr(0)`; `write(0x11)` (toggle vrai) ; `read(0x00)==0` (traite comme octet haut de `word(0)==0`), `addr()==1`, `toggle()==false`."),
         ("hash", "hash identique pour deux `Cgram::new()` ; differe apres `set_word(0,1)` ; differe apres `set_addr(1)` ; differe apres un `write(5)` seul (latch/toggle).")])

T("E04-03", "ppu/oam.rs: OAM 544 octets, OAMADD, OAMDATA, rechargement",
  size=130,
  context="L'OAM est une structure pure; l'appelant passe `ok` (acces autorise: forced blank ou VBlank, spec 05 section 3). " + CHAIN,
  embed=["05_PPU.md#2.1 OAMDATA ($2104) et lecture OAMDATAREAD ($2138)", "05_PPU.md#3. Regles d'acces pendant l'affichage"],
  write=[SRC + "/ppu/oam.rs", TST + "/e04_03_oam.rs"],
  spec="""Dans `ppu/oam.rs`:
```rust
pub struct Oam { mem: [u8; 544], oamadd_reg: u16 /* 9 bits */, internal: u16 /* 10 bits, adresse octet */, latch: u8, rotation: bool }
impl Oam {
    pub fn new() -> Oam;                           // tout a 0 / false
    pub fn reset_regs(&mut self);                  // oamadd_reg = internal = latch = 0, rotation = false ; memoire CONSERVEE
    pub fn set_addr_l(&mut self, v: u8);           // OAMADDL: oamadd_reg = (oamadd_reg & 0x100) | v ; recalcul
    pub fn set_addr_h(&mut self, v: u8);           // OAMADDH: bit 0 -> bit 8 de oamadd_reg ; bit 7 -> rotation ; recalcul
    pub fn reload(&mut self);                      // recalcul: internal = (oamadd_reg & 0x1FF) << 1
    pub fn write(&mut self, v: u8, ok: bool);      // OAMDATA, section 2.1
    pub fn read(&mut self, ok: bool) -> u8;        // OAMDATAREAD, section 2.1 ; 0 si !ok ; l'adresse s'incremente dans les deux cas
    pub fn internal(&self) -> u16;  pub fn oamadd_reg(&self) -> u16;  pub fn rotation(&self) -> bool;
    pub fn byte(&self, i: usize) -> u8;            // mem[i]
    pub fn set_byte(&mut self, i: usize, v: u8);   // debug/test
    pub fn mem(&self) -> &[u8; 544];
    pub fn hash_into(&self, h: &mut crate::hash::Fnv64);   // mem (write), oamadd_reg (u16), internal (u16), latch (u8), rotation (u8)
}
```
`write(v, ok)` (a = internal) :
- table haute (`a & 0x200 != 0`) : si `ok`, `mem[0x200 + (a & 0x1F)] = v`.
- sinon, `a` pair : si `ok`, `latch = v` ; `a` impair : si `ok`, `mem[a-1] = latch ; mem[a] = v`.
- dans tous les cas `internal = (a + 1) & 0x3FF`.
`read(ok)` : `idx = if a & 0x200 != 0 { 0x200 + (a & 0x1F) } else { a }` ; octet = `mem[idx]` si `ok` sinon 0 ; `internal = (a + 1) & 0x3FF`.""",
  tests=[("addr_regs", "`set_addr_l(0x05)` -> `internal()==0x0A`, `oamadd_reg()==5` ; `set_addr_h(0x01)` -> `oamadd_reg()==0x105`, `internal()==0x20A` ; `set_addr_h(0x80)` -> `oamadd_reg()==5`, `internal()==0x0A`, `rotation()==true` ; `set_addr_h(0x00)` -> `rotation()==false`."),
         ("word_write", "`set_addr_l(0)`; `write(0x11,true)` -> `byte(0)==0` (seulement memorise), `internal()==1` ; `write(0x22,true)` -> `byte(0)==0x11`, `byte(1)==0x22`, `internal()==2`."),
         ("sequence", "`set_addr_l(0)`; ecrire 0x11,0x22,0x33,0x44 -> octets 0..4 = [0x11,0x22,0x33,0x44], `internal()==4`."),
         ("odd_start", "`set_addr_l(0)`; `write(0x55,true)` (internal 1) ; `set_addr_l(0)` repositionne internal a 0 et le latch garde 0x55 ; `write(0x66,true)` (internal pair) memorise 0x66 ; `write(0x77,true)` -> `byte(0)==0x66`, `byte(1)==0x77`. Autre cas (nouvel `Oam`, sans repositionner): apres `write(0x55,true)` (internal 1) un 2eme `write(0x66,true)` donne `byte(0)==0x55`, `byte(1)==0x66`."),
         ("high_table", "`set_addr_h(0x01)` (internal 0x200) : `write(0xAA,true)` -> `byte(0x200)==0xAA`, `internal()==0x201` ; `write(0xBB,true)` -> `byte(0x201)==0xBB`."),
         ("high_mirror", "`set_addr_h(0x01); set_addr_l(0x10)` -> internal 0x220 ; `write(0xCC,true)` -> `byte(0x200)==0xCC` (miroir tous les 32), `internal()==0x221` ; `set_addr_l(0x11)` -> internal 0x222 ; `write(0xDD,true)` -> `byte(0x202)==0xDD`."),
         ("wrap_10bits", "`set_addr_l(0xFF); set_addr_h(0x01)` -> `internal()==0x3FE` ; `write(1,true); write(2,true)` -> `byte(0x21E)==1`, `byte(0x21F)==2`, `internal()==0`."),
         ("read_low_table", "`set_byte(0,0x10); set_byte(1,0x20)` ; `set_addr_l(0)` : `read(true)==0x10`, `read(true)==0x20`, `internal()==2`."),
         ("read_high_table", "`set_byte(0x204,0x66); set_byte(0x205,0x77)` ; `set_addr_h(0x01); set_addr_l(0x02)` (internal 0x204) : `read(true)==0x66`, `read(true)==0x77`, `internal()==0x206`. Miroir: `set_byte(0x200,0x99)`, `set_addr_l(0x10)` (internal 0x220) : `read(true)==0x99`."),
         ("blocked", "`set_addr_l(0)`; `write(9,false)` -> `byte(0)==0`, `byte(1)==0`, `internal()==1` ; `write(0x31,true)` (impair) -> `byte(0)==0` (latch non modifie par l'ecriture bloquee), `byte(1)==0x31`. `set_byte(5,0x5A)`, `set_addr_l(2)` (internal 4) : `read(false)==0` et `internal()==5`, puis `read(true)==0x5A`."),
         ("reload", "`set_addr_l(0x07)` (internal 14) ; trois `write(1,true)` -> `internal()==17` ; `reload()` -> `internal()==14` ; avec `set_addr_h(0x01)` (reg 0x107) : `reload()` -> `internal()==0x20E`."),
         ("hash", "hash identique pour deux `Oam::new()` ; differe apres `set_byte(7,1)` ; differe apres `set_addr_l(1)` ; differe apres `set_addr_h(0x80)` (rotation).")])

T("E04-04", "ppu/regs.rs: registres de scroll BGnHOFS/BGnVOFS et latchs partages",
  size=90,
  context="""Premier contenu de `ppu/regs.rs` (nouveau module: ajoute `pub mod regs;` a `ppu/mod.rs` en gardant le reste du fichier tel quel). Les registres de scroll des 4 fonds partagent des latchs (spec 05 section 2.2).
Le scroll du mode 7 ($210D/$210E aussi) est fait en E04-05. """ + CHAIN,
  embed=["05_PPU.md#2.2 Registres de scroll"],
  write=[SRC + "/ppu/regs.rs", SRC + "/ppu/mod.rs", TST + "/e04_04_scroll.rs"],
  spec="""Dans `ppu/regs.rs`:
```rust
pub struct Scroll { hofs: [u16; 4], vofs: [u16; 4], bgofs_latch: u8, bghofs_latch: u8 }
impl Scroll {
    pub fn new() -> Scroll;                        // tout a 0
    pub fn write_hofs(&mut self, n: usize, v: u8); // n = 0..=3 (BG1..BG4) ; n > 3: ignore
    pub fn write_vofs(&mut self, n: usize, v: u8);
    pub fn hofs(&self, n: usize) -> u16;  pub fn vofs(&self, n: usize) -> u16;    // 0 si n > 3
    pub fn hash_into(&self, h: &mut crate::hash::Fnv64);   // hofs[0..4], vofs[0..4] (u16), bgofs_latch, bghofs_latch (u8)
}
```
`write_hofs(n, v)` : `hofs[n] = (((v as u16) << 8) | (bgofs_latch & 0xF8) as u16 | (bghofs_latch & 7) as u16) & 0x3FF` ; puis `bgofs_latch = v ; bghofs_latch = v`.
`write_vofs(n, v)` : `vofs[n] = (((v as u16) << 8) | bgofs_latch as u16) & 0x3FF` ; puis `bgofs_latch = v`.
Les 4 fonds partagent les MEMES deux latchs.""",
  tests=[("power_on", "`Scroll::new()`: hofs(n)==0 et vofs(n)==0 pour n in 0..4 ; `hofs(4)==0`."),
         ("hofs_two_writes", "`write_hofs(0,0x34); write_hofs(0,0x01)` -> `hofs(0)==0x134`."),
         ("hofs_mask_10bits", "`write_hofs(0,0xFF); write_hofs(0,0xFF)` -> `hofs(0)==0x3FF`."),
         ("hofs_latch_shared_between_bg", "`write_hofs(0,0x34); write_hofs(1,0x01)` -> `hofs(1)==0x134`, et `hofs(0)==0` (la 1ere ecriture donne (0x34<<8)&0x3FF = 0)."),
         ("vofs_two_writes", "`write_vofs(0,0x34); write_vofs(0,0x02)` -> `vofs(0)==0x234`."),
         ("vofs_other_bg", "`write_vofs(3,0xFF); write_vofs(3,0x03)` -> `vofs(3)==0x3FF` ; `vofs(0)==0`."),
         ("hofs_uses_both_latches", "`write_vofs(0,0x35)` (bgofs_latch=0x35, bghofs_latch=0) puis `write_hofs(0,0x01)` -> `hofs(0)==0x130` (bits 2-0 viennent du latch H, ici 0)."),
         ("latch_leftovers", "`write_hofs(0,0x07)`; `write_vofs(0,0x01)` -> `vofs(0)==0x107` ; puis `write_hofs(0,0x00)` -> `hofs(0)==0x007` (bgofs_latch=1 -> (1&0xF8)=0, bghofs_latch=7 -> 7)."),
         ("bad_index", "`write_hofs(4,1)` et `write_vofs(9,1)` ne paniquent pas et ne modifient pas les latchs (hash identique avant/apres)."),
         ("hash", "hash identique pour deux `Scroll::new()` ; differe apres `write_hofs(0,1)` ; differe apres `write_vofs(2,1)`.")])

T("E04-05", "ppu/mode7.rs: registres mode 7 (M7SEL, M7A-D, M7X/Y, scroll) et MPY",
  size=120,
  context="""Seulement les REGISTRES du mode 7 et le produit MPY ($2134-$2136). Le rendu du mode 7 vient en E05.
Rappel: $210D/$210E ecrivent a la fois le scroll BG1 (E04-04) et le scroll mode 7 (ici); l'aiguillage est fait par le Ppu en E04-08. """ + CHAIN,
  embed=["05_PPU.md#2. Registres PPU en ecriture", "05_PPU.md#4. Registres PPU en lecture"],
  write=[SRC + "/ppu/mode7.rs", TST + "/e04_05_mode7_regs.rs"],
  spec="""Dans `ppu/mode7.rs` (n'ecris QUE les registres):
```rust
pub fn sign_extend_13(x: u16) -> i16;              // ((x as i16) << 3) >> 3   (en u16/i16 sans panique: utilise wrapping_shl ou passe par u16)
pub struct M7Regs { pub flip_x: bool, pub flip_y: bool, pub fill: bool, pub outside: bool,
                    pub a: i16, pub b: i16, pub c: i16, pub d: i16, pub cx: i16, pub cy: i16, pub hofs: i16, pub vofs: i16,
                    latch: u8, b_last_byte: u8 }
impl M7Regs {
    pub fn new() -> M7Regs;                        // tout a 0 / false
    pub fn write_sel(&mut self, v: u8);            // M7SEL: bit0 flip_x, bit1 flip_y, bit6 fill, bit7 outside
    pub fn write_matrix(&mut self, i: usize, v: u8);   // i = 0..=3 -> M7A, M7B, M7C, M7D ; valeur = ((v << 8) | latch) as i16 ; latch = v ; si i == 1: b_last_byte = v
    pub fn write_center(&mut self, i: usize, v: u8);   // i = 0 -> M7X (cx), 1 -> M7Y (cy) ; valeur = sign_extend_13(((v & 0x1F) << 8) | latch) ; latch = v
    pub fn write_scroll(&mut self, i: usize, v: u8);   // i = 0 -> M7HOFS (hofs), 1 -> M7VOFS (vofs) ; valeur = sign_extend_13((v << 8) | latch) ; latch = v
    pub fn mpy(&self) -> i32;                      // (a as i32) * ((b_last_byte as i8) as i32)
    pub fn mpy_bits(&self) -> u32;                 // (mpy() as u32) & 0xFF_FFFF
    pub fn hash_into(&self, h: &mut crate::hash::Fnv64);   // 4 bools (u8), a b c d cx cy hofs vofs (u16 de la valeur i16 as u16), latch, b_last_byte (u8)
}
```
Les quatre familles d'ecritures partagent le MEME latch `latch`. Indices hors plage: ignores (sans toucher au latch).""",
  tests=[("sign_extend", "`sign_extend_13(0x0FFF)==4095` ; `0x1000 -> -4096` ; `0x1FFF -> -1` ; `0x0000 -> 0` ; `0x2345 -> 837` (les bits 13-15 sont ignores) ; `0x1234 -> 4660 - 8192 = -3532`."),
         ("sel_bits", "`write_sel(0xC3)`: flip_x, flip_y, fill, outside tous vrais ; `write_sel(0x00)`: tous faux ; `0x01`: flip_x seul ; `0x02`: flip_y seul ; `0x40`: fill seul ; `0x80`: outside seul."),
         ("matrix_two_writes", "`write_matrix(0,0x34); write_matrix(0,0x12)` -> `a==0x1234` ; `write_matrix(0,0x00); write_matrix(0,0x80)` -> `a==-32768` ; `write_matrix(0,0xFF); write_matrix(0,0xFF)` -> `a==-1`."),
         ("matrix_shared_latch", "`write_matrix(0,0x11); write_matrix(1,0x22)` -> `b==0x2211` ; `a==0x1100` (valeur issue de la 1ere ecriture seule: (0x11<<8)|0)."),
         ("matrix_cd", "`write_matrix(2,0x01); write_matrix(2,0x02)` -> `c==0x0201` ; `write_matrix(3,0x03); write_matrix(3,0x04)` -> `d==0x0403`."),
         ("center", "`write_center(0,0xFF); write_center(0,0x01)` -> `cx==511` ; `write_center(0,0x00); write_center(0,0x10)` -> `cx==-4096` ; `write_center(0,0x34); write_center(0,0x1F)` -> `cx==-204` ; `write_center(1,0x34); write_center(1,0x01)` -> `cy==308`."),
         ("scroll", "`write_scroll(0,0x34); write_scroll(0,0x01)` -> `hofs==308` ; `write_scroll(0,0xFF); write_scroll(0,0x1F)` -> `hofs==-1` ; `write_scroll(1,0x00); write_scroll(1,0x10)` -> `vofs==-4096`."),
         ("mpy_examples", "`write_matrix(0,0x00); write_matrix(0,0x01)` (a=256) puis `write_matrix(1,0xFF)` -> `mpy()==-256`, `mpy_bits()==0xFFFF00`. A=-1 (0xFF,0xFF) et `write_matrix(1,0x7F)` -> `mpy()==-127`, `mpy_bits()==0xFFFF81`."),
         ("mpy_extremes", "a=0x7FFF (ecrire 0xFF puis 0x7F), b_last_byte=0x80 -> `mpy()==-4194176`, `mpy_bits()==0xC00080` ; a=0x0100, b_last_byte=0x7F -> `mpy()==32512`, `mpy_bits()==0x007F00`."),
         ("bad_index", "`write_matrix(4,1)`, `write_center(2,1)`, `write_scroll(2,1)` ne paniquent pas et ne changent pas le hash."),
         ("hash", "hash identique pour deux `M7Regs::new()` ; differe apres `write_sel(1)` ; differe apres `write_matrix(0,1)`.")])
