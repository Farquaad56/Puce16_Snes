from _dsl import T

CORE = "crates/puce16-core"
SRC = CORE + "/src"
TST = CORE + "/tests"

CHAIN = ("Execution strictement sequentielle (D-020): cette tache suppose que toutes les precedentes sont terminees. "
         "Ne modifie QUE les fichiers de 'Write only in'.")

HDR = ("Chaque fichier de test commence par `mod common; #[path = \"common/prog.rs\"] mod prog;` (et `#[path = \"common/fake_dma_bus.rs\"]` n'est pas utile ici), "
       "puis `use puce16_core::snes::Snes; use prog::build_rom;`. Helper de test: `fn snes(rom: Vec<u8>) -> Snes { Snes::new(&rom, None).ok().unwrap() }`. "
       "Les programmes sont du code machine 65816 assemble A LA MAIN (octets donnes); l'adresse CPU de l'octet i du programme est 0x8000 + i. "
       "`settle(&mut s)` = `for _ in 0..12 { s.step_instruction(); }` (laisse le temps au CPU de prendre une interruption apres `run_frame`).")

# ------------------------------------------------------------------ E03 part E: integration tests on synthetic ROMs (E03-36..43)

T("E03-36", "tests/common/prog.rs et test de demarrage (boot)",
  size=100, deps=["E03-35"],
  context="""Premier test d'integration de bout en bout: une ROM synthetique minuscule demarre, le CPU execute du vrai code qui ecrit en WRAM via le bus.
Cette tache cree aussi l'assembleur-jouet `build_rom` utilise par toutes les taches d'integration suivantes. """ + CHAIN,
  embed=["06_TEST_ROMS.md#7. Tests internes"],
  write=[TST + "/common/prog.rs", TST + "/e03_36_boot.rs"],
  spec="""1. `tests/common/prog.rs` (declare par `#[path = "common/prog.rs"] mod prog;` apres `mod common;` ; utilise `crate::common::{make_rom_with, set_vector, Map}`):
```rust
/// blocks: (offset relatif a $00:8000, octets). vectors: (nom, adresse) pour set_vector. LoRom 32 Kio.
pub fn build_rom(blocks: &[(usize, &[u8])], vectors: &[(&str, u16)]) -> Vec<u8>;
pub fn build_rom_with(blocks: &[(usize, &[u8])], vectors: &[(&str, u16)], sram_log2: u8, chipset: u8) -> Vec<u8>;   // pays = 1
```
Le tampon de code est de longueur max(offset+len) (zeros ailleurs), passe a `make_rom_with(&code, Map::LoRom, 15, sram_log2, chipset, 1)`; puis pour chaque vecteur `set_vector(&mut rom, Map::LoRom, nom, addr)`. `build_rom` = `build_rom_with(.., 0, 0x00)`.
2. Programme BOOT (a mettre dans le test), `PROG_BOOT`:
`78 18 FB  A9 42  8D 10 00  A9 99  8F 20 00 7E  80 FE` = SEI ; CLC ; XCE ; LDA #$42 ; STA $0010 ; LDA #$99 ; STA $7E0020 ; BRA -2 (la boucle `BRA` est a l'offset 0x0E, soit pc 0x800E).""" + " " + HDR,
  tests=[("build_rom_layout", "`build_rom(&[(0,&[1,2]),(0x10,&[3])],&[])`: len==32768, rom[0..2]==[1,2], rom[0x10]==3, rom[2]==0, rom[0x0F]==0. Avec `&[(\"nmi\",0x8123)]`: rom[0x7FEA..0x7FEC]==[0x23,0x81] ; avec `&[(\"reset\",0x8010)]`: rom[0x7FFC..0x7FFE]==[0x10,0x80]."),
         ("build_rom_checksum", "`rom = build_rom(&[(0,&[1,2,3])], &[(\"nmi\",0x8100)])` : copie `rom`, ecrase les octets 0x7FDC..0x7FE0 par [0xFF,0xFF,0x00,0x00], somme u16 (wrapping) de tous les octets == mot LE lu a 0x7FDE de l'original ; et mot LE a 0x7FDC == !checksum."),
         ("boot_writes_wram", "`snes(build_rom(&[(0, PROG_BOOT)], &[]))` ; `run_frame()` : `peek_wram(0x10)==0x42`, `peek_wram(0x20)==0x99`, `debug().pc==0x800E`, `debug().pb==0`, `debug().e==false`."),
         ("reset_vector_respected", "`build_rom(&[(0x10, PROG_BOOT)], &[(\"reset\",0x8010)])` : juste apres `Snes::new`, `debug().pc==0x8010` ; apres `run_frame()`: `peek_wram(0x10)==0x42` et `debug().pc==0x801E`."),
         ("wram_initial_untouched", "avant tout `step_instruction`, `peek_wram(0x10)==0x55` et `peek_wram(0x20)==0x55` ; apres `run_frame()` ils valent 0x42 / 0x99 ; `peek_wram(0x11)` reste 0x55."),
         ("starts_in_emulation_mode", "juste apres `new`, `debug().e==true` ; apres 3 `step_instruction()` : `debug().e==false`.")])

