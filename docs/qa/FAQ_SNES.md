# FAQ SNES (pre-repondue - Hermes la lit avant de poser une question)

Chaque reponse est issue du wiki SNESdev (reference du projet) ou marquee `UNKNOWN - to confirm`.

**Q: Combien de master clocks dure un cycle CPU?**
R: 6, 8 ou 12. 6 = ROM rapide (banque >= $80 avec MEMSEL bit0=1), la plupart des MMIO ($21xx, $42xx, $43xx), et cycles internes. 8 = ROM lente, WRAM ($7E-$7F et miroirs), SRAM. 12 = $4016/$4017 (JOYSER).
Detail et table complete: specs/02_BUS_TIMING.md.

**Q: Taille d'une scanline et d'une frame?**
R: 1364 clocks (340 dots), NTSC 262 lignes (357 368 clocks), PAL 312 lignes (425 568). Ligne courte NTSC (entrelace off, field=1, V=240): 1360. Ligne longue PAL (entrelace on, field=1, V=311): 1368. Le CPU est arrete 40 clocks par ligne (rafraichissement DRAM).

**Q: Ou commence le VBlank?**
R: V=225 (V=240 si overscan SETINI bit2). Fin apres V=261 (NTSC) / 311 (PAL). Flag RDNMI ($4210 bit7) mis au debut du VBlank, efface a la lecture.

**Q: Le vecteur de reset?**
R: $00:FFFC, toujours en mode emulation. Apres reset: E=1, M=1, X=1, I=1, D=0, S=$01xx, DB=0, PB=0, D(direct page)=0. Voir specs/01_CPU_65816.md.

**Q: Que fait une division par zero ($4204-$4206)?**
R: Quotient = $FFFF, reste = dividende.

**Q: Que faire si une adresse ou un bit n'est dans aucune spec?**
R: Ecrire `UNKNOWN - to confirm`, poser une question (02_QA_PROTOCOL.md). Ne jamais deviner.

**Q: Quelle est la forme d'un test?**
R: `eNN_MM_<nom>`, dans `crates/puce16-core/tests/eNN_MM_*.rs`, lance par `cargo test eNN_MM_ -p puce16-core`.

**Q: Ou ecrire un nouveau fichier non liste dans mon scope?**
R: Nulle part. Question de type `decision`.

**Q: Les flottants sont-ils permis?**
R: Non dans `puce16-core` (D-009). Oui seulement dans `puce16-desktop`.

**Q: Les ROM de test sont ou?**
R: Pas dans le depot. `tests/roms/` est ignore par git. La liste et les noms sont dans specs/06_TEST_ROMS.md; l'utilisateur les place a la main.
