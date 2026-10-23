from e06_a import CHAIN
from e03_f import TINY
from _dsl import T

DESK = "crates/puce16-desktop"
SRC = DESK + "/src"
DT = DESK + "/tests"
EMBD = ["08_INPUT_DESKTOP.md#5. Audio"]
TMP = ("Dossier temporaire de test: `let dir = std::env::temp_dir().join(format!(\"puce16_<id>_{}\", std::process::id()))` cree puis supprime a la fin du test.")
SRAMROM = ("Aide de test (a recopier): `fn sram_rom(code: &[u8]) -> Vec<u8> { let mut r = vec![0u8; 0x8000]; r[..code.len()].copy_from_slice(code); "
           "r[0x7FC0..0x7FC0 + 21].copy_from_slice(b\"PUCE16 TEST          \"); r[0x7FD5] = 0x20; r[0x7FD6] = 0x02; r[0x7FD7] = 0x05; r[0x7FD8] = 0x03; r[0x7FD9] = 0x01; r[0x7FDA] = 0x33; "
           "r[0x7FFC] = 0x00; r[0x7FFD] = 0x80; r[0x7FDC] = 0xFF; r[0x7FDD] = 0xFF; r[0x7FDE] = 0x00; r[0x7FDF] = 0x00; "
           "let sum: u32 = r.iter().map(|&b| b as u32).sum(); let c = (sum & 0xFFFF) as u16; r[0x7FDE] = c as u8; r[0x7FDF] = (c >> 8) as u8; let n = !c; r[0x7FDC] = n as u8; r[0x7FDD] = (n >> 8) as u8; r }` "
           "(LoROM 32 Kio avec 8 Kio de SRAM, chipset 0x02, vecteur RESET 0x8000; si `Snes::new` la refuse, corrige l'en-tete d'apres spec 04). "
           "Programmes (code a 0x8000): `PROG_W` = `78 18 FB A9 5A 8F 00 00 70 80 FE` (SEI CLC XCE LDA #$5A STA $700000 BRA -2) ; `PROG_R` = `78 18 FB AF 00 00 70 1A 8F 01 00 70 80 FE` (LDA $700000 ; INC A ; STA $700001 ; BRA -2).")

# ------------------------------------------------------------------ E08 part B: desktop audio, fichiers, SRAM, save states (E08-15..24)

T("E08-15", "Desktop audio.rs: anneau circulaire AudioRing (pur)",
  size=90, crate="puce16-desktop",
  context="Tampon entre le thread d'emulation (producteur, ~32 040 trames/s) et le callback audio (consommateur). Partie pure de `audio.rs`, testee sans peripherique. " + CHAIN,
  embed=EMBD,
  write=[SRC + "/audio.rs", DT + "/e08_15_ring.rs"],
  spec="""Dans `audio.rs` (module deja declare):
```rust
pub const RING_CAPACITY: usize = 8192;        // trames
pub const RING_TARGET: usize = 2048;          // remplissage vise (~64 ms)
pub const RING_WAIT_ABOVE: usize = 3072;      // l'emulation attend au-dela (~96 ms)
pub struct AudioRing { buf: Vec<[i16; 2]>, head: usize, len: usize }
impl AudioRing {
    pub fn new() -> AudioRing;                              // capacite RING_CAPACITY, vide
    pub fn len(&self) -> usize;  pub fn capacity(&self) -> usize;  pub fn free(&self) -> usize;
    pub fn push(&mut self, frames: &[[i16; 2]]) -> usize;   // ecrit autant de trames que possible, renvoie le nombre accepte (jamais d'ecrasement)
    pub fn pop(&mut self, out: &mut [[i16; 2]]) -> usize;   // lit jusqu'a out.len() trames dans l'ordre d'ecriture, renvoie le nombre lu
    pub fn clear(&mut self);
}
pub type SharedRing = std::sync::Arc<std::sync::Mutex<AudioRing>>;
pub fn new_shared_ring() -> SharedRing;
```""",
  tests=[("new_is_empty", "`AudioRing::new()`: `len()==0`, `capacity()==8192`, `free()==8192`."),
         ("push_pop_order", "push de `[[1,1],[2,2],[3,3]]` puis `pop` de 2 -> `[[1,1],[2,2]]`, `len()==1` ; `pop` de 5 -> 1 trame `[3,3]`."),
         ("push_full", "push de 8192 trames -> 8192 ; push de 10 de plus -> 0 ; `len()==8192` ; apres `pop` de 4 : push de 10 -> 4."),
         ("wraparound", "push 6000, pop 6000, push 6000 (franchit la fin du tampon), pop : les 6000 valeurs relues sont dans l'ordre (valeurs `[i as i16, -(i as i16)]`)."),
         ("pop_empty", "`pop` sur anneau vide -> 0 et le tampon de sortie n'est pas modifie."),
         ("clear", "apres `clear()`: `len()==0` ; push/pop refonctionnent."),
         ("shared_ring_threads", "`new_shared_ring()` ; un thread ecrit 10 x 500 trames (par blocs, en reessayant tant que `push` renvoie moins que demande, avec `sleep(1ms)`), le thread de test lit jusqu'a 5000 trames : valeurs recues = suite croissante complete, aucune perte."),
         ("constants", "`RING_TARGET < RING_WAIT_ABOVE` et `RING_WAIT_ABOVE < RING_CAPACITY`.")],
  accept=["cargo test -p puce16-desktop e08_15_"])

