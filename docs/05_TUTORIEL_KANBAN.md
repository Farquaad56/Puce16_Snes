# 05 - TUTORIEL: INSTALLER ET LANCER PUCE16 DANS LE KANBAN DE NOUS HERMES

Public: toi (utilisateur, Windows, AZERTY). Duree: ~30 min la premiere fois. Chemin du depot: `H:\__Emulator__\Puce16_Snes`.
Reference Hermes: https://github.com/nousresearch/hermes-agent/blob/main/website/docs/user-guide/features/kanban.md

## 0. Comment ca marche (resume en 6 lignes)

1. Le depot contient 248 taches ecrites (`docs/tasks/E01-01.md` ... `E09-07.md`), chacune de 50 a 150 lignes de code.
2. Chaque tache devient UNE carte Kanban. Les cartes forment UNE chaine: chaque carte a la precedente pour parent => une seule carte `ready` a la fois (D-020).
3. Hermes (LLM local) prend la carte `ready`, code seulement dans les fichiers autorises ("Write only in"), lance les tests de la tache, puis la complete.
4. TOI (ou Claude, avec ta sortie collee) lances `task.py check <ID>`: c'est la verification officielle. Hermes ne la lance jamais.
5. Si `check` est OK: `task.py done <ID>` puis `git add / commit / push` (un commit = une tache).
6. Les taches GATE (fin de jalon) n'ont pas d'assignee: la chaine s'arrete la jusqu'a ce que TU fasses `approve`.

## 1. Prerequis a installer (une seule fois)

| Outil | Verification | Installation |
|---|---|---|
| Git | `git --version` | https://git-scm.com/download/win |
| Rust (stable, MSVC) | `cargo --version` | https://rustup.rs |
| Python 3.10+ | `python --version` | https://www.python.org (cocher "Add to PATH") |
| Hermes Agent | `hermes --version` | voir le README de nousresearch/hermes-agent |
| Modele local | selon ton serveur (LM Studio, Ollama...) | un modele code capable, contexte >= 32k recommande |

Sous Windows, installe aussi les "Build Tools for Visual Studio" (charge C++), sinon `cargo build` echoue.
Les ROM de test: Hermes les telecharge lui-meme (tache E07-01). Ne commite jamais de ROM commerciale (`.gitignore`).

## 2. Recuperer le depot dans le dossier de travail

```
cd H:\__Emulator__
git clone https://github.com/Farquaad56/Puce16_Snes.git
cd Puce16_Snes
git config user.name  "ton nom"
git config user.email "legarrec.s56@gmail.com"
python docs\tools\build_tasks.py --check
```

Attendu: `OK (validation only)` et `248 tasks`.

