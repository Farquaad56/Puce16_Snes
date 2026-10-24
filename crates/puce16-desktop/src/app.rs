//! Desktop UI window (milestone D0): shows the test pattern from the core.

use std::sync::Arc;

use eframe::egui;

use crate::emu_thread::{Command, EmuHandle, Event, FrameData};
use crate::video::{self, Ratio};

pub struct App {
    pub handle: EmuHandle,
    texture: Option<egui::TextureHandle>,
    last: Option<Arc<FrameData>>,
    scale: usize,
    ratio: Ratio,
    paused: bool,
    status: String,
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
        }
    }

    fn send(&self, cmd: Command) {
        let _ = self.handle.tx.send(cmd);
    }

    /// Top menu bar: File, Emulation, Video.
    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("Fichier", |ui| {
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
            }
        }
        if let Some(frame) = latest { self.last = Some(frame); }

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
