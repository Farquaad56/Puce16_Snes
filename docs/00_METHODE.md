# 00 - METHODE (Puce16)

Puce16 = emulateur Nintendo SNES / Super Famicom, ecrit en Rust, cycle-accurate,
construit avec la meme methode que Puce8 (Game Boy).

Principe central: **toute la reflexion est faite ICI, dans la documentation et
dans les taches. Le LLM local (Hermes) ne fait que CODER a partir de la
documentation.** Une tache doit donc etre autosuffisante.

## 1. Roles

| Qui | Fait | Ne fait jamais |
|---|---|---|
| Hermes (LLM local, opencode) | Lit la tache + les specs citees, ecrit le code et les tests dans le scope autorise, lance les tests de sa tache | Lancer `task.py check/done/approve`, modifier hors scope, inventer une valeur materielle |
| Claude | Diagnostic (en francais), prompts prets a coller, ecriture/maintenance des docs et des taches, reponses aux questions bloquantes | Ecrire le code a la place de Hermes sauf correctif minimal demande |
| Utilisateur | Valide les GATE a la main, repond aux questions (Q&R), push | - |

## 2. Pipeline (identique a Puce8)

```
docs/tasks/<ID>.md      une tache = un fichier (spec + scope + tests + acceptance)
docs/tasks/INDEX.tsv    ordre officiel des taches (ID, titre, epopee, deps, gate)
docs/checks/<ID>.sh     script de verification automatique de la tache
docs/specs/*.md         specifications techniques (source unique de verite)
docs/qa/QA_LOG.md       toutes les questions/reponses (append-only)
docs/qa/DECISIONS.md    decisions durables extraites des Q&R
PROGRESS.json           etat: todo / in_progress / checked / done / approved
python docs/tools/task.py check <ID>     (utilisateur ou Claude, PAS Hermes)
python docs/tools/task.py done <ID>
python docs/tools/task.py approve <ID>   (GATE: utilisateur uniquement)
python docs/tools/task.py next           (affiche la prochaine tache selon INDEX)
```

Regles de l'ordre: on respecte INDEX.tsv. Une tache n'est ouverte que si la precedente
est `done` (ou `approved` pour une GATE). Les taches GATE exigent un `approve` manuel.

## 3. Regles de redaction de TOUTE tache (format obligatoire)

Chaque `docs/tasks/<ID>.md` contient exactement ces sections:

1. `# <ID> - <titre>`
2. `## Contexte` : 2-5 lignes, pourquoi cette tache existe.
3. `## Lire d'abord` : liste exacte des fichiers specs (avec sections).
4. `## Write only in` : liste fermee de chemins ecrivables. Tout autre chemin est interdit.
5. `## Specification` : le comportement exact, avec tables de bits/valeurs/cycles
   RECOPIES dans la tache (pas de "voir le wiki"). Le LLM ne doit pas chercher.
6. `## Tests` : noms exacts de tests, tous prefixes par l'ID de tache en minuscules
   (ex: `e02_03_adc_imm_binary`). Entrees/sorties attendues chiffrees.
7. `## Acceptance` : commandes exactes a faire passer (`cargo test e02_03_ -p puce16-core`).
8. `## Si tu bloques` : renvoi au protocole Q&R (02_QA_PROTOCOL.md).

## 4. Regles de code (donnees a Hermes dans chaque prompt)

- Code, identifiants et commentaires: **anglais, ASCII uniquement**.
- Toute valeur inconnue ou non specifiee: ecrire le commentaire exact
  `UNKNOWN - to confirm` et **bloquer la tache avec une question** (voir QA).
  Ne jamais deviner un registre, un timing, un opcode.
- Edits ancres exacts: modifier uniquement les blocs indiques, ne pas reformater le reste.
- `puce16-core`: deterministe, **entiers uniquement** (pas de f32/f64), pas de HashMap
  iteree, pas d'horloge systeme, pas d'aleatoire non seede, `#![forbid(unsafe_code)]`.
- Un tick = un cycle maitre (ou sous-multiple documente). Aucune "magie" par instruction.
- Pas de dependance externe dans `puce16-core` sauf celles listees dans 01_ARCHITECTURE.md.
- Hermes repond uniquement avec les lignes/fichiers demandes, sans recap.

## 5. Cloture d'une tache (commandes completes, donnees par Claude)

```
python docs/tools/task.py check <ID>
python docs/tools/task.py done <ID>
git add -A
git commit -m "<ID>: <titre>"
git push
```

Les messages de commit des taches Puce16 ne contiennent ni `Co-Authored-By`
ni `Claude-Session` (decision utilisateur heritee de Puce8; voir DECISIONS.md D-001).

## 6. Test pilote par ROM

- Commande `puce16-cli suite <manifest.toml>`: execute chaque ROM, compare a une baseline
  de non-regression (`tests/baseline.json`).
- Un echec est TOUJOURS rapporte avec l'etat des registres (PC, A, X, Y, S, D, DB, P,
  scanline, dot, master clock). Jamais un `TIMEOUT` nu.
- Oracle CPU: SingleStepTests/65816 et /spc700 (JSON, cycles de bus inclus).
- Les ROM de test ne sont PAS dans le repo: `tests/roms/README.md` liste noms + sources a confirmer.

## 7. Environnement utilisateur

- Windows, clavier AZERTY, commande `python` (pas `python3`).
- `task.py check` est entierement natif Python (aucun `bash` requis): il verifie l'ordre, le scope, les tests nommes,
  l'hygiene du code puis lance les commandes `acceptance` de la tache (voir 01_ARCHITECTURE.md section 8).
  Les `docs/checks/<ID>.sh` generes sont facultatifs (shell bash).
- D-020: **une seule tache a la fois**, dans l'ordre de `docs/tasks/ORDER.md`. Aucun travail en parallele.
- Tous les chemins dans les taches sont relatifs a la racine du repo, avec `/`.

## 8. Granularite des taches

- Une tache = 1 a 3 heures de code pour un LLM local = un seul sujet.
- Une sous-etape = une tache (50 a 150 lignes de code, D-014). Les epopees (E01..E09) sont des groupes de taches.
- Execution strictement sequentielle (D-020): chaque tache a la precedente comme seul parent dans le Kanban.
  Hermes ne prend jamais deux taches en meme temps.
- Si une tache depasse 250 lignes de code attendues, la decouper (par Claude, pas par Hermes).
