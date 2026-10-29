//! Desktop UI window (milestone D1): test pattern, AZERTY keyboard input and hotkeys.

use std::sync::Arc;

use eframe::egui;

use puce16_core::joypad::filter_opposites;

use crate::emu_thread::{Command, EmuHandle, Event, FrameData};
use crate::files;
use crate::input::{self, Hotkey};
use crate::video::{self, Ratio};

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
    paused: bool,
    status: String,
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
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>, handle: EmuHandle) -> Self {
        Self {
            handle,
            texture: None,
            last: None,
            scale: 3,
            ratio: Ratio::Square,
            paused: false,
            status: "test pattern (NTSC)".to_string(),
            last_joypad: 0,
            slot: 0,
            muted: false,
            fast_forward: false,
            rom_path: None,
        }
    }

    fn send(&self, cmd: Command) {
        let _ = self.handle.tx.send(cmd);
    }

    /// Load a ROM from a file path. On success sends Command::LoadRom; on error sets status.
    pub fn open_path(&mut self, path: &std::path::Path) {
        match files::load_rom_file(path) {
            Ok((name, bytes)) => {
                // Send the ROM path first so SRAM can be loaded/saved.
                let _ = self.handle.tx.send(Command::SetRomPath(path.to_path_buf()));
                self.rom_path = Some(path.to_path_buf());
                self.send(Command::LoadRom(bytes, name));
            }
            Err(e) => {
                self.status = format!("ERROR: {}", e);
            }
        }
    }

    /// Show the ROM file picker dialog and open the selected file.
    fn pick_and_open(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("ROM SNES", &["sfc", "smc", "swc", "fig", "zip"])
            .pick_file()
        {
            self.open_path(&path);
        }
    }

    /// Poll the keyboard each frame (spec 08 section 2): held keys drive joypad
    /// port 0 (opposite directions filtered), key presses trigger hotkeys.
    fn poll_keyboard(&mut self, ui: &egui::Ui) {
        let ctx = ui.ctx();

        // Fast forward: Tab held -> send SetFastForward only on change.
        let tab_down = ctx.input(|i| i.key_down(egui::Key::Tab));
        if tab_down != self.fast_forward {
            self.fast_forward = tab_down;
            self.send(Command::SetFastForward(tab_down));
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
            let bits = filter_opposites(input::default_buttons(&held));
            if bits != self.last_joypad {
                self.last_joypad = bits;
                self.send(Command::SetJoypad(0, bits));
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
                        // audio.set_muted will be handled in E08-25 when AudioOut is available.
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
                    Hotkey::Fullscreen | Hotkey::Debugger => {
                        // Not implemented yet (later tasks).
                        self.status = "hotkey not implemented".to_string();
                    }
                    Hotkey::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
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
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("Fichier", |ui| {
                if ui.button("Ouvrir une ROM... (Ctrl+O)").clicked() {
                    self.pick_and_open();
                }
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
                // Fast forward toggle.
                if ui.checkbox(&mut self.fast_forward, "Avance rapide (Tab)").changed() {
                    self.send(Command::SetFastForward(self.fast_forward));
                }
            });
            ui.menu_button("Video", |ui| {
                ui.add(egui::Slider::new(&mut self.scale, 1..=6).text("Echelle"));
                ui.radio_value(&mut self.ratio, Ratio::Square, "Pixels carres");
                ui.radio_value(&mut self.ratio, Ratio::Par87, "Ratio 8:7");
            });
        });
    }

    /// Bottom status line: status text, frame number, scale, slot.
    fn status_bar(&mut self, ui: &mut egui::Ui) {
        let frame_no = self.last.as_ref().map(|f| f.frame_no).unwrap_or(0);
        ui.horizontal(|ui| {
            ui.label(self.status.as_str());
            ui.separator();
            ui.label(format!("frame {}", frame_no));
            ui.separator();
            ui.label(format!("x{}", self.scale));
            ui.separator();
            ui.label(format!("slot {}", self.slot));
        });
    }
}
impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let mut latest = None;
        while let Ok(event) = self.handle.rx.try_recv() {
            match event {
                Event::Frame(frame) => latest = Some(frame),
                Event::Status(text) => self.status = text,
                Event::Error(e) => self.status = format!("ERROR: {}", e),
                Event::SramSaved(path) => self.status = format!("SRAM saved to {}", path.display()),
            }
        }
        if let Some(frame) = latest { self.last = Some(frame); }

        // Handle drag-and-drop (spec 08 section 6): open the first dropped file.
        ui.ctx().input(|i| {
            for dropped in &i.raw.dropped_files {
                self.open_path(dropped.path());
                break;
            }
        });

        self.poll_keyboard(ui);

        if let Some(frame) = &self.last {
            let rgba = video::xrgb_to_rgba(&frame.pixels);
            let image = egui::ColorImage::from_rgba_unmultiplied([frame.width, frame.height], &rgba);
            match &mut self.texture {
                Some(texture) => texture.set(image, egui::TextureOptions::NEAREST),
                None => self.texture = Some(ui.ctx().load_texture("frame", image, egui::TextureOptions::NEAREST)),
            }
        }

        // Side panels must be added BEFORE the CentralPanel: it takes all the space left, so a
        // menu or status bar drawn after it has no room and stays invisible.
        egui::Panel::top("menu_bar").show(ui, |panel_ui| self.menu_bar(panel_ui));
        egui::Panel::bottom("status_bar").show(ui, |panel_ui| self.status_bar(panel_ui));

        egui::CentralPanel::default().show(ui, |panel_ui| {
            if let (Some(texture), Some(frame)) = (&self.texture, &self.last) {
                let (w, h) = video::display_size(frame.width, frame.height, self.scale, self.ratio);
                let rect = egui::Rect::from_center_size(panel_ui.max_rect().center(), egui::vec2(w as f32, h as f32));
                let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                panel_ui.painter().image(texture.id(), rect, uv, egui::Color32::WHITE);
            }
        });

        ui.ctx().request_repaint();
    }
}

impl Drop for App { fn drop(&mut self) { let _ = self.handle.tx.send(Command::Quit); } }