T("E08-16", "Desktop audio.rs: re-echantillonneur sinc fenetre (pur)",
  size=110, crate="puce16-desktop",
  context="Conversion de ~32 040 Hz vers la frequence du peripherique (44 100 / 48 000 Hz...), en flux continu (appels successifs avec des blocs de taille quelconque). Les flottants sont autorises ici (D-009: desktop seulement). " + CHAIN,
  embed=EMBD,
  write=[SRC + "/audio.rs", DT + "/e08_16_resampler.rs"],
  spec="""Ajoute dans `audio.rs` (sans modifier `AudioRing`):
```rust
pub struct Resampler { step: f64, pos: f64, buf: Vec<[f32; 2]> }
impl Resampler {
    pub fn new(src_rate: u32, dst_rate: u32) -> Resampler;      // step = src_rate / dst_rate ; buf = 8 trames nulles (historique) ; pos = 8.0
    pub fn process(&mut self, input: &[[f32; 2]], out: &mut Vec<[f32; 2]>);   // ajoute input a buf puis produit des sorties
}
```
Algorithme: tant que `pos + 8.0 <= (buf.len() - 1) as f64`: `n = pos.floor() as usize`, `f = pos - n as f64` (0 <= f < 1) ; pour `k` de -7 a 8 inclus: `x = f - k as f64` ; `w[k] = sinc(x) * hann(x)` avec `sinc(x) = if x == 0 {1} else {sin(pi x)/(pi x)}` et `hann(x) = if |x| < 8 {0.5 * (1 + cos(pi x / 8))} else {0}` ; sortie = `sum(buf[n + k] * w[k]) / sum(w)` (normalisation: gain continu exactement 1) pour chaque canal ; `pos += step`. Ensuite jeter les trames devenues inutiles: garder a partir de l'indice `floor(pos) - 7` et decaler `pos` en consequence (flux inchange). Indices `n + k` entre `n - 7` et `n + 8` : jamais hors du tampon grace a la condition de la boucle et aux 8 trames d'historique.""",
  tests=[("identity_rate", "`Resampler::new(32040,32040)`, entree de 100 trames `[i as f32 / 1000.0, -(i as f32) / 1000.0]` : `out.len()==92` et pour i in 0..92 : `|out[i] - in[i]| < 1e-4` sur les deux canaux."),
         ("dc_is_preserved_upsampling", "`new(32040,48000)`, 1000 trames `[0.5,0.5]` : `out.len()` entre 1480 et 1500 ; toutes les sorties d'indice >= 16 sont a moins de 1e-4 de 0.5."),
         ("dc_preserved_downsampling", "`new(48000,32040)`, 1000 trames `[0.25,-0.25]` : sorties d'indice >= 16 a moins de 1e-4 de `[0.25,-0.25]` ; `out.len()` entre 660 et 680."),
         ("streaming_equals_one_shot", "meme signal (sinusoide 440 Hz echantillonnee a 32040, 2000 trames) traite en un bloc ou en blocs de 7, 100, 1 (cycle) avec `new(32040,44100)` : memes longueurs de sortie et ecart max < 1e-6."),
         ("sine_amplitude", "sinusoide 1000 Hz amplitude 0.5 a 32040 Hz, `new(32040,48000)`, 4000 trames : l'amplitude max des sorties (apres les 100 premieres) est entre 0.49 et 0.51."),
         ("empty_input", "`process(&[], ..)` n'ajoute aucune sortie et ne panique pas."),
         ("output_rate", "`new(32040,48000)` sur 32040 trames d'entree : `out.len()` entre 47 975 et 48 000."),
         ("channels_independent", "entree `[0.5, 0.0]` constante : sorties (indice >= 16) `[~0.5, ~0.0]`.")],
  accept=["cargo test -p puce16-desktop e08_16_"])