Authentification du push: le plus simple est le **Git Credential Manager** livre avec Git for Windows
(une fenetre de connexion GitHub s'ouvre au premier `git push`). Sinon cree un NOUVEAU token (le precedent doit etre revoque)
et colle-le quand Git demande le mot de passe. **Ne mets jamais un token dans un fichier du depot.**

Le dossier `H:\__Emulator__\Puce16_Snes` est le WORKSPACE des cartes: Hermes y ecrit directement.

## 3. Initialiser Hermes Kanban

```
hermes kanban init
hermes kanban boards create puce16 --name "Puce16 SNES" --switch
```

### 3.1 Le profil Hermes `Puce16_Snes` (obligatoire)

L'assignee d'une carte est un NOM DE PROFIL Hermes. Si le profil n'existe pas, le dispatcher echoue au lancement de la carte
puis la bloque. Cree-le et configure-le sur ton modele local:

```
hermes profile create Puce16_Snes
hermes profile describe Puce16_Snes --text "Code Rust de l'emulateur SNES Puce16 a partir des docs/tasks/*.md, une tache a la fois"
hermes -p Puce16_Snes config
```

(la derniere commande ouvre la configuration du profil: choisis ton fournisseur/modele local et active les outils fichiers + terminal).
Syntaxe exacte des sous-commandes `profile`: UNKNOWN - to confirm (verifie avec `hermes profile --help`).

### 3.2 Desactiver la decomposition automatique des cartes "triage"

Les 7 taches differees E09 (coprocesseurs) ne sont PAS importees par defaut. Si un jour tu les importes (`--include-deferred`),
elles arrivent en colonne `triage`, et Hermes les decomposerait tout seul. Pour garder le controle, mets dans la config Hermes:

```
kanban:
  auto_decompose: false
```

(cle documentee: `kanban.auto_decompose`; fichier de config: `~/.hermes/config.yaml`, emplacement a confirmer avec `hermes config`).

### 3.3 Demarrer le dispatcher

Le dispatcher (qui lance Hermes sur la carte `ready`, verifie toutes les 60 s) tourne dans la gateway:

```
hermes gateway start
```

Laisse cette fenetre ouverte. N'utilise PAS en plus `hermes kanban daemon` (deprecie, non supporte avec la gateway).

## 4. Importer les cartes

Toujours depuis `H:\__Emulator__\Puce16_Snes`:

```
python docs\tools\kanban_import.py --dry-run --limit 5     (affiche les commandes, ne cree rien)
python docs\tools\kanban_import.py --limit 10              (essai: les 10 premieres cartes)
hermes kanban list
```

Verifie dans `hermes kanban list` ou `hermes dashboard` (onglet Kanban): E01-01 est `ready` (ou `todo` puis `ready` apres ~60 s),
E01-02 est `todo` (en attente de son parent). Si c'est bon, importe le reste:

```
python docs\tools\kanban_import.py
```

L'import est rejouable: cles d'idempotence `puce16-<ID>` + fichier `docs/kanban/id_map.json` (ID Puce16 -> ID Kanban, NON commite).
Si l'import s'arrete sur une erreur, corrige puis relance la meme commande: pas de doublon.

Points `UNKNOWN - to confirm` du premier import (forme de la sortie `--json`, tache sans assignee, limite 8191 caracteres de `hermes.cmd`):
voir `docs/03_KANBAN_HERMES.md` section 7.

## 5. La boucle de travail: UNE tache a la fois

```
        Hermes (automatique)                       Toi (apres que la carte soit "done" cote Hermes)
  ------------------------------------      ---------------------------------------------------------
  prend la carte ready                       python docs\tools\task.py check E01-01
  lit docs/tasks/E01-01.md + specs           -> OK ?  oui: python docs\tools\task.py done E01-01
  code dans "Write only in"                             puis: git add -A
  lance les commandes d'Acceptance                            git commit -m "E01-01: <titre>"
  complete la carte avec un resume                            git push
                                              -> KO ?  colle la sortie a Claude (voir 04_PROMPT_TEMPLATES.md)
```

Regles:
- Une carte = une sous-etape (50-150 lignes). Jamais deux cartes en parallele: la chaine l'empeche.
- ATTENTION (comportement Kanban): quand Hermes complete la carte N, le dispatcher passe la carte N+1 en `ready` et la lance
  dans la minute, SANS attendre ton `check`. La chaine garantit "une seule carte a la fois", pas "validee avant la suivante".
  Deux modes:
  - mode strict (recommande au debut, jusqu'a E02): apres chaque carte terminee, arrete la gateway (`hermes gateway stop`),
    fais `check` / `done` / commit / push, puis `hermes gateway start` pour lancer la suivante;
  - mode groupe (une fois le rythme etabli): laisse tourner, puis valide les cartes dans l'ordre avec `check`. Si une carte N echoue,
    les cartes suivantes deja faites sont suspectes: `task.py reset` puis refaire dans l'ordre.
- `task.py check` verifie: ordre strict, fichiers modifies = scope autorise, tests nommes presents, hygiene du code, commandes d'acceptance.
- Commits: `<ID>: <titre>`, SANS lignes `Co-Authored-By` ni `Claude-Session` (D-001).

### Quand pousser sur GitHub ?

- **Apres CHAQUE tache validee** (check OK + done): un commit + un push. C'est le rythme normal (248 petits commits propres, retour arriere facile).
- Les **GATE** (fin de jalon, ex. E08-39) sont les etapes cles: apres ton `approve`, commit + push, puis mets un tag:
  `git tag J1` ... `git push --tags` (un tag par jalon).
- Hermes ne fait jamais `git commit` ni `git push`.

### Taches GATE

Une GATE n'a pas d'assignee: elle reste en attente. Tu executes les commandes de validation decrites dans la tache (lancer l'emulateur,
tester une ROM...), puis:

```
python docs\tools\task.py approve <ID>
git add -A ; git commit -m "<ID>: <titre>" ; git push
hermes kanban complete <id_kanban>        (l'id est dans docs\kanban\id_map.json)
```

### Quand Hermes est bloque

Il ecrit sa question dans `docs/qa/inbox/`. Tu (ou Claude) reponds avec:

```
python docs\tools\qa.py list
python docs\tools\qa.py answer <QID> --text "reponse" [--durable]
```

La commande imprime les `hermes kanban comment` et `hermes kanban unblock` a lancer. `--durable` ajoute la reponse a `docs/qa/DECISIONS.md`.

## 6. Commandes utiles

```
hermes kanban list | stats | watch | show <id> | tail <id> | log <id>
hermes dashboard                                 (tableau visuel, onglet Kanban)
python docs\tools\task.py status                 (etat de toutes les taches)
python docs\tools\task.py next                   (prochaine tache a faire)
python docs\tools\task.py show <ID>              (affiche une tache)
python docs\tools\task.py reset <ID>             (remet une tache a todo)
```

## 7. Depannage rapide

| Symptome | Cause probable | Action |
|---|---|---|
| Carte `blocked` des le debut | profil `Puce16_Snes` absent ou sans modele | section 3.1, puis `hermes kanban unblock <id>` |
| Aucune carte ne passe `ready` | gateway arretee | `hermes gateway start` |
| `kanban_import.py` echoue sur la longueur | limite de ligne de commande Windows | garder `--max-body 7500` (defaut), voire le baisser |
| `check` refuse: "fichier hors scope" | Hermes a modifie un fichier non autorise | `git checkout -- <fichier>` ou colle la sortie a Claude |
| `check` refuse: test introuvable | nom de test different de celui demande | Claude fournit un prompt de correction |
| Carte `blocked`, log "Repetition Detected" | le modele local a boucle en ecrivant sa reponse finale (le code est souvent deja fait) | `cargo test` + `task.py check`; si OK: `hermes kanban complete <id> --result "..."` (section 8) |
| Hermes boucle / derive | tache trop vague pour le modele | Claude decoupe la tache en une `TUNE-nn` plus petite |
| `cargo build` echoue (link.exe) | Build Tools C++ manquants | installer les Build Tools Visual Studio |

## 8. Cas pratique: carte bloquee par "Repetition Detected" (ou cartes terminees sans avoir fait `check`)

1. `hermes kanban show <id>` puis `hermes kanban log <id> --tail 20000`: le message "Repetition Detected" vient du MODELE (boucle de generation), pas de la spec.
2. Le code est parfois deja ecrit: lance les tests de la tache (`cargo test -p puce16-core <prefix>`), puis `python docs\tools\task.py check <ID>`.
3. Si OK: `python docs\tools\task.py done <ID>`, commit, push, puis debloquer la chaine sans relancer le modele:
   `hermes kanban complete <id> --result "code ecrit par Hermes, check OK"` (la carte suivante passe alors `ready`).
4. Si KO ou fichiers incoherents: `git restore` / supprimer les fichiers de la tache, `hermes kanban comment <id> "..."`, `hermes kanban unblock <id>`.
5. Pour limiter les boucles: regler le modele local (penalite de repetition, temperature 0.2-0.5, contexte suffisant) ou changer de modele pour le profil.

Rattrapage par lot (cartes E01-01..E01-08 terminees sans `check`, tout est non commite): le controle de scope git ne peut pas
separer les taches. Pour chaque tache DANS L'ORDRE: `python docs\tools\task.py check <ID> --no-scope` puis `python docs\tools\task.py done <ID>`;
a la fin, UN commit groupe `E01-01..E01-08: workspace, consts, hash, framebuf, helpers de test`. Ensuite reprendre la regle normale (un commit par tache).