T("E03-37", "Integration: NMI (compteur par frame, RDNMI, activation en VBlank)",
  size=120,
  context="Verifie la chaine complete flag RDNMI -> front NMI -> interruption CPU -> handler -> RTI. Les compteurs sont en WRAM. " + CHAIN,
  embed=["02_BUS_TIMING.md#6.2 NMI", "09_SYSTEM.md#8. Algorithme de `Timing::on_dot`"],
  write=[TST + "/e03_37_nmi.rs"],
  spec="""Programmes (offsets relatifs a 0x8000). Vecteur NMI -> 0x8100 pour les programmes A, B et G; handler `HANDLER_INC0100` a l'offset 0x100: `EE 00 01  40` = INC $0100 ; RTI.
- `PROG_A` (NMI active): `78 18 FB  9C 00 01  A9 80  8D 00 42  80 FE` = SEI ; CLC ; XCE ; STZ $0100 ; LDA #$80 ; STA $4200 ; BRA -2.
- `PROG_B` (NMI inactive): `78 18 FB  9C 00 01  80 FE`.
- `PROG_C` (comptage par RDNMI, sans NMI): `78 18 FB  9C 02 01  AD 10 42  10 FB  EE 02 01  80 F6` = ... STZ $0102 ; L: LDA $4210 ; BPL L ; INC $0102 ; BRA L. (L = offset 0x06.)
- `PROG_G` (active NMI en plein VBlank): `78 18 FB  9C 00 01  AD 12 42  10 FB  A9 80  8D 00 42  80 FE` = ... STZ $0100 ; L: LDA $4212 ; BPL L ; LDA #$80 ; STA $4200 ; BRA -2.""" + " " + HDR,
  tests=[("nmi_counts_frames", "PROG_A + handler + vecteur nmi: pour k = 1..=5: `run_frame(); settle();` puis `peek_wram(0x100)==k` (un NMI par frame)."),
         ("nmi_not_before_vblank", "PROG_A: `run_master(100_000)` (avant le VBlank, ~306 900 clocks): `peek_wram(0x100)==0` et `frame_count()==0`."),
         ("nmi_disabled", "PROG_B (meme handler/vecteur): 3 frames avec settle : `peek_wram(0x100)==0` (jamais de NMI)."),
         ("rdnmi_polling", "PROG_C (sans handler): pour k = 1..=4: `run_frame(); settle();` puis `peek_wram(0x102)==k` (le bit 7 de $4210 est vu une seule fois par frame, la lecture l'efface)."),
         ("enable_in_vblank_fires", "PROG_G + handler + vecteur nmi: apres le 1er `run_frame(); settle();` : `peek_wram(0x100)==1` (activer NMITIMEN.7 alors que RDNMI est deja a 1 declenche un NMI) ; apres le 2eme : 2."),
         ("nmi_handler_returns", "PROG_A: apres `run_frame(); settle();` le CPU est revenu dans la boucle principale: `debug().pc` vaut l'offset de la boucle `BRA` + 0x8000 (0x800B pour PROG_A) et `debug().e==false`."),
         ("nmi_native_vs_emulation", "`PROG_A` modifie pour NE PAS passer en mode natif (sans `CLC ; XCE`: `78  9C 00 01  A9 80  8D 00 42  80 FE`) : le NMI utilise le vecteur d'EMULATION ($FFFA), qui est aussi 0x8100 grace a `set_vector` : le compteur vaut 1 apres la 1re frame (verifie `debug().e==true` apres le handler).")])

