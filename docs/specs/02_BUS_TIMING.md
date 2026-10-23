# SPEC 02 - BUS, MEMOIRE, TIMING, MMIO CPU ($4016/$4017, $4200-$421F, $2180-$2183)

Source: wiki SNESdev (Memory_map, Timing, MMIO_registers) + decisions D-004, D-008. Les points marques `UNKNOWN - to confirm` ont une valeur par defaut.

## 1. Constantes (fichier `consts.rs`)

```rust
pub const MASTER_HZ_NTSC: u64 = 21_477_272;
pub const MASTER_HZ_PAL:  u64 = 21_281_370;
pub const APU_RESONATOR_HZ: u64 = 24_606_720;      // D-004
pub const CLOCKS_PER_LINE: u32 = 1364;             // ligne normale
pub const CLOCKS_SHORT_LINE: u32 = 1360;           // NTSC, entrelace off, field=1, V=240
pub const CLOCKS_LONG_LINE: u32 = 1368;            // PAL, entrelace on, field=1, V=311
pub const DOTS_PER_LINE: u32 = 340;                // H = 0..339 (ligne longue: 341 dots, H = 0..340)
pub const LINES_NTSC: u32 = 262;                   // 263 en entrelace, field=0? voir 3.2
pub const LINES_PAL: u32 = 312;
pub const VBLANK_START_NORMAL: u32 = 225;
pub const VBLANK_START_OVERSCAN: u32 = 240;
pub const DRAM_REFRESH_CLOCKS: u32 = 40;
pub const DRAM_REFRESH_HCLOCK: u32 = 538;          // UNKNOWN - to confirm (D-018 provisoire)
pub const VISIBLE_DOT_FIRST: u32 = 22;             // premier pixel a clock 88 = dot 22
pub const VISIBLE_CLOCK_FIRST: u32 = 88;
pub const VISIBLE_CLOCK_END: u32 = 1112;           // exclu: 256 pixels * 4 clocks
```

## 2. Carte memoire CPU (24 bits: bank:addr)

| Banks | Adresses | Contenu | Vitesse (clocks) |
|---|---|---|---|
| $00-$3F, $80-$BF | $0000-$1FFF | WRAM (miroir des 8 premiers KiB de $7E) | 8 |
| | $2000-$20FF | non mappe (open bus) | 6 |
| | $2100-$213F | registres PPU (voir 05_PPU.md) | 6 |
| | $2140-$217F | ports APU (APUIO0-3 miroir tous les 4) | 6 |
| | $2180 | WMDATA | 6 |
| | $2181-$2183 | WMADDL/M/H (ecriture seule) | 6 |
| | $2184-$21FF | non mappe (open bus) | 6 |
| | $2200-$3FFF | non mappe en v1 (coprocesseurs) -> open bus | 6 |
| | $4000-$41FF | JOYSER ($4016/$4017); le reste open bus | **12** (tout le bloc $4000-$41FF, decision provisoire; wiki: JOYSER0/1) |
| | $4200-$42FF | registres CPU 5A22 ($4200-$421F); le reste open bus | 6 |
| | $4300-$437F | registres DMA | 6 |
| | $4380-$5FFF | open bus | 6 |
| | $6000-$7FFF | cartouche (SRAM / expansion selon mapper) | 8 |
| | $8000-$FFFF | ROM cartouche | 8, ou 6 si banque >= $80 et MEMSEL bit0=1 |
| $40-$6F | $0000-$FFFF | cartouche (selon mapper) | 8 |
| $70-$7D | $0000-$FFFF | cartouche / SRAM (selon mapper) | 8 |
| $7E-$7F | $0000-$FFFF | WRAM 128 KiB ($7E0000-$7FFFFF) | 8 |
| $C0-$FF | $0000-$FFFF | cartouche (ROM) | 8, ou 6 si MEMSEL bit0=1 |

Cycles internes (`IO`): 6 clocks.

Algorithme de vitesse (fonction `access_clocks(addr24, memsel) -> u8`), a implementer EXACTEMENT ainsi (equivalent au tableau):
```
bank = addr >> 16 ; a = addr & 0xFFFF
if (addr & 0x408000) != 0 {                 // A15 ou A22 = 1 : zone ROM
    return if (addr & 0x800000) != 0 && memsel { 6 } else { 8 }
}
if ((a + 0x6000) & 0x4000) != 0 { return 8 }   // $0000-$1FFF et $6000-$7FFF
if ((a - 0x4000) as u16 & 0x7E00) != 0 { return 6 }   // hors $4000-$41FF
return 12                                             // $4000-$41FF
```
(`UNKNOWN - to confirm` pour banques $40-$7D lorsque A22=1 et A15=0: la formule les classe "ROM" => 8, OK car ce sont des banques cartouche/WRAM a 8.)

