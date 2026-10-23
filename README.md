# Puce16 - emulateur Nintendo SNES / Super Famicom en Rust

Puce16 est un emulateur SNES (65816, PPU dot par dot, SPC700 + S-DSP, DMA/HDMA, NTSC et PAL) ecrit en Rust,
construit avec la meme methode que Puce8 (Game Boy): **toute la reflexion est dans la documentation et dans des
taches tres petites; le LLM local (Hermes) ne fait que coder a partir de ces taches.**

Etat: documentation et taches pretes (248 entrees: 241 taches v1 + 7 coprocesseurs differes). Le code n'est pas encore ecrit.

## Contenu du depot

```
docs/00_METHODE.md            la methode (roles, pipeline, regles de code)
docs/01_ARCHITECTURE.md       decisions, workspace, modele de temps, API publique
docs/02_QA_PROTOCOL.md        questions / reponses (Hermes ne devine jamais)
docs/03_KANBAN_HERMES.md      import dans le Kanban de Nous Hermes
docs/04_PROMPT_TEMPLATES.md   prompts prets a coller
docs/05_TUTORIEL_KANBAN.md   tutoriel pas a pas: installation, import, boucle de travail, push
docs/specs/                   specifications techniques (source de verite) + donnees (opcodes, IPL, gauss...)
docs/qa/                      DECISIONS.md, FAQ_SNES.md, QA_LOG.md (+ inbox/ et answered/)
docs/tasks/                   une tache = un fichier (genere), INDEX.tsv, ORDER.md
docs/tasks_src/               SOURCE des taches (Python); on edite ici puis on regenere
docs/tools/                   build_tasks.py, task.py, qa.py, kanban_import.py, check_gauss.py, ppu_ref.py
docs/kanban/tasks.json        cartes Kanban (genere)
PROGRESS.json                 etat des taches (todo / in_progress / checked / done / approved)
```

## Demarrage rapide (Windows, `python`)

```
python docs/tools/task.py next                       # la prochaine tache (ordre strict, une seule a la fois)
python docs/tools/task.py status                     # avancement par epopee
python docs/tools/kanban_import.py --dry-run --limit 5
python docs/tools/kanban_import.py --limit 10        # cree les 10 premieres cartes dans le board `puce16`
# ... Hermes code la tache ...
python docs/tools/task.py check E01-01
python docs/tools/task.py done E01-01
git add -A && git commit -m "E01-01: <titre>" && git push
```

Les taches GATE (fin de jalon) se valident a la main: `python docs/tools/task.py approve <ID>`.

## Regenerer les taches

Apres toute modification de `docs/tasks_src/*.py` ou d'une spec citee par une tache:

```
python docs/tools/build_tasks.py --check    # validation seule
python docs/tools/build_tasks.py            # regenere docs/tasks, INDEX.tsv, ORDER.md, docs/kanban/tasks.json
```

## Regles essentielles

- Code, identifiants et commentaires en anglais ASCII. Valeur inconnue: `UNKNOWN - to confirm` + question, jamais de supposition.
- `puce16-core`: entiers uniquement, deterministe, `#![forbid(unsafe_code)]`.
- Une seule tache a la fois (D-020). Commits de taches sans `Co-Authored-By` ni `Claude-Session` (D-001).
- Aucun jeton, mot de passe ou cle n'est ecrit dans le depot.
- Les ROM de test et les ROM commerciales ne sont jamais commitees (`.gitignore`).