T("E08-17", "Desktop audio.rs: rendu du callback (volume, muet, canaux, sous-alimentation)",
  size=110, crate="puce16-desktop",
  context="Fonction PURE appelee par le callback cpal: tire des trames de l'anneau, les re-echantillonne, applique le volume et ecrit les canaux du peripherique. Testable sans carte son. " + CHAIN,
  embed=EMBD,
  write=[SRC + "/audio.rs", DT + "/e08_17_playback.rs"],
  spec="""Ajoute dans `audio.rs`:
```rust
pub const SRC_RATE: u32 = 32_040;
pub struct Playback { rs: Resampler, pending: std::collections::VecDeque<[f32; 2]>, last: [f32; 2], scratch: Vec<[i16; 2]> }
impl Playback {
    pub fn new(device_rate: u32) -> Playback;       // Resampler::new(SRC_RATE, device_rate), last = [0,0]
    /// Remplit `out` (entrelace, `channels` canaux). volume 0.0..=1.0.
    pub fn render(&mut self, ring: &mut AudioRing, volume: f32, muted: bool, channels: usize, out: &mut [f32]);
}
```
`render`: `frames = out.len() / channels` ; tant que `pending.len() < frames` : lire dans l'anneau au plus `(frames - pending.len()) * SRC_RATE / device_rate + 16` trames (min 16) dans `scratch` ; si 0 trame lue : sous-alimentation -> pousser `last` dans `pending` jusqu'a `frames` et arreter ; sinon convertir (`x as f32 / 32768.0`), `rs.process`, ajouter a `pending`. Ensuite, pour chacune des `frames` trames retirees de `pending` : `last = trame` ; `l, r` = trame * volume (0 si `muted`) ; 1 canal -> `(l + r) / 2` ; 2 canaux -> `l, r` ; plus de 2 -> `l, r` puis des zeros. Si `channels == 0` ou `out` vide : ne rien faire.""",
  tests=[("constant_stereo", "anneau rempli de 4000 trames `[16384,16384]`, `Playback::new(48000)`, `render(vol 1.0, !muted, 2, out[512*2])` : apres les 16 premieres trames, toutes les valeurs sont a moins de 0.01 de 0.5."),
         ("volume_scales", "meme avec volume 0.5 -> valeurs proches de 0.25 (apres warmup)."),
         ("muted_is_silent", "meme avec `muted=true` -> toutes les valeurs exactement 0.0 ; l'anneau a quand meme ete consomme (`len()` a diminue)."),
         ("mono_downmix", "anneau `[16384, -16384]` constant, 1 canal : valeurs proches de 0.0 (moyenne) apres warmup ; avec `[16384, 0]` -> proches de 0.25."),
         ("six_channels", "anneau `[16384,16384]`, `channels=6`: pour chaque trame, indices 0 et 1 proches de 0.5 (apres warmup) et indices 2..6 exactement 0.0."),
         ("underrun_repeats_last", "anneau avec 100 trames `[16384,16384]`, `Playback::new(32040)`, `render` de 600 trames stereo : les dernieres 50 trames valent exactement la derniere valeur produite avant la sous-alimentation (aucun saut a 0)."),
         ("empty_ring_start_is_silent", "anneau vide, `render` -> toutes les valeurs 0.0 (last initial = 0)."),
         ("consumption_rate", "`Playback::new(48000)`, anneau de 8000 trames : un `render` de 4800 trames consomme entre 3 200 et 3 260 trames de l'anneau."),
         ("zero_channels_noop", "`render(.., channels=0, out)` ne panique pas et ne touche pas a l'anneau.")],
  accept=["cargo test -p puce16-desktop e08_17_"])

T("E08-18", "Desktop audio.rs: sortie cpal (AudioOut)",
  size=130, kind="ui", crate="puce16-desktop",
  context="Ouvre le peripherique audio par defaut avec `cpal` et branche `Playback::render` sur son callback. Pas de test automatique (materiel): la compilation doit etre propre. " + CHAIN,
  embed=EMBD,
  write=[SRC + "/audio.rs"],
  spec="""Ajoute dans `audio.rs` (lis la doc de la version de `cpal` resolue dans `Cargo.lock` avec `cargo doc -p cpal --no-deps` ou dans `~/.cargo/registry`; ne devine pas l'API):
```rust
pub struct AudioOut { _stream: cpal::Stream, volume: std::sync::Arc<std::sync::atomic::AtomicU32>, muted: std::sync::Arc<std::sync::atomic::AtomicBool>, pub device_rate: u32, pub channels: usize }
impl AudioOut {
    pub fn new(ring: SharedRing) -> Result<AudioOut, String>;   // Err("no audio output device") ou le message cpal
    pub fn set_volume(&self, v: f32);                            // 0.0..=1.0 (stocke `v.to_bits()` dans l'AtomicU32)
    pub fn set_muted(&self, m: bool);
}
```
`new`: `cpal::default_host().default_output_device()` ; configuration par defaut du peripherique (`default_output_config`) ; construire le flux selon le format d'echantillon (`F32`, `I16`, `U16`) : pour F32 le callback appelle `Playback::render` sur le tampon ; pour I16/U16 il rend dans un `Vec<f32>` temporaire puis convertit (`(x * 32767.0) as i16`, ou `(x * 32767.0 + 32768.0) as u16`). Le callback verrouille l'anneau (`lock()`, en cas d'empoisonnement: `into_inner`), lit volume et muet atomiques. Erreur du flux : `eprintln!("audio stream error: {}", e)`. `stream.play()` apres creation. Aucun `unwrap` sur le peripherique ou le flux.""",
  tests=[],
  accept=["cargo build -p puce16-desktop"])