T("E03-38", "Integration: IRQ H/V (modes 1, 2, 3, masque I, acquittement)",
  size=130,
  context="Verifie IRQ timer -> niveau IRQ -> interruption CPU (masquable par I) -> acquittement par lecture de $4211. " + CHAIN,
  embed=["02_BUS_TIMING.md#6.3 IRQ timer (H/V)"],
  write=[TST + "/e03_38_irq.rs"],
  spec="""Vecteur IRQ -> 0x8100 (`set_vector(\"irq\", 0x8100)`). Handler `HANDLER_IRQ` a l'offset 0x100: `AD 11 42  EE 03 01  40` = LDA $4211 (acquitte) ; INC $0103 ; RTI.
- `PROG_H` (mode 1, HTIME=100): `78 18 FB  9C 03 01  A9 64  8D 07 42  9C 08 42  A9 10  8D 00 42  58  80 FE`.
- `PROG_V` (mode 2, VTIME=100): `78 18 FB  9C 03 01  A9 64  8D 09 42  9C 0A 42  A9 20  8D 00 42  58  80 FE`.
- `PROG_HV` (mode 3, HTIME=50, VTIME=100): `78 18 FB  9C 03 01  A9 32  8D 07 42  9C 08 42  A9 64  8D 09 42  9C 0A 42  A9 30  8D 00 42  58  80 FE`.
- `PROG_V_MASKED` = `PROG_V` avec `58` (CLI) remplace par `EA` (NOP): l'IRQ reste masque.""" + " " + HDR,
  tests=[("irq_h_every_line", "PROG_H + handler + vecteur irq: un `run_frame()` (jusqu'au VBlank, ligne 225): `peek_wram(0x103)` est entre 224 et 225 inclus (un IRQ par ligne 0..=224)."),
         ("irq_v_once_per_frame", "PROG_V: pour k = 1..=3: `run_frame()` puis `peek_wram(0x103)==k` (IRQ a la ligne 100, bien avant le VBlank)."),
         ("irq_hv_once_per_frame", "PROG_HV: pour k = 1..=3: `run_frame()` puis `peek_wram(0x103)==k`."),
         ("irq_masked_by_i", "PROG_V_MASKED: 2 frames: `peek_wram(0x103)==0` (le niveau IRQ reste a 1 mais I=1)."),
         ("irq_not_before_line_100", "PROG_V: `run_master(1364*99)` (sans atteindre la ligne 100): `peek_wram(0x103)==0` ; puis `run_master(1364*3)`: `peek_wram(0x103)==1`."),
         ("irq_position", "PROG_V: au retour de `run_master(1364*100 + 400)` l'IRQ de la ligne 100 a deja eu lieu: `peek_wram(0x103)==1` ; `debug().v==100`."),
         ("irq_ack_prevents_retrigger", "PROG_V: apres la 1re frame, `peek_wram(0x103)==1` exactement (pas de re-declenchement sur la meme ligne tant que le handler acquitte)."),
         ("irq_without_ack_loops", "handler SANS acquittement (`EE 03 01  40`, sans le `AD 11 42`) avec PROG_V : le niveau IRQ reste vrai, le CPU reprend l'IRQ des le retour du handler : apres `run_master(1364*103)`, `peek_wram(0x103) > 10` (boucle d'IRQ). (Preuve que l'IRQ est un NIVEAU.)")])

