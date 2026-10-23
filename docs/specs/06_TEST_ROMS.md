# SPEC 06 - TESTS, ORACLES ET ROMS DE TEST

Decisions liees: D-006 (jeux cibles), D-015 (desktop tot), D-016 (Hermes telecharge les ROM de test).
Regle d'or: **aucune valeur materielle n'est inventee**. Quand un test externe contredit la spec, le test gagne et la spec est corrigee par Claude (Q&R).

## 1. Quatre niveaux de tests

| Niveau | Source | Telechargement | Qui valide |
|---|---|---|---|
| N1 - unitaire | donnees synthetiques ecrites dans le test (aucun fichier) | non | `cargo test` automatique |
| N2 - oracle JSON | SingleStepTests 65816 et spc700 (un test = 1 instruction, etat avant/apres) | oui (tache `T-ORACLE`) | automatique (`puce16-cli cpu-test` / `spc-test`) |
| N3 - ROM de test homebrew | ROM d'auteurs de tests (CPU, PPU, DMA, APU) | oui (tache `T-ROMS`) | automatique (baseline/hash) + GATE visuel de l'utilisateur |
| N4 - jeux commerciaux | SMW, ALttP, All-Stars (D-006) | **non: fournis par l'utilisateur** dans `tests/roms/games/` | GATE manuel de l'utilisateur |

Les ROM commerciales ne sont jamais telechargees ni commitees. Les entrees N4 du manifeste ont `optional = true` (ignorees si le fichier est absent).

## 2. Dossiers (tous ignores par git sauf README et manifeste)

```
tests/manifest.toml          liste des ROM de test (commite)
tests/baseline.json          hash attendus valides par l'utilisateur (commite)
tests/roms/README.md         sources + etat (commite)
tests/roms/homebrew/         ROM N3 telechargees (ignore)
tests/roms/games/            ROM N4 de l'utilisateur (ignore)
tests/oracle/65816/          JSON SingleStepTests 65816 (ignore)
tests/oracle/spc700/         JSON SingleStepTests spc700 (ignore)
tests/out/                   sorties .ppm, rapports (ignore)
```
`.gitignore` contient: `tests/roms/homebrew/`, `tests/roms/games/`, `tests/oracle/`, `tests/out/`, `target/`.

## 3. Oracle SingleStepTests (N2)

Sources (a confirmer au telechargement, `UNKNOWN - to confirm`): depots GitHub `SingleStepTests/65816` et `SingleStepTests/spc700`.
Les fichiers sont gros (des dizaines de Mo au total): telecharger avec `git clone --depth 1 <url> tests/oracle/<nom>` ou un clone partiel. Si le reseau est bloque -> question `tooling` (02_QA_PROTOCOL.md), ne pas contourner.

