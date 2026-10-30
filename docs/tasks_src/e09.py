from _dsl import T

# E09: coprocesseurs. HORS v1 (D-005). E09-00 est un correctif; E09-01..E09-12 = S-DD1 (specifie, D-023); les autres puces sont des PLACEHOLDERS: aucune specification n'existe encore.
# Elles ne sont jamais assignees a Hermes. Quand l'utilisateur decide de lancer l'un des coprocesseurs, Claude ecrit
# d'abord la specification (docs/specs/10_COPRO_<nom>.md) et decoupe la tache en sous-etapes de 50 a 150 lignes.

DEFER = ("HORS v1 (D-005). Placeholder: aucune specification n'existe. Ne pas assigner a Hermes. "
         "Quand l'utilisateur lance ce coprocesseur, Claude ecrit d'abord `docs/specs/10_COPRO_<nom>.md` "
         "(registres, timing, tables, jeux de test), la decoupe en taches de 50 a 150 lignes, puis marque cette tache comme remplacee.")

def D(id, title, what, order):
    T(id, title, kind="doc", deferred=True, size=0,
      context="%s Priorite %s. Perimetre: %s" % (DEFER, order, what),
      spec="Aucune action pour la v1. Quand le coprocesseur est lance: (1) specification par Claude, (2) acceptation d'en-tete `CartInfo.coprocessor` dans `cart/header.rs` (remplacer le refus `UnsupportedCoprocessor`), (3) crate ou module dedie `crates/puce16-core/src/coprocessor/<nom>.rs`, (4) integration au bus (mappage des plages $2200-$3FFF ou cartouche selon la puce), (5) tests unitaires sur les routines de la puce puis ROM de jeu connue.",
      write=[], tests=[], accept=[])

CHAIN9 = ("Execution strictement sequentielle (D-020): cette tache suppose que toutes les precedentes sont terminees. Ne modifie QUE les fichiers de 'Write only in'.")

T("E09-00", "TUNE: sprites decales d'une ligne (SMW: debris colores sur la ligne 0 en haut de l'image) - reporte apres la v1",
  kind="code", size=60, crate="puce16-core",
  context="""Bug trouve au GATE E07-14 (SMW, ecran de niveau et carte), accepte comme defaut connu et reporte ici a la demande de l'utilisateur (le gate E07-14 peut etre approuve sans ce correctif): de petits debris colores (vert, rouge, orange, blanc) apparaissent sur la toute premiere ligne de l'image (ligne de sortie 0, x environ 90 a 135). Diagnostic fait par Claude (non a deviner): SMW cache ses sprites inutilises en mettant Y=240 (OAM $F0, taille petite = 16x16 avec OBSEL de SMW). `crates/puce16-core/src/ppu/obj.rs` calcule la rangee du sprite avec `row = (r - y - 1) & 0xFF` (dans `sprite_list` ET dans `render_obj_line`, spec 05b section 6.2 ligne `row = (r - y - 1) & 0xFF`, avec `r` = ligne de sortie 0..223). Avec cette formule un sprite 16x16 a Y=240 couvre r=241..256, donc la rangee 15 revient se dessiner sur r=0: c'est l'artefact. Sur la console un sprite a Y=0 commence sur la premiere ligne visible (l'evaluation des sprites de la ligne v se fait pendant la ligne v-1, donc rangee = `(r - y) & 0xFF`, le « +1 » de la doc materielle est deja absorbe par `r = v - 1`). Experience faite en local puis ANNULEE: avec `row = (r - y) & 0xFF` dans les deux fonctions, la ligne 0 de SMW est propre (seule la colline de fond reste), mais ~30 tests e05_ changent car ils ont ete ecrits avec l'ancienne formule: `e05_10_row_visibility_y255`, `e05_10_row_visibility_y0`, `e05_10_large_height`, et la plupart des tests de `e05_11_sprite_pixels.rs` (ils utilisent y=255 comme « premiere rangee visible »), ainsi que le helper de `e05_14_pick.rs` (ligne 414).
Cette tache est donc une CORRECTION DE SPEC approuvee par la gate: la spec 05b et les tests e05_ qui l'encodent doivent etre alignes sur le comportement materiel. Ce n'est pas un affaiblissement de test: chaque test garde son intention (visibilite, hauteur, flips, limites 32/34), seul le Y utilise est decale d'une unite.""",
  reads=["specs/05b_PPU_RENDER.md (section 6, lignes 69-110 et 240)", "crates/puce16-core/src/ppu/obj.rs", "crates/puce16-core/tests/e05_10_sprite_list.rs", "crates/puce16-core/tests/e05_11_sprite_pixels.rs", "crates/puce16-core/tests/e05_14_pick.rs",
         "docs/qa/DECISIONS.md et docs/qa/FAQ_SNES.md (a consulter avant toute question)"],
  write=["crates/puce16-core/src/ppu/obj.rs", "specs/05b_PPU_RENDER.md",
         "crates/puce16-core/tests/e05_10_sprite_list.rs", "crates/puce16-core/tests/e05_11_sprite_pixels.rs", "crates/puce16-core/tests/e05_14_pick.rs",
         "crates/puce16-core/tests/e09_00_*.rs", "tests/baseline.json", "docs/qa/SUITE_REPORT.md", "tests/out/**", "docs/qa/inbox/** (questions uniquement)"],
  spec="""1. Reproduire (l'option `--input` du CLI est deja disponible): `cargo run --release -q -p puce16-cli -- run tests/roms/games/smw.sfc --frames 1500 --input "700-705:start,800-805:a,900-905:a,1000-1005:a,1100-1105:a,1200-1205:a,1300-1305:a" --dump-ppm tests/out/e09_00_smw.ppm`. Avant correction, la ligne 0 de l'image contient des pixels non fond (x 90..135) en plus de la colline bleue (x >= 151). L'exemple `cargo run --release -p puce16-core --example smw_oam -- tests/roms/games/smw.sfc 1500` liste les sprites a Y=240 responsables.
2. Corriger `obj.rs`: `row = ((r as i32) - (s.y as i32)) & 0xFF` dans `sprite_list` ET dans `render_obj_line` (les deux doivent rester identiques; en faire une petite fonction commune `sprite_row(r, y)` pour qu'elles ne divergent plus).
3. Mettre a jour la spec 05b: section 6.2 (formule et commentaire: `Y = 0` -> premiere rangee visible, `Y = 255` -> la rangee 1 du sprite sur la ligne 0, un sprite 16x16 a `Y = 240` n'atteint PAS la ligne 0, un sprite de hauteur 32 ou 64 deborde bien par le haut), et la ligne 240 (liste des tests: remplacer `Y=255 visible a la rangee 0; Y=0 non visible a la rangee 0` par la nouvelle regle). Ajouter une ligne d'historique dans la spec indiquant que la formule a ete corrigee par E09-00 et pourquoi (SMW cache ses sprites a Y=240).
4. Mettre a jour les tests e05_10, e05_11 et e05_14: decaler le Y des sprites de test d'une unite (255 -> 0 pour « premiere rangee visible », etc.) et ajuster les attentes de visibilite. Chaque test garde sa signification d'origine; ne supprimer aucune assertion. Si un test devient impossible a adapter sans en changer le sens, question bloquante (`docs/qa/inbox`).
5. Test de non-regression `e09_00_*` (nouveau fichier) avec valeurs concretes, qui ECHOUE avant la correction: sprite 16x16 a Y=240 invisible sur la ligne 0 (et sur toutes les lignes 0..223); sprite 8x8 a Y=0 visible sur les lignes 0..7 avec rangee = ligne; sprite 32x32 a Y=240 visible sur les lignes 0..15 (rangees 16..31).
6. Relancer la suite: `target/release/puce16-cli suite tests/manifest.toml`. Les ROM avec sprites changeront de hash (decalage d'une ligne): regarder les images `tests/out/suite/*.ppm` des ROM de sprites avant de refiger, puis `--update-baseline <nom>` pour chacune. Noter dans `docs/qa/SUITE_REPORT.md` la liste des ROM re-figees. Si une ROM de test sprite (ex: celles de E05) devient fausse visuellement, STOP et question bloquante: la formule est alors a reconsiderer.
7. Fin de tache: la ligne 0 de la capture SMW ne contient plus de debris (voir `accept`). L'utilisateur reverifie SMW dans le desktop apres la correction.""",
  tests=[("sprite_16x16_y240_hidden_on_all_rows", "16x16 a Y=240: aucune ligne de sortie 0..223 ne contient le sprite (regression SMW)"),
         ("sprite_y0_first_visible_row", "8x8 a Y=0: visible sur les lignes 0..7, ligne 0 = rangee 0"),
         ("sprite_32x32_y240_wraps_to_top", "32x32 a Y=240: visible sur les lignes 0..15 avec les rangees 16..31 (debordement materiel)"),
         ("sprite_list_and_render_use_same_row", "sprite_list et render_obj_line donnent la meme rangee pour tous les Y de 0 a 255")],
  accept=["cargo build --release -p puce16-cli",
          "cargo test -p puce16-core e09_00_",
          "cargo test -p puce16-core e05_",
          "cargo run --release -q -p puce16-cli -- run tests/roms/games/smw.sfc --frames 1500 --input 700-705:start,800-805:a,900-905:a,1000-1005:a,1100-1105:a,1200-1205:a,1300-1305:a --dump-ppm tests/out/e09_00_smw.ppm",
          "python -c \"import sys; d=open('tests/out/e09_00_smw.ppm','rb').read(); o=len(d)-256*239*3; r=d[o:o+150*3]; sys.exit(0 if all(r[i:i+3]==bytes([255,231,181]) for i in range(0,len(r),3)) else 1)\""],
  notes=CHAIN9)


