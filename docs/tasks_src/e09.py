from _dsl import T

# E09: coprocesseurs. HORS v1 (D-005). Ces taches sont des PLACEHOLDERS: aucune specification n'existe encore.
# Elles ne sont jamais assignees a Hermes. Quand l'utilisateur decide de lancer l'un des coprocesseurs, Claude ecrit
# d'abord la specification (docs/specs/10_COPRO_<nom>.md) et decoupe la tache en sous-etapes de 50 a 150 lignes.

DEFER = ("HORS v1 (D-005). Placeholder: aucune specification n'existe. Ne pas assigner a Hermes. "
         "Quand l'utilisateur lance ce coprocesseur, Claude ecrit d'abord `docs/specs/10_COPRO_<nom>.md` "
         "(registres, timing, tables, jeux de test), la decoupe en taches de 50 a 150 lignes, puis marque cette tache comme remplacee.")

def D(id, title, what, order):
    T(id, title, kind="doc", deferred=True, size=0,
      context="%s Priorite %s. Perimetre: %s" % (DEFER, order, what),
      spec="Aucune action pour la v1. Quand le coprocesseur est lance: (1) specification par Claude, (2) acceptation d'en-tete `CartInfo.coprocessor` dans `cart/header.rs` (remplacer le refus `UnsupportedCoprocessor`), (3) crate ou module dedie `crates/puce16-core/src/coprocessor/<nom>.rs`, (4) integration au bus (mappage des plages $2200-$3FFF ou cartouche selon la puce), (5) tests unitaires sur les routines de la puce puis ROM de jeu connue.",
      write=[], tests=[], accept=[])

CHAIN9 = ("Execution strictement sequentielle (D-020): cette tache suppose que toutes les precedentes sont terminees. Ne modifie QUE les fichiers de 'Write only in'.")

