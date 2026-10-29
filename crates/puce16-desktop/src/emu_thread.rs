//! Emulation thread (spec 08_INPUT_DESKTOP section 7, milestone D0).
//!
//! A dedicated thread owns the `Snes` state and talks to the UI only through
//! channels: commands in (`Command`), events out (`Event`). No emulation
//! state is shared by lock. This module runs the test pattern (E01-06); ROM
//! loading, save states, audio and debug data come with later tasks.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use puce16_core::{Region, Snes};

/// Debug data sent to the UI when debugging is active.
#[derive(Debug)]
pub struct DebugData {
    pub snapshot: puce16_core::snes::DebugSnapshot,
    pub vram: Vec<u16>,
    pub cgram: Vec<u16>,
    pub oam: Vec<u8>,
    pub wram_page: Vec<u8>,
    pub ppu_regs: Option<puce16_core::ppu::PpuRegsSnapshot>,
    pub smp: Option<puce16_core::snes::SmpSnapshot>,
    pub dma: Option<[puce16_core::snes::DmaChanSnapshot; 8]>,
}

/// Maximum instructions per block when running to a PC breakpoint.
pub const RUN_TO_PC_BLOCK: u32 = 100_000;
/// Maximum number of blocks for RunToPc before giving up.
pub const RUN_TO_PC_MAX_BLOCKS: u32 = 100;

use crate::files;

/// Commands sent from the UI thread to the emulation thread.
pub enum Command {
    /// Pause or resume (`true` = paused).
    Pause(bool),
    /// While paused: run exactly one frame and publish it.
    FrameAdvance,
    /// Set the buttons of a joypad port (0 or 1; other ports ignored).
    SetJoypad(usize, u16),
    /// Load a ROM from bytes; name is for display only.
    LoadRom(Vec<u8>, String),
    /// Recreate the emulator state; the next frame is number 1.
    Reset,
    /// Run frames without waiting for cadence (`true` = fast forward on).
    SetFastForward(bool),
    /// Stop the emulation thread.
    Quit,
    /// Set the ROM path (sent before LoadRom) so SRAM can be loaded/saved.
    SetRomPath(PathBuf),
    /// Force an immediate SRAM save to disk.
    FlushSram,
    /// Save emulator state to slot (0..9).
    SaveState(u8),
    /// Load emulator state from slot (0..9).
    LoadState(u8),
    /// Enable or disable debug data output.
    RequestDebug(bool),
    /// Set the WRAM page for debug inspection (0..255).
    SetWramPage(u16),
    /// Step one CPU instruction (only when paused).
    StepInstruction,
    /// Run until PC reaches address (pb << 16) | pc; only when paused.
    RunToPc(u32),
}

/// One published frame: compact XRGB8888 pixels plus metadata.
#[derive(Debug)]
pub struct FrameData {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
    pub frame_no: u64,
}

/// Events sent from the emulation thread to the UI thread.
#[derive(Debug)]
pub enum Event {
    /// A new frame was emulated (the UI displays the most recent one).
    Frame(Arc<FrameData>),
    /// Human-readable status line.
    Status(String),
    /// Error message (ROM load failure, emulator panic).
    Error(String),
    /// SRAM was saved to disk at this path.
    SramSaved(PathBuf),
    /// Debug data (sent after each frame when debugging is active).
    Debug(Box<DebugData>),
}

/// Interval between automatic SRAM saves (spec 08 section 6).
pub const SRAM_SAVE_INTERVAL: Duration = Duration::from_secs(5);

/// Tracks whether SRAM has changed since the last save and when to check again.
pub struct SramSync {
    /// Last saved SRAM contents (None means nothing saved yet).
    pub last_saved: Option<Vec<u8>>,
    /// Time of the last periodic save check.
    pub last_check: Instant,
}

impl SramSync {
    /// Create a new sync tracker with no saved state.
    pub fn new(now: Instant) -> SramSync {
        SramSync {
            last_saved: None,
            last_check: now,
        }
    }

    /// Reset the tracker after loading a ROM or SRAM file.
    /// If `current` is Some, treat it as already saved to disk.
    pub fn reset(&mut self, now: Instant, current: Option<&[u8]>) {
        self.last_saved = current.map(|c| c.to_vec());
        self.last_check = now;
    }

