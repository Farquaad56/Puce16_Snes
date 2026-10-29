//! E08-33: emu_thread debug commands (RequestDebug, SetWramPage, StepInstruction, RunToPc) and Event::Debug.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use puce16_core::Region;
use puce16_desktop::emu_thread::{Command, Event, spawn_with_audio};

/// Minimal LoROM (32 KiB) that loops forever: BRA -2 at 0x8000, RESET vector -> 0x8000.
fn tiny_rom() -> Vec<u8> {
    let mut r = vec![0u8; 0x8000];
    r[0] = 0x80; // BRA -2 (infinite loop)
    r[1] = 0xFE;
    r[0x7FFC] = 0x00; // RESET vector low byte
    r[0x7FFD] = 0x80; // RESET vector high byte
    r
}

/// ROM that stores $42 to WRAM at $0010 then loops: SEI CLC XCE LDA #$42 STA $0010 BRA -2.
fn store_rom() -> Vec<u8> {
    let mut r = tiny_rom();
    // Overwrite first 10 bytes with the program.
    r[0] = 0x78; // SEI
    r[1] = 0x18; // CLC
    r[2] = 0xFB; // XCE (enter emulation mode)
    r[3] = 0xA9; // LDA #$42
    r[4] = 0x42;
    r[5] = 0x8D; // STA $0010
    r[6] = 0x10;
    r[7] = 0x00;
    r[8] = 0x80; // BRA -2 (loop)
    r[9] = 0xFE;
    r
}

fn recv_event(rx: &mpsc::Receiver<Event>, timeout_ms: u64) -> Option<Event> {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(ev) => return Some(ev),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if Instant::now() >= deadline {
                    return None;
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => return None,
        }
    }
}

fn recv_until(rx: &mpsc::Receiver<Event>, timeout_ms: u64, pred: fn(&Event) -> bool) -> Option<Event> {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(ev) => {
                if pred(&ev) {
                    return Some(ev);
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if Instant::now() >= deadline {
                    return None;
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => return None,
        }
    }
}

#[test]
fn e08_33_no_debug_by_default() {
    let handle = spawn_with_audio(Region::Ntsc, true, None);
    handle.tx.send(Command::LoadRom(tiny_rom(), "tiny".to_string())).unwrap();
    // Wait for load status.
    recv_until(&handle.rx, 2000, |e| matches!(e, Event::Status(_))).unwrap();

    // LoadRom sets paused=false; pause again before testing.
    handle.tx.send(Command::Pause(true)).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(50));
    while handle.rx.try_recv().is_ok() {} // drop frames produced before the pause took effect

    handle.tx.send(Command::FrameAdvance).unwrap();
    let ev = recv_event(&handle.rx, 1000);
    assert!(matches!(&ev, Some(Event::Frame(_))));

    // No debug event should arrive within 200ms.
    let ev2 = recv_event(&handle.rx, 200);
    assert!(ev2.is_none(), "unexpected event: {:?}", ev2);

    handle.quit();
}

#[test]
fn e08_33_debug_after_frame() {
    let handle = spawn_with_audio(Region::Ntsc, true, None);
    handle.tx.send(Command::LoadRom(tiny_rom(), "tiny".to_string())).unwrap();
    recv_until(&handle.rx, 2000, |e| matches!(e, Event::Status(_))).unwrap();

    // LoadRom sets paused=false; pause again before testing.
    handle.tx.send(Command::Pause(true)).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(50));
    while handle.rx.try_recv().is_ok() {} // drop frames produced before the pause took effect

    handle.tx.send(Command::RequestDebug(true)).unwrap();
    handle.tx.send(Command::FrameAdvance).unwrap();

    // Should receive Frame then Debug.
    let ev1 = recv_event(&handle.rx, 2000);
    assert!(matches!(&ev1, Some(Event::Frame(_))));

    let ev2 = recv_event(&handle.rx, 2000);
    match ev2 {
        Some(Event::Debug(data)) => {
            assert_eq!(data.vram.len(), 32768);
            assert_eq!(data.cgram.len(), 256);
            assert_eq!(data.oam.len(), 544);
            assert_eq!(data.wram_page.len(), 256);
            assert!(data.ppu_regs.is_some());
        }
        other => panic!("expected Event::Debug, got {:?}", other),
    }

    handle.quit();
}