T("E03-39", "Integration: DMA general ROM -> WRAM par $2180 et relecture",
  size=110,
  context="Le DMA demarre apres l'instruction suivant l'ecriture de $420B, copie la ROM vers la WRAM via le port $2180, puis le CPU reprend. " + CHAIN,
  embed=["03_DMA.md#3. DMA general (MDMAEN $420B)", "03_DMA.md#3.1 Timing DMA (cycle-accurate, valeurs par defaut)"],
  write=[TST + "/e03_39_dma.rs"],
  spec="""Donnees ROM: bloc `(0x200, &[0x11,0x22,0x33,0x44,0x55,0x66,0x77,0x88])` (adresse CPU 0x8200). Programme `PROG_DMA` (offset 0):
```
78 18 FB                 SEI CLC XCE
A9 00 8D 81 21           WMADDL = 0
A9 02 8D 82 21           WMADDM = 2     (WMADD = 0x200)
9C 83 21                 STZ $2183
9C 00 43                 DMAP0 = 0      (A->B, pattern 0, A++)
A9 80 8D 01 43           BBAD0 = $80    ($2180)
9C 02 43                 A1T0L = 0
A9 82 8D 03 43           A1T0H = $82    (A = 00:8200)
9C 04 43                 A1B0 = 0
A9 08 8D 05 43           DAS0L = 8
9C 06 43                 DAS0H = 0
A9 01 8D 0B 42           MDMAEN = 1
EA                       NOP            (le DMA part apres le fetch de cette instruction)
A9 7B 8D 10 01           marqueur: $0110 = $7B
A9 00 8D 81 21           WMADDL = 0
A9 02 8D 82 21           WMADDM = 2
AD 80 21 8D 11 01        $0111 = lecture $2180
AD 80 21 8D 12 01        $0112 = lecture $2180
80 FE                    BRA -2
```""" + " " + HDR,
  tests=[("data_copied", "apres `run_frame()`: `peek_wram(0x200+i)==[0x11,0x22,0x33,0x44,0x55,0x66,0x77,0x88][i]` pour i in 0..8 ; `peek_wram(0x208)==0x55` (intact)."),
         ("cpu_resumes", "apres `run_frame()`: `peek_wram(0x110)==0x7B` (le CPU a repris apres le DMA)."),
         ("wmdata_readback", "apres `run_frame()`: `peek_wram(0x111)==0x11` et `peek_wram(0x112)==0x22` (relecture par $2180 avec WMADD ramene a 0x200)."),
         ("dma_takes_time", "deux `Snes`: l'un avec PROG_DMA, l'autre avec PROG_DMA dont l'octet de valeur de `A9 01 8D 0B 42` est remplace par 0x00 (offset 0x2C : MDMAEN = 0, meme duree d'instruction). Dans chacun, executer exactement 22 `step_instruction()` (jusqu'au NOP inclus). `master_dma - master_nodma` est compris entre 80 et 130 inclus (8 depart + 8 canal + 8*8 octets = 80, + alignement 0..7, + eventuellement un rafraichissement DRAM de 40 clocks decale par le DMA)."),
         ("dma_data_not_before_trigger", "`step_instruction()` pas a pas depuis `new` jusqu'a ce que `debug().pc == 0x8030` (STA $420B vient d'etre executee ; le NOP est la prochaine instruction): `peek_wram(0x200)==0x55` (rien copie) ; apres UN `step_instruction()` de plus (le NOP), `peek_wram(0x200)==0x11` et `peek_wram(0x207)==0x88`."),
         ("deterministic", "deux `Snes` identiques: meme `state_hash` apres `run_frame()` x2."),
         ("mdmaen_zero_no_dma", "variante avec `A9 00 8D 0B 42` (MDMAEN=0): `peek_wram(0x200)==0x55` apres une frame.")])