## 3. Temps

### 3.1 Position dans la ligne
- `hclock` (0..1363) = master clocks depuis le debut de la ligne. `dot` = H counter (0..339).
- Conversion clock -> dot: 340 dots; les 4 dots de 5 clocks sont aux H = 323, 324, 327, 328 (D-008, provisoire), tous les autres 4 clocks. Table precalculee `DOT_START_CLOCK[0..=340]`.
- Ligne longue (PAL entrelace field=1 V=311): 1368 clocks, 341 dots (337 de 4 + 4 de 5): dot supplementaire a la fin. Ligne courte: 1360 clocks, 340 dots tous a 4 clocks.
- Debut du picture: pixel x=0 a clock 88 (dot 22). 256 pixels: clocks 88..1111.
- HBlank: flag HVBJOY.6 = 1 pour dot >= 274 (clock >= 1096) ou dot < 1 (clock < 4). `UNKNOWN - to confirm` bornes exactes; defaut: H=1 si hclock >= 1096 ou hclock < 4... (valeurs de depart, ROM de test timing trancheront).

### 3.2 Position dans la frame
- NTSC: V = 0..261 (262 lignes). PAL: V = 0..311.
- Entrelace (SETINI bit0=1): une ligne supplementaire toutes les 2 frames, sur les frames "even" (field=0): NTSC V=0..262 (263 lignes), PAL V=0..312. `UNKNOWN - to confirm` quel field porte la ligne en plus; defaut: la frame ou STAT78 field=0.
- Field: bascule a CHAQUE debut de frame (V=0), entrelace ou non (le wiki: "every odd frame may have one adjusted scanline"); valeur initiale 0. STAT78 bit7 = field. Le PPU possede V/dot/field (voir 05_PPU.md section 5).
- VBlank: debut a V=225 (overscan SETINI.2=0) ou V=240 (overscan=1); fin apres V=261 (NTSC)/V=311 (PAL), la frame suivante commence a V=0.
- Ligne courte NTSC: V=240, interlace off, field=1 : 1360 clocks. Ligne longue PAL: V=311, interlace on, field=1: 1368.
- Total frame NTSC (sans entrelace): 262 * 1364 = 357 368 clocks (field=1 : 357 364 si ligne courte appliquee). PAL: 312 * 1364 = 425 568.
- Ligne 0: cachee (affiche noir). Lignes visibles V=1..224 (ou 1..239 en overscan). Le pixel de sortie y = V - 1.

### 3.3 Rafraichissement DRAM
Le CPU est arrete 40 clocks par ligne: quand un cycle CPU commencerait a `hclock >= DRAM_REFRESH_HCLOCK` et que le refresh de la ligne n'a pas eu lieu, on insere 40 clocks (le cycle est decale). Les autres composants (PPU, APU, DMA HDMA) continuent. `UNKNOWN - to confirm` position exacte (538 par defaut) et alignement.

### 3.4 Avance du temps par le bus
`Bus::read(addr)`: `t = access_clocks(addr)`; applique refresh si necessaire; **le temps passe en deux parties**: les 6/8/12 clocks sont decoupes ainsi: lecture de la valeur a `master + (t - 4)`... `UNKNOWN - to confirm` le point d'echantillonnage exact dans le cycle. Valeur de depart simple: lecture/ecriture effectuee a la **fin** du cycle (apres avoir fait avancer `master` de `t` clocks et appele `catch_up`). Les effets de bord PPU/MMIO sont donc vus a l'instant `master_end`. Un ecart de quelques clocks peut etre corrige plus tard par E03-T (tuning) sans changer l'architecture.

## 4. Open bus
- `mdr: u8` = derniere valeur vue sur le bus de donnees (mise a jour a chaque lecture ET ecriture CPU, et a chaque octet DMA lu cote A-bus... voir 03_DMA.md).
- Lecture d'une adresse non mappee: retourne `mdr`. Lecture d'un registre partiellement defini (ex: RDNMI bits 6-4, HVBJOY bits 5-1, TIMEUP bits 6-0, JOYSER bits 7-2): les bits non definis valent les bits correspondants de `mdr`.
- JOYSER1 ($4017): bits 4-2 = 1 toujours; bits 7-5 = mdr.
- PPU: les lectures PPU1 ($2134-$2137? voir 05_PPU.md) utilisent un open bus PPU separe (ppu1_mdr / ppu2_mdr). Defaut v1: ppu1_mdr = derniere valeur lue d'un registre PPU1 ($2134-$2136, $2138-$213A, $213E), ppu2_mdr = derniere valeur lue de PPU2 ($213B-$213D, $213F). `UNKNOWN - to confirm` si necessaire pour les ROM de test.

