# SPEC 10 - Coprocesseur S-DD1 (decompresseur a la volee)

Jeux concernes: Star Ocean (6 Mio, SRAM 8 Kio, puce `SHVC-LN3B-01`), Street Fighter Alpha 2 / Zero 2 (4 Mio, sans SRAM, puces `SHVC-1N0N-01` / `SNSP-1N0N-01`). Le S-DD1 est un decompresseur place entre la ROM et le S-CPU: pendant un DMA configure d'une facon precise, il remplace les octets lus dans la ROM par des octets decompresses.

## 1. Sources et fiabilite
- Algorithme: Andreas Naive (2003-2004, description et code en **domaine public**), tel qu'implemente dans bsnes (`sfc/coprocessor/sdd1/decompressor.cpp`, port de byuu). Brevet de reference: Ricoh ABS (US 5,381,145). Page de synthese: https://wiki.superfamicom.org/s-dd1 (decrit l'algorithme; ne donne NI les registres NI le mappage).
- Registres, mappage ROM/SRAM et interception DMA: bsnes (`sdd1.cpp`) et la base de cartes d'ares (`mia/Database/Super Famicom Boards.bml`, cartes `SHVC-LN3B-01`, `SHVC-1N0N-(01,10)`, `SDD1`, `SDD1-RAM`).
- Les tables sont dans `docs/specs/data/sdd1_tables.tsv`; les vecteurs de test dans `docs/specs/data/sdd1_vectors.tsv`; le modele de reference Python est `docs/tools/sdd1_ref.py` (transcription independante de l'algorithme: les vecteurs valent ce que vaut la transcription; la preuve finale est le gate avec les vrais jeux).
- Tout ce qui n'est pas verifie est marque `UNKNOWN - to confirm` (section 10): ne pas deviner.

## 2. Detection et perimetre
- Active si le chipset `$FFD6` a le quartet haut = 4 et le quartet bas >= 3 (`$43` = S-DD1, `$45` = S-DD1 + SRAM sur pile). Le mappage du S-DD1 remplace alors le mappage LoROM/HiROM normal; l'octet de mode `$FFD5` n'est PAS utilise pour decider (les jeux ont `$32`, mais les ROM de test des taches utilisent `$20`).
- L'en-tete est lu a la position LoROM (`$7FC0`); `pick_header` reste inchange.
- Hors perimetre: HDMA depuis la zone decompressee, timing du decompresseur (instantane), autres puces.

## 3. Registres ($4800-$4807, banques 00-3F et 80-BF)
| Adresse | Nom | Reset | Ecriture | Lecture |
|---|---|---|---|---|
| $4800 | activation "hard" des canaux DMA (bit n = canal n) | $00 | valeur entiere | valeur |
| $4801 | activation "soft" (bit n = canal n); le S-DD1 efface le bit du canal quand la decompression de ce canal est finie | $00 | valeur entiere | valeur |
| $4804 | banque MMC 0 (fenetre C0-CF) | $00 | `v & $8F` | valeur |
| $4805 | banque MMC 1 (fenetre D0-DF) | $01 | `v & $8F` | valeur |
| $4806 | banque MMC 2 (fenetre E0-EF) | $02 | `v & $8F` | valeur |
| $4807 | banque MMC 3 (fenetre F0-FF) | $03 | `v & $8F` | valeur |
| $4802-$4803, $4808-$480F | non utilises | - | ignore | `UNKNOWN - to confirm` (defaut: bus ouvert, valeur `mdr`) |

Seuls les 4 bits bas de $4804-$4807 selectionnent le bloc de 1 Mio; le bit 7 de $4805 et de $4807 est utilise par le mappage des banques 20-3F / A0-BF (section 4). Le canal n est decompresse seulement si `($4800 & $4801) >> n & 1 == 1`.

## 4. Mappage de la ROM et de la SRAM
Toutes les adresses sont sur 24 bits (`addr = bank << 16 | offset`). `rom[x]` = `rom[x & rom_mask]` (la ROM est deja normalisee en puissance de 2 par `normalize`).

**Banques 00-3F et 80-BF, offsets $8000-$FFFF** (fenetre "LoROM"):
```
a = addr
if bank in 20..=3F  and (r4805 & 0x80) != 0 { a = a with bit 21 cleared }      // bit 21 = banque & 0x20
if bank in A0..=BF  and (r4807 & 0x80) != 0 { a = a with bit 21 cleared }
offset_rom = ((a >> 1) & 0x1F8000) | (a & 0x7FFF)
```
**Banques C0-FF, tous offsets** (fenetre MMC, blocs de 1 Mio):
```
block = (addr >> 20) & 3           // C0-CF -> 0, D0-DF -> 1, E0-EF -> 2, F0-FF -> 3
offset_rom = ((reg[4804 + block] & 0x0F) << 20) | (addr & 0xFFFFF)
```
**Banques 40-7D**: pas de ROM (bus ouvert). Banques 7E-7F: WRAM (inchange).

**SRAM** (cartes avec SRAM, ex. Star Ocean 8 Kio), carte `SHVC-LN3B-01`: banques 00-3F et 80-BF offsets $6000-$7FFF: `index = (offset & $1FFF) % sram_len`; banques 70-73 offsets $0000-$FFFF: `index = (((bank & 3) << 16) | offset) % sram_len`. (Le miroir 70-73 vient de la carte d'ares; `UNKNOWN - to confirm` si un jeu l'utilise.)

## 5. Interception DMA
Le S-DD1 espionne les ecritures du S-CPU aux registres DMA `$43x2-$43x6` (x = canal 0..7) et garde pour chaque canal une copie:
```
$43x2 -> addr bits 0-7     $43x3 -> addr bits 8-15     $43x4 -> addr bits 16-23
$43x5 -> size bits 0-7     $43x6 -> size bits 8-15      (size: u16; la valeur 0 reste 0, voir ci-dessous)
```
L'ecriture est quand meme transmise au DMA normal. Les jeux programment un DMA en mode **adresse A fixe** (bit 3 de `$43x0`): chaque lecture de l'octet suivant se fait donc a la meme adresse source, et c'est le S-DD1 qui renvoie un octet decompresse different a chaque lecture.

Lecture a une adresse `addr` dans les banques C0-FF (`mcu_read`):
```
si (r4800 & r4801) != 0:
    pour n = 0..7:
        si (r4800 bit n) et (r4801 bit n) et addr == dma[n].addr:
            si !dma_ready: decompressor.init(addr); dma_ready = true
            octet = decompressor.read()
            dma[n].size = dma[n].size.wrapping_sub(1)
            si dma[n].size == 0: dma_ready = false; r4801 &= !(1 << n)
            retourner octet
retourner rom[mmc_offset(addr)]          // pas de decompression: ROM normale
```
Le decompresseur lit les octets compresses par la fenetre MMC (`rom[mmc_offset(a)]` pour une adresse 24 bits `a` qui commence a `addr` et s'incremente). Une taille initiale 0 signifie 65536 octets (le decompte `0 -> 0xFFFF` fait le tour).

## 6. Algorithme de decompression
Six modules cooperent. Tous les nombres sont des entiers non signes de la largeur indiquee (u8 sauf mention); les troncatures u8 font partie de l'algorithme.

**Entree**: flux d'octets compresses commencant a l'adresse `start`. Le 1er octet `h`: bits 7-6 = type de plans de bits (`bitplanes_info = h & 0xC0`), bits 5-4 = gabarit de contexte (`context_info = h & 0x30`); les bits 3-0 sont deja des donnees du flux (le flux commence au bit 4).

**IM - gestionnaire d'entree** (`offset: u32`, `bit_count: u8`; init: `offset = start`, `bit_count = 4`):
```
code_word(code_length):
    cw = ((rom(offset) << bit_count) & 0xFF) as u8       // troncature u8
    bit_count += 1
    if cw & 0x80 != 0 {
        cw |= rom(offset + 1) >> (9 - bit_count)          // bit_count deja incremente
        bit_count += code_length
    }
    if bit_count & 0x08 != 0 { offset += 1; bit_count &= 0x07 }
    return cw
```
**GCD - decodeur de Golomb** (`run_count_table`: 256 valeurs de `sdd1_tables.tsv`, section RUNCOUNT):
```
run_count(code_number) -> (mps_count: u8, lps_index: bool):
    cw = code_word(code_number)
    if cw & 0x80 != 0 { (RUNCOUNT[cw >> (code_number ^ 7)], true) } else { (1 << code_number, false) }
```
**BG - generateurs de bits** (8 instances n = 0..7, chacune `mps_count: u8`, `lps_index: bool`, init a 0):
```
bg_bit(n) -> (bit: u8, end_of_run: bool):
    if mps_count[n] == 0 && !lps_index[n] { (mps_count[n], lps_index[n]) = run_count(n) }
    if mps_count[n] != 0 { bit = 0; mps_count[n] -= 1 } else { bit = 1; lps_index[n] = false }
    end_of_run = mps_count[n] == 0 && !lps_index[n]
```
**PEM - estimation de probabilite** (32 contextes, chacun `status: u8 = 0`, `mps: u8 = 0`; table `EVOLUTION[33]` = (code_number, next_if_mps, next_if_lps), section EVOLUTION):
```
pem_bit(ctx) -> u8:
    st = status[ctx]; cur_mps = mps[ctx]
    (code_number, next_mps, next_lps) = EVOLUTION[st]
    (bit, end) = bg_bit(code_number)
    if end {
        if bit != 0 { if st & 0xFE == 0 { mps[ctx] ^= 1 }; status[ctx] = next_lps }
        else        { status[ctx] = next_mps }
    }
    return bit ^ cur_mps
```
**CM - modele de contexte** (`bit_number: u8 = 0` (tourne a 256), `prev[8]: u16 = 0`, `current_bitplane: u8`; init: `0x00 -> 1`, `0x40 -> 7`, `0x80 -> 3`, `0xC0 -> 0`):
```
cm_bit() -> u8:
    match bitplanes_info {
      0x00: current_bitplane ^= 1
      0x40: current_bitplane ^= 1; if bit_number & 0x7F == 0 { current_bitplane = (current_bitplane + 2) & 7 }
      0x80: current_bitplane ^= 1; if bit_number & 0x7F == 0 { current_bitplane ^= 2 }
      0xC0: current_bitplane = bit_number & 7
    }
    cb = prev[current_bitplane]                       // u16
    ctx = (current_bitplane & 1) << 4
    match context_info {
      0x00: ctx |= ((cb & 0x01C0) >> 5) | (cb & 0x0001)
      0x10: ctx |= ((cb & 0x0180) >> 5) | (cb & 0x0001)
      0x20: ctx |= ((cb & 0x00C0) >> 5) | (cb & 0x0001)
      0x30: ctx |= ((cb & 0x0180) >> 5) | (cb & 0x0003)
    }
    bit = pem_bit(ctx)
    prev[current_bitplane] = (cb << 1) | bit         // u16, troncature
    bit_number = bit_number.wrapping_add(1)
    return bit
```
**OL - logique de sortie** (`r0 = 1`, `r1`, `r2` u8):
```
read() -> u8:
    if bitplanes_info != 0xC0 {
        if r0 == 0 { r0 = !r0; return r2 }            // 2e octet de la paire (r0 devient 0xFF)
        r0 = 0x80; r1 = 0; r2 = 0
        while r0 != 0 { if cm_bit() != 0 { r1 |= r0 }; if cm_bit() != 0 { r2 |= r0 }; r0 >>= 1 }
        return r1                                      // r0 vaut 0 en fin de boucle
    } else {
        r0 = 1; r1 = 0
        while r0 != 0 { if cm_bit() != 0 { r1 |= r0 }; r0 <<= 1 }   // r0 u8: 0x80 << 1 = 0
        return r1
    }
```
Note: le test `r0 == 0` de la 1ere branche distingue "octet pair deja produit": apres un `read` qui a rempli r1/r2 la boucle laisse `r0 = 0`, donc le `read` suivant renvoie `r2` puis remet `r0 = 0xFF` (le `read` d'apres relance la boucle car `r0 != 0`).

Types de plans de bits (information): `00` = tuiles 2 bpp, `01` (0x40) = 8 bpp, `10` (0x80) = 4 bpp, `11` (0xC0) = mode 7.

## 7. Interface Rust (module `puce16-core/src/coprocessor/sdd1/`)
```
coprocessor/mod.rs            pub mod sdd1;
coprocessor/sdd1/mod.rs       pub mod tables; pub mod decomp; pub mod decomp_bits; pub mod decomp_ctx; pub mod regs; pub mod dma; (+ re-exports)
coprocessor/sdd1/tables.rs    pub const RUNCOUNT: [u8; 256]; pub const EVOLUTION: [(u8, u8, u8); 33];
coprocessor/sdd1/decomp.rs    pub struct Decompressor { ... } + new / init / code_word / run_count / im_position
coprocessor/sdd1/decomp_bits.rs  impl Decompressor { pub fn bg_bit / pem_bit / pem_state }
coprocessor/sdd1/decomp_ctx.rs   impl Decompressor { pub fn cm_bit / read }
coprocessor/sdd1/regs.rs      pub struct Sdd1 { ... } + registres, mmc_offset, lorom_offset
coprocessor/sdd1/dma.rs       impl Sdd1 { snoop, streaming read }
```
La ROM est fournie au decompresseur par une fermeture `rom: &F` avec `F: Fn(u32) -> u8` (adresse 24 bits -> octet) pour que le module reste pur et testable sans `Cart`. Signatures (tests dans `tests/`: tout est `pub`):
```
impl Decompressor {
    pub fn new() -> Decompressor;                                         // etat nul, non demarre
    pub fn init<F: Fn(u32) -> u8>(&mut self, start: u32, rom: &F);        // IM, BG, PEM, CM, OL (lit l'octet d'en-tete a `start`)
    pub fn code_word<F: Fn(u32) -> u8>(&mut self, code_length: u8, rom: &F) -> u8;
    pub fn run_count<F: Fn(u32) -> u8>(&mut self, code_number: u8, rom: &F) -> (u8, bool);
    pub fn bg_bit<F: Fn(u32) -> u8>(&mut self, n: usize, rom: &F) -> (u8, bool);
    pub fn pem_bit<F: Fn(u32) -> u8>(&mut self, ctx: usize, rom: &F) -> u8;
    pub fn cm_bit<F: Fn(u32) -> u8>(&mut self, rom: &F) -> u8;
    pub fn read<F: Fn(u32) -> u8>(&mut self, rom: &F) -> u8;
    pub fn im_position(&self) -> (u32, u8);                               // (offset, bit_count)
    pub fn pem_state(&self, ctx: usize) -> (u8, u8);                      // (status, mps)
}
```
`Decompressor::new()` + `init()` produisent le meme etat qu'une instance neuve (re-`init` complet, y compris BG et PEM).

## 8. Etat, sauvegardes d'etat, hash
Etat a sauver (quand la cartouche a un S-DD1 uniquement; le format des autres cartouches ne change PAS): `r4800, r4801, r4804..r4807` (6 octets), pour chaque canal `addr` (u32 -> 3 octets) et `size` (u16). Le decompresseur n'est pas sauve: le DMA se termine toujours dans la frame (les sauvegardes se font entre deux frames) et `dma_ready = false` apres chargement. `hash_into` inclut les memes champs.

## 9. Plan de test
1. Tables: `RUNCOUNT` et `EVOLUTION` egales aux lignes de `sdd1_tables.tsv`.
2. Etages: valeurs concretes des taches E09-02..04 (IM, GCD, BG, PEM, CM).
3. Vecteurs d'or: 16 en-tetes `$00,$10,...,$F0`, flux = 4096 octets d'un generateur congruentiel (`x = x * 1664525 + 1013904223 mod 2^32`, octet = `x >> 24`, graine `0x1000 + n`, premier octet `(b & 0x0F) | (n << 4)`), lecture cyclique (`rom(a) = data[a % 4096]`): les 64 premiers octets decompresses et les FNV-64 (offset `0xcbf29ce484222325`, premier `0x100000001b3`) des 256 puis 8192 premiers octets doivent egaler `sdd1_vectors.tsv`.
4. Registres et mappage: valeurs de la section 3 et 4.
5. DMA: espionnage, decompte, effacement de `r4801`.
6. Bout en bout: ROM synthetique avec chipset `$43`, DMA d'une zone decompressee vers la WRAM (E09-10).
7. Gate: Street Fighter Alpha 2 et Star Ocean fournis par l'utilisateur (E09-11).

## 10. UNKNOWN - to confirm
- Lecture de $4802-$4803 et $4808-$480F (bsnes lit la ROM a une adresse sans rapport; defaut choisi: bus ouvert).
- Comportement du mappage MMC quand le numero de bloc depasse la taille de la ROM (Star Ocean: 6 Mio, ROM normalisee a 8 Mio par miroir): defaut = le masque `& rom_mask` de `Cart`.
- Miroir SRAM aux banques 70-73 (carte d'ares) et taille exacte de `mask`.
- Precision de la transcription Python (voir section 1): a valider par le gate.
- HDMA vers/depuis une zone decompressee: non gere.
