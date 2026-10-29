//! Puce16 desktop entry point (milestone D0): window showing the core test pattern.

use puce16_core::Region;
use puce16_desktop::app::{App, rom_arg};
use puce16_desktop::emu_thread::{spawn_with_audio, Command};

fn main() -> eframe::Result {
    let handle = spawn_with_audio(Region::Ntsc, false, None);
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([768.0, 740.0]),
        ..Default::default()
    };

    // If a ROM path was given on the command line, open it after creating the app.
    let args: Vec<String> = std::env::args().collect();
    let rom_path: Option<String> = rom_arg(&args).map(|s| s.to_string());

    eframe::run_native("Puce16", options, Box::new(move |cc| {
        let mut app = App::new(cc, handle);
        if let Some(path) = rom_path {
            app.open_path(std::path::Path::new(&path));
        }
        Ok(Box::new(app))
    }))
}