    /// Check if it's time for a periodic save (every SRAM_SAVE_INTERVAL).
    /// Returns true and updates last_check when due.
    pub fn due(&mut self, now: Instant) -> bool {
        if now - self.last_check >= SRAM_SAVE_INTERVAL {
            self.last_check = now;
            return true;
        }
        false
    }

    /// Check if SRAM has changed since the last save.
    pub fn changed(&self, sram: &[u8]) -> bool {
        match &self.last_saved {
            None => true,
            Some(saved) => saved != sram,
        }
    }

    /// Mark SRAM as saved after a successful write.
    pub fn mark_saved(&mut self, sram: &[u8]) {
        self.last_saved = Some(sram.to_vec());
    }
}

/// Handle to a running emulation thread.
pub struct EmuHandle {
    pub tx: Sender<Command>,
    pub rx: Receiver<Event>,
    join: Option<JoinHandle<()>>,
}

impl EmuHandle {
    /// Send `Quit` and wait for the emulation thread to exit.
    pub fn quit(self) {
        let _ = self.tx.send(Command::Quit);
        if let Some(handle) = self.join {
            handle.join().expect("emu thread panicked");
        }
    }
}

/// Frame period in nanoseconds for a given region (reference cadence).
pub fn frame_period_ns(region: Region) -> u64 {
    match region {
        Region::Ntsc => 16_639_267, // 1e9 / 60.0988
        Region::Pal => 19_997_200,  // 1e9 / 50.0070
    }
}

/// Spawn an emulation thread with optional audio ring buffer.
pub fn spawn_with_audio(
    region: Region,
    start_paused: bool,
    ring: Option<crate::audio::SharedRing>,
) -> EmuHandle {
    let (tx_cmd, rx_cmd) = mpsc::channel();
    let (tx_event, rx_event) = mpsc::channel();
    let join = thread::Builder::new()
        .name("puce16-emu".to_string())
        .spawn(move || emu_loop(region, start_paused, ring, rx_cmd, tx_event))
        .expect("failed to spawn the emulation thread");
    EmuHandle {
        tx: tx_cmd,
        rx: rx_event,
        join: Some(join),
    }
}