# ---------------------------------------------------------------------------------------------------
# S-DD1 (D-023): premier coprocesseur. Spec: docs/specs/10_COPRO_SDD1.md. Donnees: docs/specs/data/sdd1_*.tsv.
# Modele de reference: docs/tools/sdd1_ref.py (Python). Verification des tables: docs/tools/check_sdd1.py.
# ---------------------------------------------------------------------------------------------------

CORE9 = "crates/puce16-core"
SDD = CORE9 + "/src/coprocessor/sdd1"
TST9 = CORE9 + "/tests"
SPEC10 = "specs/10_COPRO_SDD1.md"

LCG_NOTE = ("Generateur de flux de test (identique a `docs/tools/sdd1_ref.py`): `fn lcg_bytes(seed: u32, n: usize) -> Vec<u8> { let mut x = seed; (0..n).map(|_| { x = x.wrapping_mul(1664525).wrapping_add(1013904223); (x >> 24) as u8 }).collect() }`. "
            "Fermeture de ROM des tests: `let rom = |a: u32| data[(a as usize) % data.len()];` (lecture cyclique).")

T("E09-01", "S-DD1: module coprocessor/sdd1 et tables.rs (RUNCOUNT, EVOLUTION)",
  kind="code", size=90, crate="puce16-core",
  context="Premiere tache du S-DD1 (decompresseur de Star Ocean et Street Fighter Alpha 2). Elle cree l'arborescence du module et les deux tables de l'algorithme, copiees depuis `docs/specs/data/sdd1_tables.tsv` (donnees de reference, jamais modifiees a la main). " + CHAIN9,
  embed=[SPEC10 + "#1. Sources et fiabilite", SPEC10 + "#7. Interface Rust"],
  reads=["docs/specs/data/sdd1_tables.tsv (section RUNCOUNT: 256 lignes `index<TAB>valeur`; section EVOLUTION: 33 lignes `etat<TAB>code_number<TAB>next_if_mps<TAB>next_if_lps`)",
         "docs/tools/check_sdd1.py (verifie tables.rs contre le TSV)"],
  write=[CORE9 + "/src/lib.rs", CORE9 + "/src/coprocessor/mod.rs", SDD + "/mod.rs", SDD + "/tables.rs", TST9 + "/e09_01_*.rs", "docs/qa/inbox/** (questions uniquement)"],
  spec="""1. `lib.rs`: ajouter `pub mod coprocessor;` (apres `pub mod cart;`).
2. `coprocessor/mod.rs`: `pub mod sdd1;`. `coprocessor/sdd1/mod.rs`: `pub mod tables;` seulement (chaque tache suivante ajoute sa propre ligne `pub mod ...;`).
3. `tables.rs`: `pub const RUNCOUNT: [u8; 256] = [ ... ];` (16 valeurs par ligne, notation `0x00`) et `pub const EVOLUTION: [(u8, u8, u8); 33] = [ ... ];` avec les valeurs du TSV, dans l'ordre, SANS rien recalculer ni retaper de memoire: lire le TSV par un petit script (Python ou shell) et generer le texte. Chaque ligne d'EVOLUTION est `(code_number, next_if_mps, next_if_lps)`. En-tete du fichier: commentaire `//! S-DD1 tables (Andreas Naive, public domain). Source: docs/specs/data/sdd1_tables.tsv`.
4. Ne pas ecrire d'autre code. `puce16-core` reste `#![forbid(unsafe_code)]`.
5. Verifier avec `python docs/tools/check_sdd1.py --rust crates/puce16-core/src/coprocessor/sdd1/tables.rs` (doit afficher `CHECK SDD1 OK`).""",
  tests=[("runcount_spot_values", "RUNCOUNT[2]=1, [3]=0, [4]=3, [7]=0, [8]=7, [0x80]=0x7f, [0xFA]=0x50, [0xFF]=0"),
         ("runcount_block_range", "pour chaque n de 1 a 7 et idx dans [2^n, 2^(n+1)): RUNCOUNT[idx] < 2^n (et RUNCOUNT[0] = RUNCOUNT[1] = 0)"),
         ("evolution_spot_values", "EVOLUTION[0] = (0,25,25), [1] = (0,2,1), [24] = (7,24,23), [25] = (0,26,1), [32] = (7,24,22)"),
         ("evolution_next_states_in_range", "code_number <= 7 et next_if_mps, next_if_lps < 33 pour les 33 etats")],
  accept=["cargo test -p puce16-core e09_01_",
          "python docs/tools/check_sdd1.py --rust crates/puce16-core/src/coprocessor/sdd1/tables.rs"],
  notes=CHAIN9)