T("E08-19", "emu_thread: synchro audio (anneau), cadence PAL/NTSC, avance rapide",
  size=140, crate="puce16-desktop",
  context="Le thread d'emulation envoie l'audio de chaque frame dans l'anneau et se synchronise dessus (horloge principale). Sans audio: minuteur monotone a la cadence de la console. Avance rapide: aucune attente, audio jete. " + CHAIN,
  embed=EMBD + ["08_INPUT_DESKTOP.md#4. Fenetre, video", "08_INPUT_DESKTOP.md#7. Architecture desktop"],
  write=[SRC + "/emu_thread.rs", DT + "/e08_19_emu_audio.rs"],
  spec="""Dans `emu_thread.rs` (comportement existant conserve):
```rust
pub fn frame_period_ns(region: puce16_core::Region) -> u64;      // Ntsc 16_639_267 (1e9 / 60.0988) ; Pal 19_997_200 (1e9 / 50.0070)
pub fn spawn_with_audio(region: puce16_core::Region, start_paused: bool, ring: Option<crate::audio::SharedRing>) -> EmuHandle;
```
`spawn_test_pattern(region, paused)` devient `spawn_with_audio(region, paused, None)`.
Boucle (pas en pause): `run_frame()` ; `snes.take_audio(&mut v)` ; si `ring` est `Some` et PAS d'avance rapide : pousser `v` dans l'anneau (verrou court; trames refusees = perdues) puis ATTENDRE tant que `ring.len() > RING_WAIT_ABOVE` : dormir 1 ms, traiter les commandes pendant l'attente (`Quit`, `Pause`, ...), au plus 100 ms par frame. Si `ring` est `None` et pas d'avance rapide : cadence par echeance `next += frame_period_ns` (`Instant`), dormir jusqu'a `next` ; si le retard depasse 100 ms, `next = maintenant`. Avance rapide (`SetFastForward(true)`) : ni attente ni audio (`v` jete, anneau vide). Le motif de test ne produit pas d'audio. Region: celle de la ROM chargee (`cart_info().region`) apres `LoadRom` ; sinon celle donnee au `spawn`. En pause, ne rien pousser.""" + " " + TINY,
  tests=[("frame_period", "`frame_period_ns(Ntsc)==16_639_267` ; `frame_period_ns(Pal)==19_997_200`."),
         ("audio_flows_into_ring", "ring = `new_shared_ring()` ; `spawn_with_audio(Ntsc,false,Some(ring.clone()))` ; `LoadRom(tiny_rom(), \"t\")` ; attendre 300 ms : `ring.lock().len() > 500`."),
         ("back_pressure", "meme montage, aucun consommateur, attendre 600 ms : `len() <= RING_WAIT_ABOVE + 600` (l'emulation s'est mise en attente) et `len() >= RING_WAIT_ABOVE - 600`."),
         ("consumer_unblocks", "meme montage ; un thread consomme (`pop` de 256 toutes les 5 ms) : au moins 20 `Event::Frame` recus en 2 s."),
         ("fast_forward_ignores_ring", "meme montage sans consommateur, `SetFastForward(true)` : au moins 30 `Event::Frame` en 1 s ; apres `SetFastForward(false)` le debit retombe (le nombre de frames des 300 ms suivantes est < 30)."),
         ("timer_pacing_without_ring", "`spawn_with_audio(Ntsc,false,None)` + `LoadRom(tiny_rom())` : 12 `Event::Frame` successives arrivent en plus de 150 ms et en moins de 1000 ms (cadence ~60 Hz)."),
         ("pause_pushes_nothing", "meme montage avec ring, `Pause(true)` puis vider l'anneau et attendre 200 ms : `len()==0`.")],
  accept=["cargo test -p puce16-desktop e08_19_", "cargo test -p puce16-desktop e03_46_", "cargo test -p puce16-desktop e01_09_"])

