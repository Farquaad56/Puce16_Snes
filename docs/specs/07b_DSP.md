# SPEC 07b - S-DSP (8 voix, BRR, enveloppes, bruit, echo)

Source: wiki SNESdev (S_DSP_registers, DSP_envelopes, BRR_samples, S_SMP) + references bsnes/blargg (formules entieres). Fichiers: `puce16-core/src/apu/dsp/{mod,voice,envelope,brr,echo,tables}.rs`.
**Decision D-018 (provisoire)**: le DSP est **precis a l'echantillon** en v1: un appel `run_sample()` toutes les 32 cycles SMP (pas de micro-pas internes), avec les quirks documentes (KON/KOFF scrutes un echantillon sur deux, delai de key-on de 5 echantillons, bug de debordement du gaussien, FIR "clip puis clamp"). Le micro-ordonnancement 32 pas est une tache de tuning ulterieure. Calcul 100 % entier (D-009). `>>` sur `i32` est arithmetique. `as i16` = repliement 16 bits voulu.
Toute valeur marquee `UNKNOWN - to confirm` a un defaut; il est ajuste par comparaison avec une reference (SPC de test) dans une tache de calibration.

## 1. Registres (acces via $F2/$F3, 128 octets, `regs[0x80]`)
Voix `x` (0..7), base `x*0x10`: `+0 VOLL(i8) +1 VOLR(i8) +2 PITCHL +3 PITCHH(6 bits) +4 SRCN +5 ADSR1 +6 ADSR2 +7 GAIN +8 ENVX +9 OUTX`.
Globaux: `$0C MVOLL $1C MVOLR $2C EVOLL $3C EVOLR $4C KON $5C KOFF $6C FLG $7C ENDX $0D EFB $2D PMON $3D NON $4D EON $5D DIR $6D ESA $7D EDL`, FIR: `$0F,$1F,...,$7F` = FIR0..FIR7 (i8). `$1D`, `$xA`, `$xB`, `$xE`: stockes sans effet.
- Ecriture: `regs[a] = v`, sauf: `$4C` -> `new_kon = v` (et `regs[$4C] = v`); `$7C` (ENDX): toute ecriture met `endx = 0` (`UNKNOWN - to confirm`). ENVX/OUTX/ENDX sont mis a jour par le DSP a chaque echantillon (`regs[x8] = env >> 4`, `regs[x9] = out >> 8`, `regs[$7C] = endx`).
- Lecture: `regs[a]`.
- Power-on: tous les registres 0 sauf `FLG = $E0`; `counter = 0`, `noise = 0x4000`, `every_other = true`, echo offset/length 0, historiques FIR 0. Reset DSP: `FLG = $E0` (les autres registres sont conserves, `UNKNOWN`).

## 2. Compteur de periodes (partage: enveloppes + bruit)
`counter` (u16): a chaque echantillon `counter = if counter == 0 { 0x77FF } else { counter - 1 }`.
Tables (index = taux 0..31):
```
PERIOD = [0 (jamais),2048,1536,1280,1024,768,640,512,384,320,256,192,160,128,96,80,64,48,40,32,24,20,16,12,10,8,6,5,4,3,2,1]
OFFSET[r] = 0 si r%3==1 ; 1040 si r%3==2 ; 536 si r%3==0   (r >= 1)
fn poll(rate) -> bool { rate != 0 && (counter + OFFSET[rate]) % PERIOD[rate] == 0 }
```
(Les periodes sont en echantillons DSP; 30720 = 0x7800 est divisible par toutes.)