T("E09-02", "S-DD1: Decompressor (struct, init) et etages IM + GCD",
  kind="code", size=120, crate="puce16-core",
  context="Premier etage du decompresseur: la structure `Decompressor` (tous les champs de l'algorithme), `init`, le gestionnaire d'entree (IM) et le decodeur de Golomb (GCD). Les etages suivants (E09-03, E09-04) ajoutent leurs methodes dans d'autres fichiers du meme module, donc les champs sont `pub(super)`. " + CHAIN9,
  embed=[SPEC10 + "#6. Algorithme de decompression", SPEC10 + "#7. Interface Rust"],
  reads=[SDD + "/tables.rs"],
  write=[SDD + "/mod.rs", SDD + "/decomp.rs", TST9 + "/e09_02_*.rs", "docs/qa/inbox/** (questions uniquement)"],
  spec="""1. `decomp.rs`: `#[derive(Clone, Debug)] pub struct Decompressor` avec les champs (tous `pub(super)`): `im_offset: u32`, `im_bit_count: u8`, `mps_count: [u8; 8]`, `lps_index: [bool; 8]`, `status: [u8; 32]`, `mps: [u8; 32]`, `bitplanes_info: u8`, `context_info: u8`, `bit_number: u8`, `current_bitplane: u8`, `prev: [u16; 8]`, `r0: u8`, `r1: u8`, `r2: u8`. `impl Default` = `new()` (tout a zero).
2. `init<F: Fn(u32) -> u8>(&mut self, start: u32, rom: &F)`: remet TOUT a l'etat initial de la spec section 6 (IM: `offset = start`, `bit_count = 4`; BG: zeros; PEM: tous les `status` et `mps` a 0; CM: `bitplanes_info = h & 0xC0`, `context_info = h & 0x30` avec `h = rom(start)`, `bit_number = 0`, `prev` a zero, `current_bitplane` selon la table (0x00 -> 1, 0x40 -> 7, 0x80 -> 3, 0xC0 -> 0); OL: `r0 = 1`, `r1 = r2 = 0`).
3. `code_word<F>(&mut self, code_length: u8, rom: &F) -> u8` (IM) et `run_count<F>(&mut self, code_number: u8, rom: &F) -> (u8, bool)` (GCD, utilise `tables::RUNCOUNT`), strictement comme la spec: attention aux troncatures u8 (`(rom(off) << bit_count) as u8` doit se calculer en u32/u16 puis tronquer; `>> (9 - bit_count)` avec `bit_count` deja incremente; `RUNCOUNT[(cw >> (code_number ^ 7)) as usize]`).
4. `pub fn im_position(&self) -> (u32, u8)` renvoie `(im_offset, im_bit_count)`.
5. Ajouter `pub mod decomp;` a `sdd1/mod.rs`. Les tests construisent la ROM avec une fermeture (voir ci-dessous).
""" + LCG_NOTE,
  tests=[("im_code_word_sequence", "data=[0xB6,0x4D,0x00,0xFF,0x5A,0xC3,0x11,0x22], init(0); code_word avec les longueurs [3,3,0,1,2,7,4,5,0,6] renvoie dans l'ordre les triplets (octet, offset, bit_count apres appel): (96,0,5) (201,1,1) (154,1,2) (52,1,3) (104,1,4) (208,2,4) (0,2,5) (0,2,6) (0,2,7) (0,3,0)"),
         ("init_reads_header_at_start", "fermeture `|a| if a == 0xC00100 { 0xC0 } else { 0 }`; init(0xC00100): im_position() == (0xC00100, 4)"),
         ("run_count_lps_cases", "flux [0x0F,0xA5,0x5A,0x3C] suivi de 60 zeros; pour n = 0..7 (une instance neuve + init(0) par n) run_count(n) = [(0,true),(0,true),(0,true),(0,true),(0,true),(16,true),(16,true),(80,true)]; avec le flux [0x08,0x40,0x00,0x00] + 60 zeros: [(0,true),(1,true),(3,true),(7,true),(15,true),(15,true),(47,true),(111,true)]"),
         ("run_count_mps_cases", "flux de 64 zeros: pour n = 0..7, run_count(n) = (1 << n, false) et im_position() == (0, 5)")],
  accept=["cargo test -p puce16-core e09_02_"],
  notes=CHAIN9)

T("E09-03", "S-DD1: etages BG (generateurs de bits) et PEM (estimation de probabilite)",
  kind="code", size=100, crate="puce16-core",
  context="Deuxieme etage: les 8 generateurs de bits et le module d'estimation de probabilite, en methodes de `Decompressor` (nouveau fichier du meme module). " + CHAIN9,
  embed=[SPEC10 + "#6. Algorithme de decompression"],
  reads=[SDD + "/decomp.rs", SDD + "/tables.rs"],
  write=[SDD + "/mod.rs", SDD + "/decomp_bits.rs", TST9 + "/e09_03_*.rs", "docs/qa/inbox/** (questions uniquement)"],
  spec="""1. `decomp_bits.rs`: `impl Decompressor { ... }` (acces aux champs `pub(super)`) avec `pub fn bg_bit<F: Fn(u32) -> u8>(&mut self, n: usize, rom: &F) -> (u8, bool)` (BG), `pub fn pem_bit<F: Fn(u32) -> u8>(&mut self, ctx: usize, rom: &F) -> u8` (PEM, utilise `tables::EVOLUTION` et `bg_bit(code_number as usize)`) et `pub fn pem_state(&self, ctx: usize) -> (u8, u8)` = `(status[ctx], mps[ctx])`.
2. Copier la logique de la spec section 6 (BG puis PEM) sans la simplifier. Le test `if st & 0xFE == 0` (basculement du MPS) porte sur l'etat AVANT mise a jour.
3. Ajouter `pub mod decomp_bits;` a `sdd1/mod.rs`.
4. Les flux de test: `data = lcg_bytes(seed, 64)` puis `data[0] = 0x0F` (le quartet bas 0xF force un premier bit a 1, donc des cas LPS). `init(0, &rom)` une fois par test.
""" + LCG_NOTE,
  tests=[("bg_bit_n1_sequence", "seed 0x4444 (octets 0f6ac8908c9d2554...), n = 1, 16 appels de bg_bit(1): (1,t) (1,t) (0,f) (0,t) (1,t) (0,f) (0,t) (0,f) (1,t) (0,f) (1,t) (1,t) (0,f) (0,t) (0,f) (0,t) (t = true, f = false)"),
         ("bg_bit_n0_sequence", "meme flux, n = 0, 16 appels: (1,t) (1,t) (1,t) (1,t) (0,t) (1,t) (1,t) (0,t) (1,t) (0,t) (1,t) (0,t) (1,t) (1,t) (0,t) (0,t)"),
         ("pem_bit_ctx5_sequence", "seed 0x7777 (octets 0f4540bdebdcbf58...), 40 appels pem_bit(5): 1001101110101011111111111001000110000001, puis pem_state(5) == (1, 0)"),
         ("pem_bit_ctx17_sequence", "seed 0x8888 (octets 0f8ebe76b69cf304...), 40 appels pem_bit(17): 1001000011101011010001111000101110001001, puis pem_state(17) == (2, 0)")],
  accept=["cargo test -p puce16-core e09_03_"],
  notes=CHAIN9)