T("E08-20", "files.rs: chemins SRAM/save states, ecriture atomique, en-tete .p16s",
  size=130, crate="puce16-desktop",
  context="Fonctions pures de la spec 08 section 6: chemins derives de la ROM, ecriture atomique, conteneur de save state `.p16s`, chemin de capture. " + CHAIN,
  embed=["08_INPUT_DESKTOP.md#6. Fichiers"],
  write=[SRC + "/files.rs", DT + "/e08_20_files.rs"],
  spec="""Ajoute dans `files.rs` (sans toucher aux fonctions de E03-45):
```rust
use std::path::{Path, PathBuf};
pub fn srm_path(rom: &Path) -> PathBuf;                 // meme dossier, nom sans extension + ".srm"
pub fn state_path(rom: &Path, slot: u8) -> PathBuf;     // "<nom>.<slot>.p16s" (slot 0..=9, sinon slot % 10)
pub fn screenshot_path(rom: &Path, exists: &dyn Fn(&Path) -> bool) -> PathBuf;   // "<nom>_<n>.ppm", n = premier entier >= 1 tel que !exists(chemin)
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String>;            // ecrit "<chemin>.tmp" puis rename ; Err("cannot write <chemin>: <err>")
pub fn rom_hash(bytes: &[u8]) -> u64;                   // FNV-1a 64 (puce16_core::hash::Fnv64) des octets
pub const STATE_MAGIC: [u8; 4] = *b"P16S";
pub const STATE_CONTAINER_VERSION: u16 = 1;
pub fn encode_state(rom_hash: u64, region: u8, blob: &[u8]) -> Vec<u8>;   // magic, u16 version LE, u64 hash LE, u8 region, puis blob
pub fn decode_state(data: &[u8], expect_rom_hash: u64) -> Result<(u8, &[u8]), String>;
```
`decode_state` renvoie `(region, blob)` ou `Err`: `"not a Puce16 state"` (fichier < 15 octets ou magie fausse), `"unsupported state container version <v>"`, `"state belongs to another ROM"` (hash different). L'en-tete fait 4 + 2 + 8 + 1 = 15 octets. "Nom" = nom de fichier sans extension (`file_stem`) ; dossier = parent de la ROM (`.` si absent).""",
  tests=[("srm_path", "`srm_path(\"/g/Super Mario World.sfc\")==\"/g/Super Mario World.srm\"` ; `srm_path(\"rom.smc\")==\"rom.srm\"` ; un nom sans extension `\"game\"` -> `\"game.srm\"`."),
         ("state_path", "`state_path(\"/g/a.sfc\", 3)==\"/g/a.3.p16s\"` ; slot 12 -> `a.2.p16s`."),
         ("screenshot_path", "`exists` toujours faux -> `\"/g/a_1.ppm\"` ; `exists` vrai pour `a_1.ppm` et `a_2.ppm` seulement -> `a_3.ppm`."),
         ("write_atomic", "dans un dossier temporaire: ecrit [1,2,3] ; relu == [1,2,3] ; le fichier `.tmp` n'existe plus ; une seconde ecriture remplace le contenu ; dossier inexistant -> `Err` contenant `cannot write`."),
         ("rom_hash", "`rom_hash(&[])==0xcbf29ce484222325` ; `rom_hash(&[1,2,3])` == valeur calculee dans le test avec `Fnv64` ; deux ROM differentes -> hash differents."),
         ("encode_header_layout", "`encode_state(0x0102030405060708, 1, &[9,9])` == `b\"P16S\"` + `[1,0]` + `[8,7,6,5,4,3,2,1]` + `[1]` + `[9,9]` (longueur 17)."),
         ("decode_roundtrip", "`decode_state(&encode_state(h, 1, blob), h) == Ok((1, blob))`."),
         ("decode_errors", "magie `XXXX...` -> `Err(\"not a Puce16 state\")` ; 10 octets -> idem ; version 2 -> `unsupported state container version 2` ; autre hash -> `state belongs to another ROM`.")],
  accept=["cargo test -p puce16-desktop e08_20_"])

