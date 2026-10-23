# SPEC 03 - DMA et HDMA

Source: wiki SNESdev (DMA_registers, Timing). Valeurs UNKNOWN avec defaut.

## 1. Registres (8 canaux, n = 0..7, adresse $43n0-$43nF)

| Reg | Addr | Power-on | Description |
|---|---|---|---|
| DMAPn | $43n0 | $FF | `DI.A APPP`: D=direction (0: A->B, 1: B->A); I=HDMA indirect; bit5 inutilise; A (bits 4-3)=ajustement adresse A (0 incremente; 2 decremente; 1 ou 3 fixe); P (bits2-0) = pattern |
| BBADn | $43n1 | $FF | adresse B-bus: $2100 + valeur |
| A1TnL/H/B | $43n2/3/4 | $FFFFFF | adresse A-bus source (DMA) ou debut de table (HDMA) |
| DASnL/H | $43n5/6 | $FFFF | nombre d'octets (DMA; 0 = 65536) ou adresse indirecte (HDMA) |
| DASBn | $43n7 | $FF | banque de l'adresse indirecte HDMA |
| A2AnL/H | $43n8/9 | $FFFF | adresse courante dans la table HDMA (banque = A1Bn) |
| NLTRn | $43nA | $FF | HDMA: `RLLL LLLL` flag repetition + compteur de lignes |
| UNUSEDn | $43nB et $43nF | $FF | un octet partage, lecture/ecriture sans effet |
| (autres) | $43nC-$43nE | - | open bus |

Tous les registres $43xx sont lisibles et ecrivables (acces a 6 clocks).

## 2. Patterns de transfert (octets envoyes a BBADn + offset)

| P | Octets par unite | Offsets B-bus | HDMA: octets par entree |
|---|---|---|---|
| 0 | 1 | 0 | 1 |
| 1 | 2 | 0,1 | 2 |
| 2 | 2 | 0,0 | 2 |
| 3 | 4 | 0,0,1,1 | 4 |
| 4 | 4 | 0,1,2,3 | 4 |
| 5 | 4 | 0,1,0,1 | 4 |
| 6 | 2 | 0,0 (= 2) | 2 |
| 7 | 4 | 0,0,1,1 (= 3) | 4 |

L'offset B est applique a BBADn: adresse B = $2100 + ((BBADn + offset) & 0xFF).

## 3. DMA general (MDMAEN $420B)

Ecriture non nulle: les canaux dont le bit est a 1 s'executent dans l'ordre 0..7. Le CPU est arrete pendant toute la duree.

Pour chaque canal actif, tant que le compteur n'est pas 0:
1. Lire un octet (source selon direction), ecrire a destination.
   - Direction A->B: lire A-bus `(A1Bn:A1Tn)`, ecrire B-bus `$2100 + ...`.
   - Direction B->A: lire B-bus, ecrire A-bus.
2. Passer a l'octet suivant du pattern (offset B suivant, cyclique sur la longueur du pattern).
3. Ajuster A1Tn (16 bits, la banque ne change jamais): mode 0: +1; mode 2: -1; modes 1 et 3: inchange.
4. Decrementer DASn (16 bits). A 0 -> fin du canal. Le transfert s'arrete meme au milieu d'un pattern.

Apres le canal: DASn = 0, A1Tn = adresse apres l'avance (wrap 16 bits dans la banque). Compteur initial 0 = 65536 octets.