T("E09-04", "S-DD1: etages CM (contexte) et OL (sortie) + vecteurs d'or",
  kind="code", size=130, crate="puce16-core",
  context="Dernier etage de l'algorithme: modele de contexte (CM) et logique de sortie (OL). A la fin, `Decompressor::read` produit les octets decompresses; les 16 vecteurs d'or de `docs/specs/data/sdd1_vectors.tsv` (generes par le modele de reference Python) doivent passer. " + CHAIN9,
  embed=[SPEC10 + "#6. Algorithme de decompression", SPEC10 + "#9. Plan de test"],
  reads=[SDD + "/decomp.rs", SDD + "/decomp_bits.rs", "docs/specs/data/sdd1_vectors.tsv (colonnes: header, seed, input_hex_first32, output_hex_first64, fnv64_first256, fnv64_first8192)"],
  write=[SDD + "/mod.rs", SDD + "/decomp_ctx.rs", TST9 + "/e09_04_*.rs", "docs/qa/inbox/** (questions uniquement)"],
  spec="""1. `decomp_ctx.rs`: `impl Decompressor { pub fn cm_bit<F: Fn(u32) -> u8>(&mut self, rom: &F) -> u8; pub fn read<F: Fn(u32) -> u8>(&mut self, rom: &F) -> u8 }` strictement comme la spec section 6 (CM puis OL). `current_bitplane`, `r0`, `r1`, `r2`, `bit_number` sont des u8: les decalages gauche de `r0` (`r0 <<= 1` a partir de 0x80 donne 0) se font avec `wrapping_shl`/masque; `prev[...]` est un u16 (`(cb << 1) | bit` tronque a 16 bits); `!r0` sur u8.
2. Ajouter `pub mod decomp_ctx;` a `sdd1/mod.rs`.
3. Test des vecteurs d'or: lire `docs/specs/data/sdd1_vectors.tsv` avec `std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/specs/data/sdd1_vectors.tsv"))`. Pour chaque ligne: `header` (hex, quartet haut), `seed` (decimal): `data = lcg_bytes(seed, 4096)` avec `data[0] = (data[0] & 0x0F) | header`; verifier `hex(data[..32]) == input_hex_first32` (controle du generateur), puis `init(0)` et 8192 appels de `read` avec la ROM cyclique: les 64 premiers octets en hex == `output_hex_first64`, FNV-64 des 256 premiers == `fnv64_first256`, FNV-64 des 8192 == `fnv64_first8192` (hex sur 16 chiffres, FNV-1a: `h = 0xcbf29ce484222325; pour chaque octet: h ^= b; h = h.wrapping_mul(0x100000001b3)`).
4. Si un vecteur echoue, NE PAS modifier le fichier TSV ni le test: chercher l'erreur dans le code (le modele Python `docs/tools/sdd1_ref.py` est lisible et fait foi pour cette tache); si le doute persiste, question `docs/qa/inbox`.
""" + LCG_NOTE,
  tests=[("cm_bit_first16_headers", "seed 0x6666, data = lcg_bytes(0x6666, 64) avec data[0] = (data[0] & 0x0F) | h; init(0); 16 appels de cm_bit donnent (bits en ordre): h=0x00 -> 0101101001001110; h=0x30 -> 0101111010101101; h=0x40 -> 0101101001001110; h=0x80 -> 0101101001001110; h=0xC0 -> 0100101110101010"),
         ("read_golden_vectors_all_headers", "les 16 lignes du TSV (64 premiers octets, FNV-64 des 256 et des 8192 premiers)"),
         ("reinit_restarts_stream", "apres 100 appels de read, init(0) de nouveau: les 32 octets suivants egalent les 32 premiers d'une instance neuve"),
         ("mode7_header_reads_eight_bit_values", "header 0xC0, meme flux (seed 0x6666): une instance A appelle read() une fois; une instance B appelle cm_bit() 8 fois et assemble l'octet bit 0 en premier (LSB first): les deux octets sont egaux, et ce pour 16 octets successifs")],
  accept=["cargo test -p puce16-core e09_04_"],
  notes=CHAIN9)

