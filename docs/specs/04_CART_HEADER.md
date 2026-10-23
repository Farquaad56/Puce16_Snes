# SPEC 04 - Cartouche, en-tete ROM, mappers (v1: LoROM, HiROM, ExHiROM)

Source: wiki SNESdev (ROM_header, Memory_map). Fichier cible: `puce16-core/src/cart/{mod,header,mapper}.rs`.
Les tests de cette spec n'ont besoin d'AUCUNE vraie ROM: ils fabriquent des ROM synthetiques en code (`fn make_rom(mode, size) -> Vec<u8>`).

## 1. Chargement d'un fichier ROM

1. Si `len % 1024 == 512`: ignorer les 512 premiers octets (en-tete de copieur). Sinon utiliser tel quel.
2. Si `len < 0x8000`: erreur `CartError::TooSmall`.
3. Normalisation en puissance de 2 (algorithme du wiki):
   - `big` = plus grande puissance de 2 <= len. Si `len == big`: rien a faire.
   - Sinon `rest = len - big`; `small` = plus petite puissance de 2 >= rest; completer la partie restante avec des 0 jusqu'a `small`, puis la repeter jusqu'a ce qu'elle fasse `big` octets. Taille finale = 2 * big.
   - `rom_mask = rom.len() - 1`. Tout acces ROM fait `offset & rom_mask`.

## 2. En-tete interne (adresses CPU $00FFC0-$00FFDF, offset dans le fichier selon le mapper)

| Offset | Taille | Champ |
|---|---|---|
| $FFC0 | 21 | titre ASCII ($20-$7E) |
| $FFD5 | 1 | `001s mmmm`: s=vitesse (0 lent, 1 rapide), mmmm = mapper ($0 LoROM, $1 HiROM, $5 ExHiROM; $2 S-DD1, $3 SA-1, $A SPC7110 = non supportes v1) |
| $FFD6 | 1 | chipset |
| $FFD7 | 1 | taille ROM: `1 << N` Kio |
| $FFD8 | 1 | taille RAM: `1 << N` Kio (0 = aucune) |
| $FFD9 | 1 | code pays |
| $FFDA | 1 | developpeur (si $33: en-tete etendu) |
| $FFDB | 1 | version |
| $FFDC | 2 | complement du checksum (LE) |
| $FFDE | 2 | checksum (LE) |
| $FFE0-$FFFF | 32 | vecteurs |

Position de l'en-tete dans le fichier normalise: LoROM `0x7FC0`, HiROM `0xFFC0`, ExHiROM `0x40FFC0`.

### 2.1 Vecteurs (adresses CPU, banque 00)
Natif: COP $FFE4, BRK $FFE6, ABORT $FFE8, NMI $FFEA, IRQ $FFEE. Emulation: COP $FFF4, ABORT $FFF8, NMI $FFFA, RESET $FFFC, IRQ/BRK $FFFE. Lus par le CPU via le bus (pas par `cart/`).

### 2.2 Chipset ($FFD6)
- Quartet bas: `0` ROM; `1` ROM+RAM; `2` ROM+RAM+batterie; `3` ROM+coproc; `4` ROM+coproc+RAM; `5` ROM+coproc+RAM+batterie; `6` ROM+coproc+batterie.
- Quartet haut (si bas >= 3): `0` DSP-n, `1` GSU, `2` OBC1, `3` SA-1, `4` S-DD1, `5` S-RTC, `E` autre, `F` custom (voir $FFBF).
- v1 (D-005): si bas >= 3 -> `CartError::UnsupportedCoprocessor(nom)` (`UNKNOWN - to confirm`: defaut = refuser plutot que lancer sans coprocesseur).
- `has_battery` = quartet bas dans {2, 5, 6}.
- Taille SRAM = `0` si $FFD8 == 0, sinon `1024 << N` octets si 1 <= N <= 8; N > 8 -> traiter comme 0 (avertissement). Contenu initial SRAM: `0x00` (`UNKNOWN - to confirm`).

### 2.3 Region ($FFD9)
NTSC si code dans {`$00`, `$01`, `$0D`, `$0F`, `$10`}; PAL sinon (`UNKNOWN - to confirm` pour $0D/$0F/$10: les codes $00 et $01 sont surs). L'utilisateur peut forcer (`--region ntsc|pal`). `enum Region { Ntsc, Pal }` (decision D-003).

### 2.4 Checksum
- Calcul sur la ROM normalisee (puissance de 2): mettre temporairement complement=$FFFF et checksum=$0000 dans l'en-tete, additionner tous les octets sur 16 bits (debordement ignore).
- Valide si `checksum_calcule == checksum_en_tete` et `checksum ^ complement == $FFFF`.
- Le checksum n'est PAS bloquant: c'est un critere de score (ROM homebrew sans checksum valides).

## 3. Choix de l'en-tete (heuristique `score_header`)
Candidats: les trois positions ci-dessus dont `pos + 0x40 <= rom.len()`. Pour chacun, `score` entier:

| Critere | Points |
|---|---|
| checksum + complement == $FFFF | +4 |
| checksum calcule == checksum en-tete | +2 |
| bits 7-5 de $FFD5 == `001` et mapper (mmmm) coherent avec la position (LoROM: 0,2,3,A?; HiROM: 1; ExHiROM: 5) | +2 |
| `1<<N_rom` Kio >= taille normalisee / 2 (taille declaree plausible) | +1 |
| vecteur RESET ($FFFC) >= $8000 | +2, sinon -4 |
| 1er octet a l'adresse RESET (lu via le mapper candidat) dans {$78 SEI, $18 CLC, $38 SEC, $9C/$64/$74/$9E STZ, $4C JMP, $5C JML, $DC JMP[], $A9 LDA#, $C2 REP, $E2 SEP} | +1 |
| 1er octet dans {$00 BRK, $02 COP, $DB STP, $42 WDM, $FF} | -2 |
| titre entierement ASCII $20-$7E | +1 |
| N_rom dans 7..=14 et N_ram dans 0..=8 | +1 |

