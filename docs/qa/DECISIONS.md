# DECISIONS (durables, le plus recent gagne)

Format: `D-nnn | date | source | decision`

- D-001 | 2026-10-22 | utilisateur | Commits des taches: pas de `Co-Authored-By` ni `Claude-Session` (heritage Puce8). Ordre INDEX.tsv respecte. Hermes ne lance jamais `task.py check/done/approve`.
- D-002 | 2026-10-22 | utilisateur (Q&R cadrage, REMPLACE la version scanline) | Cycle-accurate partout: CPU/bus/DMA/timers/IRQ/SPC700/DSP **et PPU dot par dot (pixel-accurate)**: un pixel/dot compose a chaque dot, les ecritures de registres PPU prennent effet au dot courant (effets mid-scanline supportes).
- D-003 | 2026-10-22 | utilisateur | NTSC et PAL des la v1.
- D-004 | 2026-10-22 | utilisateur (Q-001 CONFIRME) | Horloge APU: 24 606 720 Hz (DSP 32 040 Hz), constante `APU_RESONATOR_HZ`.
- D-005 | 2026-10-22 | utilisateur | Aucun coprocesseur en v1. Plus tard: DSP-1, SuperFX, SA-1 (E07 optionnel).
- D-006 | 2026-10-22 | utilisateur | Jeux cibles: Super Mario World, Zelda ALttP, Super Mario All-Stars (SMB3).
- D-007 | 2026-10-22 | utilisateur | Depot neuf, template vierge + pipeline Puce8.
- D-008 | 2026-10-22 | Claude (a confirmer Q-002) | Scanline NTSC normale = 1364 clocks, 340 dots (H = 0..339). Les 4 dots de 5 clocks (wiki) sont places en H = 323, 324, 327, 328 par defaut. Seule la conversion clock<->H des lectures OPHCT et des IRQ H est affectee.
- D-009 | 2026-10-22 | Claude | `puce16-core` n'utilise aucun flottant. Les flottants sont autorises seulement dans `puce16-desktop` (resampling audio, UI).
- D-010 | 2026-10-22 | Claude | Format framebuffer: XRGB8888 `u32`; conversion 5 bits -> 8 bits par replication des bits hauts: `c8 = (c5 << 3) | (c5 >> 2)`. Brillance INIDISP (provisoire, comme bsnes): sur le canal 5 bits, `c5' = c5 * b / 15` (division entiere) avec b = 0..15 (b=0 => noir, b=15 => inchange), appliquee AVANT la conversion 8 bits. A valider par ROM de test.
- D-011 | 2026-10-22 | Claude | Etat au power-on: WRAM remplie d'un motif deterministe (`0x55`) sauf demande de test; registres CPU/PPU selon specs; toute valeur non specifiee = question.
- D-012 | 2026-10-22 | utilisateur | Profil Hermes (assignee Kanban): `Puce16_Snes`. Board Kanban: `puce16`, tenant `puce16`.
- D-013 | 2026-10-22 | utilisateur | Depot Puce16: `H:\__Emulator__\Puce16_Snes` (workspace Kanban `dir:H:/__Emulator__/Puce16_Snes`). Documentation source: `H:\__Emulator__\Nintendo_Doc_Emulation`.
- D-014 | 2026-10-22 | utilisateur | Taille de tache: petite, ~50-150 lignes de code par tache (beaucoup de sous-etapes, plusieurs en parallele).
- D-015 | 2026-10-22 | utilisateur | Priorite: **desktop minimal tres tot** (fenetre + affichage des que le PPU sort une image) pour voir les progres; puis CPU/PPU/APU valides par ROMs de test.
- D-016 | 2026-10-22 | utilisateur | ROMs de test: **Hermes les telecharge** (tache dediee, dossier `tests/roms/` ignore par git, liste de sources dans specs/06_TEST_ROMS.md, URLs a confirmer: UNKNOWN - to confirm). Si le reseau est bloque -> question `tooling`.
- D-017 | 2026-10-22 | utilisateur | Desktop v1: save states + SRAM, manette (gamepad), debugger (registres + VRAM viewer), filtres/plein ecran.
- D-018 | 2026-10-22 | Claude (provisoire, a valider) | DSP precis a l'echantillon en v1 (un `run_sample` toutes les 32 cycles SMP, quirks documentes), pas de micro-ordonnancement 32 pas; tache de tuning ulterieure. Table gaussienne = donnee externe a obtenir (specs/07b section 8), jamais inventee.
- D-019 | 2026-10-22 | Claude (decoule de D-017, a valider) | Dependance `gilrs` autorisee dans `puce16-desktop` pour la manette. Raccourcis clavier sans touches chiffres (AZERTY). Detail: specs/08_INPUT_DESKTOP.md.
- D-020 | 2026-10-22 | utilisateur | **Execution strictement sequentielle: UNE tache a la fois** (pas de taches en parallele). L'ordre de declaration dans `docs/tasks_src/` est l'ordre d'execution; `build_tasks.py` chaine chaque tache a la precedente (Kanban: un seul parent). Les dependances declarees restent comme documentation des prerequis. Remplace "plusieurs taches en parallele" de D-014.
- D-021 | 2026-10-23 | utilisateur | Depot GitHub de sauvegarde: https://github.com/Farquaad56/Puce16_Snes (le jeton d'acces n'est JAMAIS ecrit dans la doc ni dans le depot; il est fourni hors-bande par l'utilisateur).