T("E09-00", "TUNE: sprites decales d'une ligne (SMW: debris colores sur la ligne 0 en haut de l'image) - reporte apres la v1",
  kind="code", size=60, crate="puce16-core",
  context="""Bug trouve au GATE E07-14 (SMW, ecran de niveau et carte), accepte comme defaut connu et reporte ici a la demande de l'utilisateur (le gate E07-14 peut etre approuve sans ce correctif): de petits debris colores (vert, rouge, orange, blanc) apparaissent sur la toute premiere ligne de l'image (ligne de sortie 0, x environ 90 a 135). Diagnostic fait par Claude (non a deviner): SMW cache ses sprites inutilises en mettant Y=240 (OAM $F0, taille petite = 16x16 avec OBSEL de SMW). `crates/puce16-core/src/ppu/obj.rs` calcule la rangee du sprite avec `row = (r - y - 1) & 0xFF` (dans `sprite_list` ET dans `render_obj_line`, spec 05b section 6.2 ligne `row = (r - y - 1) & 0xFF`, avec `r` = ligne de sortie 0..223). Avec cette formule un sprite 16x16 a Y=240 couvre r=241..256, donc la rangee 15 revient se dessiner sur r=0: c'est l'artefact. Sur la console un sprite a Y=0 commence sur la premiere ligne visible (l'evaluation des sprites de la ligne v se fait pendant la ligne v-1, donc rangee = `(r - y) & 0xFF`, le « +1 » de la doc materielle est deja absorbe par `r = v - 1`). Experience faite en local puis ANNULEE: avec `row = (r - y) & 0xFF` dans les deux fonctions, la ligne 0 de SMW est propre (seule la colline de fond reste), mais ~30 tests e05_ changent car ils ont ete ecrits avec l'ancienne formule: `e05_10_row_visibility_y255`, `e05_10_row_visibility_y0`, `e05_10_large_height`, et la plupart des tests de `e05_11_sprite_pixels.rs` (ils utilisent y=255 comme « premiere rangee visible »), ainsi que le helper de `e05_14_pick.rs` (ligne 414).
Cette tache est donc une CORRECTION DE SPEC approuvee par la gate: la spec 05b et les tests e05_ qui l'encodent doivent etre alignes sur le comportement materiel. Ce n'est pas un affaiblissement de test: chaque test garde son intention (visibilite, hauteur, flips, limites 32/34), seul le Y utilise est decale d'une unite.""",
  reads=["specs/05b_PPU_RENDER.md (section 6, lignes 69-110 et 240)", "crates/puce16-core/src/ppu/obj.rs", "crates/puce16-core/tests/e05_10_sprite_list.rs", "crates/puce16-core/tests/e05_11_sprite_pixels.rs", "crates/puce16-core/tests/e05_14_pick.rs",
         "docs/qa/DECISIONS.md et docs/qa/FAQ_SNES.md (a consulter avant toute question)"],
  write=["crates/puce16-core/src/ppu/obj.rs", "specs/05b_PPU_RENDER.md",
         "crates/puce16-core/tests/e05_10_sprite_list.rs", "crates/puce16-core/tests/e05_11_sprite_pixels.rs", "crates/puce16-core/tests/e05_14_pick.rs",
         "crates/puce16-core/tests/e09_00_*.rs", "tests/baseline.json", "docs/qa/SUITE_REPORT.md", "tests/out/**", "docs/qa/inbox/** (questions uniquement)"],
  spec="""1. Reproduire (l'option `--input` du CLI est deja disponible): `cargo run --release -q -p puce16-cli -- run tests/roms/games/smw.sfc --frames 1500 --input "700-705:start,800-805:a,900-905:a,1000-1005:a,1100-1105:a,1200-1205:a,1300-1305:a" --dump-ppm tests/out/e09_00_smw.ppm`. Avant correction, la ligne 0 de l'image contient des pixels non fond (x 90..135) en plus de la colline bleue (x >= 151). L'exemple `cargo run --release -p puce16-core --example smw_oam -- tests/roms/games/smw.sfc 1500` liste les sprites a Y=240 responsables.
2. Corriger `obj.rs`: `row = ((r as i32) - (s.y as i32)) & 0xFF` dans `sprite_list` ET dans `render_obj_line` (les deux doivent rester identiques; en faire une petite fonction commune `sprite_row(r, y)` pour qu'elles ne divergent plus).
3. Mettre a jour la spec 05b: section 6.2 (formule et commentaire: `Y = 0` -> premiere rangee visible, `Y = 255` -> la rangee 1 du sprite sur la ligne 0, un sprite 16x16 a `Y = 240` n'atteint PAS la ligne 0, un sprite de hauteur 32 ou 64 deborde bien par le haut), et la ligne 240 (liste des tests: remplacer `Y=255 visible a la rangee 0; Y=0 non visible a la rangee 0` par la nouvelle regle). Ajouter une ligne d'historique dans la spec indiquant que la formule a ete corrigee par E09-00 et pourquoi (SMW cache ses sprites a Y=240).
4. Mettre a jour les tests e05_10, e05_11 et e05_14: decaler le Y des sprites de test d'une unite (255 -> 0 pour « premiere rangee visible », etc.) et ajuster les attentes de visibilite. Chaque test garde sa signification d'origine; ne supprimer aucune assertion. Si un test devient impossible a adapter sans en changer le sens, question bloquante (`docs/qa/inbox`).
5. Test de non-regression `e09_00_*` (nouveau fichier) avec valeurs concretes, qui ECHOUE avant la correction: sprite 16x16 a Y=240 invisible sur la ligne 0 (et sur toutes les lignes 0..223); sprite 8x8 a Y=0 visible sur les lignes 0..7 avec rangee = ligne; sprite 32x32 a Y=240 visible sur les lignes 0..15 (rangees 16..31).
6. Relancer la suite: `target/release/puce16-cli suite tests/manifest.toml`. Les ROM avec sprites changeront de hash (decalage d'une ligne): regarder les images `tests/out/suite/*.ppm` des ROM de sprites avant de refiger, puis `--update-baseline <nom>` pour chacune. Noter dans `docs/qa/SUITE_REPORT.md` la liste des ROM re-figees. Si une ROM de test sprite (ex: celles de E05) devient fausse visuellement, STOP et question bloquante: la formule est alors a reconsiderer.
7. Fin de tache: la ligne 0 de la capture SMW ne contient plus de debris (voir `accept`). L'utilisateur reverifie SMW dans le desktop apres la correction.""",
  tests=[("sprite_16x16_y240_hidden_on_all_rows", "16x16 a Y=240: aucune ligne de sortie 0..223 ne contient le sprite (regression SMW)"),
         ("sprite_y0_first_visible_row", "8x8 a Y=0: visible sur les lignes 0..7, ligne 0 = rangee 0"),
         ("sprite_32x32_y240_wraps_to_top", "32x32 a Y=240: visible sur les lignes 0..15 avec les rangees 16..31 (debordement materiel)"),
         ("sprite_list_and_render_use_same_row", "sprite_list et render_obj_line donnent la meme rangee pour tous les Y de 0 a 255")],
  accept=["cargo build --release -p puce16-cli",
          "cargo test -p puce16-core e09_00_",
          "cargo test -p puce16-core e05_",
          "cargo run --release -q -p puce16-cli -- run tests/roms/games/smw.sfc --frames 1500 --input 700-705:start,800-805:a,900-905:a,1000-1005:a,1100-1105:a,1200-1205:a,1300-1305:a --dump-ppm tests/out/e09_00_smw.ppm",
          "python -c \"import sys; d=open('tests/out/e09_00_smw.ppm','rb').read(); o=len(d)-256*239*3; r=d[o:o+150*3]; sys.exit(0 if all(r[i:i+3]==bytes([255,231,181]) for i in range(0,len(r),3)) else 1)\""],
  notes=CHAIN9)

