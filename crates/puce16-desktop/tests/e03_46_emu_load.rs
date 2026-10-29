use std::time::{Duration, Instant};
use puce16_core::Region;
use puce16_desktop::emu_thread::{Command, Event};

fn tiny_rom() -> Vec<u8> {
    let mut r = vec![0u8; 0x8000];
    r[0] = 0x80; // BRA -2 (infinite loop at 0x8000)
    r[1] = 0xFE;
    r[0x7FFC] = 0x00; // RESET vector low byte: 0x8000
    r[0x7FFD] = 0x80; // RESET vector high byte
    r
}

fn recv_event(handle: &puce16_desktop::emu_thread::EmuHandle, timeout_s: u64) -> Option<Event> {
    let deadline = Instant::now() + Duration::from_secs(timeout_s);
    loop {
        match handle.rx.recv_timeout(deadline - Instant::now()) {
            Ok(event) => return Some(event),
            Err(_) => return None,
        }
    }
}

fn recv_until(handle: &puce16_desktop::emu_thread::EmuHandle, timeout_s: u64, pred: fn(&Event) -> bool) -> Option<Event> {
    let deadline = Instant::now() + Duration::from_secs(timeout_s);
    loop {
        match handle.rx.recv_timeout(deadline - Instant::now()) {
            Ok(event) => {
                if pred(&event) { return Some(event); }
            }
            Err(_) => return None,
        }
    }
}

#[test]
fn e03_46_load_ok_status() {
    let handle = puce16_desktop::emu_thread::spawn_with_audio(Region::Ntsc, true, None);
    handle.tx.send(Command::LoadRom(tiny_rom(), "tiny.sfc".into())).unwrap();

    let event = recv_until(&handle, 2, |e| matches!(e, Event::Status(s) if s.contains("Loaded tiny.sfc")));
    assert!(event.is_some());
    let status = match event.unwrap() {
        Event::Status(s) => s,
        _ => panic!("wrong event"),
    };
    assert!(status.contains("LoRom"));
    handle.quit();
}

#[test]
fn e03_46_load_ok_black_frame() {
    let handle = puce16_desktop::emu_thread::spawn_with_audio(Region::Ntsc, true, None);
    handle.tx.send(Command::LoadRom(tiny_rom(), "tiny.sfc".into())).unwrap();

    // Wait for status event first.
    recv_until(&handle, 2, |e| matches!(e, Event::Status(s) if s.contains("Loaded")));

    // FrameAdvance to get a frame from the loaded ROM.
    handle.tx.send(Command::FrameAdvance).unwrap();
    let event = recv_event(&handle, 2);
    assert!(matches!(&event, Some(Event::Frame(f)) if f.frame_no == 1));
    match event.unwrap() {
        Event::Frame(frame) => {
            assert_eq!(frame.width, 256);
            assert_eq!(frame.height, 224);
            // Pixel (0,0) should be black after loading a ROM that doesn't draw.
            let pixel = frame.pixels[0];
            assert_eq!(pixel, 0xFF000000);
        }
        _ => panic!("expected Frame event"),
    }
    handle.quit();
}

#[test]
fn e03_46_load_resumes_running() {
    let handle = puce16_desktop::emu_thread::spawn_with_audio(Region::Ntsc, true, None);
    handle.tx.send(Command::LoadRom(tiny_rom(), "tiny.sfc".into())).unwrap();

    // Wait for status event.
    recv_until(&handle, 2, |e| matches!(e, Event::Status(s) if s.contains("Loaded")));

    // After loading, the emulator should be running (not paused).
    // We should receive at least 2 Frame events without sending FrameAdvance.
    let mut frames = 0;
    let deadline = Instant::now() + Duration::from_secs(3);
    while frames < 2 && Instant::now() < deadline {
        match handle.rx.recv_timeout(deadline - Instant::now()) {
            Ok(Event::Frame(_)) => frames += 1,
            Ok(_) => {} // Skip other events.
            Err(_) => break,
        }
    }
    assert!(frames >= 2, "expected at least 2 frames while running, got {}", frames);
    handle.quit();
}

#[test]
fn e03_46_load_error_keeps_old() {
    let handle = puce16_desktop::emu_thread::spawn_with_audio(Region::Ntsc, true, None);

    // Try to load a ROM that's too small.
    handle.tx.send(Command::LoadRom(vec![0u8; 10], "bad.sfc".into())).unwrap();

    let event = recv_until(&handle, 2, |e| matches!(e, Event::Error(s) if s.contains("cannot load bad.sfc")));
    assert!(event.is_some());
    let error = match event.unwrap() {
        Event::Error(s) => s,
        _ => panic!("wrong event"),
    };
    assert!(error.contains("ROM too small"));

    // The old test pattern should still work. FrameAdvance should produce a frame.
    handle.tx.send(Command::FrameAdvance).unwrap();
    let event = recv_event(&handle, 2);
    assert!(matches!(&event, Some(Event::Frame(_))));
    handle.quit();
}

#[test]
fn e03_46_reset_real() {
    let handle = puce16_desktop::emu_thread::spawn_with_audio(Region::Ntsc, true, None);
    handle.tx.send(Command::LoadRom(tiny_rom(), "tiny.sfc".into())).unwrap();

    // Wait for status event.
    recv_until(&handle, 2, |e| matches!(e, Event::Status(s) if s.contains("Loaded")));

    // Advance two frames (frame_no should be 1 then 2).
    handle.tx.send(Command::FrameAdvance).unwrap();
    let e1 = recv_event(&handle, 2);
    assert!(matches!(&e1, Some(Event::Frame(f)) if f.frame_no == 1));

    handle.tx.send(Command::FrameAdvance).unwrap();
    let e2 = recv_event(&handle, 2);
    assert!(matches!(&e2, Some(Event::Frame(f)) if f.frame_no == 2));

    // Reset and advance again - frame_no should be back to 1.
    handle.tx.send(Command::Reset).unwrap();
    handle.tx.send(Command::FrameAdvance).unwrap();
    let e3 = recv_event(&handle, 2);
    assert!(matches!(&e3, Some(Event::Frame(f)) if f.frame_no == 1));
    handle.quit();
}

#[test]
fn e03_46_joypad_survives_load() {
    let handle = puce16_desktop::emu_thread::spawn_with_audio(Region::Ntsc, true, None);

    // Set joypad before loading.
    handle.tx.send(Command::SetJoypad(0, 0x8000)).unwrap();

    // Load ROM - should not crash even with joypad state set.
    handle.tx.send(Command::LoadRom(tiny_rom(), "tiny.sfc".into())).unwrap();

    let event = recv_until(&handle, 2, |e| matches!(e, Event::Status(s) if s.contains("Loaded")));
    assert!(event.is_some());
    handle.quit();
}

#[test]
fn e03_46_quit_after_load() {
    let handle = puce16_desktop::emu_thread::spawn_with_audio(Region::Ntsc, true, None);
    handle.tx.send(Command::LoadRom(tiny_rom(), "tiny.sfc".into())).unwrap();

    // Wait for status event.
    recv_until(&handle, 2, |e| matches!(e, Event::Status(s) if s.contains("Loaded")));

    let start = Instant::now();
    handle.quit();
    assert!(start.elapsed() < Duration::from_secs(1));
}