**Le format JSON exact n'est PAS suppose.** La premiere tache (`T-FORMAT`) :
1. liste le dossier clone, ouvre UN fichier de chaque depot (ex: premiers ~2 Kio avec `head -c 2000`);
2. ecrit `docs/specs/data/ssjson_format.md` : nom des fichiers (convention d'opcode, mode emulation/natif), noms exacts des cles de `initial` / `final`, forme de `ram`, forme de `cycles` (adresse, valeur, type), nombre de tests par fichier;
3. s'arrete pour revue de Claude (GATE doc). Le parseur n'est ecrit qu'APRES.

Strategie de comparaison (par ordre de difficulte, a activer au fur et a mesure):
1. etat final des registres + octets de RAM modifies;
2. nombre de cycles de bus;
3. journal complet des cycles (adresse + valeur + lecture/ecriture/interne) si le format le permet.
Le bus de test est PLAT (24 bits pour le 65816, 16 bits pour le SPC700), sans timing SNES, sans registres: les registres $2100-$21FF, $4xxx ne sont PAS mappes dans ces tests (RAM plate partout). Le CPU est donc teste via le trait `Bus` (spec 01 section 9) et le SPC700 via `SmpBus` (spec 07).
`puce16-cli cpu-test <dossier> [--filter NN]` : par fichier, imprime `OK`/`FAIL n/m` ; en cas d'echec, imprime le nom du test, l'etat attendu et obtenu (diff des champs differents seulement), jamais plus de 5 echecs par fichier. Code de sortie 1 si au moins un echec.
Les opcodes sont valides par groupes (une tache par groupe, specs 01 / 07), pour que chaque tache reste petite.

## 4. ROM de test homebrew (N3): candidats (tous `UNKNOWN - to confirm` : URL, nom de fichier, licence)

La tache `T-ROMS` : pour chaque candidat, retrouver le depot, telecharger les `.sfc`/`.smc` (ou construire s'ils sont fournis en source ET qu'un assembleur est installe: sinon ignorer), les ranger dans `tests/roms/homebrew/<auteur>/`, et remplir `tests/roms/README.md` (nom, URL reelle, date, taille, sha256). Un candidat introuvable est note `not found` (pas d'invention d'URL).

| Candidat | Contenu attendu | Utilite | Epopee |
|---|---|---|---|
| PeterLemon/SNES (depot GitHub, aussi nomme "krom") | nombreux petits programmes: tests CPU, modes graphiques, mode 7, fenetres, mosaique, HDMA, DMA, SPC700/BRR | rendu PPU (images attendues connues visuellement) | E04-E06 |
| gilyon/snes-tests | tests d'instructions CPU (basic/full), resultat affiche a l'ecran | validation CPU globale | E02 |
| undisbeliever/snes-test-roms | tests de timing (HDMA, DMA, IRQ, registres) | timing exact | E03-E04 |
| tests SPC700/DSP de blargg (programmes `spc_*`) | tests SMP/DSP, resultat par checksum ou texte | SPC700 + DSP | E06 |
| autres tests de bsnes/higan (depot de tests) | divers | optionnel | E08 |

Pour toute ROM sans critere de reussite automatique: la reussite = **hash framebuffer stable** apres N frames, valide UNE FOIS par l'utilisateur sur une capture (GATE), puis fige dans `tests/baseline.json`.

## 5. Manifeste `tests/manifest.toml`

```toml
[[rom]]
name = "peterlemon_gfx_mode1"          # unique, [a-z0-9_]
path = "tests/roms/homebrew/peterlemon/GFX/Mode1/Mode1.sfc"   # chemin relatif a la racine du repo
region = "auto"                        # auto | ntsc | pal
frames = 120                           # nombre de frames a executer
optional = false                       # true: ignore si le fichier manque
epic = "E04"                           # plus petite epopee ou la ROM doit passer
check = "hash"                         # hash | ram | ports | none
# check = "ram"  -> expect_ram = [ { addr = 0x7E0010, value = 0x01 } ]
# check = "ports" -> expect_ports = [0xAA, 0xBB, 0x00, 0x00]   (valeurs vues par le CPU sur $2140-$2143)
```
`baseline.json`: `{ "version": 1, "roms": { "<name>": { "frames": 120, "frame_hash": "0x...", "state_hash": "0x...", "audio_hash": "0x..." } } }` (`audio_hash`: FNV-1a 64 de toutes les paires `[i16; 2]` produites par `Snes::take_audio` pendant les `frames` frames, chaque `i16` en little-endian, L puis R. Pour `check = "hash"` les TROIS hash sont calcules et compares.).

## 6. Commande `puce16-cli suite <manifest.toml>`

- Pour chaque `[[rom]]` dont `epic <= --max-epic` (option, defaut: toutes): charge, execute `frames` frames, calcule `frame_hash` (FNV-1a 64 de `width` puis `height` (chacun `u32` little-endian) puis des pixels `u32` little-endian du `FrameView`, ligne par ligne -- identique a `puce16-cli run --hash`), `state_hash` (`Snes::state_hash`) et `audio_hash` (voir section 5).
- Compare a `baseline.json` selon `check`. Rapport `PASS`/`FAIL`/`SKIP` (optional absente) / `NEW` (pas de baseline).
- `--update-baseline <name>` : reecrit l'entree (uniquement demande par l'utilisateur apres validation visuelle, jamais par Hermes).
- `--dump-ppm DIR` : ecrit `<name>.ppm` (P6, 8 bits, largeur/hauteur du `FrameView`, pixels RGB) pour la revue visuelle.
- Echec = toujours `DebugSnapshot` (PC, A, X, Y, S, D, DB, PB, P, e, H/V, master, frame) ; jamais un `TIMEOUT` nu.
- Code de sortie: 0 = aucun FAIL, 1 = au moins un FAIL, 2 = erreur d'usage.

