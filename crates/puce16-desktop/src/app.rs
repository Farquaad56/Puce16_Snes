//! Desktop UI window (milestone D1): test pattern, AZERTY keyboard input and hotkeys.

use std::sync::Arc;

use eframe::egui;

use puce16_core::joypad::filter_opposites;
use puce16_core::Region;

use crate::audio::{AudioOut, SharedRing};
use crate::config::{self, Config};
use crate::debugger::DebuggerUi;
use crate::emu_thread::{Command, EmuHandle, Event, FrameData};
use crate::files;
use crate::input::{self, Hotkey, PadPoller};
use crate::presets::{self, Preset};
use crate::video::{self, Filter, Interp, Ratio};

/// Return the first command-line argument that is not a flag (does not start with '-').
pub fn rom_arg(args: &[String]) -> Option<&str> {
    args.iter().skip(1).find(|a| !a.starts_with('-')).map(|s| s.as_str())
}

pub struct App {
    pub handle: EmuHandle,
    texture: Option<egui::TextureHandle>,
    last: Option<Arc<FrameData>>,
    scale: usize,
    ratio: Ratio,
    filter: Filter,
    /// Scanline darkness in percent (0 = off), used when the filter is Scanlines.
    scan_strength: u32,
    /// SNES interpolation applied when scaling (needs scale >= 2).
    interp: Interp,
    /// MiSTer display preset (replaces filter/interpolation when active, scale >= 2).
    preset: Preset,
    /// Persistent configuration (spec 08 section 6) and its save bookkeeping.
    cfg: Config,
    cfg_dirty: bool,
    last_cfg_change: Option<std::time::Instant>,
    /// True when the pause was caused by the window losing focus.
    auto_paused: bool,
    paused: bool,
    status: String,
    /// Title read from the ROM header (empty = no ROM).
    rom_title: String,
    region: Region,
    /// Last joypad bits sent to the emulation thread (port 0).
    last_joypad: u16,
    /// Current save state slot (0..=9).
    slot: u8,
    /// Mute toggle.
    muted: bool,
    /// Fast forward toggle (held Tab).
    fast_forward: bool,
    /// ROM path for SRAM/save states/screenshots.
    rom_path: Option<std::path::PathBuf>,
    /// Audio output stream (None if no device or error).
    audio: Option<AudioOut>,
    /// Shared ring buffer between emulation thread and audio callback.
    ring: SharedRing,
    /// Last time the mouse moved (for cursor hiding in fullscreen).
    last_mouse_move: std::time::Instant,
    /// True until the first UI frame has been drawn (used to apply initial fullscreen).
    first_frame: bool,
    /// Gamepad poller (gilrs).
    poller: PadPoller,
    /// Last joypad bits sent for port 1 (gamepad only).
    last_joypad_1: u16,
    /// Debugger UI state.
    debugger: DebuggerUi,
    /// Emulated frames per second, measured over ~0.5 s windows.
    fps: f32,
    fps_last_frame: u64,
    fps_last_time: std::time::Instant,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>, handle: EmuHandle, cfg: Config, audio: Option<AudioOut>, ring: SharedRing) -> Self {
        // Apply initial volume and mute from config.
        if let Some(ref a) = audio {
            a.set_volume(cfg.volume as f32 / 100.0);
            a.set_muted(cfg.muted);
            a.set_lpf(cfg.audio_lpf);
        }
        Self {
            handle,
            texture: None,
            last: None,
            scale: (cfg.scale as usize).clamp(1, 8),
            ratio: video::ratio_from_name(&cfg.ratio),
            filter: Filter::from_name(&cfg.filter),
            scan_strength: cfg.scan_strength as u32,
            interp: Interp::from_name(&cfg.interp),
            preset: Preset::from_name(&cfg.preset),
            muted: cfg.muted,
            cfg,
            cfg_dirty: false,
            last_cfg_change: None,
            auto_paused: false,
            paused: false,
            status: "test pattern (NTSC)".to_string(),
            rom_title: String::new(),
            region: Region::Ntsc,
            last_joypad: 0,
            slot: 0,
            fast_forward: false,
            rom_path: None,
            audio,
            ring,
            last_mouse_move: std::time::Instant::now(),
            first_frame: true,
            poller: PadPoller::new(),
            last_joypad_1: 0,
            debugger: DebuggerUi::new(),
            fps: 0.0,
            fps_last_frame: 0,
            fps_last_time: std::time::Instant::now(),
        }
    }

    /// Apply current volume and mute settings to the audio output stream.
    fn apply_audio_settings(&self) {
        if let Some(ref a) = self.audio {
            a.set_volume(self.cfg.volume as f32 / 100.0);
            a.set_muted(self.muted);
            a.set_lpf(self.cfg.audio_lpf);
        }
    }

    /// Copy the UI state into the config and schedule a save (1 s after the last change).
    fn touch_cfg(&mut self) {
        if !self.cfg.fit_window {
            self.cfg.scale = self.scale.clamp(1, 8) as u8;
        }
        self.cfg.ratio = video::ratio_name(self.ratio).to_string();
        self.cfg.filter = self.filter.name().to_string();
        self.cfg.scan_strength = self.scan_strength.min(90) as u8;
        self.cfg.interp = self.interp.name().to_string();
        self.cfg.preset = self.preset.name().to_string();
        self.cfg.muted = self.muted;
        self.cfg_dirty = true;
        self.last_cfg_change = Some(std::time::Instant::now());
    }

    /// Write the config to disk; a failure only sets the status line.
    fn save_cfg(&mut self) {
        if let Err(e) = config::save(&self.cfg) {
            self.status = format!("ERROR: {}", e);
        }
        self.cfg_dirty = false;
    }

    fn send(&self, cmd: Command) {
        let _ = self.handle.tx.send(cmd);
    }

    /// Load a ROM from a file path. On success sends Command::LoadRom; on error sets status.
    pub fn open_path(&mut self, path: &std::path::Path) {
        match files::load_rom_file(path) {
            Ok((name, bytes)) => {
                // Clear the audio ring to avoid leftover sound from the previous ROM.
                if let Ok(mut r) = self.ring.lock() {
                    r.clear();
                }
                // Send the ROM path first so SRAM can be loaded/saved.
                let _ = self.handle.tx.send(Command::SetRomPath(path.to_path_buf()));
                self.rom_path = Some(path.to_path_buf());
                self.cfg.add_recent(&path.to_string_lossy());
                if let Some(dir) = path.parent() {
                    self.cfg.last_dir = dir.to_string_lossy().to_string();
                }
                self.touch_cfg();
                self.send(Command::LoadRom(bytes, name));
            }
            Err(e) => {
                self.status = format!("ERROR: {}", e);
            }
        }
    }

    /// Show the ROM file picker dialog and open the selected file.
    fn pick_and_open(&mut self) {
        let mut dialog = rfd::FileDialog::new().add_filter("ROM SNES", &["sfc", "smc", "swc", "fig", "zip"]);
        if !self.cfg.last_dir.is_empty() && std::path::Path::new(&self.cfg.last_dir).exists() {
            dialog = dialog.set_directory(&self.cfg.last_dir);
        }
        if let Some(path) = dialog.pick_file() {
            self.open_path(&path);
        }
    }

    /// Poll keyboard and gamepads each frame (spec 08 sections 2 & 3): held keys drive joypad
    /// port 0 (OR'd with gamepad 0, opposite directions filtered), key presses trigger hotkeys.
    fn poll_keyboard(&mut self, ui: &egui::Ui) {
        let ctx = ui.ctx();

        // Fast forward: Tab held -> send SetFastForward only on change.
        let tab_down = ctx.input(|i| i.key_down(egui::Key::Tab));
        if tab_down != self.fast_forward {
            self.fast_forward = tab_down;
            self.send(Command::SetFastForward(tab_down));
        }

        // Poll gamepads.
        let pads = self.poller.poll();

        // Check connection state changes for status messages.
        let conn = self.poller.connected();
        if conn[0] {
            if !self.status.contains("Pad 1 connected") && !self.status.starts_with("ERROR:") {
                self.status = "Pad 1 connected".to_string();
            }
        } else if self.status == "Pad 1 connected" {
            self.status = "Pad 1 disconnected".to_string();
        }

        // Joypad buttons from held keys.
        let (held, presses) = ctx.input(|i| {
            let mut held = Vec::new();
            for k in input::MAPPED_KEYS {
                if i.key_down(k) {
                    held.push(input::from_egui_key(k));
                }
            }
            let presses: Vec<(egui::Key, bool)> = i.events.iter().filter_map(|e| {
                match e {
                    egui::Event::Key { key, pressed, repeat, modifiers, .. } if *pressed && !*repeat => Some((*key, modifiers.ctrl)),
                    _ => None,
                }
            }).collect();
            (held, presses)
        });

        // Don't send joypad input when a dialog has keyboard focus.
        let wants_keyboard = ctx.egui_wants_keyboard_input();
        if !wants_keyboard {
            // Port 0: keyboard OR gamepad 0.
            let kb_bits = input::keyboard_buttons_mapped(&held, &self.cfg.keymap);
            let raw = kb_bits | pads[0];
            let bits = if self.cfg.block_opposite_dirs { filter_opposites(raw) } else { raw };
            if bits != self.last_joypad {
                self.last_joypad = bits;
                self.send(Command::SetJoypad(0, bits));
            }

            // Port 1: gamepad only.
            let pad1_bits = pads[1];
            if pad1_bits != self.last_joypad_1 {
                self.last_joypad_1 = pad1_bits;
                self.send(Command::SetJoypad(1, pad1_bits));
            }
        }

        for (key, ctrl) in presses {
            if let Some(hotkey) = input::hotkey_for(key, ctrl) {
                match hotkey {
                    Hotkey::Pause => {
                        self.paused = !self.paused;
                        self.send(Command::Pause(self.paused));
                    }
                    Hotkey::FrameAdvance => self.send(Command::FrameAdvance),
                    Hotkey::Reset => self.send(Command::Reset),
                    Hotkey::SaveState => self.send(Command::SaveState(self.slot)),
                    Hotkey::LoadState => self.send(Command::LoadState(self.slot)),
                    Hotkey::SlotPrev => {
                        self.slot = (self.slot + 9) % 10;
                        self.status = format!("slot {}", self.slot);
                    }
                    Hotkey::SlotNext => {
                        self.slot = (self.slot + 1) % 10;
                        self.status = format!("slot {}", self.slot);
                    }
                    Hotkey::Mute => {
                        self.muted = !self.muted;
                        self.touch_cfg();
                        self.apply_audio_settings();
                    }
                    Hotkey::Screenshot => {
                        if let Some(frame) = &self.last {
                            let path = match &self.rom_path {
                                Some(p) => files::screenshot_path(&p, &|path: &std::path::Path| path.exists()),
                                None => std::path::PathBuf::from("screenshot_0.ppm"),
                            };
                            if let Err(e) = self.write_ppm(frame, &path) {
                                self.status = format!("ERROR: {}", e);
                            } else {
                                self.status = format!("Screenshot saved: {}", path.display());
                            }
                        }
                    }
                    Hotkey::Fullscreen => {
                        self.cfg.fullscreen = !self.cfg.fullscreen;
                        self.touch_cfg();
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.cfg.fullscreen));
                        if self.cfg.fullscreen {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::None);
                        } else {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
                        }
                    }
                    Hotkey::Debugger => {
                        self.debugger.open = !self.debugger.open;
                        if self.debugger.open {
                            self.send(Command::RequestDebug(true));
                        } else {
                            self.send(Command::RequestDebug(false));
                        }
                    }
                    Hotkey::Quit => {
                        if self.cfg.fullscreen {
                            // In fullscreen, Escape exits fullscreen first.
                            self.cfg.fullscreen = false;
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                            ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
                        } else {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                    Hotkey::OpenRom => self.pick_and_open(),
                    _ => {} // FastForward handled above via held state.
                }
            }
        }
    }

    /// Write the last frame as a PPM file (P6 format, same encoding as puce16-cli).
    fn write_ppm(&self, frame: &Arc<FrameData>, path: &std::path::Path) -> Result<(), String> {
        let rgba = video::xrgb_to_rgba(&frame.pixels);
        // PPM is RGB only (no alpha), 3 bytes per pixel.
        let mut rgb = Vec::with_capacity(frame.width * frame.height * 3);
        for i in 0..rgba.len() {
            if i % 4 != 3 {
                rgb.push(rgba[i]);
            }
        }
        let header = format!("P6\n{} {}\n255\n", frame.width, frame.height);
        let mut data = Vec::with_capacity(header.len() + rgb.len());
        data.extend_from_slice(header.as_bytes());
        data.extend_from_slice(&rgb);
        files::write_atomic(path, &data)
    }

    /// Top menu bar: File, Emulation, Video.
    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        let mut to_open: Option<String> = None;
        let mut changed = false;
        let mut debug_toggled = false;
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("Fichier", |ui| {
                if ui.button("Ouvrir une ROM... (Ctrl+O)").clicked() {
                    self.pick_and_open();
                }
                ui.menu_button("ROM recentes", |ui| {
                    if self.cfg.recent_roms.is_empty() {
                        ui.label("(aucune)");
                    }
                    for rom in self.cfg.recent_roms.clone() {
                        if ui.button(rom.as_str()).clicked() {
                            to_open = Some(rom);
                            ui.close();
                        }
                    }
                });
                if ui.button("Quitter").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
            ui.menu_button("Emulation", |ui| {
                if ui.checkbox(&mut self.paused, "Pause (P)").changed() {
                    self.send(Command::Pause(self.paused));
                }
                if ui.add_enabled(self.paused, egui::Button::new("Image suivante (N)")).clicked() {
                    self.send(Command::FrameAdvance);
                }
                if ui.button("Reset (F12)").clicked() {
                    self.send(Command::Reset);
                }
                ui.separator();
                // Save state submenu.
                ui.menu_button(format!("Etat (slot {})...", self.slot), |ui| {
                    if ui.button(format!("Sauvegarder (F5)")).clicked() {
                        self.send(Command::SaveState(self.slot));
                    }
                    if ui.button(format!("Charger (F7)")).clicked() {
                        self.send(Command::LoadState(self.slot));
                    }
                    ui.separator();
                    for s in 0..10u8 {
                        let label = format!("Slot {}", s);
                        if ui.selectable_value(&mut self.slot, s, &label).changed() {
                            self.status = format!("slot {}", self.slot);
                        }
                    }
                });
                if ui.checkbox(&mut self.cfg.pause_on_focus_loss, "Pause si focus perdu").changed() {
                    changed = true;
                }
                // Fast forward toggle.
                if ui.checkbox(&mut self.fast_forward, "Avance rapide (Tab)").changed() {
                    self.send(Command::SetFastForward(self.fast_forward));
                }
            });
            ui.menu_button("Video", |ui| {
                if ui.checkbox(&mut self.cfg.fit_window, "Ajuster a la fenetre").changed() {
                    changed = true;
                }
                ui.add_enabled_ui(!self.cfg.fit_window, |ui| {
                    if ui.add(egui::Slider::new(&mut self.scale, 1..=8).text("Echelle")).changed() {
                        changed = true;
                    }
                });
                changed |= ui.radio_value(&mut self.ratio, Ratio::Square, "Pixels carres").changed();
                changed |= ui.radio_value(&mut self.ratio, Ratio::Par87, "Ratio 8:7").changed();
                ui.separator();
                changed |= ui.radio_value(&mut self.filter, Filter::Nearest, "Filtre: nearest").changed();
                changed |= ui.radio_value(&mut self.filter, Filter::Linear, "Filtre: linear").changed();
                changed |= ui.radio_value(&mut self.filter, Filter::Scanlines, "Filtre: scanlines (echelle >= 2)").changed();
                ui.add_enabled_ui(self.filter == Filter::Scanlines, |ui| {
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut self.scan_strength, 0..=video::SCAN_STRENGTH_MAX)
                                .step_by(5.0)
                                .suffix(" %")
                                .text("Scanlines"),
                        )
                        .changed();
                });
                ui.label("Interpolation SNES (echelle >= 2)");
                changed |= ui.radio_value(&mut self.interp, Interp::Off, "   Aucune").changed();
                changed |= ui.radio_value(&mut self.interp, Interp::Soft, "   Douce (soft)").changed();
                changed |= ui.radio_value(&mut self.interp, Interp::Medium, "   Moyenne (medium)").changed();
                changed |= ui.radio_value(&mut self.interp, Interp::Sharp, "   Nette (sharp)").changed();
                ui.label("Preset ecran MiSTer (echelle >= 2, remplace les filtres)");
                changed |= ui.radio_value(&mut self.preset, Preset::None, "   Aucun").changed();
                changed |= ui.radio_value(&mut self.preset, Preset::SonyTrinitron, "   Sony Trinitron").changed();
                changed |= ui.radio_value(&mut self.preset, Preset::FadedTube, "   Faded Tube").changed();
                ui.add_enabled_ui(self.preset == Preset::SonyTrinitron, |ui| {
                    changed |= ui.checkbox(&mut self.cfg.preset_mask, "   Grille d'ouverture RGB (echelle 3 ideale)").changed();
                });
                ui.separator();
                if ui.checkbox(&mut self.cfg.fullscreen, "Plein ecran (F11)").changed() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.cfg.fullscreen));
                    if self.cfg.fullscreen {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::None);
                    } else {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
                    }
                }
            });
            ui.menu_button("Audio", |ui| {
                let mut vol = self.cfg.volume as u32;
                if ui.add(egui::Slider::new(&mut vol, 0..=100).text("Volume")).changed() {
                    self.cfg.volume = vol as u8;
                    changed = true;
                    self.apply_audio_settings();
                }
                if ui.checkbox(&mut self.muted, "Muet (M)").changed() {
                    changed = true;
                    self.apply_audio_settings();
                }
                if ui.checkbox(&mut self.cfg.audio_lpf, "Filtre SNES GPM-02 LPF").changed() {
                    changed = true;
                    self.apply_audio_settings();
                }
            });
            ui.menu_button("Debug", |ui| {
                if ui.checkbox(&mut self.debugger.open, "Debugger (F10)").changed() {
                    debug_toggled = true;
                }
            });
            ui.menu_button("Entrees", |ui| {
                ui.label("Fleches: croix directionnelle");
                ui.label("X / S: A / B   (Z / A selon le clavier: voir spec 08 section 2)");
                ui.label("Entree: Start   Retour arriere: Select");
                ui.label("F5 / F7: sauver / charger l'etat   F12: reset");
                ui.label("P: pause   N: image suivante   Tab: avance rapide");
                ui.label("F2: capture d'ecran   M: muet");
                ui.separator();
                if ui.checkbox(&mut self.cfg.block_opposite_dirs, "Bloquer les directions opposees").changed() {
                    changed = true;
                }
            });
        });
        if changed {
            self.touch_cfg();
        }
        if debug_toggled {
            self.send(Command::RequestDebug(self.debugger.open));
        }
        if let Some(rom) = to_open {
            self.open_path(std::path::Path::new(&rom));
        }
    }

    /// Update the FPS estimate from the emulation frame counter (called every UI update).
    fn update_fps(&mut self) {
        let frame_no = self.last.as_ref().map(|f| f.frame_no).unwrap_or(0);
        let now = std::time::Instant::now();
        if frame_no < self.fps_last_frame {
            // Counter restarted (ROM loaded or reset): start a new window.
            self.fps_last_frame = frame_no;
            self.fps_last_time = now;
            self.fps = 0.0;
            return;
        }
        let dt = now.duration_since(self.fps_last_time).as_secs_f32();
        if dt >= 0.5 {
            self.fps = (frame_no - self.fps_last_frame) as f32 / dt;
            self.fps_last_frame = frame_no;
            self.fps_last_time = now;
        }
    }

    /// Bottom status line: status text, FPS, scale, slot, audio buffer level.
    fn status_bar(&mut self, ui: &mut egui::Ui) {
        // Audio ring fill: the colour shows how close the buffer is to running dry.
        // Steady state is RING_TARGET frames: green >= half of it, yellow >= 1/8, red below.
        let stopped = self.paused || self.auto_paused;
        let (audio_text, audio_color) = match &self.audio {
            Some(_) => match self.ring.lock() {
                Ok(r) => {
                    let frames = r.len();
                    let ms = frames * 1000 / 32000;
                    let color = if stopped {
                        egui::Color32::GRAY
                    } else if frames >= crate::audio::RING_TARGET / 2 {
                        egui::Color32::from_rgb(60, 200, 60)
                    } else if frames >= crate::audio::RING_TARGET / 8 {
                        egui::Color32::from_rgb(230, 190, 30)
                    } else {
                        egui::Color32::from_rgb(220, 50, 50)
                    };
                    (format!("audio {:>4} ms", ms), color)
                }
                Err(_) => (format!("{:<13}", "audio error"), egui::Color32::from_rgb(220, 50, 50)),
            },
            None => (format!("{:<13}", "audio off"), egui::Color32::GRAY),
        };
        // Fixed-width (monospace, padded) texts so the items to the right never shift.
        let fps_text = if stopped { format!("{:<7}", "pause") } else { format!("{:>3.0} FPS", self.fps) };
        let title = if self.rom_title.is_empty() { "Aucune ROM" } else { self.rom_title.as_str() };
        let region = if self.region == Region::Pal { "PAL" } else { "NTSC" };
        let control = if self.poller.connected()[0] { "Pad" } else { "Clavier" };
        // Transient messages (errors, save states, screenshots) go at the end of the bar.
        let message = ["ERROR", "State", "Screenshot", "SRAM", "pause first"]
            .iter()
            .any(|p| self.status.starts_with(p))
            .then(|| self.status.clone());
        ui.horizontal(|ui| {
            ui.label(title);
            ui.separator();
            ui.label(region);
            ui.separator();
            ui.label(control);
            ui.separator();
            ui.label(egui::RichText::new(fps_text).monospace());
            ui.separator();
            ui.label(format!("x{}", self.scale));
            ui.separator();
            ui.label(format!("slot {}", self.slot));
            ui.separator();
            ui.label(egui::RichText::new(audio_text).monospace().color(audio_color));
            if let Some(m) = message {
                ui.separator();
                ui.label(m);
            }
        });
    }
}
impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Apply initial fullscreen on first frame if configured.
        if self.first_frame && self.cfg.fullscreen {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
            ui.ctx().set_cursor_icon(egui::CursorIcon::None);
        }
        self.first_frame = false;

        let mut latest = None;
        while let Ok(event) = self.handle.rx.try_recv() {
            match event {
                Event::Frame(frame) => latest = Some(frame),
                Event::Status(text) => self.status = text,
                Event::Error(e) => self.status = format!("ERROR: {}", e),
                Event::RomInfo(title, region) => {
                    self.rom_title = title;
                    self.region = region;
                }
                Event::SramSaved(path) => self.status = format!("SRAM saved to {}", path.display()),
                Event::Debug(data) => self.debugger.last = Some(data),
            }
        }
        if let Some(frame) = latest { self.last = Some(frame); }
        self.update_fps();

        // Handle drag-and-drop (spec 08 section 6): open the first dropped file.
        ui.ctx().input(|i| {
            for dropped in &i.raw.dropped_files {
                self.open_path(dropped.path());
                break;
            }
        });

        self.poll_keyboard(ui);

        // Optional auto-pause when the window loses focus.
        let focused = ui.ctx().input(|i| i.focused);
        if self.cfg.pause_on_focus_loss && !focused && !self.paused {
            self.paused = true;
            self.auto_paused = true;
            self.send(Command::Pause(true));
        } else if focused && self.auto_paused {
            self.auto_paused = false;
            if self.paused {
                self.paused = false;
                self.send(Command::Pause(false));
            }
        }

        // Debounced config save: 1 s after the last change.
        if self.cfg_dirty && self.last_cfg_change.map_or(true, |t| t.elapsed().as_secs_f32() >= 1.0) {
            self.save_cfg();
        }

        if let Some(frame) = &self.last {
            let mut rgba = video::xrgb_to_rgba(&frame.pixels); // fresh copy, source untouched
            let (mut tex_width, mut tex_height) = (frame.width, frame.height);
            if self.scale >= 2 {
                // MiSTer-style filters on the CPU: interpolation on both axes, scanlines
                // (when enabled) replace the vertical coefficients.
                let h_table = self.interp.table();
                let v_table = if video::scanlines_active(self.filter, self.scale) {
                    video::scanline_table(self.scan_strength).or(h_table)
                } else {
                    h_table
                };
                if self.preset != Preset::None {
                    let (scaled, w, h) = presets::apply_preset(&rgba, frame.width, self.scale as usize, self.preset, self.cfg.preset_mask);
                    rgba = scaled;
                    tex_width = w;
                    tex_height = h;
                } else if h_table.is_some() || v_table.is_some() {
                    let (scaled, w, h) = video::filter_scale(&rgba, frame.width, self.scale, h_table, v_table);
                    rgba = scaled;
                    tex_width = w;
                    tex_height = h;
                }
            }
            let options = if self.filter == Filter::Linear {
                egui::TextureOptions::LINEAR
            } else {
                egui::TextureOptions::NEAREST
            };
            let image = egui::ColorImage::from_rgba_unmultiplied([tex_width, tex_height], &rgba);
            match &mut self.texture {
                Some(texture) => texture.set(image, options),
                None => self.texture = Some(ui.ctx().load_texture("frame", image, options)),
            }
        }

        // Cursor hiding: in fullscreen, hide after 2s of no mouse movement.
        if self.cfg.fullscreen {
            let pos = ui.ctx().input(|i| i.pointer.hover_pos());
            if pos.is_some() {
                // Check for recent pointer events (movement or clicks).
                let has_recent_event = ui.ctx().input(|i| {
                    i.events.iter().any(|e| matches!(e, egui::Event::PointerMoved(_) | egui::Event::PointerButton { .. }))
                });
                if has_recent_event {
                    self.last_mouse_move = std::time::Instant::now();
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
                } else if self.last_mouse_move.elapsed().as_secs_f32() >= 2.0 {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::None);
                }
            }
        }

        // Side panels must be added BEFORE the CentralPanel: it takes all the space left, so a
        // menu or status bar drawn after it has no room and stays invisible.
        let show_menu = !self.cfg.fullscreen || ui.ctx().input(|i| i.pointer.hover_pos().map_or(false, |p| p.y < 8.0));
        if show_menu {
            egui::Panel::top("menu_bar").show(ui, |panel_ui| self.menu_bar(panel_ui));
        }
        egui::Panel::bottom("status_bar").show(ui, |panel_ui| self.status_bar(panel_ui));
        self.debugger.show(ui.ctx(), &self.handle.tx, self.paused);
        if let Some(p) = self.debugger.pause_request.take() {
            self.paused = p;
            self.send(Command::Pause(p));
        }

        egui::CentralPanel::default().show(ui, |panel_ui| {
            if let (Some(texture), Some(frame)) = (&self.texture, &self.last) {
                // Pixel perfect: the image is always drawn on whole physical screen pixels
                // (egui points x pixels_per_point), at an integer scale. Filters and presets
                // need an undistorted image, so they force square pixels.
                let ppp = panel_ui.ctx().pixels_per_point();
                let filtered = texture.size()[0] != frame.width; // texture already scaled on the CPU
                let square = filtered || self.ratio == Ratio::Square;
                // Pixel aspect ratio of the source: 8:7 on NTSC, 11:8 on PAL (frame height differs too).
                let (par_num, par_den) = if square { (1, 1) } else if self.region == Region::Pal { (11, 8) } else { (8, 7) };
                if self.cfg.fit_window || self.cfg.fullscreen {
                    let avail = panel_ui.available_size() * ppp;
                    let unit_w = frame.width * par_num / par_den;
                    self.scale = video::fit_scale(unit_w, frame.height, avail.x.max(0.0) as usize, avail.y.max(0.0) as usize).clamp(1, 8);
                }
                let (w, h) = if filtered {
                    (texture.size()[0], texture.size()[1]) // already scale x source, 1:1
                } else {
                    (frame.width * self.scale * par_num / par_den, frame.height * self.scale)
                };
                let size = egui::vec2(w as f32, h as f32) / ppp;
                let center = panel_ui.max_rect().center();
                let min = egui::pos2(
                    ((center.x - size.x / 2.0) * ppp).round() / ppp,
                    ((center.y - size.y / 2.0) * ppp).round() / ppp,
                );
                let rect = egui::Rect::from_min_size(min, size);
                let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                panel_ui.painter().image(texture.id(), rect, uv, egui::Color32::WHITE);
            }
        });

        ui.ctx().request_repaint();
    }
}

impl Drop for App {
    fn drop(&mut self) {
        if self.cfg_dirty {
            let _ = config::save(&self.cfg);
        }
        let _ = self.handle.tx.send(Command::Quit);
    }
}
