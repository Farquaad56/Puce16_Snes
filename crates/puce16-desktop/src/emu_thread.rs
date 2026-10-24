//! Emulation thread (spec 08_INPUT_DESKTOP section 7, milestone D0).
//!
//! A dedicated thread owns the `Snes` state and talks to the UI only through
//! channels: commands in (`Command`), events out (`Event`). No emulation
//! state is shared by lock. This module runs the test pattern (E01-06); ROM
//! loading, save states, audio and debug data come with later tasks.

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use puce16_core::{Region, Snes};

/// Commands sent from the UI thread to the emulation thread.
pub enum Command {
    /// Pause or resume (`true` = paused).
    Pause(bool),
    /// While paused: run exactly one frame and publish it.
    FrameAdvance,
    /// Set the buttons of a joypad port (0 or 1; other ports ignored).
    SetJoypad(usize, u16),
    /// Recreate the emulator state; the next frame is number 1.
    Reset,
    /// Run frames without waiting for cadence (`true` = fast forward on).
    SetFastForward(bool),
    /// Stop the emulation thread.
    Quit,
}

/// One published frame: compact XRGB8888 pixels plus metadata.
pub struct FrameData {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
    pub frame_no: u64,
}

/// Events sent from the emulation thread to the UI thread.
pub enum Event {
    /// A new frame was emulated (the UI displays the most recent one).
    Frame(Arc<FrameData>),
    /// Human-readable status line.
    Status(String),
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

/// Spawn an emulation thread running the test pattern for `region`.
pub fn spawn_test_pattern(region: Region, start_paused: bool) -> EmuHandle {
    let (tx_cmd, rx_cmd) = mpsc::channel();
    let (tx_event, rx_event) = mpsc::channel();
    let join = thread::Builder::new()
        .name("puce16-emu".to_string())
        .spawn(move || emu_loop(region, start_paused, rx_cmd, tx_event))
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
    rx_cmd: Receiver<Command>,
    tx_event: Sender<Event>,
) {
    let mut snes = Snes::new_test_pattern(region);
    let mut paused = start_paused;
    let mut fast_forward = false;

    loop {
        // Drain every pending command before doing any work.
        while let Ok(cmd) = rx_cmd.try_recv() {
            match cmd {
                Command::Pause(pause) => paused = pause,
                Command::FrameAdvance => {
                    if paused {
                        snes.run_frame();
                        publish_frame(&snes, &tx_event);
                    }
                }
                Command::SetJoypad(port, buttons) => snes.set_joypad(port, buttons),
                Command::Reset => snes = Snes::new_test_pattern(region),
                Command::SetFastForward(on) => fast_forward = on,
                Command::Quit => return,
            }
        }

        if paused {
            thread::sleep(Duration::from_millis(5));
        } else {
            snes.run_frame();
            publish_frame(&snes, &tx_event);
            // Cadence is not exact in this milestone; fast forward skips the wait.
            if !fast_forward {
                thread::sleep(Duration::from_millis(16));
            }
        }
    }
}

/// Copy the current frame into a `FrameData` and send it as an event.
fn publish_frame(snes: &Snes, tx_event: &Sender<Event>) {
    let view = snes.frame();
    let data = FrameData {
        width: view.width,
        height: view.height,
        pixels: view.pixels.to_vec(),
        frame_no: snes.frame_count(),
    };
    // The channel is unbounded and the UI consumes it; at most one event per
    // emulated frame. If the receiver was dropped there is nothing to do.
    let _ = tx_event.send(Event::Frame(Arc::new(data)));
}