T("E03-40", "Integration: HDMA vers $2180 (table directe, repetition)",
  size=120,
  context="HDMA ecrit une valeur par ligne dans le port WRAM $2180. L'init se fait a (0,3) de chaque frame si HDMAEN != 0; le programme l'active pendant la frame 0 apres ce point, donc le 1er HDMA effectif est en frame 1. " + CHAIN,
  embed=["03_DMA.md#4. HDMA (HDMAEN $420C)", "09_SYSTEM.md#5. HDMA: declenchement"],
  write=[TST + "/e03_40_hdma.rs"],
  spec="""Table 1 (non repetee) a l'offset 0x300 (adresse 0x8300): `01 A1  01 A2  03 A3  00`. Table 2 (repetee) a l'offset 0x400: `83 B1 B2 B3  00`.
Programme `PROG_HDMA` (offset 0; canal 1; `T_HI` = 0x83 pour la table 1, 0x84 pour la table 2; `EN` = 0x02):
```
78 18 FB
A9 00 8D 81 21     WMADDL = 0
A9 03 8D 82 21     WMADDM = 3         (WMADD = 0x300)
9C 83 21           STZ $2183
9C 10 43           DMAP1 = 0
A9 80 8D 11 43     BBAD1 = $80
9C 12 43           A1T1L = 0
A9 T_HI 8D 13 43   A1T1H = T_HI
9C 14 43           A1B1 = 0
A9 EN 8D 0C 42     HDMAEN = EN
80 FE              BRA -2
```""" + " " + HDR,
  tests=[("frame0_no_hdma", "table 1: apres le 1er `run_frame()`: `peek_wram(0x300..0x306)` valent tous 0x55 (l'init de la frame 0, a (0,3), est passee avant l'activation)."),
         ("frame1_three_bytes", "table 1: apres le 2eme `run_frame()`: `peek_wram(0x300)==0xA1`, `0x301==0xA2`, `0x302==0xA3`, `0x303==0x55`."),
         ("frame2_repeats", "table 1: apres le 3eme `run_frame()`: `peek_wram(0x300..0x306)==[0xA1,0xA2,0xA3,0xA1,0xA2,0xA3]` et `0x306==0x55` (WMADD continue d'avancer, la table est reinitialisee a chaque frame)."),
         ("repeat_mode_table", "table 2 (A1T1H=0x84): apres le 2eme `run_frame()`: `peek_wram(0x300..0x303)==[0xB1,0xB2,0xB3]` et `0x303==0x55`."),
         ("hdma_disabled", "meme programme avec `EN = 0x00`: apres 3 frames `peek_wram(0x300)==0x55`."),
         ("other_channel_bit", "`EN = 0x01` (canal 0 au lieu de 1; le canal 0 n'est pas configure: BBAD0/A1T0 a 0xFF au power-on): aucune ecriture dans 0x300 (ne doit pas planter): apres 3 frames `peek_wram(0x300)==0x55`."),
         ("deterministic", "deux `Snes` identiques : meme `state_hash` apres 3 frames.")])