/// Main loop of the emulation thread (spec 08 section 7).
fn emu_loop(
    region: Region,
    start_paused: bool,
    ring: Option<crate::audio::SharedRing>,
    rx_cmd: Receiver<Command>,
    tx_event: Sender<Event>,
) {
    let mut snes = Snes::new_test_pattern(region);
    let mut paused = start_paused;
    let mut fast_forward = false;
    let mut joy = [0u16; 2];
    let mut frame_no: u64 = 0;
    // Region from loaded ROM overrides the spawn region.
    let mut active_region = region;
    // Timer-based pacing deadline (used when no audio ring).
    let mut next: Option<std::time::Instant> = None;
    // SRAM sync state.
    let mut rom_path: Option<PathBuf> = None;
    let mut rom_hash: u64 = 0;
    let mut rom_bytes: Vec<u8> = Vec::new();
    let mut sync = SramSync::new(Instant::now());

    // Debug state.
    let mut debug_active = false;
    let mut wram_page = 0u16;
    let mut rom_loaded = false;

    loop {
        // Drain every pending command before doing any work.
        while let Ok(cmd) = rx_cmd.try_recv() {
            match cmd {
                Command::Pause(pause) => { paused = pause; next = None; }
                Command::FrameAdvance => {
                    if paused {
                        run_one_frame(&mut snes, &mut frame_no, &tx_event, debug_active, wram_page, rom_loaded);
                    }
                }
                Command::SetJoypad(port, buttons) => {
                    if port < 2 { joy[port] = buttons; }
                    snes.set_joypad(port, buttons);
                }
                Command::SetRomPath(path) => {
                    // Save SRAM for the previous ROM before switching.
                    save_sram(&snes, &rom_path, &mut sync, &tx_event);
                    let _ = rom_path.replace(path);
                }
                Command::FlushSram => {
                    if let Some(ref path) = rom_path {
                        save_sram(&snes, &Some(path.clone()), &mut sync, &tx_event);
                    }
                }
                Command::SaveState(slot) => {
                    handle_save_state(&snes, slot, &rom_path, rom_hash, active_region, &tx_event);
                }
                Command::LoadState(slot) => {
                    let result = handle_load_state(
                        &mut snes, slot, &rom_path, rom_hash, active_region,
                        &rom_bytes, &joy, &mut frame_no, &mut sync, &tx_event);
                    if let Some(new_snes) = result {
                        snes = new_snes;
                    }
                }
                Command::RequestDebug(on) => {
                    debug_active = on;
                }
                Command::SetWramPage(page) => {
                    wram_page = page;
                    if debug_active && rom_loaded {
                        send_debug_data(&snes, wram_page, true, &tx_event);
                    }
                }
                Command::StepInstruction => {
                    if paused {
                        snes.step_instruction();
                        if debug_active && rom_loaded {
                            send_debug_data(&snes, wram_page, true, &tx_event);
                        }
                    } else {
                        let _ = tx_event.send(Event::Status("pause first".to_string()));
                    }
                }
                Command::RunToPc(addr) => {
                    if !paused {
                        let _ = tx_event.send(Event::Status("pause first".to_string()));
                    } else {
                        match run_to_pc(&mut snes, addr, &rx_cmd, &tx_event, debug_active, wram_page, rom_loaded) {
                            RunToPcOutcome::Finished => {}
                            RunToPcOutcome::Cancelled => { paused = false; next = None; }
                            RunToPcOutcome::Quit => {
                                save_sram(&snes, &rom_path, &mut sync, &tx_event);
                                return;
                            }
                        }
                    }
                }
                Command::LoadRom(bytes, name) => {
                    // Save SRAM for the previous ROM before loading a new one.
                    save_sram(&snes, &rom_path, &mut sync, &tx_event);

                    match Snes::new(&bytes, None) {
                        Ok(new_snes) => {
                            snes = new_snes;
                            rom_loaded = true;
                            rom_hash = files::rom_hash(&bytes);
                            rom_bytes = bytes.clone();
                            // Re-apply last joypad settings.
                            for (port, buttons) in joy.iter().enumerate() {
                                snes.set_joypad(port, *buttons);
                            }
                            frame_no = 0;
                            paused = false;
                            next = None;
                            if let Some(info) = snes.cart_info() {
                                active_region = info.region;
                                let _ = tx_event.send(Event::Status(format!(
                                    "Loaded {} ({:?}, {:?})", name, info.map, info.region)));
                            } else {
                                let _ = tx_event.send(Event::Status(format!("Loaded {}", name)));
                            }

                            // Load SRAM if file exists.
                            if let Some(ref path) = rom_path {
                                load_sram(&mut snes, &path, &mut sync, &tx_event);
                            }
                        }
                        Err(puce16_core::snes::LoadError::Cart(e)) => {
                            let msg = match e {
                                puce16_core::cart::CartError::TooSmall => "ROM too small".to_string(),
                                puce16_core::cart::CartError::NoHeader => "no valid ROM header found".to_string(),
                                puce16_core::cart::CartError::UnsupportedCoprocessor(n) => {
                                    format!("unsupported coprocessor: {}", n)
                                }
                            };
                            let _ = tx_event.send(Event::Error(format!(
                                "cannot load {}: {}", name, msg)));
                        }
                    }
                }
                Command::Reset => {
                    snes.reset();
                    frame_no = 0;
                }
                Command::SetFastForward(on) => { fast_forward = on; next = None; }
                Command::Quit => {
                    save_sram(&snes, &rom_path, &mut sync, &tx_event);
                    return;
                }
            }
        }

        if paused {
            thread::sleep(Duration::from_millis(5));
        } else {
            run_one_frame(&mut snes, &mut frame_no, &tx_event, debug_active, wram_page, rom_loaded);

            // Periodic SRAM save check.
            let now = Instant::now();
            if sync.due(now) {
                if let Some(ref path) = rom_path {
                    save_sram(&snes, &Some(path.clone()), &mut sync, &tx_event);
                }
            }

            // Audio: take samples from SNES and push to ring (if any).
            let mut audio_buf = Vec::new();
            snes.take_audio(&mut audio_buf);

            if fast_forward {
                // Fast forward: discard audio, no wait.
            } else if let Some(ring_ref) = &ring {
                if !audio_buf.is_empty() {
                    // Back-pressure: wait while the ring is above RING_WAIT_ABOVE, but never longer
                    // than one frame period (the consumer may be stalled or absent).
                    let max_wait_ms = (frame_period_ns(active_region) / 1_000_000) as u32 + 1;
                    let mut waited_ms = 0;
                    let mut has_room = false;
                    loop {
                        match ring_ref.lock() {
                            Ok(r) => {
                                if r.len() <= crate::audio::RING_WAIT_ABOVE {
                                    has_room = true;
                                    break;
                                }
                            }
                            Err(_) => break, // Poisoned: give up waiting.
                        }
                        if waited_ms >= max_wait_ms {
                            break;
                        }
                        thread::sleep(Duration::from_millis(1));
                        waited_ms += 1;
                    }
                    // Still above the threshold after the wait: drop this frame's audio so the
                    // ring stays bounded (a stalled consumer must not make it grow).
                    if has_room {
                        match ring_ref.lock() {
                            Ok(mut r) => { r.push(&audio_buf); }
                            Err(poisoned) => { poisoned.into_inner().push(&audio_buf); }
                        }
                    }
                }
            } else {
                // No audio: timer-based pacing at console cadence.
                let period = Duration::from_nanos(frame_period_ns(active_region));
                let now = Instant::now();
                let mut target = match next {
                    Some(t) => t + period,
                    None => now + period,
                };
                if now > target + Duration::from_millis(100) {
                    // More than 100 ms behind: resynchronise instead of running a burst.
                    target = now + period;
                }
                if now < target {
                    thread::sleep(target - now);
                }
                next = Some(target);
            }
        }
    }
}