#[test]
fn e08_33_debug_off_again() {
    let handle = spawn_with_audio(Region::Ntsc, true, None);
    handle.tx.send(Command::LoadRom(tiny_rom(), "tiny".to_string())).unwrap();
    recv_until(&handle.rx, 2000, |e| matches!(e, Event::Status(_))).unwrap();

    // LoadRom sets paused=false; pause again before testing.
    handle.tx.send(Command::Pause(true)).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(50));
    while handle.rx.try_recv().is_ok() {} // drop frames produced before the pause took effect

    handle.tx.send(Command::RequestDebug(true)).unwrap();
    handle.tx.send(Command::RequestDebug(false)).unwrap();
    handle.tx.send(Command::FrameAdvance).unwrap();

    let ev1 = recv_event(&handle.rx, 2000);
    assert!(matches!(&ev1, Some(Event::Frame(_))));

    // No debug event after turning it off.
    let ev2 = recv_event(&handle.rx, 200);
    assert!(ev2.is_none(), "unexpected event: {:?}", ev2);

    handle.quit();
}

#[test]
fn e08_33_pattern_debug_empty() {
    // No ROM loaded - test pattern mode.
    let handle = spawn_with_audio(Region::Ntsc, true, None);

    handle.tx.send(Command::RequestDebug(true)).unwrap();
    handle.tx.send(Command::FrameAdvance).unwrap();

    let ev1 = recv_event(&handle.rx, 2000);
    assert!(matches!(&ev1, Some(Event::Frame(_))));

    let ev2 = recv_event(&handle.rx, 2000);
    match ev2 {
        Some(Event::Debug(data)) => {
            assert!(data.vram.is_empty());
            assert!(data.cgram.is_empty());
            assert!(data.oam.is_empty());
            assert!(data.wram_page.is_empty());
            assert!(data.ppu_regs.is_none());
            assert!(data.smp.is_none());
            assert!(data.dma.is_none());
        }
        other => panic!("expected Event::Debug, got {:?}", other),
    }

    handle.quit();
}

#[test]
fn e08_33_step_instruction() {
    let handle = spawn_with_audio(Region::Ntsc, true, None);
    handle.tx.send(Command::LoadRom(store_rom(), "store".to_string())).unwrap();
    recv_until(&handle.rx, 2000, |e| matches!(e, Event::Status(_))).unwrap();

    // LoadRom sets paused=false; pause again before testing.
    handle.tx.send(Command::Pause(true)).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(50));
    while handle.rx.try_recv().is_ok() {} // drop frames produced before the pause took effect

    // The ROM ran a few frames before the pause: go back to the reset vector (PC 0x8000).
    handle.tx.send(Command::Reset).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(50));
    while handle.rx.try_recv().is_ok() {}

    // Enable debug.
    handle.tx.send(Command::RequestDebug(true)).unwrap();

    // Step 3 instructions: SEI(1) CLC(2) XCE(3) -> PC should be at 0x8003 after XCE.
    for _ in 0..3 {
        handle.tx.send(Command::StepInstruction).unwrap();
        let ev = recv_event(&handle.rx, 2000);
        match ev {
            Some(Event::Debug(data)) => {
                // Record PC after each step.
            }
            other => panic!("expected Event::Debug during step, got {:?}", other),
        }
    }

    // After 3 steps (SEI, CLC, XCE), PC should be at 0x8003.
    handle.tx.send(Command::SetWramPage(0)).unwrap();
    let ev = recv_event(&handle.rx, 2000);
    match ev {
        Some(Event::Debug(data)) => {
            assert_eq!(data.snapshot.pc, 0x8003, "PC should be 0x8003 after 3 steps");
        }
        other => panic!("expected Event::Debug, got {:?}", other),
    }

    handle.quit();
}

#[test]
fn e08_33_step_requires_pause() {
    let handle = spawn_with_audio(Region::Ntsc, false, None); // Not paused.
    handle.tx.send(Command::LoadRom(tiny_rom(), "tiny".to_string())).unwrap();
    recv_until(&handle.rx, 2000, |e| matches!(e, Event::Status(_))).unwrap();

    handle.tx.send(Command::StepInstruction).unwrap();

    // Should get "pause first" status.
    let ev = recv_until(&handle.rx, 2000, |e| {
        if let Event::Status(s) = e {
            return s.contains("pause first");
        }
        false
    });
    assert!(ev.is_some(), "expected 'pause first' status");

    handle.quit();
}

