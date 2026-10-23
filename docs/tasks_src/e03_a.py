from _dsl import T

CORE = "crates/puce16-core"
CLI = "crates/puce16-cli"
DESK = "crates/puce16-desktop"
SRC = CORE + "/src"
TST = CORE + "/tests"

CHAIN = ("Execution strictement sequentielle (D-020): cette tache suppose que toutes les precedentes sont terminees. "
         "Ne modifie QUE les fichiers de 'Write only in'.")

# ------------------------------------------------------------------ E03 part A: cartridge (E03-01..06)

T("E03-01", "cart/header.rs: types, en-tete interne, checksum, region, SRAM",
  size=140, deps=["E02-41"],
  context="""Premiere tache de l'epopee E03 (assemblage du systeme). On commence par la cartouche: fonctions PURES sur des octets,
testables sans ROM reelle. Le choix de l'en-tete (heuristique) et les mappers viennent dans les taches suivantes. """ + CHAIN,
  embed=["04_CART_HEADER.md#2. En-tete interne", "04_CART_HEADER.md#2.2 Chipset", "04_CART_HEADER.md#2.3 Region", "04_CART_HEADER.md#2.4 Checksum"],
  write=[SRC + "/cart/mod.rs", SRC + "/cart/header.rs", TST + "/e03_01_header.rs"],
  spec="""Dans `cart/mod.rs` (garde `pub mod header; pub mod mapper;`) ajoute:
```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum MapMode { LoRom, HiRom, ExHiRom }
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum CartError { TooSmall, NoHeader, UnsupportedCoprocessor(&'static str) }
pub use header::Header;
```
(`Region` reste dans `crate::region`.) Dans `cart/header.rs`:
```rust
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Header { pub title: String, pub map_byte: u8, pub chipset: u8, pub rom_log2: u8, pub ram_log2: u8,
                    pub country: u8, pub dev_id: u8, pub version: u8, pub checksum: u16, pub complement: u16 }
pub fn header_pos(map: MapMode) -> usize;                  // LoRom 0x7FC0, HiRom 0xFFC0, ExHiRom 0x40FFC0
pub fn parse_header(rom: &[u8], pos: usize) -> Option<Header>;   // None si pos + 0x40 > rom.len()
pub fn compute_checksum(rom: &[u8], pos: usize) -> u16;    // somme u16 (wrapping) de tous les octets, avec complement=0xFFFF et checksum=0x0000 dans l'en-tete
pub fn region_from_country(c: u8) -> Region;               // Ntsc si c dans {0x00,0x01,0x0D,0x0F,0x10}, sinon Pal
pub fn sram_bytes(ram_log2: u8) -> usize;                  // 0 si ram_log2 == 0 ou > 8 ; sinon 1024 << ram_log2
pub fn chipset_info(chipset: u8) -> (bool, Option<&'static str>);   // (has_battery, coprocessor name)
```
- `parse_header`: octets relatifs a `pos`: titre `+0x00..0x15` (chaque octet 0x20..=0x7E -> le caractere, sinon `'?'`, puis supprimer les espaces de fin), `map_byte +0x15`, `chipset +0x16`, `rom_log2 +0x17`, `ram_log2 +0x18`, `country +0x19`, `dev_id +0x1A`, `version +0x1B`, `complement +0x1C` (u16 LE), `checksum +0x1E` (u16 LE).
- `compute_checksum`: somme de tous les octets de `rom`, MOINS les 4 octets reels `pos+0x1C..pos+0x20`, PLUS `0xFF+0xFF` (ceux du complement force a 0xFFFF; le checksum force a 0 ajoute 0).
- `chipset_info`: `low = chipset & 0xF`; `has_battery = low in {2,5,6}`; coprocesseur `Some(nom)` si `low >= 3`, nom selon `chipset >> 4`: 0 `"DSP"`, 1 `"GSU"`, 2 `"OBC1"`, 3 `"SA-1"`, 4 `"S-DD1"`, 5 `"S-RTC"`, 0xE `"other"`, 0xF `"custom"`, autres `"unknown"`.""",
  tests=[("pos", "`header_pos(LoRom)==0x7FC0`, `header_pos(HiRom)==0xFFC0`, `header_pos(ExHiRom)==0x40FFC0`."),
         ("parse_fields", "tampon de 0x8000 octets a zero; a 0x7FC0 ecris `b\"HELLO WORLD          \"` (21 octets, espaces de fin); octets: [0x7FD5]=0x20, [0x7FD6]=0x02, [0x7FD7]=0x0A, [0x7FD8]=0x03, [0x7FD9]=0x01, [0x7FDA]=0x33, [0x7FDB]=0x02, [0x7FDC..0x7FDE]=[0x34,0x12], [0x7FDE..0x7FE0]=[0xCB,0xED]. `parse_header(&buf,0x7FC0)` == Header{title:\"HELLO WORLD\", map_byte:0x20, chipset:2, rom_log2:10, ram_log2:3, country:1, dev_id:0x33, version:2, checksum:0xEDCB, complement:0x1234}."),
         ("parse_none", "`parse_header(&buf[..0x7FFF], 0x7FC0)==None` ; `parse_header(&buf, 0x7FC1)==None` (0x7FC1+0x40 > 0x8000)."),
         ("title_nonascii", "octet de titre 0x00 ou 0xFF -> `'?'` ; un titre tout a zero (21 octets) donne 21 fois `'?'`."),
         ("checksum_zero", "tampon de 0x8000 zeros: `compute_checksum(&buf,0x7FC0)==0x01FE`."),
         ("checksum_ignores_field", "meme tampon avec buf[0]=1, buf[1]=2 et les 4 octets 0x7FDC..0x7FE0 = [0x12,0x34,0x56,0x78]: `compute_checksum==0x0201` (3 + 0x01FE)."),
         ("region", "`region_from_country`: 0x00 et 0x01 -> Ntsc ; 0x0D, 0x0F, 0x10 -> Ntsc ; 0x02, 0x11, 0xFF -> Pal."),
         ("sram", "`sram_bytes`: 0->0 ; 1->2048 ; 3->8192 ; 8->262144 ; 9->0 ; 255->0."),
         ("chipset", "`chipset_info(0x00)==(false,None)` ; `(0x01)==(false,None)` ; `(0x02)==(true,None)` ; `(0x13)==(false,Some(\"GSU\"))` ; `(0x34)==(false,Some(\"SA-1\"))` ; `(0x25)==(true,Some(\"OBC1\"))` ; `(0x03)==(false,Some(\"DSP\"))` ; `(0x56)==(true,Some(\"unknown\"))`.")])