/// Load SRAM from disk into the emulator (if file exists).
fn load_sram(
    snes: &mut Snes,
    rom_path: &PathBuf,
    sync: &mut SramSync,
    tx_event: &Sender<Event>,
) {
    let srm = files::srm_path(&rom_path);
    if !srm.exists() {
        return;
    }

    match std::fs::read(&srm) {
        Ok(data) => {
            snes.load_sram(&data);
            sync.reset(Instant::now(), Some(&data));
        }
        Err(e) => {
            let _ = tx_event.send(Event::Error(format!(
                "cannot load SRAM {}: {}", srm.display(), e)));
        }
    }
}

/// Save SRAM to disk if it has changed.
fn save_sram(
    snes: &Snes,
    rom_path: &Option<PathBuf>,
    sync: &mut SramSync,
    tx_event: &Sender<Event>,
) {
    let sram = match snes.sram() {
        Some(s) => s.to_vec(),
        None => return, // No SRAM -> no file.
    };

    if !sync.changed(&sram) {
        return; // Unchanged since last save.
    }

    let path = match rom_path {
        Some(p) => files::srm_path(&p),
        None => return,
    };

    match files::write_atomic(&path, &sram) {
        Ok(()) => {
            sync.mark_saved(&sram);
            let _ = tx_event.send(Event::SramSaved(path));
        }
        Err(e) => {
            let _ = tx_event.send(Event::Error(format!(
                "cannot save SRAM {}: {}", path.display(), e)));
        }
    }
}

/// Run one emulated frame with panic protection. Sends debug data if active.
fn run_one_frame(
    snes: &mut Snes,
    frame_no: &mut u64,
    tx_event: &Sender<Event>,
    debug_active: bool,
    wram_page: u16,
    rom_loaded: bool,
) {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| snes.run_frame()));
    match result {
        Ok(()) => {
            *frame_no += 1;
            publish_frame(snes, *frame_no, tx_event);
            if debug_active && rom_loaded {
                send_debug_data(snes, wram_page, true, tx_event);
            } else if debug_active && !rom_loaded {
                // Test pattern mode: send empty debug data.
                let data = DebugData {
                    snapshot: puce16_core::snes::DebugSnapshot {
                        pc: 0, a: 0, x: 0, y: 0, s: 0, d: 0, db: 0, pb: 0, p: 0, e: false,
                        v: 0, dot: 0, master: 0, frame: 0,
                    },
                    vram: Vec::new(),
                    cgram: Vec::new(),
                    oam: Vec::new(),
                    wram_page: Vec::new(),
                    ppu_regs: None,
                    smp: None,
                    dma: None,
                };
                let _ = tx_event.send(Event::Debug(Box::new(data)));
            }
        }
        Err(payload) => {
            let msg = if let Some(s) = payload.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = payload.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "unknown panic".to_string()
            };
            let _ = tx_event.send(Event::Error(format!("emulator panic: {}", msg)));
        }
    }
}

