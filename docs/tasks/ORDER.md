# ORDER - ordre d'execution strict (genere)

D-020: UNE seule tache a la fois, dans cet ordre. Une tache n'est prete que quand la precedente est `done` (et approuvee si GATE).


## E01
  1. E01-01 Workspace Cargo et squelette de puce16-core
  2. E01-02 Crates puce16-cli et puce16-desktop (manifestes + main minimal)
  3. E01-03 consts.rs et region.rs (constantes d'horloge et de ligne)
  4. E01-04 hash.rs: Fnv64 (hash d'etat deterministe)
  5. E01-05 FrameBuf 512x478 et FrameView compacte
  6. E01-06 Snes de test (motif degrade) pour le desktop minimal D0
  7. E01-07 puce16-cli run: motif de test, --hash, --dump-ppm
  8. E01-08 Helper de test make_rom (ROM synthetiques)
  9. E01-09 Desktop: thread d'emulation (Command / Event) sur le motif de test
 10. E01-10 Desktop: video.rs (conversion XRGB -> RGBA et calcul de taille)
 11. E01-11 Desktop: fenetre eframe affichant le motif de test (jalon D0)
 12. E01-12 joypad.rs: constantes de boutons et filtre des directions opposees
 13. E01-13 Desktop: clavier AZERTY (input.rs) et raccourcis (jalon D1)
 14. E01-14 GATE J0: le desktop affiche le motif et reagit au clavier  [GATE]

## E02
 15. E02-01 T-ORACLE: telecharger l'oracle SingleStepTests 65816
 16. E02-02 T-FORMAT: documenter le format JSON exact de l'oracle
 17. E02-03 GATE: Claude relit ssjson_format.md et valide le format  [GATE]
 18. E02-04 gen_tables.py: cpu/table.rs (enums Mn, Am et tables MN, AM)
 19. E02-05 Cpu: registres, trait Bus, helpers, step et squelettes de familles
 20. E02-06 testbus.rs: FlatBus (bus plat journalise)
 21. E02-07 addr.rs: Ea, read_data/write_data, read_operand, rmw et squelettes de modes
 22. E02-08 addr/absolute.rs: modes Abs, AbsX, AbsY, Long, LongX
 23. E02-09 addr/direct.rs: modes Dp, DpX, DpY, Sr
 24. E02-10 addr/indirect.rs: modes (d) [d] (d,x) (d),y [d],y (d,s),y
 25. E02-11 CLI: enregistrer les sous-commandes cpu-test, spc-test et suite (squelettes)
 26. E02-12 cpu-test: chargeur des fichiers JSON de l'oracle
 27. E02-13 cpu-test: comparaison d'etats et de cycles (fonctions pures)
 28. E02-14 cpu-test: commande complete (boucle sur les fichiers, rapport, code de sortie)
 29. E02-15 mem/load.rs: LDA, LDX, LDY (tous modes)
 30. E02-16 mem/store.rs: STA, STX, STY, STZ (tous modes)
 31. E02-17 VERIF oracle: modes d'adressage via LDA/LDX/LDY/STA/STX/STY/STZ
 32. E02-18 alu/adc.rs: ADC binaire (8 et 16 bits, tous modes)
 33. E02-19 alu/adc.rs: ADC decimal (BCD 8 et 16 bits)
 34. E02-20 alu/sbc.rs: SBC binaire (8 et 16 bits, tous modes)
 35. E02-21 alu/sbc.rs: SBC decimal (BCD 8 et 16 bits)
 36. E02-22 alu/logic.rs: AND, ORA, EOR, BIT
 37. E02-23 alu/cmp.rs: CMP, CPX, CPY
 38. E02-24 alu/incdec.rs: INC, DEC, INX, INY, DEX, DEY
 39. E02-25 alu/shift.rs: ASL, LSR, ROL, ROR
 40. E02-26 alu/tsbtrb.rs: TSB, TRB
 41. E02-26a VERIF oracle ALU 1/5: AND, ORA, EOR
 42. E02-26b VERIF oracle ALU 2/5: BIT, TSB, TRB, CMP, CPX, CPY
 43. E02-26c VERIF oracle ALU 3/5: INC, DEC, INX, INY, DEX, DEY, ASL, LSR, ROL, ROR
 44. E02-26d VERIF oracle ALU 4/5: ADC (binaire et decimal)
 45. E02-27 VERIF oracle ALU 5/5: SBC puis toute la famille ALU
 46. E02-28 mem/push.rs: PHA PHX PHY PHP PHD PHB PHK PEA PEI PER
 47. E02-29 mem/pull.rs: PLA PLX PLY PLP PLD PLB
 48. E02-30 mem/block.rs: MVN, MVP
 49. E02-30a VERIF oracle pile 1/4: PHA, PHX, PHY, PHP, PHB, PHK
 50. E02-30b VERIF oracle pile 2/4: PHD, PLD, PEA, PEI, PER
 51. E02-30c VERIF oracle pile 3/4: PLA, PLX, PLY, PLP, PLB, MVN, MVP
 52. E02-31 VERIF oracle pile 4/4: toute la famille pile et blocs
 53. E02-32 flow/branch.rs: Bxx, BRA, BRL
 54. E02-33 flow/jump.rs: JMP et JML (tous modes)
 55. E02-34 flow/call.rs: JSR, JSL, RTS, RTL
 56. E02-35 flow/sys.rs: BRK, COP, RTI, WAI, STP, WDM
 57. E02-36 VERIF oracle: controle de flux (Bxx, JMP, JSR, RTS, BRK, RTI...)
 58. E02-37 misc/transfer.rs: TAX TAY TXA TYA TSX TXS TXY TYX TCD TDC TCS TSC
 59. E02-38 misc/flags.rs: CLC SEC CLI SEI CLD SED CLV REP SEP XCE XBA NOP
 60. E02-39 VERIF oracle: transferts, drapeaux, XCE/XBA/NOP
 61. E02-40 interrupts.rs: poll (NMI/IRQ/WAI/STP), sequence d'interruption et RESET
 62. E02-41 GATE J1: tous les opcodes 65816 passent l'oracle  [GATE]

## E03
 63. E03-01 cart/header.rs: types, en-tete interne, checksum, region, SRAM
 64. E03-02 cart/mapper.rs: formules LoROM / HiROM / ExHiROM (fonctions pures)
 65. E03-03 cart/header.rs: score_header et choix de l'en-tete
 66. E03-04 cart/mod.rs: strip_copier_header et normalize (ROM en puissance de 2)
 67. E03-05 Cart: from_bytes / from_bytes_forced et accesseurs
 68. E03-06 Cart: read / write / load_sram
 69. E03-07 ppu/dot.rs: tables de dots et types de ligne
 70. E03-08 ppu/mod.rs: Ppu (compteurs v/dot/field, step_dot, evenements)
 71. E03-09 ppu/mod.rs: registres $2100 et $2133 (minimal), read, hash
 72. E03-10 bus/wram.rs: WRAM 128 Kio et registres $2180-$2183
 73. E03-11 joypad.rs: ports $4016/$4017 et valeurs d'auto-read
 74. E03-12 apu/mod.rs: squelette Apu (ports $2140-$2143)
 75. E03-13 bus/mod.rs: access_clocks (6/8/12 clocks par acces)
 76. E03-14 bus/mmio.rs: struct Mmio, power-on, ecritures $4200-$420D
 77. E03-15 bus/mmio.rs: unite math (multiplication 8x8, division 16/8) et lecture $4214-$4217
 78. E03-16 bus/mmio.rs: lecture RDNMI, TIMEUP, HVBJOY, RDIO, JOYn
 79. E03-17 dma/mod.rs: Channel, Dma::new et registres $4300-$437F
 80. E03-18 dma/mod.rs: trait DmaBus et fonctions de pattern
 81. E03-19 dma/mod.rs: run_general (A -> B), patterns, ajustement A, temps
 82. E03-20 dma/mod.rs: run_general sens B -> A et cas WRAM <-> $2180
 83. E03-21 dma/hdma.rs: hdma_init (debut de frame)
 84. E03-22 dma/hdma.rs: hdma_run_line (un transfert par scanline)
 85. E03-23 timing/mod.rs: Timing (ligne, refresh, VBlank/NMI) + banc de test Rig
 86. E03-24 timing/mod.rs: IRQ H/V
 87. E03-25 timing/mod.rs: auto-read joypad et declencheurs HDMA
 88. E03-26 bus/mod.rs: struct SysBus (possession des composants), new, reset, hash
 89. E03-27 bus/route.rs: read_inner (routage des lectures, open bus)
 90. E03-28 bus/route.rs: write_inner (routage des ecritures et effets)
 91. E03-29 bus/cycle.rs: avance du temps, catch_up et impl cpu::Bus
 92. E03-30 bus/cycle.rs: rafraichissement DRAM (40 clocks par ligne)
 93. E03-31 bus/mod.rs: impl DmaBus pour SysBus
 94. E03-32 bus/cycle.rs: execution du DMA general (apres le cycle suivant MDMAEN)
 95. E03-33 bus/cycle.rs: execution HDMA (init et run) en debut de cycle
 96. E03-34 snes.rs: Snes reel (new_with, reset) a cote du motif de test
 97. E03-35 snes.rs: run_frame / run_master / step_instruction / hash / SRAM / debug (Snes reel)
 98. E03-36 tests/common/prog.rs et test de demarrage (boot)
 99. E03-37 Integration: NMI (compteur par frame, RDNMI, activation en VBlank)
100. E03-38 Integration: IRQ H/V (modes 1, 2, 3, masque I, acquittement)
101. E03-39 Integration: DMA general ROM -> WRAM par $2180 et relecture
102. E03-40 Integration: HDMA vers $2180 (table directe, repetition)
103. E03-41 Integration: auto-read des manettes ($4218-$421F)
104. E03-42 Integration: unite de multiplication/division et determinisme
105. E03-43 Integration: SRAM, open bus et valeurs de power-on des registres
106. E03-44 CLI: puce16-cli run <rom> (vraie ROM, --state-hash, --mapper, --debug)
107. E03-45 Desktop: files.rs (extensions ROM, extraction d'un .zip, lecture d'un fichier ROM)
108. E03-46 Desktop: emu_thread LoadRom, Reset reel, Event::Error
109. E03-47 Desktop: ouvrir une ROM (menu, Ctrl+O, glisser-deposer, argument de ligne de commande)
110. E03-48 GATE J2: le systeme assemble demarre une ROM (CLI + desktop)  [GATE]

## E04
111. E04-01 ppu/vram.rs: VRAM, VMAIN, VMADD, prefetch, lecture/ecriture
112. E04-02 ppu/cgram.rs: CGRAM (256 mots, ecriture/lecture par paires)
113. E04-03 ppu/oam.rs: OAM 544 octets, OAMADD, OAMDATA, rechargement
114. E04-04 ppu/regs.rs: registres de scroll BGnHOFS/BGnVOFS et latchs partages
115. E04-05 ppu/mode7.rs: registres mode 7 (M7SEL, M7A-D, M7X/Y, scroll) et MPY
116. E04-06 ppu/regs.rs: registres de configuration des fonds et des sprites
117. E04-07 ppu/regs.rs: fenetres, TM/TS, color math, COLDATA
118. E04-08 ppu/mod.rs: aiguillage des ecritures $2100-$2133 vers les memoires et registres
119. E04-09 ppu/mod.rs: registres de lecture MPY, OAM, VRAM, CGRAM et open bus PPU
120. E04-10 ppu/mod.rs: verrou des compteurs H/V, OPHCT/OPVCT, STAT77, STAT78
121. E04-11 ppu/mod.rs: INIDISP/SETINI complets, rechargement OAM et drapeaux au debut de frame
122. E04-12 ppu/mod.rs: reset vs power-on, regs_snapshot (debugger) et hash complet
123. E04-13 ppu/output.rs: conversion couleur BGR555 -> XRGB8888 et rangee de sortie
124. E04-14 ppu/mod.rs: framebuffer, pixel de fond (backdrop) et FrameView
125. E04-15 snes.rs: le Snes reel affiche la sortie du PPU (jalon video V1)

## E05
126. E05-01 ppu/bg.rs: types Pix/Pick, tables BPP et ordres de priorite
127. E05-02 ppu/bg.rs: indices de palette et couleur directe
128. E05-03 ppu/bg.rs: decode_row (tuiles 2/4/8 bpp)
129. E05-04 ppu/bg.rs: entree de tilemap et adresse (map_entry_addr)
130. E05-05 ppu/bg.rs: bg_pixel (tuiles 8x8, defilement, palette)
131. E05-06 ppu/bg.rs: tuiles 16x16 et retournements (flips)
132. E05-07 ppu/bg.rs: mosaique
133. E05-08 ppu/bg.rs: offset-per-tile (modes 2 et 4)
134. E05-09 ppu/window.rs: fenetres (window_out) et groupes de couche
135. E05-10 ppu/obj.rs: tailles, decodage d'un sprite et liste des 32 sprites d'une rangee
136. E05-11 ppu/obj.rs: tranches de sprites, tampon de ligne, limites et erratum de flip
137. E05-12 ppu/mode7.rs: echantillonnage mode 7 (m7_sample, clip13)
138. E05-13 ppu/mode7.rs: pixel mode 7 (BG1) et EXTBG (BG2)
139. E05-14 ppu: pick_layers (ordre de priorite), fenetres par ecran, pick_screen
140. E05-15 ppu/colormath.rs: color math, regions de la fenetre couleur
141. E05-16 ppu/mod.rs: compose_pixel branche dans step_dot (rendu non hi-res complet)
142. E05-17 ppu/bg.rs: fonds hi-res des modes 5 et 6 (bg_pixel_hires)
143. E05-18 ppu/mod.rs: hi-res et pseudo hi-res dans step_dot, export 512 colonnes
144. E05-19 tests: harness de scenes PPU (hash de frame compare a un renderer Python independant)

## E06
145. E06-01 T-ORACLE: telecharger l'oracle SingleStepTests spc700
146. E06-02 T-FORMAT: documenter le format JSON exact de l'oracle spc700
147. E06-03 GATE: Claude relit spcjson_format.md et valide le format  [GATE]
148. E06-04 gen_spc_tables.py: spc700/table.rs (BYTES, CYCLES, BRANCH_EXTRA, NAME)
149. E06-05 Spc700: registres, trait SmpBus, helpers, step et squelettes de familles
150. E06-06 spc700/testbus.rs: FlatSmpBus (bus plat journalise)
151. E06-07 spc-test: chargeur des fichiers JSON de l'oracle
152. E06-08 spc-test: comparaison d'etats et de cycles (fonctions pures)
153. E06-09 spc-test: commande complete (boucle sur les fichiers, rapport, code de sortie)
154. E06-10 spc700/addr.rs: recettes d'adressage (dp, dp+X, abs, (X), [dp+X], [dp]+Y...)
155. E06-11 spc700/mov.rs: MOV de chargement (A, X, Y depuis la memoire ou un immediat)
156. E06-12 spc700/mov.rs: MOV de stockage (avec lecture factice de la destination)
157. E06-13 spc700/mov.rs: transferts, MOV dp,#imm, MOV dp,dp, (X)+ et MOVW
158. E06-14 VERIF oracle: toutes les instructions MOV / MOVW
159. E06-15 spc700/alu.rs: OR, AND, EOR (toutes les formes)
160. E06-16 spc700/alu.rs: ADC et SBC (toutes les formes)
161. E06-17 spc700/alu.rs: CMP (A, X, Y et formes memoire)
162. E06-18 VERIF oracle: OR AND EOR ADC SBC CMP
163. E06-19 spc700/incdec.rs: INC, DEC, ASL, LSR, ROL, ROR, XCN
164. E06-20 spc700/misc.rs: DAA, DAS, MUL, DIV
165. E06-21 spc700/word.rs: ADDW, SUBW, CMPW, INCW, DECW
166. E06-22 VERIF oracle: INC DEC ASL LSR ROL ROR XCN DAA DAS MUL DIV et mots 16 bits
167. E06-23 spc700/bits.rs: SET1, CLR1, BBS, BBC, TSET1, TCLR1
168. E06-24 spc700/bits.rs: OR1, AND1, EOR1, MOV1, NOT1 (bits de la memoire absolue)
169. E06-25 VERIF oracle: instructions de bits
170. E06-26 spc700/flow.rs: branches (Bxx, BRA, CBNE, DBNZ)
171. E06-27 spc700/flow.rs: JMP, CALL, PCALL, TCALL, BRK, RET, RETI
172. E06-28 spc700/misc.rs: PUSH, POP, flags, NOP, SLEEP, STOP
173. E06-29 VERIF oracle: flux de controle, pile, indicateurs
174. E06-30 GATE J4a: tous les opcodes SPC700 passent l'oracle  [GATE]
175. E06-31 dsp/mod.rs: Dsp (fichier de registres, reset, hash) et Voice (structure)
176. E06-32 apu/timer.rs: les trois timers du SMP
177. E06-33 apu/ipl.rs: la ROM IPL de 64 octets
178. E06-34 apu/io.rs: ApuIo (RAM 64 Kio, registres $F0-$FF, ports, timers, IPL)
179. E06-35 apu/mod.rs: Apu complet (SMP + ApuIo), rattrapage step_to, region
180. E06-36 Tests d'integration IPL: handshake et envoi d'un programme par les ports
181. E06-37 Snes: le S-CPU voit le handshake de l'IPL (jalon audio A1)
182. E06-38 DSP-DATA: obtenir la table gaussienne GAUSS[512] (reseau)
183. E06-39 dsp/gauss.rs (genere) et dsp/tables.rs: PERIOD, OFFSET, poll
184. E06-40 dsp/brr.rs: decodage d'un groupe de 4 echantillons BRR
185. E06-41 dsp/voice.rs: debut de note (repertoire) et flux BRR (decode_group, fin de bloc)
186. E06-42 dsp/voice.rs: interpolation gaussienne (avec le bug de debordement)
187. E06-43 dsp/envelope.rs: enveloppe ADSR / GAIN
188. E06-44 dsp/envelope.rs: compteur de periodes et generateur de bruit
189. E06-45 dsp/voice.rs: un echantillon d'une voix (delai de key-on, interpolation, enveloppe, PMON, volumes)
190. E06-46 dsp/mod.rs: run_sample (scrutation KON/KOFF, soft reset, 8 voix, sortie sans echo)
191. E06-47 dsp/echo.rs: echo (tampon en RAM, FIR a 8 coefficients, feedback, mute)
192. E06-48 Snes::take_audio reel et constante de cadence audio
193. E06-49 Tests d'integration DSP: une note BRR en boucle de bout en bout
194. E06-50 GATE J4: IPL, SPC700 et DSP valides (jalon audio)  [GATE]

## E07
195. E07-01 T-ROMS: telecharger les ROM de test homebrew (reseau)
196. E07-02 tests/manifest.toml et tests/baseline.json initiaux
197. E07-03 GATE: revue du manifeste et des ROM de test (Claude)  [GATE]
198. E07-04 suite/manifest.rs: lecture et validation du manifeste TOML
199. E07-05 suite/baseline.rs: lecture/ecriture de baseline.json
200. E07-06 suite/report.rs: verdicts, lignes de rapport et code de sortie
201. E07-07 core: Snes::apu_port (ports vus par le S-CPU)
202. E07-08 suite/runner.rs: executer une ROM du manifeste et la juger
203. E07-09 suite.rs: la commande suite (manifeste, baseline, filtre d'epopee, PPM, mise a jour)
204. E07-10 Tests d'integration de `puce16-cli suite` (ROM synthetiques)
205. E07-11 T-SUITE-REPORT: lancer la suite sur les vraies ROM et documenter l'etat
206. E07-11a TUNE: DSP echo, spc_dsp6 echoue a Echo/echo calc (Failed 03)
207. E07-11b TUNE: temps d'acces memoire du SPC700 (spc_mem_access_times / spc_smp: Failed 02)
208. E07-11c TUNE: Mode 7, bruit blanc dans le haut de l'image (PeterLemon StarWars)
209. E07-11d TUNE: CPU 65816, gilyon cputest_full echoue au test 0027
210. E07-11e TUNE: DSP enveloppe, spc_dsp6 echoue a Envelope/gain $E0 threshold (Failed 02)
211. E07-12 GATE J3/J5: captures validees, baseline fige  [GATE]
212. E07-13 Preparer les jeux: README des ROM commerciales et verification du manifeste
213. E07-14 GATE J6: Super Mario World demarre et se joue  [GATE]
214. E07-15 GATE J7: ALttP et Super Mario All-Stars jouables  [GATE]

## E08
215. E08-01 savestate.rs: StateWriter, StateReader, StateError et trait State
216. E08-02 State pour Cpu (cpu/mod.rs)
217. E08-03 State pour Joypads, Cart (SRAM) et Wram
218. E08-04 State pour Mmio et Timing
219. E08-05 State pour Dma (canaux et etat HDMA)
220. E08-06 State pour Vram, Cgram et Oam
221. E08-07 State pour Scroll, M7Regs et Regs (registres PPU)
222. E08-08 State pour Ppu (position, tampons de ligne, framebuffer)
223. E08-09 State pour Spc700 et Timer
224. E08-10 State pour Dsp et Voice
225. E08-11 State pour ApuIo et Apu
226. E08-12 Snes::save_state et load_state (assemblage, en-tete du blob)
227. E08-13 Tests d'integration des save states sur des scenarios (NMI, DMA, HDMA, PPU, APU)
228. E08-14 CLI: puce16-cli run --save-state / --load-state
229. E08-15 Desktop audio.rs: anneau circulaire AudioRing (pur)
230. E08-16 Desktop audio.rs: re-echantillonneur sinc fenetre (pur)
231. E08-17 Desktop audio.rs: rendu du callback (volume, muet, canaux, sous-alimentation)
232. E08-18 Desktop audio.rs: sortie cpal (AudioOut)
233. E08-19 emu_thread: synchro audio (anneau), cadence PAL/NTSC, avance rapide
234. E08-20 files.rs: chemins SRAM/save states, ecriture atomique, en-tete .p16s
235. E08-21 emu_thread: SRAM (chargement, sauvegarde toutes les 5 s, a la fermeture)
236. E08-22 emu_thread: save states (SaveState / LoadState, slots 0..9)
237. E08-23 app.rs: raccourcis save states / slots / capture d'ecran / muet / avance rapide
238. E08-24 config.rs: configuration JSON (defauts, aller-retour, ROM recentes)
239. E08-25 video.rs: filtres, ratio par nom, scanlines (pur)
240. E08-26 app.rs: configuration (chargement, application, ROM recentes, menus Video/Audio)
241. E08-27 app.rs / main.rs: audio reel (anneau, flux cpal, volume, muet)
242. E08-28 app.rs: plein ecran (F11), curseur masque, pause a la perte du focus
243. E08-29 input.rs: tables manette (PadButton), stick, mappings personnalises (pur)
244. E08-30 input.rs / app.rs: lecture des manettes avec gilrs (PadPoller)
245. E08-31 GATE D5: desktop jouable (audio, SRAM, save states, manette, filtres, config)  [GATE]
246. E08-32 snes.rs: accesseurs de debug (VRAM, CGRAM, OAM, registres PPU/SMP/DMA, page WRAM, run_until_pc)
247. E08-33 emu_thread: commandes de debug (RequestDebug, SetWramPage, StepInstruction, RunToPc) et Event::Debug
248. E08-34 debugger.rs: decodage de tuiles, couleurs, palettes, hexdump, drapeaux (pur)
249. E08-35 debugger.rs: rendu des cartes BG, carte mode 7 et decodage OAM (pur)
250. E08-36 debugger.rs / app.rs: fenetre du debugger (F10), registres et commandes d'execution
251. E08-37 debugger.rs: onglets Tuiles, Cartes et Palettes
252. E08-38 debugger.rs: onglets OAM et Memoire, parse_pc (+ tests purs)
253. E08-39 GATE J8: save states, SRAM et debugger valides (jalon desktop final)  [GATE]

## E09
254. E09-00 TUNE: sprites decales d'une ligne (SMW: debris colores sur la ligne 0 en haut de l'image) - reporte apres la v1
255. E09-01 S-DD1: module coprocessor/sdd1 et tables.rs (RUNCOUNT, EVOLUTION)
256. E09-02 S-DD1: Decompressor (struct, init) et etages IM + GCD
257. E09-03 S-DD1: etages BG (generateurs de bits) et PEM (estimation de probabilite)
258. E09-04 S-DD1: etages CM (contexte) et OL (sortie) + vecteurs d'or
259. E09-05 S-DD1: registres $4800-$4807 et fonctions de mappage (Sdd1)
260. E09-06 S-DD1: espionnage DMA et lecture decompressee en flux
261. E09-07 S-DD1: acceptation de l'en-tete et integration dans Cart (ROM, SRAM)
262. E09-08 S-DD1: routage du bus ($4800-$480F, espionnage $4300-$437F, lectures C0-FF)
263. E09-09 S-DD1: sauvegarde d'etat et hash
264. E09-10 S-DD1: test de bout en bout (ROM synthetique, DMA vers la WRAM)
265. E09-11 S-DD1: entrees optionnelles du manifeste pour les jeux (Street Fighter Alpha 2, Star Ocean)
266. E09-12 GATE S-DD1: Street Fighter Alpha 2 et Star Ocean valides par l'utilisateur  [GATE]
267. E09-13 Coprocesseur DSP-1 (Super Mario Kart, Pilotwings)
268. E09-14 Coprocesseur Super FX / GSU (Star Fox, Yoshi's Island)
269. E09-15 Coprocesseur SA-1 (Super Mario RPG, Kirby Super Star)
270. E09-16 Decompresseur SPC7110 (Far East of Eden Zero, Momotaro Dentetsu Happy)
271. E09-17 Coprocesseur Cx4 (Mega Man X2, X3)
272. E09-18 Autres puces: OBC1, S-RTC, ST010/ST011/ST018, MSU-1, BS-X

## E10
273. E10-01 Triches: module cheat et analyse d'un code PAR (parse_par)
274. E10-02 Triches: fichier .cht (lecture, ecriture, erreurs avec numero de ligne)
275. E10-03 Triches: application sur la console (Snes::set_cheats, ecriture silencieuse a chaque image)
276. E10-04 Triches: options CLI --cheat et --cheat-code
277. E10-05 Triches: fichiers .cht du desktop et commande SetCheats de la console
278. E10-06 Triches: modele CheatPanel, menu Triches a cases a cocher et fenetre d'ajout
279. E10-07 GATE Triches: Pro Action Replay valide par l'utilisateur avec de vrais codes  [GATE]
280. E10-08 Game Genie (placeholder, hors perimetre)