Restrictions:
- L'adresse A-bus ne peut pas viser $2100-$21FF, $4000-$41FF, $4200-$421F, $4300-$437F (banques 00-3F/80-BF): ces acces sont ignores (lecture = open bus; ecriture = rien). `UNKNOWN - to confirm` le detail: defaut = lecture open bus (`mdr`), ecriture ignoree.
- DMA WRAM->$2180 : la lecture A-bus en WRAM n'a pas d'effet utile (rien n'est ecrit, adresse WMADD non incrementee). DMA $2180->WRAM: ecrit open bus; WMADD non incremente. (wiki)
- BBADn = $00 sur certaines revisions de CPU: ne pas emuler la panne; documenter seulement.
- DMA ne peut pas copier WRAM -> WRAM.

### 3.1 Timing DMA (cycle-accurate, valeurs par defaut)
- L'ecriture de MDMAEN fixe un drapeau "DMA en attente". Le DMA demarre **apres la lecture de l'opcode de l'instruction suivante** (le CPU ne se fait pas interrompre au milieu d'un cycle).
- Au demarrage: temps maitre aligne sur le multiple de 8 clocks suivant (`master_phase`; `UNKNOWN - to confirm` l'origine: defaut = origine de la ligne 0 de la frame), puis **8 clocks de surcout**.
- Par canal actif: **8 clocks** de surcout de canal.
- Par octet transfere: **8 clocks**.
- A la fin: **le CPU reprend apres un alignement sur son prochain cycle** (6/8/12 clocks selon l'adresse du cycle suivant): `UNKNOWN - to confirm`; defaut: pas de realignement supplementaire.
- HDMA peut interrompre un DMA general: le DMA est mis en pause pendant l'execution de HDMA (voir 4).
- Les composants PPU/APU/timers avancent normalement pendant le DMA (via `catch_up`). Le CPU ne s'execute pas.

## 4. HDMA (HDMAEN $420C)

HDMA est actif sur les canaux dont le bit HDMAEN est a 1.

### 4.1 Initialisation (debut de chaque frame)
A V=0, H ~ debut de ligne (`HDMA_INIT_HCLOCK` defaut 12 clocks, `UNKNOWN - to confirm`), pour chaque canal actif:
1. `A2An = A1Tn` (16 bits bas).
2. Lire l'octet de ligne `NLTRn` depuis `(A1Bn:A2An)`, `A2An++`.
3. Si NLTRn == $00: le canal est termine pour la frame (flag `done`).
4. Mode indirect (DMAPn bit6): lire 2 octets suivants `(A1Bn:A2An)` -> `DASn` (lo, hi), `A2An += 2`; banque indirecte = DASBn.
5. Premier transfert (voir 4.2) avec le meme flag "do transfer".

### 4.2 Table et transfert (chaque scanline, V = 0..derniere ligne visible-1)
Octet de ligne `NLTRn` = `R LLLLLLL`:
- `$00`: fin de la table; arret du canal pour la frame.
- `$01-$80` (R=0): **une** ecriture (pattern complet), puis attendre L lignes (L = 1..128; $80 = 128).
- `$81-$FF` (R=1): une ecriture **a chaque ligne** pendant L-$80... : nombre de lignes = (valeur & 0x7F); une ecriture par ligne.
  (Le wiki: `$81-$FF` = X-$80 lignes en mode repete.)

A chaque scanline (fin de la partie active, juste apres la zone visible; `HDMA_RUN_HCLOCK` defaut = 1104 clocks, `UNKNOWN - to confirm`), pour chaque canal actif non termine:
1. Si "do transfer" (compteur de ligne = debut d'entree ou flag repetition): transferer `octets du pattern` (table 2) depuis:
   - direct: `(A1Bn:A2An)`, `A2An += n`;
   - indirect: `(DASBn:DASn)`, `DASn += n`;
   vers B-bus (`BBADn + offset`). (Direction bit D=1 interdit en HDMA: comportement `UNKNOWN`, ne pas implementer.)
2. Decrementer le compteur de lignes (`NLTRn & 0x7F`).
3. Si le compteur atteint 0: lire le prochain octet de ligne (comme 4.1 items 2-4), etablir le flag "do transfer" = vrai; si l'octet vaut 0: canal termine.
   Sinon: flag "do transfer" = vrai seulement si R=1 (repetition).
- La premiere entree est executee a la fin de la scanline 0. L'entree pour la ligne N doit donc etre prevue avec le compteur de l'entree precedente.
- Dernier cycle: les canaux HDMA termines ne font plus rien jusqu'a l'initialisation suivante.

### 4.3 Timing HDMA (defauts)
- Surcout par scanline si au moins un canal HDMA actif: 18 clocks (`UNKNOWN - to confirm`) + 8 clocks par canal actif + 8 clocks par octet transfere.
- HDMA a priorite sur DMA general et sur le CPU: execute a des dates fixes de la ligne.
- HDMA ecrit meme pendant le forced blanking (INIDISP bit7 ne le coupe pas).

### 4.4 Erreurs connues a NE PAS reproduire en v1
- Plantage CPU rev2 si un DMA se termine juste quand HDMA commence; BBADn=0 sur CPU rev2. Ne pas emuler.