T("E03-02", "cart/mapper.rs: formules LoROM / HiROM / ExHiROM (fonctions pures)",
  size=140,
  context="""Les mappers sont des fonctions pures (bank, addr) -> offset. `Cart` (E03-05/06) les appelle. Les cas de test sont calcules a la main
dans les tests ci-dessous. """ + CHAIN,
  embed=["04_CART_HEADER.md#4. Mappers"],
  write=[SRC + "/cart/mapper.rs", TST + "/e03_02_mapper.rs"],
  spec="""Dans `cart/mapper.rs`:
```rust
/// Offset into the (power-of-two) ROM BEFORE masking. None = the cartridge does not answer with ROM here.
pub fn rom_offset(map: MapMode, bank: u8, addr: u16) -> Option<usize>;
/// Offset into SRAM (already reduced modulo sram_len). None if not an SRAM address or sram_len == 0.
pub fn sram_offset(map: MapMode, bank: u8, addr: u16, sram_len: usize) -> Option<usize>;
```
`rom_offset` (banques `$7E` et `$7F` = WRAM: toujours `None`; pour toute autre banque suivre la spec):
- LoRom: `addr >= 0x8000` et banque `0x00..=0x7D | 0x80..=0xFF`: `((bank & 0x7F) << 15) | (addr & 0x7FFF)`. `addr < 0x8000` et banque `0x40..=0x7D | 0xC0..=0xFF`: meme formule. Sinon `None`.
- HiRom: banques `0xC0..=0xFF | 0x40..=0x7D` (toute adresse) ou banques `0x00..=0x3F | 0x80..=0xBF` avec `addr >= 0x8000`: `((bank & 0x3F) << 16) | addr`. Sinon `None`.
- ExHiRom: memes conditions que HiRom, offset `(if bank & 0x80 == 0 { 0x40_0000 } else { 0 }) | ((bank & 0x3F) << 16) | addr`.
`sram_offset` (si `sram_len == 0` -> `None`):
- LoRom: banques `0x70..=0x7D | 0xF0..=0xFF` et `addr < 0x8000`: `(((bank & 0x0F) as usize) << 15 | addr as usize) % sram_len`.
- HiRom et ExHiRom: banques `0x20..=0x3F | 0xA0..=0xBF` et `0x6000 <= addr <= 0x7FFF`: `((((bank & 0x1F) as usize) << 13) | (addr as usize - 0x6000)) % sram_len`.""",
  tests=[("lorom_rom", "LoRom, (bank,addr)->Some(offset): (0x00,0x8000)->0 ; (0x00,0xFFFF)->0x7FFF ; (0x01,0x8000)->0x8000 ; (0x3F,0x8000)->0x1F8000 ; (0x80,0x8000)->0 ; (0xBF,0xFFFF)->0x1FFFFF ; (0x40,0x0000)->0x200000 ; (0xC0,0x1234)->0x201234 ; (0x7D,0xFFFF)->0x3EFFFF ; (0xFF,0x0000)->0x3F8000. None: (0x00,0x7FFF), (0x3F,0x0000), (0x7E,0x8000), (0x7F,0xFFFF)."),
         ("hirom_rom", "HiRom: (0xC0,0x0000)->0 ; (0xC0,0xFFFF)->0xFFFF ; (0xC1,0x0000)->0x10000 ; (0xFF,0xFFFF)->0x3FFFFF ; (0x40,0x1234)->0x1234 ; (0x7D,0x0000)->0x3D0000 ; (0x00,0x8000)->0x8000 ; (0x3F,0xFFFF)->0x3FFFFF ; (0x80,0x8000)->0x8000 ; (0xBF,0x9000)->0x3F9000. None: (0x00,0x7FFF), (0x7E,0x0000), (0x7F,0x8000), (0x20,0x6000)."),
         ("exhirom_rom", "ExHiRom: (0xC0,0x0000)->0 ; (0xFF,0xFFFF)->0x3FFFFF ; (0x40,0x0000)->0x400000 ; (0x7D,0xFFFF)->0x7DFFFF ; (0x00,0x8000)->0x408000 ; (0x3F,0xFFFF)->0x7FFFFF ; (0x80,0x8000)->0x8000 ; (0xBF,0x8000)->0x3F8000. None: (0x00,0x7FFF), (0x7E,0x0000)."),
         ("lorom_sram", "LoRom: sram_len=0x2000: (0x70,0x0000)->0 ; (0x70,0x1234)->0x1234 ; (0x70,0x2000)->0 ; (0x71,0x0000)->0 ; (0xF0,0x7FFF)->0x1FFF. sram_len=0x8000: (0x71,0x0010)->0x10 ; sram_len=0x20000: (0x73,0x0100)->0x18100. None: (0x70,0x8000), (0x6F,0x0000), (0x00,0x6000), (0x7E,0x0000) ; sram_len=0 -> None partout."),
         ("hirom_sram", "HiRom et ExHiRom (memes resultats): sram_len=0x2000: (0x20,0x6000)->0 ; (0x20,0x7FFF)->0x1FFF ; (0x21,0x6000)->0 ; sram_len=0x8000: (0x21,0x6000)->0x2000 ; (0x3F,0x7FFF)->0x7FFF ; (0xA0,0x6010)->0x10. None: (0x20,0x5FFF), (0x20,0x8000), (0x40,0x6000), (0x00,0x6000)."),
         ("no_overlap_lorom_sram_rom", "LoRom: `(0x70,0x0000)` donne `rom_offset == Some(0x380000)` ET `sram_offset(.., 0x2000) == Some(0)`: la fonction `Cart::read` (E03-06) devra tester la SRAM AVANT la ROM. Verifie les deux valeurs dans ce test.")])