T("E09-05", "S-DD1: registres $4800-$4807 et fonctions de mappage (Sdd1)",
  kind="code", size=120, crate="puce16-core",
  context="Etat de la puce (registres, banques MMC) et fonctions pures de mappage de la ROM; pas encore de DMA ni de lien avec `Cart`. " + CHAIN9,
  embed=[SPEC10 + "#3. Registres", SPEC10 + "#4. Mappage de la ROM et de la SRAM"],
  reads=[SDD + "/mod.rs"],
  write=[SDD + "/mod.rs", SDD + "/regs.rs", TST9 + "/e09_05_*.rs", "docs/qa/inbox/** (questions uniquement)"],
  spec="""1. `regs.rs`: `#[derive(Clone, Debug)] pub struct Sdd1 { pub(super) r4800: u8, pub(super) r4801: u8, pub(super) mmc: [u8; 4], pub(super) dma_addr: [u32; 8], pub(super) dma_size: [u16; 8], pub(super) dma_ready: bool, pub(super) dec: Decompressor }`; `impl Sdd1 { pub fn new() -> Sdd1 }` met les valeurs de reset de la section 3 (`mmc = [0, 1, 2, 3]`).
2. `pub fn io_read(&self, reg: u16) -> Option<u8>`: `$4800`, `$4801`, `$4804..$4807` renvoient la valeur; tout autre registre de `$4800..$480F` renvoie `None` (bus ouvert, voir UNKNOWN).
3. `pub fn io_write(&mut self, reg: u16, v: u8)`: `$4800` et `$4801` = `v`; `$4804..$4807` = `v & 0x8F` (les autres registres sont ignores).
4. `pub fn mmc_offset(&self, addr: u32) -> usize` (fenetre C0-FF, section 4: `block = ((addr >> 20) & 3)`, `((mmc[block] & 0x0F) << 20) | (addr & 0xFFFFF)`) et `pub fn lorom_offset(&self, addr: u32) -> usize` (fenetre 00-3F/80-BF: regles du bit 21 selon `mmc[1] & 0x80` pour les banques 20-3F et `mmc[3] & 0x80` pour A0-BF, puis `((a >> 1) & 0x1F8000) | (a & 0x7FFF)`). Les deux fonctions renvoient l'offset AVANT masquage par la taille de la ROM.
5. Ajouter `pub mod regs;` a `sdd1/mod.rs` et `pub use regs::Sdd1; pub use decomp::Decompressor;`.""",
  tests=[("reset_values", "apres new(): io_read($4800) = Some(0), $4801 = Some(0), $4804 = Some(0), $4805 = Some(1), $4806 = Some(2), $4807 = Some(3); $4802, $4803, $4808 et $480F = None"),
         ("bank_registers_mask_8f", "io_write($4804, 0xFF) puis io_read($4804) = Some(0x8F); io_write($4800, 0xFF) et ($4801, 0xA5): lectures 0xFF et 0xA5"),
         ("mmc_offset_defaults", "avec les valeurs de reset: 0xC00000 -> 0x000000, 0xD01234 -> 0x101234, 0xE0ABCD -> 0x20ABCD, 0xFFFFFF -> 0x3FFFFF; apres io_write($4804, 0x03): 0xC00010 -> 0x300010; apres io_write($4804, 0x8F): 0xC00000 -> 0xF00000 (le bit 7 est ignore par la fenetre MMC)"),
         ("lorom_offset_values", "reset: 0x008000 -> 0x000000, 0x018004 -> 0x008004, 0x3F8000 -> 0x1F8000, 0x3FFFFF -> 0x1FFFFF, 0x808000 -> 0x000000, 0xBFFFFF -> 0x1FFFFF, 0x208000 -> 0x100000, 0xA08000 -> 0x100000"),
         ("lorom_offset_bit21_cleared", "apres io_write($4805, 0x80) et io_write($4807, 0x83): 0x208000 -> 0x000000, 0xA08000 -> 0x000000, 0x3F8000 -> 0x0F8000, 0xBFFFFF -> 0x0FFFFF, 0x018004 -> 0x008004 (banques < 20 inchangees)")],
  accept=["cargo test -p puce16-core e09_05_"],
  notes=CHAIN9)

T("E09-06", "S-DD1: espionnage DMA et lecture decompressee en flux",
  kind="code", size=110, crate="puce16-core",
  context="Logique d'interception du DMA (section 5): recopie des registres `$43x2-$43x6`, selection du canal actif, decompression en flux octet par octet, decompte de la taille et effacement de `$4801`. Toujours pur: la ROM est fournie en tranche. " + CHAIN9,
  embed=[SPEC10 + "#5. Interception DMA"],
  reads=[SDD + "/regs.rs", SDD + "/decomp_ctx.rs"],
  write=[SDD + "/mod.rs", SDD + "/dma.rs", TST9 + "/e09_06_*.rs", "docs/qa/inbox/** (questions uniquement)"],
  spec="""1. `dma.rs`: `impl Sdd1 { ... }` (champs `pub(super)`):
   - `pub fn dma_write(&mut self, reg: u16, v: u8)`: `reg` dans `0x4300..=0x437F`; canal `n = ((reg >> 4) & 7) as usize`; selon `reg & 0xF`: 2 -> bits 0-7 de `dma_addr[n]`, 3 -> bits 8-15, 4 -> bits 16-23, 5 -> bits 0-7 de `dma_size[n]`, 6 -> bits 8-15; les autres sont ignores.
   - `pub fn dma_channel(&self, n: usize) -> (u32, u16)` renvoie `(dma_addr[n], dma_size[n])`.
   - `pub fn mmc_read(&self, addr: u32, rom: &[u8]) -> u8` = `rom[self.mmc_offset(addr) & (rom.len() - 1)]` (la longueur de `rom` est une puissance de 2).
   - `pub fn mcu_read(&mut self, addr: u32, rom: &[u8]) -> u8`: algorithme de la section 5 (bloc `si (r4800 & r4801) != 0`). La fermeture de ROM du decompresseur est `|a: u32| rom[mmc_offset(a) & mask]` avec une COPIE de `mmc` (pour ne pas emprunter `self` deux fois): par exemple `let mmc = self.mmc; let f = |a: u32| rom[((((mmc[((a >> 20) & 3) as usize] & 0x0F) as usize) << 20) | (a as usize & 0xFFFFF)) & mask];`.
2. Le decompte utilise `wrapping_sub(1)` sur `dma_size[n]`; quand il atteint 0: `dma_ready = false` et `r4801 &= !(1 << n)`.
3. Ajouter `pub mod dma;` a `sdd1/mod.rs`.
4. Construction des donnees de test: ROM = `vec![0u8; 1 << 20]`; flux compresse = `lcg_bytes(0x1000 + k, 4096)` avec `data[0] = (data[0] & 0x0F) | (k << 4)`, copie en `rom[0x100..0x100 + 4096]`; les adresses `0xC00100` pointent dessus (mmc par defaut).
""" + LCG_NOTE,
  tests=[("stream_matches_golden_header_00", "k = 0: io_write($4800,1), io_write($4801,1), dma_write($4302,0x00), ($4303,0x01), ($4304,0xC0), ($4305,4), ($4306,0): 4 mcu_read(0xC00100) renvoient 0x75 0x12 0x24 0x91; ensuite io_read($4801) = Some(0) (canal 0 termine)"),
         ("after_finish_reads_plain_rom", "apres le test precedent, mcu_read(0xC00100) renvoie rom[0x100] (octet d'en-tete compresse, ici 0x02)"),
         ("stream_matches_golden_header_30", "k = 3 (en-tete 0x30), canal 2: dma_write($4322.. $4326) taille 8; 8 mcu_read(0xC00100) renvoient 45 56 11 a4 ff bb 37 ed; $4800 = $4801 = 0x04 au depart, io_read($4801) = Some(0) a la fin"),
         ("inactive_cases_return_rom", "r4800 = 0 (mais r4801 = 1), ou adresse differente de dma_addr (ex. 0xC00101), ou canal non active: mcu_read renvoie la ROM normale et ne touche pas dma_size"),
         ("size_zero_wraps", "dma_size = 0: apres 1 lecture, dma_channel(0).1 == 0xFFFF et $4801 inchange"),
         ("dma_write_assembles_fields", "ecritures de 0x12, 0x34, 0xC5 en $4332,$4333,$4334 et 0x56, 0x78 en $4335,$4336: dma_channel(3) == (0xC53412, 0x7856)")],
  accept=["cargo test -p puce16-core e09_06_"],
  notes=CHAIN9)