Le candidat de score maximal gagne; egalite: ordre LoROM, HiROM, ExHiROM. Si aucun candidat n'existe -> `CartError::NoHeader`. `UNKNOWN - to confirm`: ces poids sont un point de depart (inspires de bsnes/snes9x); ils sont ajustes par la suite si une ROM de la suite de test est mal detectee. Option CLI `--mapper lorom|hirom|exhirom` pour forcer.

## 4. Mappers: fonction `read(bank: u8, addr: u16) -> Option<u8>` (None = la cartouche ne repond pas: open bus)
`rom_offset` est toujours suivi de `& rom_mask`. Les zones systeme (WRAM, MMIO, banques $7E-$7F) sont traitees par `bus/` avant d'appeler `cart/`.

### 4.1 LoROM (banques de 32 Kio)
- Banques `$00-$7D` et `$80-$FF`, `addr >= $8000`: `rom_offset = ((bank & 0x7F) << 15) | (addr & 0x7FFF)`.
- Banques `$40-$7D` et `$C0-$FF`, `addr < $8000`: miroir de la moitie haute: meme formule (`addr & 0x7FFF`), sauf SRAM.
- SRAM (si taille > 0): banques `$70-$7D` et `$F0-$FF`, `addr < $8000`: `sram_off = (((bank & 0x0F) << 15) | addr) % sram_len` (`UNKNOWN - to confirm`: les variantes sont nombreuses, ce defaut couvre la plupart des jeux <= 2 Mio).
- Banques `$00-$3F`/`$80-$BF`, `addr < $8000`: systeme, pas la cartouche.

### 4.2 HiROM (banques de 64 Kio)
- Banques `$C0-$FF`, toute l'adresse: `rom_offset = ((bank & 0x3F) << 16) | addr`.
- Banques `$40-$7D`: miroir de `$C0-$FD` (meme formule avec `bank & 0x3F`).
- Banques `$00-$3F` et `$80-$BF`, `addr >= $8000`: `rom_offset = ((bank & 0x3F) << 16) | addr`.
- SRAM: banques `$20-$3F` et `$A0-$BF`, `$6000 <= addr <= $7FFF`: `sram_off = (((bank & 0x1F) << 13) | (addr - 0x6000)) % sram_len`.

### 4.3 ExHiROM
- `rom_offset = (((bank & 0x80) == 0) as usize) << 22 | ((bank & 0x3F) << 16) | addr` pour: banques `$40-$7D` et `$C0-$FF` (toute l'adresse), banques `$00-$3F` et `$80-$BF` avec `addr >= $8000`.
  (A23 inverse -> A22: les banques `$00-$7D` voient la 2e moitie de 4 Mio, `$80-$FF` la 1re moitie.) L'en-tete est donc a `0x40FFC0`.
- SRAM: comme HiROM (banques `$20-$3F`/`$A0-$BF`, `$6000-$7FFF`).

### 4.4 Ecriture `write(bank, addr, v) -> bool`
Seules les zones SRAM sont inscriptibles (retourne true si geree). Ecriture dans la ROM: ignoree (retourne false).

## 5. Interface Rust (cart/mod.rs)
```rust
pub enum MapMode { LoRom, HiRom, ExHiRom }
pub enum Region { Ntsc, Pal }
pub enum CartError { TooSmall, NoHeader, UnsupportedCoprocessor(&'static str) }
pub struct Header { pub title: String, pub map_byte: u8, pub chipset: u8, pub rom_log2: u8, pub ram_log2: u8,
                    pub country: u8, pub dev_id: u8, pub version: u8, pub checksum: u16, pub complement: u16 }
pub struct Cart { /* rom (normalisee), rom_mask, map, sram, has_battery, header, region */ }
impl Cart {
    pub fn from_bytes(data: &[u8]) -> Result<Cart, CartError>;
    pub fn from_bytes_forced(data: &[u8], map: Option<MapMode>, region: Option<Region>) -> Result<Cart, CartError>;
    pub fn read(&self, bank: u8, addr: u16) -> Option<u8>;
    pub fn write(&mut self, bank: u8, addr: u16, v: u8) -> bool;
    pub fn header(&self) -> &Header;
    pub fn map_mode(&self) -> MapMode;
    pub fn region(&self) -> Region;
    pub fn has_battery(&self) -> bool;
    pub fn sram(&self) -> &[u8];
    pub fn load_sram(&mut self, data: &[u8]); // tronque ou complete par 0
}
```

## 6. Tests (prefixe = id de tache)
- Fichier avec en-tete copieur de 512 octets: ignore.
- ROM de 3 Mio (2 Mio + 1 Mio): normalisee a 4 Mio, derniere partie dupliquee (verifier offsets).
- ROM synthetique LoROM/HiROM/ExHiROM avec checksum valide: detection correcte, lecture `read(0x00,0x8000)` = octet 0 de la ROM (LoROM/HiROM), `read(0xC0,0x0000)` = octet 0 (HiROM/ExHiROM).
- Formules de mapping: tableau de cas (bank, addr) -> offset attendu pour chaque mapper (au moins 8 cas chacun, calcules a la main dans le test).
- SRAM: ecriture puis lecture dans la zone de chaque mapper; miroir modulo `sram_len`.
- Coprocesseur ($FFD6 = $13) -> `UnsupportedCoprocessor`.