T("E03-03", "cart/header.rs: score_header et choix de l'en-tete",
  size=120,
  context="""Heuristique de detection du mapper (spec 04 section 3). Utilise `mapper::rom_offset` pour lire le 1er octet du code RESET. """ + CHAIN,
  embed=["04_CART_HEADER.md#3. Choix de l'en-tete"],
  write=[SRC + "/cart/header.rs", TST + "/e03_03_score.rs"],
  spec="""Ajoute a `cart/header.rs` (sans toucher aux fonctions existantes):
```rust
pub fn score_header(rom: &[u8], map: MapMode) -> Option<i32>;   // rom = ROM NORMALISEE (longueur puissance de 2). None si pos + 0x40 > rom.len()
pub fn pick_from_scores(s: [Option<i32>; 3]) -> Option<MapMode>;  // s = [LoRom, HiRom, ExHiRom]; le plus grand score gagne; egalite: premier dans cet ordre
pub fn pick_header(rom: &[u8]) -> Result<MapMode, CartError>;      // NoHeader si aucun candidat
```
`score_header`: `pos = header_pos(map)`, `h = parse_header(rom,pos)?`, puis somme (voir tableau de la spec):
+4 si `checksum as u32 + complement as u32 == 0xFFFF` ; +2 si `compute_checksum(rom,pos) == h.checksum` ;
+2 si `(h.map_byte & 0xE0) == 0x20` ET `mmmm = h.map_byte & 0x0F` coherent (LoRom: 0,2,3,0xA ; HiRom: 1 ; ExHiRom: 5) ;
+1 si `h.rom_log2 < 32` ET `(1u64 << h.rom_log2) * 1024 >= rom.len() as u64 / 2` (`h.rom_log2` = octet `$FFD7` tel quel, taille declaree `1 << N` Kio) ;
RESET = u16 LE a `pos + 0x3C`: +2 si `>= 0x8000`, sinon -4 ;
1er octet du code: `off = mapper::rom_offset(map, 0x00, reset)` ; si `Some(o)` lire `b = rom[o & (rom.len()-1)]`: +1 si b dans {0x78,0x18,0x38,0x9C,0x64,0x74,0x9E,0x4C,0x5C,0xDC,0xA9,0xC2,0xE2}, -2 si b dans {0x00,0x02,0xDB,0x42,0xFF} ; si `None`: 0 ;
+1 si le titre brut (21 octets) est entierement dans 0x20..=0x7E ;
+1 si `h.rom_log2` dans 7..=14 ET `h.ram_log2` dans 0..=8.
NB: pour une `make_rom` de 32 Kio, N = 5 ; de 64 Kio, N = 6 ; de 8 Mio, N = 13.""",
  tests=[("lorom_score", "ROM = `common::make_rom(&[0x78], Map::LoRom)` (32 Kio, N=5): `score_header(&rom, LoRom)==Some(13)` (4+2+2+1+2+1+1) ; avec code `[0x00]` -> Some(10) ; avec `[0xEA]` -> Some(12). `score_header(&rom, HiRom)==None` et `ExHiRom -> None`."),
         ("hirom_picks", "ROM = `make_rom(&[0x78], Map::HiRom)` (64 Kio, N=6): `score_header(&rom,HiRom)==Some(13)` ; `score_header(&rom,LoRom)==Some(-4)` (en-tete a 0x7FC0 vide: seul RESET=0 < 0x8000 compte -4) ; `pick_header(&rom)==Ok(HiRom)`."),
         ("exhirom_picks", "ROM = `make_rom(&[0x78], Map::ExHiRom)` (8 Mio, N=13): `score_header(&rom,ExHiRom)==Some(14)` ; LoRom et HiRom -> Some(-4) ; `pick_header(&rom)==Ok(ExHiRom)`."),
         ("pick_scores", "`pick_from_scores([Some(5),Some(5),None])==Some(LoRom)` ; `[Some(3),Some(5),Some(5)]==Some(HiRom)` ; `[None,None,None]==None` ; `[Some(-4),Some(-4),Some(-4)]==Some(LoRom)` ; `[None,Some(-1),Some(-2)]==Some(HiRom)`."),
         ("no_header", "`pick_header(&vec![0u8; 0x4000])` == `Err(CartError::NoHeader)` (aucun candidat).")])

