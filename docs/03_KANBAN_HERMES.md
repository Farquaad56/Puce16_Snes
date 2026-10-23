# 03 - IMPORT DANS LE KANBAN DE NOUS HERMES

Reference: https://github.com/nousresearch/hermes-agent/blob/main/website/docs/user-guide/features/kanban.md
Il n'existe pas de format d'import natif: on cree les taches avec `hermes kanban create`
(une commande par tache, avec cles d'idempotence => l'import est rejouable sans doublon).

Conventions du projet (D-012, D-013, D-020):
- assignee Hermes: **`Puce16_Snes`** ; board et tenant: **`puce16`** ;
- dossier de travail (workspace): `H:/__Emulator__/Puce16_Snes` (le depot Git du projet, chemin ABSOLU);
- **execution strictement sequentielle**: les taches forment UNE chaine (chaque tache a la precedente pour seul parent).
  Une seule tache est `ready` a la fois.

## 1. Prerequis

```
hermes kanban init
hermes kanban boards create puce16 --name "Puce16 SNES" --switch
```

Le depot doit exister localement et contenir le dossier `docs/` genere (le workspace est le depot lui-meme;
le corps des cartes renvoie vers `docs/tasks/<ID>.md` quand il est long).

## 2. Import

```
python docs/tools/kanban_import.py --dry-run                 # affiche les commandes, n'execute rien
python docs/tools/kanban_import.py --limit 10                # importe les 10 prochaines taches (recommande pour un premier essai)
python docs/tools/kanban_import.py                           # importe tout (hors taches differees E09)
python docs/tools/kanban_import.py --only E01                # une epopee
python docs/tools/kanban_import.py --from E02-05 --to E02-20 # une plage
```

Options utiles: `--assignee Puce16_Snes` (defaut), `--board puce16` (defaut), `--workspace H:/__Emulator__/Puce16_Snes` (defaut),
`--max-body 7500` (les cartes plus longues ont un corps court qui renvoie vers `docs/tasks/<ID>.md`; `0` = toujours le corps complet),
`--include-deferred` (cree aussi les taches E09 avec `--triage`).

Le script lit `docs/kanban/tasks.json` (genere par `build_tasks.py`), cree les taches dans l'ordre, passe
`--parent <id kanban de la tache precedente>`, `--idempotency-key puce16-<ID>`, et ecrit **apres chaque creation**
`docs/kanban/id_map.json` (ID Puce16 -> ID Kanban): un import interrompu reprend la ou il s'est arrete.
Les taches GATE sont creees **sans assignee** (validation humaine, voir 4.).

## 3. Mapping tache -> carte

| Champ Kanban | Valeur |
|---|---|
| title | `<ID> <titre>` (ex: `E02-03 ADC: tous les modes`) |
| body | contenu de `docs/tasks/<ID>.md` + pied de page (regles permanentes + protocole Q&R) |
| assignee | `Puce16_Snes` (sauf GATE: aucun) |
| tenant | `puce16` |
| workspace | `dir:H:/__Emulator__/Puce16_Snes` |
| priority | 10 - numero d'epopee (E01 = plus haute) |
| parent | la tache precedente de `INDEX.tsv` (chaine unique) |
| idempotency-key | `puce16-<ID>` |
| max-runtime / max-retries | `2h` / `1` |

## 4. Taches GATE (validation humaine)

Une tache GATE n'a pas d'assignee. Quand la tache precedente est `done`, elle apparait sans worker:
la chaine s'arrete la jusqu'a ta validation. L'utilisateur:
1. execute les commandes de validation indiquees dans la tache;
2. `python docs/tools/task.py approve <ID>` (puis commit/push);
3. `hermes kanban complete <id_kanban>` (l'ID est dans `docs/kanban/id_map.json`) pour debloquer la suite.

## 5. Boucle de travail pour UNE tache

```
Hermes (Kanban)   : prend la tache ready, code dans "Write only in", lance les commandes d'Acceptance, complete avec un resume
Claude/Utilisateur: python docs/tools/task.py check <ID>      (verification native: scope, tests nommes, hygiene, acceptance)
                    python docs/tools/task.py done <ID>       (affiche les commandes git; pas de Co-Authored-By, D-001)
                    git add -A ; git commit -m "<ID>: <titre>" ; git push
Si check echoue   : Claude lit la sortie, diagnostique (en francais) et fournit le prompt de correction (04_PROMPT_TEMPLATES.md)
Si Hermes bloque  : question dans docs/qa/inbox/ -> python docs/tools/qa.py answer ... -> commandes `hermes kanban comment` + `unblock` imprimees
```

Important: Hermes ne lance JAMAIS `task.py check|done|approve` ni `git commit|push` (il ne fait que coder et tester).

## 6. Taches de reglage dynamiques (`TUNE-nn`)

Quand un test passe mais que le comportement est faux a l'usage (son qui craque, cadence, jeu qui plante), Claude cree une tache
`TUNE-nn` (fichier dans `docs/tasks_src/` ou carte Kanban directe), petite et ciblee, inseree AVANT la tache suivante de la chaine.
Le sequencement D-020 reste: une seule tache a la fois.

## 7. Points a confirmer au premier import (UNKNOWN - to confirm)

- Forme exacte de la sortie `--json` de `hermes kanban create` (cle de l'ID: `id` ou `task_id`; la doc n'en decrit pas la forme).
  `kanban_import.py` essaie `id`, `task_id`, puis un motif `t_[0-9a-zA-Z]+` dans la sortie.
- Comportement d'une tache sans assignee (doit rester non dispatchee). Si elle etait dispatchee quand meme, la creer avec `--triage`
  (modifier `kanban_import.py`: ajouter `--triage` aux taches GATE).
- Si le lanceur `hermes` est un `.cmd` Windows, la longueur de commande est limitee a 8191 caracteres: garder `--max-body 7500` (defaut).
- Chemins Windows pour `--workspace dir:`: le script envoie des `/` (`H:/__Emulator__/Puce16_Snes`).

Si un point echoue, `kanban_import.py` s'arrete sur la premiere erreur et affiche la commande fautive; l'import reprend ensuite sans doublon.