T("E08-21", "emu_thread: SRAM (chargement, sauvegarde toutes les 5 s, a la fermeture)",
  size=140, crate="puce16-desktop",
  context="La SRAM de la cartouche est lue a l'ouverture de la ROM et ecrite sur disque quand elle change (D-017). " + CHAIN,
  embed=["08_INPUT_DESKTOP.md#6. Fichiers"],
  write=[SRC + "/emu_thread.rs", SRC + "/app.rs", DT + "/e08_21_sram.rs"],
  spec="""Dans `emu_thread.rs`:
- `Command` gagne `SetRomPath(std::path::PathBuf)` (envoye par l'UI juste AVANT `LoadRom`) et `FlushSram`. `Event` gagne `SramSaved(std::path::PathBuf)`.
- `pub const SRAM_SAVE_INTERVAL: Duration = Duration::from_secs(5);`
- Pure et testable: `pub struct SramSync { last_saved: Option<Vec<u8>>, last_check: Instant }` avec `pub fn new(now: Instant) -> SramSync`, `pub fn reset(&mut self, now: Instant, current: Option<&[u8]>)` (memorise `current` comme deja sauve), `pub fn due(&mut self, now: Instant) -> bool` (vrai si `now - last_check >= SRAM_SAVE_INTERVAL` ; dans ce cas `last_check = now`), `pub fn changed(&self, sram: &[u8]) -> bool` (vrai si different de `last_saved` ou si `last_saved` est None), `pub fn mark_saved(&mut self, sram: &[u8])`.
- Dans le thread: apres un `LoadRom` reussi, si `rom_path` est connu et que `snes.sram()` est `Some` : si `srm_path(rom_path)` existe, `snes.load_sram(&octets)` (lecture ratee -> `Event::Error`, on continue) ; `sync.reset(..)`. Avant de remplacer le `Snes` (nouveau `LoadRom`), a `Quit` et sur `FlushSram` : sauvegarder si `changed`. Pendant l'execution: a chaque frame `if sync.due(now) && sram changed` -> `write_atomic(srm_path, sram)` puis `mark_saved` et `Event::SramSaved(path)`. Pas de SRAM (`None`) -> aucun fichier. Echec d'ecriture -> `Event::Error`.
- Dans `app.rs`: `open_path` envoie `Command::SetRomPath(path)` avant `Command::LoadRom`; la barre d'etat affiche `SRAM saved` sur `Event::SramSaved`.""" + " " + SRAMROM + " " + TMP,
  tests=[("sync_due", "`SramSync::new(t0)` : `due(t0 + 4s)` faux ; `due(t0 + 5s)` vrai ; ensuite `due(t0 + 6s)` faux (le dernier controle est a t0+5s) ; `due(t0 + 10s)` vrai."),
         ("sync_changed", "`changed(&[1])` vrai (rien de sauve) ; apres `mark_saved(&[1])` : faux ; `changed(&[2])` vrai ; `reset(now, Some(&[7]))` puis `changed(&[7])` faux."),
         ("flush_writes_file", "dossier temporaire, ROM `sram_rom(PROG_W)` ecrite dans `dir/game.sfc` ; `spawn_with_audio(Ntsc,true,None)` ; `SetRomPath(dir/game.sfc)` ; `LoadRom(rom, \"game.sfc\")` ; attendre 300 ms (le programme ecrit 0x5A en SRAM) ; `FlushSram` : `Event::SramSaved` recu (timeout 2 s) ; `dir/game.srm` existe, taille 8192, octet 0 == 0x5A."),
         ("sram_loaded_on_open", "suite du test precedent : nouveau thread, ROM `sram_rom(PROG_R)` au meme chemin `game.sfc` ; apres `LoadRom` + 300 ms + `FlushSram` : `game.srm` octet 0 == 0x5A (conserve du fichier) et octet 1 == 0x5B (le programme a lu 0x5A et ecrit +1) : preuve du chargement."),
         ("quit_flushes", "meme montage que `flush_writes_file` mais sans `FlushSram` : `quit()` du handle produit le fichier `.srm`."),
         ("no_sram_no_file", "ROM `tiny_rom()` : apres `LoadRom` + `FlushSram` + `quit`, aucun fichier `.srm` n'existe dans le dossier."),
         ("unchanged_not_rewritten", "apres un premier `FlushSram` (fichier ecrit), supprimer le fichier a la main, `FlushSram` de nouveau (SRAM inchangee) : le fichier n'est PAS recree.")],
  accept=["cargo test -p puce16-desktop e08_21_", "cargo test -p puce16-desktop e03_46_", "cargo build -p puce16-desktop"])