T("E03-04", "cart/mod.rs: strip_copier_header et normalize (ROM en puissance de 2)",
  size=100,
  context="Chargement d'un fichier ROM: retirer l'en-tete de copieur de 512 octets puis normaliser en puissance de 2 (spec 04 section 1). Fonctions libres pures. " + CHAIN,
  embed=["04_CART_HEADER.md#1. Chargement d'un fichier ROM"],
  write=[SRC + "/cart/mod.rs", TST + "/e03_04_normalize.rs"],
  spec="""Ajoute a `cart/mod.rs` (sans modifier ce qui existe):
```rust
pub fn strip_copier_header(data: &[u8]) -> &[u8];     // si data.len() % 1024 == 512: &data[512..], sinon data
pub fn normalize(data: &[u8]) -> Vec<u8>;             // precondition data.len() >= 1
```
`normalize`: `big` = plus grande puissance de 2 <= len. Si `len == big`: copie telle quelle. Sinon `rest = &data[big..]`, `small` = plus petite puissance de 2 >= `rest.len()`; construire `part` = `rest` completee de zeros jusqu'a `small` octets; resultat = `data[..big]` suivi de `part` repetee `big / small` fois (taille finale `2 * big`).""",
  tests=[("strip_512", "`data` de 0x8000+512 octets avec data[i]=(i%251) as u8: `strip_copier_header(&data)` a 0x8000 octets et son premier octet == data[512]. Pour 0x8000 octets: renvoie le meme slice (meme longueur)."),
         ("strip_edge", "une entree de exactement 512 octets: renvoie un slice vide ; une entree de 1000 octets: inchangee."),
         ("pow2_noop", "`normalize` d'un tableau de 0x8000 octets == ce tableau ; de 0x10000 aussi."),
         ("pow2_96k", "len 0x18000 (data[i]=(i%251) as u8): resultat de longueur 0x20000 ; `r[..0x18000]==data` ; `r[0x18000..0x20000]==data[0x10000..0x18000]`."),
         ("pow2_80k", "len 0x14000: resultat 0x20000 ; `r[0x10000..0x14000]==data[0x10000..]` ; `r[0x14000..0x18000]`, `r[0x18000..0x1C000]`, `r[0x1C000..0x20000]` sont tous egaux a `data[0x10000..0x14000]`."),
         ("pow2_non_pow_rest", "len 0x16000 (rest=0x6000, small=0x8000): resultat 0x20000 ; `r[0x10000..0x16000]==data[0x10000..]` ; `r[0x16000..0x18000]` tous a 0 ; `r[0x18000..0x1E000]==data[0x10000..0x16000]` ; `r[0x1E000..0x20000]` tous a 0."),
         ("pow2_3mib", "len 3*1024*1024 (data[i]=(i%251) as u8): resultat 4 Mio ; `r[3 Mio..4 Mio]==data[2 Mio..3 Mio]`.")])