## 5. WRAM et registres $2180-$2183
- WRAM = `[u8; 131072]`. Adresses $7E0000-$7FFFFF = index 0..0x1FFFF. $00-$3F/$80-$BF:$0000-$1FFF = index addr & 0x1FFF.
- WMADD = 17 bits (WMADDL, WMADDM, WMADDH bit0). `$2181` ecrit bits 7-0, `$2182` bits 15-8, `$2183` bit 0 -> bit 16 (seul bit 0 utilise).
- `$2180` ecriture: `wram[wmadd] = v; wmadd = (wmadd + 1) & 0x1FFFF`. Lecture: `v = wram[wmadd]` puis meme increment.
- Power-on: WMADD = 0 (`UNKNOWN - to confirm`).
- DMA vers/depuis $2180 : voir 03_DMA.md.

## 6. Registres 5A22 ($4200-$421F)

Valeurs au power-on / reset indiquees. Ecritures aux registres en lecture seule et lectures des registres en ecriture seule: open bus.

| Reg | Addr | Dir | Description |
|---|---|---|---|
| NMITIMEN | $4200 | W | `N.VH ...J`: bit7 NMI enable; bits5-4 (V,H) mode IRQ: 00 off, 01 IRQ quand H==HTIME, 10 IRQ quand V==VTIME et H==0, 11 V==VTIME et H==HTIME; bit0 auto-read joypad. Power-on/reset $00. |
| WRIO | $4201 | W | `21DD DDDD` port I/O. Power-on $FF. Bit7 (port 2 pin) connecte a /EXTLATCH du PPU: voir 05_PPU.md H/V latch (front 1->0 verrouille H/V quand WRIO.7=1 avant). |
| WRMPYA | $4202 | W | Facteur A (8 bits non signe). Power-on $FF. |
| WRMPYB | $4203 | W | Facteur B: demarre la multiplication. |
| WRDIVL/H | $4204/$4205 | W | Dividende 16 bits. Power-on $FFFF. |
| WRDIVB | $4206 | W | Diviseur 8 bits: demarre la division. |
| HTIMEL/H | $4207/$4208 | W | Cible H (9 bits). Power-on $1FF. Max valide 339. |
| VTIMEL/H | $4209/$420A | W | Cible V (9 bits). Power-on $1FF. |
| MDMAEN | $420B | W | Active DMA (voir 03_DMA.md). Power-on $00. |
| HDMAEN | $420C | W | Active HDMA. Power-on $00. |
| MEMSEL | $420D | W | Bit0: FastROM (banques >= $80 a 6 clocks). Power-on $00. |
| RDNMI | $4210 | R | `N... VVVV`: bit7 flag NMI (efface a la lecture, et a la fin du VBlank), bits3-0 version CPU = 2 (ici: `(mdr & 0x70) | 0x02 | (flag<<7)`). |
| TIMEUP | $4211 | R | bit7 flag IRQ timer (efface a la lecture); bits6-0 = mdr. Lecture efface aussi la ligne IRQ. |
| HVBJOY | $4212 | R | bit7 VBlank, bit6 HBlank, bit0 auto-read en cours; bits5-1 = mdr. |
| RDIO | $4213 | R | Etat du port I/O (voir WRIO: bits `21` = sorties ET entrees des manettes). |
| RDDIVL/H | $4214/$4215 | R | Quotient 16 bits. |
| RDMPYL/H | $4216/$4217 | R | Produit 16 bits ou reste. |
| JOY1L..JOY4H | $4218-$421F | R | Resultat auto-read. JOY1L=$4218 (bits7-0 du mot 1), JOY1H=$4219. |