T("E03-41", "Integration: auto-read des manettes ($4218-$421F)",
  size=100,
  context="Le CPU lit JOY1/JOY2 apres l'auto-read du VBlank. Les bits viennent de `set_joypad`. " + CHAIN,
  embed=["02_BUS_TIMING.md#6.4 Auto-read joypad", "02_BUS_TIMING.md#6.5 JOYOUT / JOYSER0 / JOYSER1 ($4016/$4017)"],
  write=[TST + "/e03_41_autoread.rs"],
  spec="""`PROG_JOY` (auto-read active, copie JOY1L, JOY1H, JOY2H en WRAM $0120-$0122 en boucle):
```
78 18 FB
A9 01 8D 00 42     NMITIMEN = 1 (auto-read, pas de NMI)
AD 18 42 8D 20 01  L: $0120 = JOY1L          (L = offset 0x08)
AD 19 42 8D 21 01  $0121 = JOY1H
AD 1B 42 8D 22 01  $0122 = JOY2H
80 EC              BRA L
```
`PROG_JOY_OFF` = `PROG_JOY` avec les 5 octets `A9 01 8D 00 42` remplaces par `EA EA EA EA EA`. Boutons: `BTN_A=0x0080`, `BTN_START=0x1000`, `BTN_B=0x8000` (constantes de `puce16_core::joypad`).""" + " " + HDR,
  tests=[("zero_before_vblank", "`set_joypad(0, BTN_A|BTN_START)` apres `new` ; `run_master(100_000)` : `peek_wram(0x120)==0`, `0x121==0`, `0x122==0` (l'auto-read n'a pas encore eu lieu ; le programme a deja copie des zeros)."),
         ("values_after_autoread", "`set_joypad(0, BTN_A|BTN_START)`, `set_joypad(1, BTN_B)` ; `run_frame()` puis `run_master(20_000)` : `peek_wram(0x120)==0x80` (octet bas de 0x1080), `0x121==0x10`, `0x122==0x80` (JOY2H = octet haut de 0x8000)."),
         ("old_value_during_autoread", "meme config ; `run_frame()` (retour juste apres le debut du VBlank, avant la fin des 4224 clocks) puis `run_master(1000)` : `peek_wram(0x120)==0` (JOY1L garde l'ancienne valeur pendant la lecture)."),
         ("next_frame_updates", "apres le test precedent complet: `set_joypad(0, 0x0001)` ; `run_frame()` (frame suivante) ; `run_master(20_000)` : `peek_wram(0x120)==0x01` et `0x121==0x00`. Avant ce `run_frame()` (juste apres `set_joypad`) `peek_wram(0x120)` valait encore 0x80."),
         ("autoread_off", "`PROG_JOY_OFF`, `set_joypad(0,0xFFF0)` : 2 frames + `run_master(20_000)` : `peek_wram(0x120)==0`."),
         ("opposite_dirs_not_filtered_by_core", "`set_joypad(0, BTN_LEFT|BTN_RIGHT)` (le filtrage est cote desktop) : apres auto-read `peek_wram(0x121)==0x03` (LEFT=0x0200, RIGHT=0x0100 -> octet haut 0x03).")])

T("E03-42", "Integration: unite de multiplication/division et determinisme",
  size=110,
  context="Lecture des resultats $4214-$4217 par le CPU apres le delai materiel (>= 48 / 96 clocks), et lecture trop precoce. " + CHAIN,
  embed=["02_BUS_TIMING.md#6.1 Multiplication / division (unite math du 5A22)"],
  write=[TST + "/e03_42_math.rs"],
  spec="""`PROG_MATH` (10 NOP = 10 octets `EA` aux deux endroits marques):
```
78 18 FB
A9 07 8D 02 42     WRMPYA = 7
A9 09 8D 03 42     WRMPYB = 9         (lance 7*9)
EA x10
AD 16 42 8D 30 01  $0130 = RDMPYL
AD 17 42 8D 31 01  $0131 = RDMPYH
A9 34 8D 04 42     WRDIVL = $34
A9 12 8D 05 42     WRDIVH = $12       (dividende $1234)
A9 07 8D 06 42     WRDIVB = 7         (lance $1234 / 7)
EA x10
AD 14 42 8D 32 01  $0132 = RDDIVL
AD 15 42 8D 33 01  $0133 = RDDIVH
AD 16 42 8D 34 01  $0134 = RDMPYL (reste)
80 FE
```
`PROG_MATH_EARLY` = lecture immediate: `78 18 FB  A9 07 8D 02 42  A9 09 8D 03 42  AD 16 42 8D 30 01  80 FE` (la lecture de $4216 se termine 30 clocks apres l'ecriture de $4203, < 48).""" + " " + HDR,
  tests=[("multiplication", "`run_frame()` sur PROG_MATH : `peek_wram(0x130)==0x3F` et `peek_wram(0x131)==0x00` (7*9 = 63)."),
         ("division", "`peek_wram(0x132)==0x99`, `peek_wram(0x133)==0x02` (quotient $0299 = 665), `peek_wram(0x134)==0x05` (reste 5)."),
         ("early_read_returns_old", "PROG_MATH_EARLY: `run_frame()` : `peek_wram(0x130)==0x00` (ancienne valeur de RDMPYL au power-on)."),
         ("determinism_same_rom", "deux `Snes` sur PROG_MATH: meme `state_hash` apres 2 frames."),
         ("determinism_input_changes_hash", "deux `Snes` identiques, l'un avec `set_joypad(0, 0x8000)` : hash differents apres 1 frame."),
         ("determinism_chunked_runs", "deux `Snes` identiques: `run_master(50_000)` x3 de chaque cote puis `run_frame()` : meme hash ; la partition identique est requise (le test ne compare PAS des partitions differentes)."),
         ("frames_do_not_change_results", "apres 3 `run_frame()`, `peek_wram(0x130..0x135)` valent toujours `[0x3F,0x00,0x99,0x02,0x05]` : le programme boucle sur BRA, il ne re-execute pas le calcul, les valeurs sont stables.")])

