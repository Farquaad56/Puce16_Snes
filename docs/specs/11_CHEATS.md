# 11 - Triches: Pro Action Replay (PAR)

Specification de l'epic E10. Perimetre v1 des triches: **codes Pro Action Replay bruts `AAAAAAVV`** uniquement.
Le Game Genie est HORS perimetre (placeholder E10-08).

## 1. Perimetre et sources

- Un code PAR est une ecriture forcee: l'octet `VV` est ecrit a l'adresse CPU 24 bits `AAAAAA`, une fois par image.
- Les codes decodes (format `AAAAAAVV`) sont publies sur gamehacking.org, rubrique Super Nintendo; l'utilisateur les recopie tels quels. Aucun code n'est invente dans le depot: les tests utilisent des valeurs arbitraires, jamais « le code d'un jeu ».
- Hors perimetre: codes qui modifient la ROM, codes a plusieurs lignes avec condition, Game Genie, import de bases de donnees.

## 2. Format d'un code

- Exactement 8 chiffres hexadecimaux (majuscules ou minuscules), sans separateur: `AAAAAA` (adresse 24 bits `bank:offset`) puis `VV` (valeur 8 bits). Exemple: `7E0DBF99` = ecrire `0x99` en `$7E:0DBF`.
- Les espaces au debut et a la fin sont ignores.
- Un texte contenant un tiret `-` est refuse avec l'erreur `LooksLikeGameGenie` (les codes Game Genie `XXXX-XXXX` n'utilisent que des chiffres hexadecimaux et seraient pris a tort pour des codes PAR).
- Erreurs (enum `ParseError`): `Length(usize)` (nombre de caracteres apres nettoyage, different de 8), `NotHex(char)` (premier caractere invalide), `LooksLikeGameGenie`.
- Affichage canonique (`Display` de `ParCode`): `{:06X}{:02X}` (majuscules, 8 caracteres).

## 3. Fichier de triches (.cht)

Fichier texte UTF-8, une triche par ligne:

```
# commentaire (ligne ignoree)
+ Vies infinies = 7E0DBE05
- Invincible = 7E1497FF 7E1490FF
```

- `+` = triche active, `-` = inactive (premier caractere non blanc de la ligne).
- Le nom est le texte entre le signe et le premier `=`, sans les blancs de debut et de fin; il ne doit pas etre vide et ne contient pas `=`.
- Apres le `=`: un ou plusieurs codes PAR separes par des espaces ou des virgules. Au moins un code.
- Les lignes vides et les lignes qui commencent par `#` sont ignorees. Fin de ligne `\n` ou `\r\n` acceptee en lecture; ecriture avec `\n`.
- Toute ligne invalide fait echouer la lecture entiere: `CheatFileError { line: usize (1 = premiere ligne), kind: CheatFileErrorKind }` avec `kind` parmi `MissingSign`, `MissingEquals`, `EmptyName`, `NoCode`, `BadCode(ParseError)`.
- Ecriture (`format_cheat_file`): un en-tete `# Puce16 cheats` puis une ligne par triche: `<+|-> <nom> = <code> <code>...` (codes au format canonique separes par un espace), chaque ligne terminee par `\n`. `parse(format(x)) == x` pour toute liste valide.

## 4. Application sur la console

- Les codes actifs sont appliques **une fois par image**, a la fin de `Snes::run_frame`, juste apres la detection du debut du VBlank (ils sont donc visibles par les lectures de l'image suivante).
- Une application est une ecriture silencieuse: zero cycle, aucune modification du temps, du `mdr` ni d'un registre. Ordre d'application = ordre de la liste (le dernier gagne).
- Routage de l'adresse `AAAAAA` = `bank:addr` (`bank = A >> 16`, `addr = A & 0xFFFF`):

| Cible | Condition | Effet |
|---|---|---|
| WRAM | `bank` 00-3F ou 80-BF, `addr` < 0x2000 | WRAM[addr] |
| WRAM | `bank` = 7E | WRAM[addr] |
| WRAM | `bank` = 7F | WRAM[0x10000 + addr] |
| Cartouche (SRAM) | `bank` 00-3F ou 80-BF avec `addr` >= 0x6000, ou `bank` 40-7D, ou `bank` C0-FF | `cart.write(bank, addr, v)` (la cartouche decide: SRAM ecrite, ROM ignoree) |
| Rien | tout le reste (registres PPU/APU/DMA/MMIO $2000-$5FFF, etc.) | ignore |

- Les triches ne font PAS partie de l'etat sauvegarde: ni dans `save_state`/`load_state`, ni dans `state_hash`. Sans triche, le comportement et tous les hashs existants sont inchanges.
- La liste de triches de `Snes` survit a `reset()` et a `load_state()`.

## 5. Interface Rust (puce16-core)

```rust
// crates/puce16-core/src/cheat/mod.rs : pub mod par; pub mod list;  (+ re-exports)
pub struct ParCode { pub addr: u32, pub value: u8 }            // addr sur 24 bits
pub enum ParseError { Length(usize), NotHex(char), LooksLikeGameGenie }
pub fn parse_par(s: &str) -> Result<ParCode, ParseError>
pub struct CheatEntry { pub name: String, pub enabled: bool, pub codes: Vec<ParCode> }
pub enum CheatFileErrorKind { MissingSign, MissingEquals, EmptyName, NoCode, BadCode(ParseError) }
pub struct CheatFileError { pub line: usize, pub kind: CheatFileErrorKind }
pub fn parse_cheat_file(text: &str) -> Result<Vec<CheatEntry>, CheatFileError>
pub fn format_cheat_file(entries: &[CheatEntry]) -> String
// Snes
pub fn set_cheats(&mut self, codes: &[ParCode])
pub fn cheats(&self) -> &[ParCode]
```

Tous ces types derivent `Clone, Debug, PartialEq, Eq`.

## 6. Ligne de commande

`puce16-cli run <rom> --cheat <fichier.cht> --cheat-code <AAAAAAVV>` (`--cheat-code` repetable). Les codes des entrees `+` du fichier, puis les `--cheat-code`, sont passes a `set_cheats` avant la premiere image. Une erreur de fichier ou de code arrete la commande avec un message clair (`erreur cheat: ...`) et un code de sortie non nul.

## 7. Desktop

- Fichier de triches d'une ROM: meme dossier et meme nom que la ROM avec l'extension `.cht` (comme `.srm`).
- Au chargement d'une ROM: lecture du `.cht` (absent = liste vide), envoi des codes actifs a la console par `Command::SetCheats(Vec<ParCode>)`. La console garde la liste apres un `Reset` et un rechargement de la console.
- Menu `Triches` > `Gerer les triches...`: fenetre avec la liste (case a cocher, nom, codes, bouton supprimer), un champ nom, un champ codes, un bouton Ajouter. Chaque modification renvoie les codes actifs et reecrit le `.cht` (ecriture atomique).

## 8. Plan de test

- E10-01: analyse d'un code (cas valides, longueurs, caracteres invalides, tiret, affichage).
- E10-02: lecture/ecriture du fichier (cas valides, chaque erreur avec son numero de ligne, aller-retour).
- E10-03: console synthetique: miroir WRAM bas, bank 7E, bank 7F, SRAM, ROM ignoree, registre PPU ignore, application a chaque image, ordre (le dernier gagne), hash inchange sans triche.
- E10-04: option CLI (fonction pure de collecte des codes).
- E10-05/06: fichiers et modele de la fenetre (fonctions pures testees; le dessin egui est seulement compile).
- E10-07: gate utilisateur avec de vrais codes de gamehacking.org.
