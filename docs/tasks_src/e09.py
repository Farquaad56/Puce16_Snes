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
