# QA_LOG (append-only - ne jamais reecrire une entree)

Format:
```
## <date> <ID-question> | tache <TASKID> | auteur <nom>
Q: ...
A: ...
Decision: D-nnn ou "aucune"
```

## 2026-10-22 Q-001 | tache E00 (cadrage) | auteur Claude
Q: Quelle frequence APU utiliser: nominale (24 576 000 Hz, DSP 32 000 Hz) ou mesuree (~24 606 720 Hz, DSP 32 040 Hz)?
A: PROPOSITION (a valider par l'utilisateur): mesuree, 24 606 720 Hz, comme bsnes. Constante unique `APU_RESONATOR_HZ`, facile a changer.
Decision: D-004 (provisoire)

## 2026-10-22 Q-002 | tache E00 (cadrage) | auteur Claude
Q: Quels dots de la ligne durent 5 (ou 6) clocks?
A: PROPOSITION: wiki = 336 dots de 4 clocks + 4 dots de 5 clocks (total 1364). Emplacement exact non precise dans le wiki lu -> placement provisoire H = 323, 324, 327, 328 (D-008). A confirmer par un test materiel (ex: rapport de lecture OPHCT).
Decision: D-008 (provisoire)

## 2026-10-22 Q-003..Q-009 | tache E00 (cadrage) | auteur utilisateur (reponses)
Q-003 PPU: dot par dot (pixel-accurate) -> D-002 (remplace scanline).
Q-001 APU: mesuree 24 606 720 Hz -> D-004 confirme.
Q-004 Assignee: Puce16_Snes -> D-012.
Q-005 Taille des taches: petite 50-150 lignes -> D-014.
Q-006 Depot: H:\__Emulator__\Puce16_Snes -> D-013.
Q-007 Priorite: desktop minimal tres tot -> D-015.
Q-008 ROMs de test: Hermes les telecharge -> D-016.
Q-009 Desktop v1: save states+SRAM, gamepad, debugger, filtres/plein ecran -> D-017.
Note: le Q&R est un processus de cadrage fait MAINTENANT avec l'utilisateur (et reste disponible pendant la construction via 02_QA_PROTOCOL.md).

## 2026-10-25 E02-31-Q1 | tache E02-31 | auteur Claude
Q: In emulation mode, what should opcode AB (PLB) do? The code currently remaps AB to DA (PHX) based on a comment citing the WDC datasheet. However, the oracle test expects PLB behavior (pull from stack into DB register). Should the remapping be removed so AB always executes PLB in both modes?
A: (choix A) A: no remap. AB always executes PLB in both modes (oracle authoritative). Fixed by pull8n + fix_stack_e; oracle 512/512 ok.
Decision: D-022
