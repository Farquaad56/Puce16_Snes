# Sources des oracles SingleStepTests (N2)

Telecharge par la tache E02-01 (T-ORACLE). Le dossier `tests/oracle/` est ignore par git.

## Depot 65816

| Champ | Valeur |
|---|---|
| URL reelle utilisee | https://github.com/SingleStepTests/65816 |
| Commande | `git clone --depth 1 https://github.com/SingleStepTests/65816 tests/oracle/65816` |
| Hash du commit clone (`git -C tests/oracle/65816 rev-parse HEAD`) | `307c63f5673c8c6b861c52f71ddb89212a227480` (branche `main`) |
| Date du telechargement | 2026-10-24 |
| Nombre de fichiers `.json` (recursif) | 512 |
| Taille totale | 3 245 817 684 octets (~3095 MiB, ~3.0 GiB) |

Contenu du depot : `LICENCE`, `README.md`, et le dossier `v1/` contenant tous les fichiers `.json`.
Convention de noms des fichiers : `<opcode>.e.json` (mode emulation) et `<opcode>.n.json` (mode natif), ex. `00.e.json`, `00.n.json`.

Exemples de noms de fichiers (5) :

1. `tests/oracle/65816/v1/00.e.json`
2. `tests/oracle/65816/v1/00.n.json`
3. `tests/oracle/65816/v1/01.e.json`
4. `tests/oracle/65816/v1/01.n.json`
5. `tests/oracle/65816/v1/02.e.json`

## Depot spc700

Pas encore telecharge (tache E02-01 ne couvre que le 65816). L'existence du depot est confirmee : https://github.com/SingleStepTests/spc700 (branche `main`, HEAD `67d15f492b2740964abd4efd1229e0ec9c342228` au 2026-10-24). A telecharger par la tache T-ORACLE spc700 dans `tests/oracle/spc700`.
