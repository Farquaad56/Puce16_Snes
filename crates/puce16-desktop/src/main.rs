//! Puce16 desktop entry point (milestone D0): window showing the core test pattern.

use puce16_core::Region;
use puce16_desktop::app::{App, rom_arg};
use puce16_desktop::audio::{new_shared_ring, AudioOut};
use puce16_desktop::emu_thread::spawn_with_audio;

fn main() -> eframe::Result {
    let cfg = puce16_desktop::config::load();
    let ring = new_shared_ring();

    // Try to open audio output; fall back to timer sync if it fails.
    let (handle, audio) = match AudioOut::new(ring.clone()) {
        Ok(out) => {
            let h = spawn_with_audio(Region::Ntsc, false, Some(ring.clone()));
            (h, Some(out))
        }
        Err(e) => {
            let h = spawn_with_audio(Region::Ntsc, false, None);
            eprintln!("No audio: {} (timer sync)", e);
            (h, None)
        }
    };

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_title("Puce16 - Snes").with_inner_size([768.0, 740.0]),
        ..Default::default()
    };

    // If a ROM path was given on the command line, open it after creating the app.
    let args: Vec<String> = std::env::args().collect();
    let rom_path: Option<String> = rom_arg(&args).map(|s| s.to_string());

    eframe::run_native("Puce16 - Snes", options, Box::new(move |cc| {
        let mut app = App::new(cc, handle, cfg, audio, ring);
        if let Some(path) = rom_path {
            app.open_path(std::path::Path::new(&path));
        }
        Ok(Box::new(app))
    }))
}