FNV-1a 64: `h = 0xcbf29ce484222325 ; pour chaque octet b: h ^= b ; h = h.wrapping_mul(0x100000001b3)`. Implementation unique dans `hash.rs` (`Fnv64::new() / write(&[u8]) / write_u64(u64) (little-endian) / finish()`).

## 7. Tests internes sans ROM externe (a faire des que le composant existe)

| Test | Verifie |
|---|---|
| Handshake IPL | apres reset, ~100 000 clocks maitres (l'IPL efface d'abord $00-$EF, ~50 000 clocks): lecture CPU de $2140 = $AA et $2141 = $BB (le SPC700 execute la ROM IPL, spec 07) |
| Transfert IPL | un programme CPU synthetique (mis dans une ROM LoROM fabriquee en code) envoie un petit programme SPC700 par le protocole IPL et le lance; la reponse sur les ports prouve l'execution |
| DMA WRAM | ROM synthetique: DMA general 256 octets ROM -> $2180; relecture par le CPU |
| NMI/IRQ | ROM synthetique: NMI incremente un compteur en WRAM une fois par frame; apres 60 frames compteur = 60 (ou 59/60 selon alignement, tolerance +-1 documentee) |
| Auto-joypad | `set_joypad` -> lecture de $4218/$4219 apres 2 frames = valeur injectee |
| Determinisme | meme ROM + memes entrees, deux executions -> `state_hash` egal a chaque frame |
| Save state | `save_state` puis `load_state` puis N frames == N frames sans passer par le save state (hash identique) |

Une ROM synthetique est fabriquee par une fonction de test `make_rom(code: &[u8], mode: MapMode) -> Vec<u8>` (en-tete valide, vecteur RESET = $8000, checksum correct) : fournie par la tache `E01-xx` dans `crates/puce16-core/tests/common/mod.rs`.

## 8. Jalons de validation (utilises pour ordonner les epopees et les GATE)

| Jalon | Condition | Epopee |
|---|---|---|
| J0 | squelette compile, desktop affiche une fenetre avec un degrade genere par le core | E01 |
| J1 | tous les opcodes 65816 passent l'oracle N2 | E02 |
| J2 | ROM synthetiques: bus, DMA/HDMA, NMI/IRQ, auto-joypad OK | E03 |
| J3 | PPU: les ROM graphiques de PeterLemon (mode 0 a 7) = captures validees par l'utilisateur | E04-E05 |
| J4 | tous les opcodes SPC700 passent l'oracle N2; handshake IPL OK | E06a |
| J5 | DSP: sons de ROM de test audibles + hash audio stable | E06b |
| J6 | SMW demarre jusqu'a l'ecran titre puis niveau 1 jouable (GATE utilisateur) | E07 |
| J7 | ALttP et All-Stars (SMB3) jouables ~5 minutes sans plantage (GATE) | E07 |
| J8 | desktop complet (audio, SRAM, save states, manette, debugger, filtres) | E08 |

## 9. Politique d'echec et de correction

1. Un test oracle qui echoue donne un diagnostic (champ fautif). Hermes corrige dans son scope; s'il pense que la spec est fausse -> question `spec` (pas de contournement).
2. Si deux ROM de test se contredisent: question a Claude, qui tranche avec le wiki et note la decision dans `DECISIONS.md`.
3. Les valeurs `UNKNOWN - to confirm` des specs sont des CANDIDATES de reglage (tuning): chaque tache de tuning change UNE constante, relance la suite complete et n'est acceptee que si aucune ROM deja `PASS` ne regresse.