### 6.1 Multiplication / division (unite math du 5A22)
- WRMPYB: `rdmpy = wrmpya * wrmpyb` (16 bits). Au power-on la valeur de RDMPY/RDDIV est `UNKNOWN - to confirm` (defaut 0).
- Valeurs intermediaires: une lecture avant la fin retourne un resultat partiel. Defaut v1: le resultat complet est visible a partir de `MUL_DONE = start + 8 * 6 = 48` clocks apres l'ecriture de WRMPYB (division: `start + 16 * 6 = 96`); avant, les registres gardent l'ancienne valeur. `UNKNOWN - to confirm` -> E03-T tuning.
- WRDIVB: si diviseur != 0: `rddiv = wrdiv / div ; rdmpy = wrdiv % div`. Diviseur == 0: `rddiv = 0xFFFF ; rdmpy = wrdiv`.
- Ecrire WRMPYB pendant une division/multiplication en cours: `UNKNOWN - to confirm` (defaut: nouvelle operation ecrase l'ancienne).

### 6.2 NMI
- Flag NMI (RDNMI.7) mis au debut du VBlank (V = vblank_start, H = 0 + `NMI_FLAG_DELAY` clocks; defaut 2 dots = ~8 clocks; `UNKNOWN - to confirm`), efface par lecture de $4210 ou a la fin du VBlank (V=0 debut).
- Ligne NMI vers CPU = `flag AND nmitimen.7`. Front montant => NMI. Voir 01_CPU_65816.md section 8.
- Power-on/reset: flag = 0 (RDNMI & $7F).

### 6.3 IRQ timer (H/V)
- Condition evaluee **a chaque dot** (en pratique: a chaque avance de H):
  - mode 1 (V=0,H=1): vrai quand `H == HTIME` (a chaque ligne);
  - mode 2 (V=1,H=0): vrai quand `V == VTIME` et `H == 0`;
  - mode 3: `V == VTIME` et `H == HTIME`.
- Quand la condition passe de faux a vrai: TIMEUP.7 = 1 (flag) et la ligne IRQ du CPU = 1 (maintenue jusqu'a lecture de $4211 ou jusqu'a ce que NMITIMEN V/H = 00; ecrire NMITIMEN avec V/H=00 efface le flag et la ligne).
- Decalage H reel (IRQ declenche ~ quelques dots apres H==HTIME): `IRQ_H_DELAY` defaut 4 clocks? `UNKNOWN - to confirm`.
- HTIME > 339 ou VTIME > derniere ligne: ne se declenche jamais.
- V=0 H=0 (modes 2,3): decale d'un dot sur certaines revisions: ignorer en v1.

### 6.4 Auto-read joypad
- Si NMITIMEN.0 = 1: au debut du VBlank (V = vblank_start), a partir de `H >= 32.5 dots` (defaut: clock 130 = ligne V, hclock 130; `UNKNOWN - to confirm` entre H=32.5 et 95.5), l'auto-read dure **4224 master clocks**; pendant ce temps HVBJOY.0 = 1.
- Au debut: equivalent a JOYOUT=1 puis 0 (latch). Puis 16 lectures de $4016 et $4017: JOY1 (bits D0 du port 1) , JOY2 (bits D0 du port 2), JOY3 (bits D1 port 1), JOY4 (bits D1 port 2); le 1er bit lu = bit 15.
- Defaut v1: les registres JOYn sont mis a jour **a la fin** des 4224 clocks (pendant l'operation, ils conservent l'ancienne valeur). `UNKNOWN - to confirm`.
- Les manettes standard (voir joypad ci-dessous) renvoient apres 16 bits: 1.

### 6.5 JOYOUT / JOYSER0 / JOYSER1 ($4016/$4017)
- `$4016` ecriture: bit0 = OUT0 (latch). Quand OUT0 passe de 1 a 0, chaque manette charge son registre a decalage (16 bits: B Y Select Start Up Down Left Right A X L R 0 0 0 0, 1 = appuye; ordre de lecture: B d'abord). Tant que OUT0 = 1, le registre est relu continuellement (chaque lecture retourne le bit B).
- `$4016` lecture: bits1-0 = (D1, D0) du port 1 (D1 = 0 pour manette standard); bits 7-2 = mdr. Chaque lecture decale d'un bit (apres 16 lectures: retourne 1).
- `$4017` lecture: bits1-0 = port 2 (D1, D0); bits 4-2 = 1; bits 7-5 = mdr.
- Etat clavier -> bits: format de `set_joypad(port, u16)`: bit15 = B, 14 = Y, 13 = Select, 12 = Start, 11 = Up, 10 = Down, 9 = Left, 8 = Right, 7 = A, 6 = X, 5 = L, 4 = R, 3-0 = 0 (identique a JOY1H:JOY1L apres auto-read).

## 7. Reset et power-on
- Power-on: WRAM = 0x55 (D-011), registres 5A22 comme ci-dessus, PPU voir 05_PPU.md, APU voir 07_SPC700.md.
- Reset (bouton): registres NMITIMEN, HDMAEN, MDMAEN = 0, flags RDNMI/TIMEUP effaces; RAM conservee. CPU: voir 01_CPU_65816.md.
- Premiere instruction apres reset: le temps initial `master` (offset de phase) est `UNKNOWN - to confirm`; defaut: H = 0 de la ligne V = 0, master = 0, puis 7 cycles de reset a 6 clocks... Les ROM ne dependent normalement pas de la phase (sauf tests de timing).
