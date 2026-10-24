//! Puce16 desktop entry point (milestone D0): window showing the core test pattern.

use puce16_core::Region;
use puce16_desktop::app::App;
use puce16_desktop::emu_thread::spawn_test_pattern;

fn main() -> eframe::Result {
    let handle = spawn_test_pattern(Region::Ntsc, false);
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([768.0, 740.0]),
        ..Default::default()
    };
    eframe::run_native("Puce16", options, Box::new(move |cc| Ok(Box::new(App::new(cc, handle)))))
}