D("E09-01", "Coprocesseur DSP-1 (Super Mario Kart, Pilotwings)",
  "calculs mathematiques 16 bits (rotation, projection, perspective), ROM de donnees interne 2 Kio + 1 Kio, commandes via un registre de donnees/statut a deux adresses (LoROM: $30-$3F:$8000-$FFFF ou $6000/$7000 selon la carte). Variantes DSP-1/1A/1B, DSP-2/3/4 plus tard.", "1")
D("E09-02", "Coprocesseur Super FX / GSU (Star Fox, Yoshi's Island)",
  "CPU RISC 16 bits propre (GSU-1/GSU-2), 16 registres, cache d'instructions 512 octets, trace de pixels dans la RAM de cartouche, double acces ROM/RAM avec le S-CPU (RON/RAN), interruption IRQ vers le S-CPU.", "2")
D("E09-03", "Coprocesseur SA-1 (Super Mario RPG, Kirby Super Star)",
  "second 65C816 a 10,74 MHz avec BW-RAM, I-RAM, DMA caracteres, multiplication/division materielles, tampon de bits variable, timers, arbitrage du bus avec le S-CPU, interruptions croisees.", "3")
D("E09-04", "Decompresseur S-DD1 (Star Ocean, Street Fighter Alpha 2)",
  "decompression a la volee des donnees ROM entre le bus et le S-CPU, 4 banques de ROM remappables, deux canaux DMA interceptes.", "4")
D("E09-05", "Decompresseur SPC7110 (Far East of Eden Zero, Momotaro Dentetsu Happy)",
  "decompression, ROM de donnees, horloge temps reel EPSON RTC-4513 optionnelle, unite de multiplication/division, mappage de banques.", "5")
D("E09-06", "Coprocesseur Cx4 (Mega Man X2, X3)",
  "processeur Hitachi HG51B: fonctions graphiques et trigonometriques (wireframe, rotations, tables), programme interne en ROM.", "6")
D("E09-07", "Autres puces: OBC1, S-RTC, ST010/ST011/ST018, MSU-1, BS-X",
  "puces a part, a traiter une par une selon la demande: OBC1 (gestion OAM), S-RTC (horloge), ST010/011 (IA, mathematiques), ST018 (ARM), MSU-1 (audio/donnees streaming, non officielle), satellaview BS-X.", "7")