T("E03-05", "Cart: from_bytes / from_bytes_forced et accesseurs",
  size=130,
  context="Assemble les briques E03-01..04 dans la structure `Cart` (sans lecture/ecriture encore). Utilise `common::make_rom_with` (E01-08) dans les tests. " + CHAIN,
  embed=["04_CART_HEADER.md#5. Interface Rust (cart/mod.rs)"],
  write=[SRC + "/cart/mod.rs", TST + "/e03_05_cart_load.rs"],
  spec="""Ajoute a `cart/mod.rs`:
```rust
#[derive(Clone)]
pub struct Cart { rom: Vec<u8>, rom_mask: usize, map: MapMode, sram: Vec<u8>, has_battery: bool, header: Header, region: Region }
impl Cart {
    pub fn from_bytes(data: &[u8]) -> Result<Cart, CartError>;
    pub fn from_bytes_forced(data: &[u8], map: Option<MapMode>, region: Option<Region>) -> Result<Cart, CartError>;
    pub fn header(&self) -> &Header;  pub fn map_mode(&self) -> MapMode;  pub fn region(&self) -> Region;
    pub fn has_battery(&self) -> bool;  pub fn sram(&self) -> &[u8];  pub fn rom_len(&self) -> usize;
}
```
`from_bytes` = `from_bytes_forced(data, None, None)`. Etapes de `from_bytes_forced`:
1. `d = strip_copier_header(data)`; si `d.len() < 0x8000` -> `Err(TooSmall)`.
2. `rom = normalize(d)`; `rom_mask = rom.len() - 1`.
3. `map = forced.unwrap_or(pick_header(&rom)?)`.
4. `header = parse_header(&rom, header_pos(map)).ok_or(NoHeader)?`.
5. `(battery, coproc) = chipset_info(header.chipset)`; si `coproc = Some(n)` -> `Err(UnsupportedCoprocessor(n))`.
6. `sram = vec![0; sram_bytes(header.ram_log2)]` ; `has_battery = battery` ; `region = forced.unwrap_or(region_from_country(header.country))`.
Les methodes `read`/`write`/`load_sram` viennent a la tache suivante.""",
  tests=[("lorom", "`Cart::from_bytes(&make_rom(&[0xA9,0x01], Map::LoRom))`: Ok ; `map_mode()==LoRom` ; `region()==Ntsc` (pays 1) ; `rom_len()==32768` ; `has_battery()==false` ; `sram().len()==0` ; `header().title==\"PUCE16 TEST ROM\"`."),
         ("hirom", "meme chose avec HiRom: `map_mode()==HiRom`, `rom_len()==65536`."),
         ("exhirom", "avec ExHiRom (8 Mio): `map_mode()==ExHiRom`, `rom_len()==8*1024*1024`."),
         ("copier_header", "`[0u8;512] ++ make_rom(..LoRom)` donne Ok avec `map_mode()==LoRom` et `rom_len()==32768`."),
         ("too_small", "`Cart::from_bytes(&[0u8; 0x7FFF]).err()==Some(CartError::TooSmall)` ; une entree vide aussi."),
         ("coprocessor", "`make_rom_with(&[],Map::LoRom,15,0,0x13,1)` -> `err()==Some(UnsupportedCoprocessor(\"GSU\"))` ; chipset 0x34 -> `UnsupportedCoprocessor(\"SA-1\")`."),
         ("sram_battery", "`make_rom_with(&[],Map::LoRom,15,3,0x02,1)`: `has_battery()==true`, `sram().len()==8192`, tous les octets de sram a 0."),
         ("forced", "`from_bytes_forced(&make_rom(&[0x78],Map::LoRom), Some(LoRom), Some(Region::Pal))` -> `region()==Pal`. `from_bytes_forced(&lorom32k, Some(HiRom), None).err()==Some(NoHeader)` (32 Kio ne contient pas l'en-tete HiRom a 0xFFC0)."),
         ("pal_country", "`make_rom_with(&[],Map::LoRom,15,0,0,2)` -> `region()==Pal`.")])