T("E09-07", "S-DD1: acceptation de l'en-tete et integration dans Cart (ROM, SRAM)",
  kind="code", size=140, crate="puce16-core",
  context="Branche le S-DD1 sur la cartouche: le chipset `$43`/`$45` n'est plus refuse, `Cart` construit un `Sdd1`, le mappage de la ROM et de la SRAM devient celui de la spec (sections 2 et 4). Les autres coprocesseurs restent refuses (`UnsupportedCoprocessor`). " + CHAIN9,
  embed=[SPEC10 + "#2. Detection et perimetre", SPEC10 + "#4. Mappage de la ROM et de la SRAM"],
  reads=[CORE9 + "/src/cart/mod.rs", CORE9 + "/src/cart/header.rs", CORE9 + "/src/cart/mapper.rs", CORE9 + "/tests/common/mod.rs (make_rom_with)"],
  write=[CORE9 + "/src/cart/mod.rs", CORE9 + "/src/cart/header.rs", CORE9 + "/src/cart/mapper.rs", CORE9 + "/src/coprocessor/sdd1/*.rs", TST9 + "/e09_07_*.rs", "docs/qa/inbox/** (questions uniquement)"],
  spec="""1. `header.rs`: `pub fn has_sdd1(chipset: u8) -> bool { chipset >> 4 == 4 && chipset & 0xF >= 3 }`; `chipset_info` ne renvoie plus de nom de coprocesseur pour ce cas (`has_sdd1`), la pile (`has_battery`) reste `low in {2,5,6}` (donc `$45` = pile, `$43` = pas de pile). Les autres chipsets (GSU, SA-1, DSP...) gardent leur refus.
2. `Cart`: nouveau champ `sdd1: Option<Box<Sdd1>>` (`Sdd1` doit etre `Clone`); `from_bytes_forced` le cree quand `has_sdd1`. Accesseurs: `pub fn has_sdd1(&self) -> bool`, `pub fn io_read(&self, reg: u16) -> Option<u8>`, `pub fn io_write(&mut self, reg: u16, v: u8)`, `pub fn dma_write(&mut self, reg: u16, v: u8)` (sans S-DD1: `None` / sans effet).
3. Mappage avec S-DD1 (a la place de `mapper::rom_offset`): banques 00-3F et 80-BF offsets $8000-$FFFF -> `sdd1.lorom_offset(addr)`; banques C0-FF -> `sdd1.mmc_offset(addr)`; les deux masquees par `rom_mask`; banques 40-7D -> `None`. SRAM: nouvelles fonctions dans `mapper.rs` (ou dans `cart/mod.rs`) selon la section 4: banques 00-3F/80-BF offsets $6000-$7FFF -> `(offset & 0x1FFF) % sram_len`; banques 70-73 offsets $0000-$FFFF -> `(((bank & 3) << 16) | offset) % sram_len`; `sram_len == 0` -> pas de SRAM.
4. `Cart::read(&self, bank, addr)` (sans effet de bord, utilise aussi par les outils de debug): sans decompression, la fenetre C0-FF renvoie la ROM normale via `mmc_offset`. NOUVELLE methode `pub fn read_bus(&mut self, bank: u8, addr: u16) -> Option<u8>` pour le bus: identique a `read`, sauf que les lectures C0-FF passent par `sdd1.mcu_read(addr24, &self.rom)`. Sans S-DD1, `read_bus` == `read`. `write` utilise le mappage SRAM du S-DD1 quand il est present.
5. Les ROM sans S-DD1 se comportent EXACTEMENT comme avant (aucun test existant ne doit changer).
6. Mettre a jour `sdd1/mod.rs` si necessaire (re-exports).""",
  tests=[("chipset_43_loads_without_battery", "make_rom_with(.., chipset 0x43): Cart::from_bytes OK, has_sdd1() = true, has_battery() = false"),
         ("chipset_45_has_battery_and_sram", "chipset 0x45, sram_log2 = 3 (8 Kio): OK, has_battery() = true, sram().len() = 8192"),
         ("other_coprocessors_still_refused", "chipset 0x13 -> Err(UnsupportedCoprocessor(\"GSU\")), 0x34 -> Err(UnsupportedCoprocessor(\"SA-1\")), 0x00 -> OK sans S-DD1 (has_sdd1() = false)"),
         ("lorom_window_reads_rom", "ROM 32 Kio avec rom[4] = 0xAB: read(0x00, 0x8004) = Some(0xAB), read(0x80, 0x8004) = Some(0xAB); read(0x40, 0x8004) = None; read(0x00, 0x2000) = None"),
         ("mmc_window_reads_rom", "meme ROM: read(0xC0, 0x0004) = Some(0xAB); apres io_write($4804, 0x01) la fenetre C0 pointe sur le bloc 1 (ROM 32 Kio: l'offset 0x100004 est masque en 0x0004, donc toujours 0xAB)"),
         ("sram_windows_mirror", "chipset 0x45 + 8 Kio: write(0x00, 0x6000, 0x5A) -> read(0x70, 0x0000) = Some(0x5A), read(0x80, 0x7FFF) apres write(0x00, 0x7FFF, 0x33) = Some(0x33); read(0x00, 0x8000) n'est pas la SRAM"),
         ("read_bus_without_sdd1_equals_read", "chipset 0x00: pour 20 couples (bank, addr) variees read_bus == read")],
  accept=["cargo test -p puce16-core e09_07_", "cargo test -p puce16-core e03_05_", "cargo test -p puce16-core e03_34_"],
  notes=CHAIN9)