#[test]
fn e08_33_run_to_pc() {
    let handle = spawn_with_audio(Region::Ntsc, true, None);
    handle.tx.send(Command::LoadRom(store_rom(), "store".to_string())).unwrap();
    recv_until(&handle.rx, 2000, |e| matches!(e, Event::Status(_))).unwrap();

    // LoadRom sets paused=false; pause again before testing.
    handle.tx.send(Command::Pause(true)).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(50));
    while handle.rx.try_recv().is_ok() {} // drop frames produced before the pause took effect

    // Debug data on, then run to PC at the BRA -2 instruction (address 0x8008).
    handle.tx.send(Command::RequestDebug(true)).unwrap();
    handle.tx.send(Command::RunToPc(0x008008)).unwrap();

    let ev = recv_until(&handle.rx, 5000, |e| {
        if let Event::Status(s) = e {
            return s.contains("Breakpoint");
        }
        false
    });
    match ev {
        Some(Event::Status(s)) => {
            assert!(s.contains("Breakpoint 00:8008"), "unexpected status: {}", s);
        }
        other => panic!("expected Breakpoint status, got {:?}", other),
    }

    // Verify WRAM was written by checking wram_page[0x10] == 0x42.
    handle.tx.send(Command::SetWramPage(0)).unwrap();
    let ev = recv_until(&handle.rx, 2000, |e| matches!(e, Event::Debug(_)));
    match ev {
        Some(Event::Debug(data)) => {
            assert_eq!(data.wram_page[0x10], 0x42, "WRAM at $0010 should be 0x42");
        }
        other => panic!("expected Event::Debug, got {:?}", other),
    }

    handle.quit();
}

#[test]
fn e08_33_run_to_pc_cancel() {
    let handle = spawn_with_audio(Region::Ntsc, true, None);
    handle.tx.send(Command::LoadRom(tiny_rom(), "tiny".to_string())).unwrap();
    recv_until(&handle.rx, 2000, |e| matches!(e, Event::Status(_))).unwrap();

    // LoadRom sets paused=false; pause again before testing.
    handle.tx.send(Command::Pause(true)).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(50));
    while handle.rx.try_recv().is_ok() {} // drop frames produced before the pause took effect

    // Run to an unreachable PC (0x9000 is outside ROM).
    handle.tx.send(Command::RunToPc(0x009000)).unwrap();
    // Immediately cancel by resuming.
    std::thread::sleep(Duration::from_millis(10));
    handle.tx.send(Command::Pause(false)).unwrap();

    let ev = recv_until(&handle.rx, 10000, |e| {
        if let Event::Status(s) = e {
            return s.contains("cancelled");
        }
        false
    });
    assert!(ev.is_some(), "expected 'cancelled' status");

    // Emulation should now be running - wait for a frame.
    let ev2 = recv_until(&handle.rx, 5000, |e| matches!(e, Event::Frame(_)));
    assert!(ev2.is_some(), "expected Frame after cancel");

    handle.quit();
}

#[test]
fn e08_33_set_wram_page() {
    let handle = spawn_with_audio(Region::Ntsc, true, None);
    handle.tx.send(Command::LoadRom(tiny_rom(), "tiny".to_string())).unwrap();
    recv_until(&handle.rx, 2000, |e| matches!(e, Event::Status(_))).unwrap();

    // LoadRom sets paused=false; pause again before testing.
    handle.tx.send(Command::Pause(true)).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(50));
    while handle.rx.try_recv().is_ok() {} // drop frames produced before the pause took effect

    handle.tx.send(Command::RequestDebug(true)).unwrap();
    handle.tx.send(Command::SetWramPage(1)).unwrap();

    let ev = recv_event(&handle.rx, 2000);
    match ev {
        Some(Event::Debug(data)) => {
            assert_eq!(data.wram_page.len(), 256);
        }
        other => panic!("expected Event::Debug after SetWramPage, got {:?}", other),
    }

    handle.quit();
}