/// Copy the current frame into a `FrameData` and send it as an event.
fn publish_frame(snes: &Snes, frame_no: u64, tx_event: &Sender<Event>) {
    let view = snes.frame();
    let data = FrameData {
        width: view.width,
        height: view.height,
        pixels: view.pixels.to_vec(),
        frame_no,
    };
    // The channel is unbounded and the UI consumes it; at most one event per
    // emulated frame. If the receiver was dropped there is nothing to do.
    let _ = tx_event.send(Event::Frame(Arc::new(data)));
}

/// Save emulator state to a slot file (.p16s container).
fn handle_save_state(
    snes: &Snes,
    slot: u8,
    rom_path: &Option<PathBuf>,
    rom_hash: u64,
    region: Region,
    tx_event: &Sender<Event>,
) {
    let path = match rom_path {
        Some(p) => files::state_path(&p, slot),
        None => {
            let _ = tx_event.send(Event::Error("no ROM loaded".to_string()));
            return;
        }
    };

    let blob = snes.save_state();

    // Region byte: 0=Ntsc, 1=Pal.
    let region_byte = match region {
        Region::Ntsc => 0u8,
        Region::Pal => 1u8,
    };

    let data = files::encode_state(rom_hash, region_byte, &blob);

    match files::write_atomic(&path, &data) {
        Ok(()) => {
            let _ = tx_event.send(Event::Status(format!("State saved to slot {}", slot)));
        }
        Err(e) => {
            let _ = tx_event.send(Event::Error(format!(
                "cannot save state {}: {}", path.display(), e)));
        }
    }
}

/// Load emulator state from a slot file. Returns Some(new_snes) if the console
/// was reloaded after an error; None otherwise.
fn handle_load_state(
    snes: &mut Snes,
    slot: u8,
    rom_path: &Option<PathBuf>,
    rom_hash: u64,
    region: Region,
    rom_bytes: &[u8],
    joy: &[u16; 2],
    frame_no: &mut u64,
    sync: &mut SramSync,
    tx_event: &Sender<Event>,
) -> Option<Snes> {
    let path = match rom_path {
        Some(p) => files::state_path(&p, slot),
        None => {
            let _ = tx_event.send(Event::Error("no ROM loaded".to_string()));
            return None;
        }
    };

    // Read the state file.
    let data = match std::fs::read(&path) {
        Ok(d) => d,
        Err(_) => {
            let _ = tx_event.send(Event::Error(format!("no state in slot {}", slot)));
            return None;
        }
    };

    // Decode and verify.
    let (stored_region_byte, blob) = match files::decode_state(&data, rom_hash) {
        Ok(r) => r,
        Err(e) => {
            let _ = tx_event.send(Event::Error(format!("cannot load state: {}", e)));
            return None;
        }
    };

    // Check region matches.
    let expected_region_byte = match region {
        Region::Ntsc => 0u8,
        Region::Pal => 1u8,
    };
    if stored_region_byte != expected_region_byte {
        let _ = tx_event.send(Event::Error("state has another region".to_string()));
        return None;
    }

    // Load the state.
    match snes.load_state(blob) {
        Ok(()) => {
            let _ = tx_event.send(Event::Status(format!("State loaded from slot {}", slot)));
            // Publish current frame immediately (even if paused).
            *frame_no = snes.frame_count();
            publish_frame(snes, *frame_no, tx_event);
            None
        }
        Err(e) => {
            let _ = tx_event.send(Event::Error(format!(
                "cannot load state: {}", e)));
            // Reload console from ROM bytes + SRAM.
            reload_console(rom_path, rom_bytes, joy, frame_no, sync, tx_event)
        }
    }
}

/// Rebuild the console from the ROM bytes kept in memory and reload its SRAM file
/// (used after a failed `load_state`, whose partial writes may leave an incoherent state).
fn reload_console(
    rom_path: &Option<PathBuf>,
    rom_bytes: &[u8],
    joy: &[u16; 2],
    frame_no: &mut u64,
    sync: &mut SramSync,
    tx_event: &Sender<Event>,
) -> Option<Snes> {
    match Snes::new(rom_bytes, None) {
        Ok(mut new_snes) => {
            for (port, buttons) in joy.iter().enumerate() {
                new_snes.set_joypad(port, *buttons);
            }
            *frame_no = 0;
            if let Some(path) = rom_path {
                load_sram(&mut new_snes, path, sync, tx_event);
            }
            Some(new_snes)
        }
        Err(_) => {
            let _ = tx_event.send(Event::Error("cannot reload ROM".to_string()));
            None
        }
    }
}