T("E08-22", "emu_thread: save states (SaveState / LoadState, slots 0..9)",
  size=140, crate="puce16-desktop",
  context="Utilise `Snes::save_state` / `load_state` (core, E08-12) et le conteneur `.p16s` (E08-20). " + CHAIN,
  embed=["08_INPUT_DESKTOP.md#6. Fichiers", "08_INPUT_DESKTOP.md#7. Architecture desktop"],
  write=[SRC + "/emu_thread.rs", DT + "/e08_22_states.rs"],
  spec="""Dans `emu_thread.rs`:
- `Command` gagne `SaveState(u8)` et `LoadState(u8)`. Le thread garde `rom_path: Option<PathBuf>` (E08-21) et `rom_hash` (calcule au `LoadRom` avec `files::rom_hash(&bytes)`).
- `SaveState(slot)`: sans ROM chargee (motif de test) ou sans `rom_path` : `Event::Error(\"no ROM loaded\")`. Sinon `blob = snes.save_state()`, `data = encode_state(rom_hash, region_u8, &blob)` (Ntsc 0, Pal 1), `write_atomic(state_path(rom_path, slot), &data)` ; `Event::Status(format!(\"State saved to slot {}\", slot))` ; erreur d'ecriture -> `Event::Error`.
- `LoadState(slot)`: lire le fichier (absent -> `Event::Error(\"no state in slot <n>\")`), `decode_state(&data, rom_hash)` (erreur -> `Event::Error(<message>)`), region differente de celle de la console -> `Event::Error(\"state has another region\")`, puis `snes.load_state(blob)` : `Err(e)` -> `Event::Error(format!(\"cannot load state: {}\", e))` ET rechargement propre de la console (`Snes::new` sur les memes octets de ROM + SRAM rechargee) car son etat peut etre incoherent ; `Ok` -> `Event::Status(format!(\"State loaded from slot {}\", slot))`, puis publier immediatement une `Event::Frame` de la frame courante (meme en pause).
- Ne pas modifier les autres comportements.""" + " " + SRAMROM + " " + TMP,
  tests=[("save_without_rom", "`spawn_with_audio(Ntsc,true,None)` (motif de test) + `SaveState(0)` -> `Event::Error` contenant `no ROM loaded`."),
         ("save_creates_file", "dossier temporaire, `sram_rom(PROG_W)` dans `dir/game.sfc`, `SetRomPath`, `LoadRom` ; `FrameAdvance` x2 ; `SaveState(3)` -> `Event::Status` contenant `State saved to slot 3` ; `dir/game.3.p16s` existe, commence par `P16S`, taille > 1_000_000."),
         ("load_restores_frame_counter", "meme montage : `FrameAdvance` x2 (frame_no 1, 2), `SaveState(0)`, `FrameAdvance` x3 (frame_no 5), `LoadState(0)` : `Event::Status` contenant `State loaded from slot 0` puis une `Event::Frame` avec `frame_no==2` ; un `FrameAdvance` de plus -> `frame_no==3`."),
         ("load_missing_slot", "`LoadState(7)` sans fichier -> `Event::Error` contenant `no state in slot 7`."),
         ("load_foreign_rom", "ecrire un etat avec la ROM A (`PROG_W`), puis charger la ROM B (`PROG_R`, MEME chemin `game.sfc` pour retrouver le fichier d'etat) : `LoadState(0)` -> `Event::Error` contenant `another ROM`."),
         ("load_corrupt_blob", "ecrire `encode_state(rom_hash, 0, &[1,2,3])` dans `game.1.p16s` : `LoadState(1)` -> `Event::Error` contenant `cannot load state` ; ensuite `FrameAdvance` fonctionne encore (console rechargee)."),
         ("load_works_while_running", "sans pause : `SaveState(0)`, attendre 200 ms, `LoadState(0)` : `Event::Status` de chargement recu et les frames continuent d'arriver."),
         ("pal_region_byte", "`encode_state` utilise l'octet 1 pour une ROM PAL : ROM `sram_rom` avec `r[0x7FD9] = 0x02` (pays Europe) rechargee : le fichier d'etat a l'octet de region (indice 14) egal a 1.")],
  accept=["cargo test -p puce16-desktop e08_22_", "cargo test -p puce16-desktop e08_21_", "cargo build -p puce16-desktop"])

T("E08-23", "app.rs: raccourcis save states / slots / capture d'ecran / muet / avance rapide",
  size=140, kind="ui", crate="puce16-desktop",
  context="Branche les raccourcis de la spec 08 section 2 qui n'etaient que journalises en E01-13, et le menu Emulation (slots, sauvegarde, chargement). Code UI: pas de test automatique. " + CHAIN,
  embed=["08_INPUT_DESKTOP.md#2. Clavier (AZERTY) - touches LOGIQUES", "08_INPUT_DESKTOP.md#4. Fenetre, video"],
  write=[SRC + "/app.rs"],
  spec="""Dans `app.rs` (utilise `input::hotkey_for` et `Hotkey` de E01-13):
- `App` gagne `slot: u8` (0..=9, defaut 0), `muted: bool`, `fast_forward: bool`, `audio: Option<AudioOut>` (rempli en E08-25), `rom_path: Option<PathBuf>`.
- `Hotkey::SaveState` -> `Command::SaveState(slot)` ; `LoadState` -> `Command::LoadState(slot)` ; `SlotPrev` / `SlotNext` -> `slot = (slot + 9) % 10` / `(slot + 1) % 10` avec statut `Slot <n>` ; `FastForward` (Tab): maintenu -> `SetFastForward(true)` tant que la touche est enfoncee, `false` au relachement (envoyer seulement sur changement) ; `Mute` (M) -> bascule `muted` (et `audio.set_muted` si present) ; `Screenshot` (F2) -> demander la derniere frame et ecrire un PPM : l'App garde la derniere `Arc<FrameData>` recue ; chemin `files::screenshot_path(rom_path, &|p| p.exists())` (si pas de ROM: `screenshot_0.ppm` dans le dossier courant) ; contenu P6 (meme encodage que `puce16-cli`: en-tete `P6\\n<w> <h>\\n255\\n` puis RGB) ; statut `Screenshot saved: <chemin>` ; `Fullscreen`, `Debugger` restent journalises (`hotkey not implemented`) jusqu'aux taches suivantes.
- Menu Emulation: sous-menu "Etat" avec "Sauvegarder (F5)", "Charger (F7)", et un selecteur de slot 0..9 ; l'entree "Avance rapide (Tab)" est une case a cocher ; la barre d'etat affiche le slot courant (`slot <n>`).
- Les touches de jeu continuent d'alimenter le port 0; ne pas envoyer `SetJoypad` quand une fenetre de dialogue a le focus clavier (`ctx.wants_keyboard_input()`).""",
  tests=[],
  accept=["cargo build -p puce16-desktop"],
  notes="Verification visuelle faite par l'utilisateur au GATE E08-31.")