T("E09-08", "S-DD1: routage du bus ($4800-$480F, espionnage $4300-$437F, lectures C0-FF)",
  kind="code", size=80, crate="puce16-core",
  context="Le bus appelle les nouvelles methodes de `Cart`: lecture/ecriture des registres $4800-$480F, espionnage des ecritures $4300-$437F et lectures de cartouche par `read_bus` (seul chemin avec effet de bord). " + CHAIN9,
  embed=[SPEC10 + "#5. Interception DMA"],
  reads=[CORE9 + "/src/bus/route.rs", CORE9 + "/src/bus/mod.rs"],
  write=[CORE9 + "/src/bus/route.rs", CORE9 + "/src/bus/mod.rs", TST9 + "/e09_08_*.rs", "docs/qa/inbox/** (questions uniquement)"],
  spec="""1. `route.rs`, `read_inner` (banques 00-3F/80-BF): ajouter `0x4800..=0x480F => self.cart.io_read(a).unwrap_or(self.mdr)` (avant `_ =>`), et remplacer les deux appels `self.cart.read(bank, a)` (zone `0x6000..=0xFFFF` ET zone cartouche banques 40-7D / C0-FF) par `self.cart.read_bus(bank, a)`. `read_inner` a deja `&mut self`.
2. `write_inner` (banques 00-3F/80-BF): `0x4300..=0x437F => { self.dma.write(a, v); self.cart.dma_write(a, v); }` et `0x4800..=0x480F => self.cart.io_write(a, v)`.
3. Ne rien changer d'autre: `dma_a_forbidden` reste tel quel (les donnees S-DD1 sont en banques C0+).
4. Les tests construisent un `SysBus::new(cart, Region::Ntsc)` avec `make_rom_with(.., chipset 0x43)` (ROM 32 Kio, le flux compresse place au decalage 0x100: `rom[0x100..]` = flux du vecteur `0x00`, graine `0x1000`, voir spec section 9) et appellent `read_inner`/`write_inner` (publics).""",
  tests=[("registers_visible_on_the_bus", "write_inner(0x004801, 0xA5) puis read_inner(0x004801) = 0xA5; avec le miroir de banque 0x804801 aussi; $4802 renvoie mdr"),
         ("dma_registers_still_work", "write_inner(0x004302, 0x34) puis read_inner(0x004302) = 0x34 (le DMA normal recoit toujours l'ecriture)"),
         ("decompressed_stream_on_c0_reads", "apres $4800 = $4801 = 1 et $4302..$4306 = 00 01 C0 04 00: 4 read_inner(0xC00100) renvoient 0x75 0x12 0x24 0x91; le 5e renvoie rom[0x100]"),
         ("no_sdd1_rom_unchanged", "chipset 0x00: $4801 est du bus ouvert (renvoie mdr); read_inner(0xC00100) renvoie rom[0x100] a chaque fois")],
  accept=["cargo test -p puce16-core e09_08_", "cargo test -p puce16-core e03_"],
  notes=CHAIN9)

T("E09-09", "S-DD1: sauvegarde d'etat et hash",
  kind="code", size=80, crate="puce16-core",
  context="Les registres du S-DD1 et les copies des registres DMA entrent dans `Cart::save` / `load` / `hash_into` quand la cartouche a un S-DD1 (spec section 8). Le format des cartouches sans S-DD1 ne change pas d'un octet. " + CHAIN9,
  embed=[SPEC10 + "#8. Etat, sauvegardes d'etat, hash"],
  reads=[CORE9 + "/src/cart/mod.rs", CORE9 + "/src/savestate.rs"],
  write=[CORE9 + "/src/cart/mod.rs", CORE9 + "/src/coprocessor/sdd1/*.rs", TST9 + "/e09_09_*.rs", "docs/qa/inbox/** (questions uniquement)"],
  spec="""1. `Sdd1`: ajouter `pub fn save(&self, w: &mut StateWriter)` / `pub fn load(&mut self, r: &mut StateReader) -> Result<(), StateError>` / `pub fn hash_into(&self, h: &mut Fnv64)` (dans un nouveau fichier ou `regs.rs`): ordre `r4800, r4801, mmc[0..4], pour n in 0..8: dma_addr[n] (u32), dma_size[n] (u16)`; au chargement `dma_ready = false` et le decompresseur est remis a `Decompressor::new()`.
2. `Cart::save/load/hash_into`: appeler les methodes de `Sdd1` APRES les donnees actuelles, uniquement si `sdd1.is_some()`.
3. Regarder comment `Cart::save` ecrit la SRAM et comment les tests `e08_*` utilisent `StateWriter` / `StateReader` pour copier le style.""",
  tests=[("roundtrip_registers", "cart S-DD1: io_write($4800,3), ($4801,2), ($4805,0x82), dma_write($4342,0x10), ($4343,0x20), ($4344,0xC0), ($4345,0x08), ($4346,0x00); save; nouvelle cartouche identique; load; io_read($4800/$4801/$4805) = Some(3)/Some(2)/Some(0x82); dma_channel(4) = (0xC02010, 8)"),
         ("non_sdd1_format_unchanged", "chipset 0x00: la taille du flux de Cart::save est la meme qu'une cartouche sans S-DD1 (comparer a la longueur de `sram().len()` + les champs existants, ou a une cartouche `make_rom` equivalente: egalite des longueurs et des octets)"),
         ("hash_changes_with_registers", "hash_into avant et apres io_write($4804, 0x05) differents; hash identique pour deux cartouches avec les memes ecritures"),
         ("load_resets_streaming", "apres load, dma_ready est faux: une decompression en cours est abandonnee (un nouveau mcu_read relance init)")],
  accept=["cargo test -p puce16-core e09_09_", "cargo test -p puce16-core e08_"],
  notes=CHAIN9)

T("E09-10", "S-DD1: test de bout en bout (ROM synthetique, DMA vers la WRAM)",
  kind="code", size=120, crate="puce16-core",
  context="Un petit programme 65816 active le S-DD1, programme un DMA a adresse fixe depuis $C0:0100 vers `$2180` (port de donnees WRAM) et on verifie la WRAM. Aucune modification du code du coeur: seulement un fichier de test. Si ce test echoue, le defaut est dans les taches E09-02..09 (ne PAS adapter le test). " + CHAIN9,
  embed=[SPEC10 + "#5. Interception DMA", SPEC10 + "#9. Plan de test"],
  reads=[CORE9 + "/tests/common/prog.rs (build_rom_with)", CORE9 + "/tests/e03_34_snes_new.rs (usage de Snes::new)"],
  write=[TST9 + "/e09_10_*.rs", "docs/qa/inbox/** (questions uniquement)"],
  spec="""Programme (a placer en tete du code, adresse CPU $00:8000 = decalage 0 de la ROM LoROM; `build_rom_with(&[(0, &PROG), (0x100, &stream)], &[("reset", 0x8000)], 0, 0x43)`):
```
78 18 FB                 SEI ; CLC ; XCE
A9 01  8D 00 48          LDA #$01 ; STA $4800        (activation hard canal 0)
       8D 01 48          STA $4801                   (activation soft canal 0)
A9 08  8D 00 43          LDA #$08 ; STA $4300        (DMAP: A->B, adresse A FIXE, mode 0)
A9 80  8D 01 43          LDA #$80 ; STA $4301        (B-bus $2180 = WRAM data)
A9 00  8D 02 43          LDA #$00 ; STA $4302        (adresse source $C0:0100)
A9 01  8D 03 43          LDA #$01 ; STA $4303
A9 C0  8D 04 43          LDA #$C0 ; STA $4304
A9 40  8D 05 43          LDA #$40 ; STA $4305        (taille $0040 = 64 octets)
A9 00  8D 06 43          LDA #$00 ; STA $4306
       8D 81 21          STA $2181                   (adresse WRAM = $7E:0000)
       8D 82 21          STA $2182
       8D 83 21          STA $2183
A9 01  8D 0B 42          LDA #$01 ; STA $420B        (MDMAEN: canal 0)
80 FE                    BRA -2
```
`stream` = `lcg_bytes(0x1000 + k, 4096)` avec `data[0] = (data[0] & 0x0F) | (k << 4)`. Le programme occupe les octets 0..0x50 de la ROM, le flux commence a 0x100: aucun recouvrement. Apres `Snes::new(&rom, None)` puis 2 appels de `run_frame()`, comparer `peek_wram(i)` pour i dans 0..64 avec les 64 octets attendus (colonne `output_hex_first64` de `docs/specs/data/sdd1_vectors.tsv`, ligne dont `header` = `k << 4`).
Controle sans S-DD1: meme programme avec chipset `0x00`: la DMA a adresse fixe copie 64 fois le meme octet, donc `peek_wram(i) == rom[0x100]` pour tout i.""" + "\n" + LCG_NOTE,
  tests=[("wram_gets_decompressed_header_00", "k = 0: WRAM[0..64] == 64 octets de la ligne header 00 du TSV (commencent par 75 12 24 91)"),
         ("wram_gets_decompressed_header_30", "k = 3: WRAM[0..64] == ligne header 30 (commencent par 45 56 11 a4)"),
         ("wram_gets_decompressed_mode7_c0", "k = 12 (en-tete 0xC0): WRAM[0..64] == ligne header c0 (commencent par 3a 66 c6 48)"),
         ("without_sdd1_same_byte_repeated", "chipset 0x00: tous les octets WRAM[0..64] egalent rom[0x100]")],
  accept=["cargo test -p puce16-core e09_10_"],
  notes=CHAIN9)

