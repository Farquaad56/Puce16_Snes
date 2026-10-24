# Format JSON de l'oracle SingleStepTests 65816

Source: `tests/oracle/65816/v1/*.json` (clone de https://github.com/SingleStepTests/65816).
Ce document a ete CORRIGE par Claude (tache E02-03) apres comparaison avec 8 fichiers reels:
`00.e`, `00.n`, `44.e`, `4c.e`, `54.n`, `a9.n`, `cb.e`, `db.n` (60 200 tests, 1 008 014 cycles).
Les exemples ci-dessous sont copies tels quels des fichiers. Les affirmations marquees (verifie) ont ete controlees par script sur ces 8 fichiers.

## 1. Fichiers

- Dossier `tests/oracle/65816/v1/`, 512 fichiers `NN.e.json` et `NN.n.json`.
- `NN` = opcode en hexadecimal **MINUSCULE** sur 2 chiffres (`0a.e.json`, `4c.e.json`, `a9.n.json`; jamais `4C`).
- `.e.` = mode emulation (E=1), `.n.` = mode natif (E=0). La cle `initial.e` correspond toujours au nom du fichier (verifie).
- Un fichier = un tableau JSON, **un test par ligne**: la premiere ligne commence par `[{`, chaque test se termine par `},` (le dernier par `}]`).
- Nombre de tests par fichier: **10 000**, SAUF les copies de blocs: `44.*` (MVP) et `54.*` (MVN) contiennent **100 tests** (verifie sur `44.e` et `54.n`; les fichiers `44.n` et `54.e` ont des tailles equivalentes, 10 a 11 Mo).
- Noms de tests: `"NN m K"` (opcode hexa minuscule, `e` ou `n`, K de 1 a 10 000), ex `"db n 1"`.
- Les fichiers font de 3,7 a 11,7 Mo: les charger un par un, ne pas tout garder en memoire.

## 2. Cles initial et final

Un test est un objet avec exactement 4 cles: `name`, `initial`, `final`, `cycles` (verifie). `initial` et `final` ont les memes 11 cles, toutes obligatoires:

| Cle | Type JSON | Signification (plages verifiees) |
|---|---|---|
| `pc` | entier | PC 16 bits, **jamais** plus de 0xFFFF (la banque est dans `pbr`) |
| `pbr` | entier | bank du programme (8 bits) |
| `a` | entier | accumulateur C **16 bits TOUJOURS** (en emulation les 8 bits hauts = registre B cache, valeurs > 255 normales) |
| `x`, `y` | entier | index 16 bits; **< 256** quand E=1 ou quand le flag X de P vaut 1 (verifie) |
| `s` | entier | pile 16 bits. En emulation: voir 6.2 (le poids fort initial est ALEATOIRE, le final vaut toujours 0x01) |
| `d` | entier | direct page 16 bits |
| `dbr` | entier | data bank (8 bits) |
| `p` | entier | registre P (8 bits). Si E=1: bits 0x30 toujours a 1 (verifie) |
| `e` | entier | 0 = natif, 1 = emulation |
| `ram` | tableau | liste de paires `[adresse, valeur]` (adresse 24 bits, valeur 0..255) |

**Semantique de `ram`** (differente de ce que supposait la premiere version): `initial.ram` = TOUS les octets dont l'instruction a besoin
(octets de l'instruction, operandes, donnees lues, vecteurs lus), a leurs valeurs AVANT; `final.ram` = les octets touches (lus ou ecrits), a leurs valeurs APRES.
L'octet de l'opcode (`pbr:pc`) est toujours dans `initial.ram` (verifie sur 60 200 tests). Toute autre adresse est consideree comme valant 0.

## 3. Cycles

`cycles` = liste de **TOUS les cycles du CPU**, internes compris: un cycle = `[adresse, valeur, "type"]` (verifie: 1 008 014 cycles, tous de cette forme).

- `adresse`: entier 24 bits, ou **`null`** (seul cas observe: le cycle final `[null, null, "--------"]` de WAI (`cb`) et STP (`db`), 20 000 occurrences).
- `valeur`: entier 0..255, ou **`null`** pour un cycle INTERNE (pas de transfert de donnee). Les cycles internes portent quand meme une adresse (celle laissee sur le bus), qui n'est PAS significative pour nous (voir 7).
- `type`: chaine de 8 caracteres, une position par signal (verifie sur tous les cycles):

| Pos | Caractere | Signification |
|---|---|---|
| 0 | `d` ou `-` | VDA: acces a une donnee |
| 1 | `p` ou `-` | VPA: acces au programme (`dp` = lecture de l'opcode; `-p` = lecture d'un operande) |
| 2 | `v` ou `-` | lecture de vecteur d'interruption (`d-vr` = lecture d'un vecteur) |
| 3 | `r`, `w` ou `-` | lecture ou ecriture (`-` seulement pour le cycle mort `--------`) |
| 4 | `e` ou `-` | etat du flag E (emulation) pendant ce cycle |
| 5 | `m` ou `-` | etat du flag M (accumulateur 8 bits) |
| 6 | `x` ou `-` | etat du flag X (index 8 bits) |
| 7 | `-` | toujours `-` (inutilise) |

Les lettres `e`, `m`, `x` decrivent l'ETAT du CPU, pas un type d'acces. Elles ne servent pas a comparer.

**Classification pour notre harnais** (regle unique):
- `valeur == null` -> cycle INTERNE (meme si la pos. 3 vaut `r`: les cycles internes sont marques `r`);
- sinon pos. 3 = `r` -> lecture, `w` -> ecriture.

Exemple copie tel quel (`00.e.json`, test 1, BRK en emulation: 7 cycles = opcode, signature, 3 ecritures de pile, 2 lectures de vecteur):
```json
{ "name": "00 e 1", "initial": {"pc": 39617, "s": 331, "p": 186, "a": 12405, "x": 169, "y": 238, "dbr": 32, "d": 28323, "pbr": 155, "e": 1, "ram": [[65535, 216], [65534, 225], [10197698, 242], [10197697, 0]]}, "final": {"pc": 55521, "s": 328, "p": 182, "a": 12405, "x": 169, "y": 238, "dbr": 32, "d": 28323, "pbr": 0, "e": 1, "ram": [[65535, 216], [65534, 225], [329, 186], [330, 195], [331, 154], [10197698, 242], [10197697, 0]]}, "cycles": [[10197697, 0, "dp-remx-"],[10197698, 242, "-p-remx-"],[331, 154, "d--wemx-"],[330, 195, "d--wemx-"],[329, 186, "d--wemx-"],[65534, 225, "d-vremx-"],[65535, 216, "d-vremx-"]]}
```

## 4. Questions ouvertes

Resolues par Claude (voir 7): adresse des cycles internes; interruptions (le BRK ci-dessus couvre la lecture de vecteur; il n'y a pas de NMI/IRQ externes dans ces tests); valeur lors des ecritures (= l'octet ecrit, verifie sur BRK).
Reste ouvert (a confirmer au fil des taches de verification): comportement exact attendu pour WAI/STP au-dela du cycle mort final (voir 6.3).

## 5. Exemple complet (copie tel quel de `db.n.json`, test 1: STP en natif; contient des cycles internes et le cycle mort)

```json
{ "name": "db n 1", "initial": {"pc": 34298, "s": 753, "p": 44, "a": 64391, "x": 10961, "y": 6389, "dbr": 224, "d": 4958, "pbr": 48, "e": 0, "ram": [[3180026, 219]]}, "final": {"pc": 34299, "s": 753, "p": 44, "a": 64391, "x": 10961, "y": 6389, "dbr": 224, "d": 4958, "pbr": 48, "e": 0, "ram": [[3180026, 219]]}, "cycles": [[3180026, 219, "dp-r-m--"],[3180027, null, "---r-m--"],[3180027, null, "---r-m--"],[null, null, "--------"]]}
```

## 6. Regles de harnais (a appliquer par E02-12, E02-13, E02-14)

1. **Etat initial**: memoire remise a 0, puis `initial.ram` ecrite; registres copies tels quels.
2. **Emulation (E=1): normaliser S avant le test**: `s = 0x0100 | (s & 0xFF)`. Les fichiers `.e.` donnent un poids fort aleatoire en entree mais un poids fort 0x01 en sortie (verifie: `final.s >> 8 == 1` sur 30 100 tests emulation des fichiers `00.e`, `44.e`, `4c.e`, `cb.e`). Sans cette regle, tous les tests ne touchant pas S (ex: JMP) echouent.
3. **Cycle mort** `[null, null, "--------"]` (WAI/STP): il ne correspond a aucun cycle de bus; `diff_cycles` l'IGNORE cote attendu (le nombre de cycles compare est celui sans ce cycle).
4. **MVN/MVP (`54`, `44`)**: un test = le bloc COMPLET (jusqu'a A = 0xFFFF), de quelques centaines a plusieurs milliers de cycles (ex 6643). Notre `step` ne copie qu'UN octet: `run_case` doit rejouer `step` tant que `pbr:pc` est egal a l'adresse de l'instruction (limite 70 000 iterations, sinon echec), en cumulant le journal de cycles.
5. **Comparaison finale**: registres + chaque `(adresse, valeur)` de `final.ram`. Cycles (niveau Full): type (lecture/ecriture/interne), et pour les cycles non internes adresse + valeur. Adresses des cycles internes: jamais comparees.

## 7. Decisions de Claude (E02-03)

- Le trait `Bus` de 01b section 2 reste INCHANGE (`idle()` sans adresse): un cycle interne dure toujours 6 clocks maitres (spec 02), quelle que soit l'adresse.
- `opcode_of_file(nom)`: les deux premiers caracteres du nom de fichier (hexa, insensible a la casse) si le nom a la forme `HH.e.json` ou `HH.n.json`, sinon `None`. Exemples: `4c.e.json` -> `Some(0x4C)`, `0a.n.json` -> `Some(0x0A)`, `readme.md` -> `None`.
- `--filter 61,63,E9`: liste d'opcodes hexa separes par des virgules, insensible a la casse, espaces tolerees; un fichier est retenu si son opcode est dans la liste (les deux modes `.e` et `.n`).
- `CycleRec` = `addr: Option<u32>`, `val: Option<u8>`, `kind: String` (la chaine de 8 caracteres telle quelle). `CpuState`: `a`, `x`, `y`, `s`, `d`, `pc` en `u16`; `p`, `dbr`, `pbr` en `u8`.

STATUS: reviewed