T("E08-24", "config.rs: configuration JSON (defauts, aller-retour, ROM recentes)",
  size=130, crate="puce16-desktop",
  context="Fichier `config.json` de la spec 08 section 6. Chargement tolerant: un fichier absent ou invalide donne les valeurs par defaut, jamais un plantage. " + CHAIN,
  embed=["08_INPUT_DESKTOP.md#6. Fichiers", "08_INPUT_DESKTOP.md#4. Fenetre, video"],
  write=[SRC + "/config.rs", DT + "/e08_24_config.rs"],
  spec="""Dans `config.rs` (serde + serde_json deja en dependances):
```rust
#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Config {
    pub scale: u8,                         // 1..=8, defaut 3
    pub fit_window: bool,                  // defaut false
    pub ratio: String,                     // "square" (defaut) | "par87"
    pub filter: String,                    // "nearest" (defaut) | "linear" | "scanlines"
    pub volume: u8,                        // 0..=100, defaut 100
    pub muted: bool,                       // defaut false
    pub fullscreen: bool,                  // defaut false
    pub block_opposite_dirs: bool,         // defaut true
    pub pause_on_focus_loss: bool,         // defaut false
    pub recent_roms: Vec<String>,          // max 8, le plus recent en premier
    pub last_dir: String,                  // defaut ""
    pub keymap: std::collections::BTreeMap<String, Vec<String>>,   // bouton SNES -> noms de touches ; vide = defauts du code
    pub pad_map: std::collections::BTreeMap<String, String>,       // bouton SNES -> nom de bouton manette ; vide = defauts
}
impl Default for Config { ... }
impl Config {
    pub fn sanitize(&mut self);                         // scale -> 1..=8 ; volume -> <= 100 ; ratio/filter inconnus -> defaut ; recent_roms tronquee a 8 sans doublons
    pub fn add_recent(&mut self, path: &str);           // met `path` en tete, retire l'ancien doublon, tronque a 8
    pub fn to_json(&self) -> String;                    // serde_json::to_string_pretty + "\\n"
    pub fn from_json(text: &str) -> Config;             // jamais d'erreur : JSON invalide -> Config::default() ; puis sanitize
}
pub fn config_dir_from(appdata: Option<&str>, home: Option<&str>) -> Option<std::path::PathBuf>;
pub fn config_path() -> Option<std::path::PathBuf>;     // lit les variables d'environnement APPDATA et HOME puis config_dir_from(..).map(|d| d.join("config.json"))
pub fn load() -> Config;                                // lit config_path ; absent/illisible -> defaut
pub fn save(cfg: &Config) -> Result<(), String>;        // cree le dossier, write_atomic de to_json
```
`config_dir_from`: si `appdata` est `Some(non vide)` -> `<appdata>/Puce16` ; sinon si `home` est `Some(non vide)` -> `<home>/.config/puce16` ; sinon `None`.""",
  tests=[("defaults", "`Config::default()`: scale 3, ratio `square`, filter `nearest`, volume 100, `muted==false`, `block_opposite_dirs==true`, `recent_roms` vide."),
         ("roundtrip", "config modifiee (scale 5, filter `scanlines`, volume 40, 2 ROM recentes, une entree dans `keymap`) -> `to_json` -> `from_json` == original."),
         ("invalid_json_gives_defaults", "`from_json(\"not json\")==Config::default()` ; `from_json(\"\")==Config::default()`."),
         ("partial_json", "`from_json(\"{\\\"scale\\\": 6}\")`: scale 6, tous les autres champs par defaut."),
         ("sanitize", "`scale 0` -> 1 ; `scale 99` -> 8 ; `volume 250` -> 100 ; `ratio \"weird\"` -> `square` ; `filter \"x\"` -> `nearest` ; 12 ROM recentes -> 8."),
         ("add_recent", "ajouter `a`, `b`, `a` -> `[\"a\",\"b\"]` (le doublon remonte en tete) ; 10 ajouts distincts -> longueur 8 et le dernier ajoute en tete."),
         ("config_dir", "`config_dir_from(Some(\"C:/Users/x/AppData/Roaming\"), None)` -> `C:/Users/x/AppData/Roaming/Puce16` ; `(None, Some(\"/home/x\"))` -> `/home/x/.config/puce16` ; `(Some(\"\"), Some(\"/home/x\"))` -> la branche HOME ; `(None, None)` -> `None`."),
         ("save_load_in_temp", "`save` n'est pas teste sur le vrai dossier utilisateur : tester `to_json`/`from_json` + `files::write_atomic` dans un dossier temporaire (fichier relu == `to_json`).")],
  accept=["cargo test -p puce16-desktop e08_24_"])