T("E03-06", "Cart: read / write / load_sram",
  size=100,
  context="Acces CPU a la cartouche. La SRAM est testee AVANT la ROM (les banques $70-$7D du LoROM recouvrent aussi des miroirs ROM). " + CHAIN,
  embed=["04_CART_HEADER.md#4.4 Ecriture"],
  write=[SRC + "/cart/mod.rs", TST + "/e03_06_cart_rw.rs"],
  spec="""Ajoute a `impl Cart`:
```rust
pub fn read(&self, bank: u8, addr: u16) -> Option<u8>;      // None = open bus
pub fn write(&mut self, bank: u8, addr: u16, v: u8) -> bool;   // true si la SRAM a ete ecrite
pub fn load_sram(&mut self, data: &[u8]);                    // copie min(len) octets, le reste de la SRAM est mis a 0 ; la taille ne change pas
pub fn hash_into(&self, h: &mut crate::hash::Fnv64);         // ecrit sram (si non vide)
```
`read`: si `!sram.is_empty()` et `mapper::sram_offset(map,bank,addr,sram.len())` donne `Some(o)` -> `Some(sram[o])`. Sinon `mapper::rom_offset(map,bank,addr)` -> `Some(rom[o & rom_mask])`, ou `None`.
`write`: si `sram_offset` donne `Some(o)`: `sram[o] = v; true`. Sinon `false` (ecritures ROM ignorees).
`load_sram`: `for i in 0..sram.len() { sram[i] = data.get(i).copied().unwrap_or(0) }`.
`hash_into`: `h.write(&self.sram)` (rien d'autre: la ROM est immuable).""",
  tests=[("lorom_read", "ROM `make_rom(&[0xA9,0x01],LoRom)`: `read(0x00,0x8000)==Some(0xA9)` ; `read(0x00,0x8001)==Some(0x01)` ; `read(0x80,0x8000)==Some(0xA9)` ; `read(0x00,0xFFFC)==Some(0x00)` ; `read(0x00,0xFFFD)==Some(0x80)` ; `read(0x00,0x7FFF)==None` ; `read(0x7E,0x8000)==None`."),
         ("hirom_read", "ROM HiRom: `read(0xC0,0x8000)==Some(0xA9)` ; `read(0x00,0x8000)==Some(0xA9)` ; `read(0x80,0x8000)==Some(0xA9)` ; `read(0xC0,0xFFFD)==Some(0x80)` ; `read(0x40,0xFFFD)==Some(0x80)` ; `read(0x00,0x7FFF)==None`."),
         ("exhirom_read", "ROM ExHiRom: `read(0x00,0x8000)==Some(0xA9)` ; `read(0x00,0xFFFD)==Some(0x80)` ; `read(0x40,0xFFFD)==Some(0x80)` ; `read(0xC0,0x8000)==Some(0x00)` (premiere moitie, zeros)."),
         ("rom_write_ignored", "`write(0x00,0x8000,0x55)==false` ; `read(0x00,0x8000)` inchange."),
         ("lorom_sram", "ROM `make_rom_with(&[],LoRom,15,3,0x02,1)` (SRAM 8 Kio): `write(0x70,0x0000,0x5A)==true` ; `read(0x70,0x0000)==Some(0x5A)` ; `read(0x70,0x2000)==Some(0x5A)` (miroir modulo 0x2000) ; `read(0xF0,0x0000)==Some(0x5A)` ; `write(0x00,0x8000,1)==false`. `sram()[0]==0x5A`."),
         ("hirom_sram", "ROM `make_rom_with(&[],HiRom,16,3,0x02,1)`: `write(0x20,0x6000,0x77)==true` ; `read(0x20,0x6000)==Some(0x77)` ; `read(0xA0,0x6000)==Some(0x77)` ; `read(0x30,0x6000)==Some(0x77)` (bank&0x1F=0x10 -> 0x20000 % 0x2000 = 0) ; `read(0x20,0x7FFF)==Some(0x00)`."),
         ("load_sram", "SRAM 8 Kio: `load_sram(&[1,2,3])` -> sram()[0..4]==[1,2,3,0], len 8192 ; apres `load_sram(&vec![9;10000])`: len 8192 et sram()[8191]==9 ; apres `load_sram(&[])` tout a 0."),
         ("hash_sram", "deux Cart identiques ont le meme hash (`Fnv64`) ; apres `write(0x70,0,1)` le hash differe.")])