T("E09-11", "S-DD1: entrees optionnelles du manifeste pour les jeux (Street Fighter Alpha 2, Star Ocean)",
  kind="data", size=0,
  context="Deux jeux a S-DD1 fournis par l'utilisateur (jamais commites) rejoignent le manifeste comme les jeux de E07, pour le gate. " + CHAIN9,
  reads=["tests/manifest.toml (modele: entrees `game_smw`, `game_alttp`, `game_allstars`)"],
  write=["tests/manifest.toml", "tests/roms/README.md"],
  spec="""1. Ajouter a la fin de `tests/manifest.toml` deux entrees `[[rom]]` `optional = true`, `check = "none"`, `region = "auto"`, `epic = "E09"`, `frames = 600`: `game_sfa2` (`path = "tests/roms/games/sfa2.sfc"`) et `game_starocean` (`path = "tests/roms/games/starocean.sfc"`), memes champs que `game_smw`.
2. `tests/roms/README.md`: dans la section des jeux commerciaux, ajouter les deux noms de fichiers `sfa2.sfc` et `starocean.sfc` (a fournir par l'utilisateur, jamais commites).
3. Controle: le meme script Python que E07-02 (noms uniques, `epic` de la forme `E` + 2 chiffres, `path` existant sauf `optional`).""",
  accept=["python -c \"import tomllib,sys; d=tomllib.load(open('tests/manifest.toml','rb')); n=[x['name'] for x in d['rom']]; sys.exit(0 if 'game_sfa2' in n and 'game_starocean' in n and len(n)==len(set(n)) else 1)\""],
  notes=CHAIN9)

T("E09-12", "GATE S-DD1: Street Fighter Alpha 2 et Star Ocean valides par l'utilisateur",
  kind="gate", gate=True, size=0,
  context="Fin du S-DD1. Les vecteurs d'or viennent d'une transcription Python de l'algorithme (pas du materiel): seuls les vrais jeux prouvent que tout est juste. Aucun code a ecrire.",
  spec="""Criteres de validation (humains):
1. `cargo test --workspace` est vert.
2. Placer `sfa2.sfc` et `starocean.sfc` dans `tests/roms/games/` (jamais commites).
3. Street Fighter Alpha 2: le logo Capcom et l'ecran titre s'affichent sans corruption (les graphismes sont decompresses par le S-DD1; une decompression fausse donne des tuiles en bruit); un combat se lance et les sprites des combattants sont corrects.
4. Star Ocean: l'ecran titre s'affiche, une nouvelle partie demarre, les decors et les combats sont propres; la sauvegarde SRAM fonctionne (sauvegarder en jeu, quitter, relancer, charger).
5. Save state / load state au milieu d'un combat ou d'une carte: image et son identiques apres chargement.
En cas de corruption graphique: noter la ROM, l'ecran et le numero de frame; Claude compare avec le modele `docs/tools/sdd1_ref.py` (aucune supposition sur le materiel).""",
  user_cmds=["cargo test --workspace",
             "cargo run --release -p puce16-cli -- run tests/roms/games/sfa2.sfc --frames 1800 --dump-ppm tests/out/sfa2.ppm",
             "cargo run --release -p puce16-cli -- run tests/roms/games/starocean.sfc --frames 1800 --dump-ppm tests/out/starocean.ppm",
             "cargo run --release -p puce16-desktop",
             "python docs/tools/task.py approve E09-12"],
  accept=[])

# ------------------------------------------------------------------ autres puces: placeholders (ordre de priorite)

D("E09-13", "Coprocesseur DSP-1 (Super Mario Kart, Pilotwings)",
  "calculs mathematiques 16 bits (rotation, projection, perspective), ROM de donnees interne 2 Kio + 1 Kio, commandes via un registre de donnees/statut a deux adresses (LoROM: $30-$3F:$8000-$FFFF ou $6000/$7000 selon la carte). Variantes DSP-1/1A/1B, DSP-2/3/4 plus tard. A specifier juste apres le S-DD1 (D-023).", "2")
D("E09-14", "Coprocesseur Super FX / GSU (Star Fox, Yoshi's Island)",
  "CPU RISC 16 bits propre (GSU-1/GSU-2), 16 registres, cache d'instructions 512 octets, trace de pixels dans la RAM de cartouche, double acces ROM/RAM avec le S-CPU (RON/RAN), interruption IRQ vers le S-CPU.", "3")
D("E09-15", "Coprocesseur SA-1 (Super Mario RPG, Kirby Super Star)",
  "second 65C816 a 10,74 MHz avec BW-RAM, I-RAM, DMA caracteres, multiplication/division materielles, tampon de bits variable, timers, arbitrage du bus avec le S-CPU, interruptions croisees.", "4")
D("E09-16", "Decompresseur SPC7110 (Far East of Eden Zero, Momotaro Dentetsu Happy)",
  "decompression, ROM de donnees, horloge temps reel EPSON RTC-4513 optionnelle, unite de multiplication/division, mappage de banques.", "5")
D("E09-17", "Coprocesseur Cx4 (Mega Man X2, X3)",
  "processeur Hitachi HG51B: fonctions graphiques et trigonometriques (wireframe, rotations, tables), programme interne en ROM.", "6")
D("E09-18", "Autres puces: OBC1, S-RTC, ST010/ST011/ST018, MSU-1, BS-X",
  "puces a part, a traiter une par une selon la demande: OBC1 (gestion OAM), S-RTC (horloge), ST010/011 (IA, mathematiques), ST018 (ARM), MSU-1 (audio/donnees streaming, non officielle), satellaview BS-X.", "7")
