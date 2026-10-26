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
        }
    }

    fn send(&self, cmd: Command) {
        let _ = self.handle.tx.send(cmd);
    }

    /// Load a ROM from a file path. On success sends Command::LoadRom; on error sets status.
    pub fn open_path(&mut self, path: &std::path::Path) {
        match files::load_rom_file(path) {
            Ok((name, bytes)) => {
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

        let bits = filter_opposites(input::default_buttons(&held));
        if bits != self.last_joypad {
            self.last_joypad = bits;
            self.send(Command::SetJoypad(0, bits));
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
                    Hotkey::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                    Hotkey::OpenRom => self.pick_and_open(),
                    _ => self.status = "hotkey not implemented".to_string(),
                }
            }
        }
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
                if ui.checkbox(&mut self.paused, "Pause").changed() {
                    self.send(Command::Pause(self.paused));
                }
                if ui.add_enabled(self.paused, egui::Button::new("Image suivante")).clicked() {
                    self.send(Command::FrameAdvance);
                }
                if ui.button("Reset").clicked() {
                    self.send(Command::Reset);
                }
            });
            ui.menu_button("Video", |ui| {
                ui.add(egui::Slider::new(&mut self.scale, 1..=6).text("Echelle"));
                ui.radio_value(&mut self.ratio, Ratio::Square, "Pixels carres");
                ui.radio_value(&mut self.ratio, Ratio::Par87, "Ratio 8:7");
            });
        });
    }

    /// Bottom status line: status text, frame number, scale.
    fn status_bar(&mut self, ui: &mut egui::Ui) {
        let frame_no = self.last.as_ref().map(|f| f.frame_no).unwrap_or(0);
        ui.horizontal(|ui| {
            ui.label(self.status.as_str());
            ui.separator();
            ui.label(format!("frame {}", frame_no));
            ui.separator();
            ui.label(format!("x{}", self.scale));
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

        egui::CentralPanel::default().show(ui, |panel_ui| {
            if let (Some(texture), Some(frame)) = (&self.texture, &self.last) {
                let (w, h) = video::display_size(frame.width, frame.height, self.scale, self.ratio);
                let rect = egui::Rect::from_center_size(panel_ui.max_rect().center(), egui::vec2(w as f32, h as f32));
                let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                panel_ui.painter().image(texture.id(), rect, uv, egui::Color32::WHITE);
            }
        });

        self.menu_bar(ui);
        self.status_bar(ui);
        ui.ctx().request_repaint();
    }
}

impl Drop for App { fn drop(&mut self) { let _ = self.handle.tx.send(Command::Quit); } }