## 3. BRR
Bloc de 9 octets: en-tete `SSSS FFLE` (shift = bits 7-4, filtre F = bits 3-2, boucle L = bit 1, fin E = bit 0) + 8 octets = 16 nibbles (nibble haut d'abord).
Repertoire: `dir = (DIR as u16) << 8`; entree de la voix = `dir + SRCN*4` (mod 65536): `start = ram16(entry)`, `loop = ram16(entry+2)`.
Decodage par **groupes de 4 echantillons** (octets `addr+1+2*j` et `+2` du groupe j = 0..3 du bloc):
```
nyb = (ram[a] << 8 | ram[a+1]) as i16                 // 16 bits = 4 nibbles
for _ in 0..4 {
    let mut s = (nyb >> 12) as i32;  nyb <<= 4;         // nibble signe -8..7
    s = if shift > 12 { (s >> 4) << 11 } else { (s << shift) >> 1 };
    // p1 = dernier echantillon decode (echelle 16 bits), p2 = avant-dernier >> 1
    if filter == 2 { s += p1; s -= p2; s += p2 >> 4; s += (p1 * -3) >> 6 }
    else if filter == 3 { s += p1; s -= p2; s += (p1 * -13) >> 7; s += (p2 * 3) >> 4 }
    else if filter == 1 { s += p1 >> 1; s += (-p1) >> 5 }
    s = clamp16(s); s = (s * 2) as i16 as i32;          // stocke a l'echelle 16 bits (repliement)
    p2 = p1 >> 1; p1 = s; push dans le tampon annulaire
}
```
(Coefficients du wiki: F1 = 15/16*p1; F2 = 61/32*p1 - 15/16*p2; F3 = 115/64*p1 - 13/16*p2.) Les `p1,p2` continuent d'un bloc au suivant; ils ne sont PAS remis a zero au key-on (`UNKNOWN`, comportement materiel: l'historique residuel influe, d'ou le conseil "commencer par un bloc filtre 0").
Tampon annulaire: 12 echantillons (3 groupes), index d'ecriture `buf_pos` (multiple de 4): le groupe decode est ecrit a `buf_pos`, puis `buf_pos = (buf_pos + 4) % 12` (=> `buf_pos` pointe toujours le plus ancien groupe).
Fin de bloc (apres le groupe 3 d'un bloc dont l'en-tete est H): `if H & 1 { endx |= bit; addr = loop; if H & 2 == 0 { mode = Release; env = 0; hidden_env = 0 } } else { addr += 9 }` (comme le materiel: l'adresse saute a `loop` des que le drapeau E est present, que L soit mis ou non; sans L la voix est coupee mais le decodeur continue de lire a `loop`, sans effet audible car env = 0.). (`UNKNOWN - to confirm`: le materiel traite la fin quelques echantillons avant la fin audible a cause de l'avance du decodeur (8 echantillons); ce modele reproduit cette avance. Isoler dans `voice.rs::advance_block`.)
`ENDX`: mis a la fin d'un bloc a drapeau E (meme avec L); remis a 0 pour la voix au key-on.

## 4. Voix (une iteration par voix, voix 0 -> 7, par echantillon)
Etat: `kon_delay: u8`, `mode: Attack|Decay|Sustain|Release`, `env: i32 (0..0x7FF)`, `hidden_env: i32`, `addr: u16`, `block_header: u8`, `group_in_block: u8`, `buf: [i16;12]`, `buf_pos: usize`, `interp_pos: u32`, `out: i32` (sortie apres enveloppe, avant volume).
```
// -- polling KON/KOFF (voir 6) puis pour la voix:
if kon_delay > 0 {
    if kon_delay == 5 { addr = start(SRCN); group_in_block = 0; read header; endx &= !bit; interp_pos = 0; buf_pos = 0 }
    env = 0; hidden_env = 0; kon_delay -= 1;
    if kon_delay is 3, 2 or 1 { decode_group() }        // 3 groupes decodes pendant le delai
    out = 0; (pas d'avance de pitch, pas de run_envelope); mix 0; continue
}
while interp_pos >= 0x4000 { decode_group(); interp_pos -= 0x4000 }
sample = if NON bit { ((noise as i32) << 1) as i16 as i32 }
         else { gauss(window = buf[(buf_pos + (interp_pos >> 12) + i) % 12], i = 0..3, frac = (interp_pos >> 4) & 0xFF) }
run_envelope()                                            // AVANT d'appliquer l'enveloppe
out = ((sample * env) >> 11) & !1
regs[x8] = env >> 4; regs[x9] = out >> 8
pitch = (PITCH & 0x3FFF); if PMON bit (x>=1) { pitch += ((prev_out >> 5) * pitch) >> 10 }     // prev_out = out de la voix x-1 (cet echantillon)
interp_pos = min(interp_pos + pitch, 0x7FFF)
l = (out * VOLL as i8) >> 7; r = (out * VOLR as i8) >> 7
sum_l = clamp16(sum_l + l); sum_r = clamp16(sum_r + r); if EON bit { echo_l = clamp16(echo_l + l); echo_r = clamp16(echo_r + r) }
```
`decode_group()`: au debut d'un bloc (`group_in_block == 0`) lire l'en-tete a `addr`; decoder 4 echantillons (section 3); `group_in_block += 1`; si 4: fin de bloc (section 3) et `group_in_block = 0`.
`prev_out` pour la voix 0 = 0 (PMON bit 0 ignore). Le bit PMON de la voix x utilise la sortie de la voix x-1 **de cet echantillon**.

### 4.1 Interpolation gaussienne (table `GAUSS[512]`, voir section 8)
```
let (a, b, c, d) = window;                // a = le plus ancien, i16 echelle 16 bits
let mut o = (GAUSS[255 - f] * a) >> 11;
o += (GAUSS[511 - f] * b) >> 11;
o += (GAUSS[256 + f] * c) >> 11;
o = o as i16 as i32;                       // repliement 16 bits: BUG materiel (3 valeurs max-negatives -> bruit fort), a conserver
o += (GAUSS[f] * d) >> 11;
sample = clamp16(o) & !1
```

### 4.2 Enveloppe (11 bits, 0..0x7FF)
```
if mode == Release { env = max(env - 8, 0); hidden = env; return }      // a chaque echantillon, sans taux
let rate; let mut e = env; let mut data = ADSR2;
if ADSR1 & 0x80 {                                   // ADSR
    if mode >= Decay { e -= 1; e -= e >> 8; rate = if mode == Decay { ((ADSR1 >> 4) & 7) * 2 + 16 } else { ADSR2 & 0x1F } }
    else { rate = (ADSR1 & 0xF) * 2 + 1; e += if rate < 31 { 32 } else { 1024 } }
} else {                                            // GAIN
    data = GAIN; let m = data >> 5;
    if m < 4 { e = (data as i32) << 4; rate = 31 }
    else { rate = data & 0x1F;
           if m == 4 { e -= 32 } else if m < 6 { e -= 1; e -= e >> 8 }
           else { e += 32; if m > 6 && hidden_env >= 0x600 { e += 8 - 32 } } }
}
if (e >> 8) == (data >> 5) && mode == Decay { mode = Sustain }
hidden_env = e
if e < 0 || e > 0x7FF { e = if e < 0 { 0 } else { 0x7FF }; if mode == Attack { mode = Decay } }
if poll(rate) { env = e }
```
(Le mode GAIN direct (m<4) a le taux 31 donc s'applique a chaque echantillon.)

## 5. Bruit
A chaque echantillon: `if poll(FLG & 0x1F) { let fb = (noise << 13) ^ (noise << 14); noise = (fb & 0x4000) ^ (noise >> 1) }` (u16, 15 bits utiles; bit 14 neuf = bit0 ^ bit1). Depart 0x4000: suite 0x2000, 0x1000, 0x0800.

## 6. KON / KOFF / FLG
```
every_other = !every_other
if every_other {                     // un echantillon sur deux
    kon = new_kon; new_kon = 0; koff = regs[KOFF]
    for each voice x: if kon bit { kon_delay = 5; mode = Attack } ; then: if koff bit { mode = Release }
}
if FLG & 0x80 (soft reset): pour chaque voix { mode = Release; env = 0 } (a chaque echantillon)
```
(Si KON et KOFF sont a 1 pour la meme voix, la note est declenchee puis coupee: voix silencieuse.) `FLG bit 6` (mute): la sortie finale est 0 (le calcul continue). `FLG bit 5`: coupe l'ECRITURE de l'echo seulement.

## 7. Echo, sortie
Ordre dans un echantillon, apres les 8 voix:
```
if echo_offset == 0 { echo_length = (regs[EDL] & 15) << 11 }
ptr = ((ESA as u16) << 8).wrapping_add(echo_offset)
l = i16(ram[ptr] | ram[ptr+1] << 8) >> 1 ; r = i16(ram[ptr+2] | ram[ptr+3] << 8) >> 1     // octets mod 65536
push (l, r) dans hist (anneau de 8; FIR0 = le plus ANCIEN, FIR7 = le plus RECENT; UNKNOWN - to confirm)
fir_l = sum_{i=0..6} (hist_l[i] * FIR_i as i8) >> 6 ; fir_l = fir_l as i16 as i32 ; fir_l += (hist_l[7] * FIR_7) >> 6 ; fir_l = clamp16(fir_l) & !1   // meme chose pour r
main_l = clamp16( ((sum_l * MVOLL as i8) >> 7) + ((fir_l * EVOLL as i8) >> 7) )        // idem r
if FLG & 0x40 { main_l = 0; main_r = 0 }
audio.push(main_l as i16); audio.push(main_r as i16)
// feedback / ecriture echo
wl = clamp16(echo_l + ((fir_l * EFB as i8) >> 7)) & !1 ; wr idem
if FLG & 0x20 == 0 { ram[ptr..ptr+4] = wl LE, wr LE }
echo_offset += 4; if echo_offset >= echo_length { echo_offset = 0 }
```
(EDL = 0 => longueur 0 => ecrit toujours 4 octets en `ESA*256`, comme le wiki.) `echo_l/r` = somme clampee des voix EON de l'echantillon, remise a 0 au debut de l'echantillon, de meme que `sum_l/r`.

## 8. Table gaussienne `GAUSS[512]` (donnee externe, NON incluse dans le wiki fourni)
Fichier `docs/specs/data/gauss_table.tsv` (colonnes `index`, `value`, 512 lignes), a **obtenir** par la tache "DSP-DATA" (sources a essayer dans l'ordre: documentation fullsnes sur `problemkaputt.de`, source `SPC_DSP.cpp` de blargg snes_spc, tables de bsnes/higan; si le reseau est bloque: question `tooling`). Ne JAMAIS inventer les valeurs. Controles automatiques a executer sur le fichier:
- 512 valeurs entieres >= 0, `GAUSS[0] == 0`, `GAUSS[511] == 1305` (0x519), non decroissante.
- Pour tout `f` dans 0..=255: `GAUSS[255-f] + GAUSS[511-f] + GAUSS[256+f] + GAUSS[f]` dans `[2045, 2051]` (somme ~ 2048 = gain unitaire).

## 9. Interface Rust
```rust
pub struct Dsp { /* regs[128], voix[8], counter, noise, every_other, new_kon, kon, endx, echo_offset, echo_length, hist, audio */ }
impl Dsp {
    pub fn new() -> Self;
    pub fn reset(&mut self);
    pub fn read(&mut self, addr: u8) -> u8;                  // addr & 0x7F
    pub fn write(&mut self, addr: u8, v: u8);                // addr & 0x7F
    pub fn run_sample(&mut self, ram: &mut [u8; 65536]);     // 1 echantillon stereo -> audio
    pub fn take_audio(&mut self) -> Vec<i16>;                // L,R,L,R...
}
```
L'`Apu` appelle `run_sample` tous les 32 cycles SMP. Debit: `APU_RESONATOR_HZ / 768` = 32 040 echantillons/s (D-004).

## 10. Tests (prefixe = id de tache; sans ROM)
- Periodes: sur 30720 echantillons, `poll(r)` est vrai exactement `30720 / PERIOD[r]` fois pour chaque r = 1..31, `poll(0)` jamais.
- Enveloppes: attaque taux 31 (A=15): 0 -> 0x7FF en 2 echantillons (+1024 par echantillon); release: -8/echantillon; GAIN direct `0x40` -> env = 0x400; exponentielle et "bent" (bascule a 0x600) sur valeurs calculees a la main dans le test.
- BRR: bloc filtre 0 shift 0 avec nibbles connus; filtres 1-3 vs formule du wiki sur 2 blocs; shift 13+ (`(s>>4)<<11`); fin/boucle (ENDX, adresse de boucle, silence si fin sans boucle).
- Bruit: suite 0x2000, 0x1000, 0x0800 depuis 0x4000, cadence avec FLG.
- Gaussien: table (section 8), debordement (3 x -32768 -> repliement visible), echantillon constant -> sortie ~constante.
- Echo: EDL=0 ecrit 4 octets a ESA*256; FLG bit5 bloque l'ecriture; FIR identite (0x7F,0...) delaye de 7 echantillons.
- Key-on: delai 5 echantillons (sortie 0 pendant 5), KON+KOFF -> silence, scrutation 1 echantillon sur 2.
- Integration: un petit programme SPC (octets assembles a la main dans le test) qui ecrit KON et une boucle BRR -> `take_audio` non nul et deterministe (hash stable).