/// Build and send debug data from the current SNES state.
fn send_debug_data(snes: &Snes, wram_page: u16, rom_loaded: bool, tx_event: &Sender<Event>) {
    if !rom_loaded {
        // Test pattern mode: empty vectors and None options (spec).
        let data = DebugData {
            snapshot: puce16_core::snes::DebugSnapshot {
                pc: 0, a: 0, x: 0, y: 0, s: 0, d: 0, db: 0, pb: 0, p: 0, e: false,
                v: 0, dot: 0, master: 0, frame: 0,
            },
            vram: Vec::new(),
            cgram: Vec::new(),
            oam: Vec::new(),
            wram_page: Vec::new(),
            ppu_regs: None,
            smp: None,
            dma: None,
        };
        let _ = tx_event.send(Event::Debug(Box::new(data)));
    } else {
        let data = DebugData {
            snapshot: snes.debug(),
            vram: snes.debug_vram().to_vec(),
            cgram: snes.debug_cgram().to_vec(),
            oam: snes.debug_oam().to_vec(),
            wram_page: snes.debug_wram_page(wram_page),
            ppu_regs: snes.debug_ppu_regs(),
            smp: snes.debug_smp(),
            dma: snes.debug_dma(),
        };
        let _ = tx_event.send(Event::Debug(Box::new(data)));
    }
}

/// How a RunToPc search ended.
enum RunToPcOutcome {
    /// Breakpoint reached or blocks exhausted: stay paused.
    Finished,
    /// A `Pause(false)` arrived: the search is cancelled and normal execution resumes.
    Cancelled,
    /// A `Quit` arrived: the emulation thread must terminate.
    Quit,
}

/// Run until PC reaches the breakpoint address, or max blocks are exhausted.
fn run_to_pc(
    snes: &mut Snes,
    addr: u32,
    rx_cmd: &Receiver<Command>,
    tx_event: &Sender<Event>,
    debug_active: bool,
    wram_page: u16,
    rom_loaded: bool,
) -> RunToPcOutcome {
    let pb = (addr >> 16) as u8;
    let pc = (addr & 0xFFFF) as u16;

    for _ in 0..RUN_TO_PC_MAX_BLOCKS {
        // Check for pending commands between blocks.
        while let Ok(cmd) = rx_cmd.try_recv() {
            match cmd {
                Command::Quit => return RunToPcOutcome::Quit,
                Command::Pause(false) => {
                    let _ = tx_event.send(Event::Status("Breakpoint cancelled".to_string()));
                    return RunToPcOutcome::Cancelled;
                }
                _ => {} // Ignore other commands during run-to-pc.
            }
        }

        if snes.debug_run_until_pc(pb, pc, RUN_TO_PC_BLOCK) {
            let _ = tx_event.send(Event::Status(format!("Breakpoint {:02X}:{:04X}", pb, pc)));
            // Publish current frame and debug data.
            publish_frame(snes, snes.frame_count(), tx_event);
            if debug_active && rom_loaded {
                send_debug_data(snes, wram_page, true, tx_event);
            } else if debug_active && !rom_loaded {
                let data = DebugData {
                    snapshot: puce16_core::snes::DebugSnapshot {
                        pc: 0, a: 0, x: 0, y: 0, s: 0, d: 0, db: 0, pb: 0, p: 0, e: false,
                        v: 0, dot: 0, master: 0, frame: 0,
                    },
                    vram: Vec::new(),
                    cgram: Vec::new(),
                    oam: Vec::new(),
                    wram_page: Vec::new(),
                    ppu_regs: None,
                    smp: None,
                    dma: None,
                };
                let _ = tx_event.send(Event::Debug(Box::new(data)));
            }
            return RunToPcOutcome::Finished;
        }
    }

    let _ = tx_event.send(Event::Status("Breakpoint not reached".to_string()));
    RunToPcOutcome::Finished
}
