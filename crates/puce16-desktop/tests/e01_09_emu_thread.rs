//! E01-09 tests for the emulation thread (Command / Event channels).

use std::sync::mpsc::{Receiver, RecvTimeoutError, TryRecvError};
use std::sync::Arc;
use std::time::{Duration, Instant};

use puce16_core::Region;
use puce16_desktop::emu_thread::*;

/// Wait up to `timeout` for the next event and return it as a frame.
fn recv_frame(rx: &Receiver<Event>, timeout: Duration) -> Arc<FrameData> {
    let deadline = Instant::now() + timeout;
    loop {
        let wait = deadline.saturating_duration_since(Instant::now());
        match rx.recv_timeout(wait) {
            Ok(Event::Frame(frame)) => return frame,
            Ok(Event::Status(msg)) => panic!("unexpected status event: {msg}"),
            Ok(Event::Error(msg)) => panic!("unexpected error event: {msg}"),
            Ok(Event::SramSaved(_)) => {} // ignore SRAM save events in this test
            Ok(Event::Debug(_)) => {} // ignore debug events in this test
            Err(RecvTimeoutError::Timeout) => panic!("timed out waiting for a frame after {timeout:?}"),
            Err(RecvTimeoutError::Disconnected) => panic!("event channel closed before a frame arrived"),
        }
    }
}

#[test]
fn e01_09_start_paused_no_frame() {
    let handle = spawn_with_audio(Region::Ntsc, true, None);
    thread_sleep_ms(100);
    assert!(
        matches!(handle.rx.try_recv(), Err(TryRecvError::Empty)),
        "no event expected while paused"
    );
    handle.tx.send(Command::FrameAdvance).expect("send FrameAdvance");
    let frame = recv_frame(&handle.rx, Duration::from_secs(2));
    assert_eq!(frame.frame_no, 1);
    handle.quit();
}

#[test]
fn e01_09_frame_advance_twice() {
    let handle = spawn_with_audio(Region::Ntsc, true, None);
    thread_sleep_ms(50); // let the thread settle into its paused loop
    handle.tx.send(Command::FrameAdvance).expect("send FrameAdvance 1");
    handle.tx.send(Command::FrameAdvance).expect("send FrameAdvance 2");
    let first = recv_frame(&handle.rx, Duration::from_secs(2));
    let second = recv_frame(&handle.rx, Duration::from_secs(2));
    assert_eq!(first.frame_no, 1);
    assert_eq!(second.frame_no, 2);
    handle.quit();
}

#[test]
fn e01_09_run_free() {
    let handle = spawn_with_audio(Region::Ntsc, false, None);
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut last: Option<u64> = None;
    for _ in 0..3 {
        let wait = deadline.saturating_duration_since(Instant::now());
        match handle.rx.recv_timeout(wait) {
            Ok(Event::Frame(frame)) => {
                assert!(
                    last.map_or(true, |l| frame.frame_no > l),
                    "frame_no must strictly increase ({} then {})",
                    last.unwrap_or(0),
                    frame.frame_no
                );
                last = Some(frame.frame_no);
            }
            Ok(Event::Status(msg)) => panic!("unexpected status event: {msg}"),
            Ok(Event::Error(msg)) => panic!("unexpected error event: {msg}"),
            Ok(Event::SramSaved(_)) => {} // ignore SRAM save events in this test
            Ok(Event::Debug(_)) => {} // ignore debug events in this test
            Err(RecvTimeoutError::Timeout) => panic!("fewer than 3 frames in 2 s"),
            Err(RecvTimeoutError::Disconnected) => panic!("event channel closed early"),
        }
    }
    handle.quit();
}

#[test]
fn e01_09_joypad_visible() {
    let handle = spawn_with_audio(Region::Ntsc, true, None);
    thread_sleep_ms(50); // let the thread settle into its paused loop
    handle.tx.send(Command::SetJoypad(0, 0x8000)).expect("send SetJoypad");
    handle.tx.send(Command::FrameAdvance).expect("send FrameAdvance");
    let frame = recv_frame(&handle.rx, Duration::from_secs(2));
    assert_eq!(frame.pixels.len(), frame.width * frame.height);
    assert_eq!(frame.pixels[0], 0xFFFF_FFFF, "pixel (0,0) must be white when a button is held");
    handle.quit();
}

#[test]
fn e01_09_quit_joins() {
    let handle = spawn_with_audio(Region::Ntsc, true, None);
    let start = Instant::now();
    handle.quit();
    assert!(start.elapsed() < Duration::from_secs(1), "quit took too long");
}

fn thread_sleep_ms(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}