T("E03-43", "Integration: SRAM, open bus et valeurs de power-on des registres",
  size=100,
  context="Quelques lectures/ecritures de bout en bout qui exercent le routage: SRAM LoROM, open bus (derniere valeur du bus), registres DMA et WRAM a leur valeur de power-on. " + CHAIN,
  embed=["02_BUS_TIMING.md#4. Open bus", "04_CART_HEADER.md#4.1 LoROM (banques de 32 Kio)"],
  write=[TST + "/e03_43_sram_openbus.rs"],
  spec="""ROM avec SRAM de 8 Kio: `build_rom_with(blocks, vectors, 3, 0x02)`. Programme `PROG_BUS` (offset 0):
```
78 18 FB
AF 00 00 70        LDA $700000       (SRAM)
8D 45 01           STA $0145
A9 5A              LDA #$5A
8F 00 00 70        STA $700000
AD 00 50           LDA $5000         (open bus)
8D 40 01           STA $0140
AD 00 43           LDA $4300         (DMAP0 power-on)
8D 41 01           STA $0141
AD FF 1F           LDA $1FFF         (WRAM power-on)
8D 42 01           STA $0142
AD 10 42           LDA $4210         (RDNMI, flag faux)
8D 43 01           STA $0143
AD 13 42           LDA $4213         (RDIO power-on)
8D 44 01           STA $0144
80 FE
```""" + " " + HDR,
  tests=[("sram_written", "`run_frame()` : `sram().unwrap()[0]==0x5A`."),
         ("sram_preloaded", "`load_sram(&[0x77])` AVANT `run_frame()` : `peek_wram(0x145)==0x77` (lu par LDA long) puis `sram().unwrap()[0]==0x5A` (reecrit)."),
         ("open_bus_value", "`peek_wram(0x140)==0x50` (LDA $5000 renvoie la derniere valeur vue sur le bus = l'octet haut de l'operande)."),
         ("dma_power_on", "`peek_wram(0x141)==0xFF` ($4300 vaut 0xFF au power-on)."),
         ("wram_power_on", "`peek_wram(0x142)==0x55`."),
         ("rdnmi_partial_open_bus", "`peek_wram(0x143)==0x42` ((0x42 & 0x70) | 0x02, flag faux)."),
         ("rdio_power_on", "`peek_wram(0x144)==0xFF`."),
         ("no_sram_rom", "meme programme avec `build_rom` (sans SRAM, chipset 0): `sram()==None` ; `peek_wram(0x145)` vaut l'open bus: l'ecriture SRAM est ignoree; `LDA $700000` lit le MIROIR ROM de LoROM (banque 0x70, adresse < 0x8000 -> offset 0 de la ROM = premier octet du programme, SEI = 0x78): `peek_wram(0x145)==0x78`.")])
