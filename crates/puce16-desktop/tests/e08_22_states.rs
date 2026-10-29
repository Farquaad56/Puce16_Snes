//! E08-22: save states (SaveState / LoadState, slots 0..9) in the emulation thread.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use puce16_core::Region;
use puce16_desktop::emu_thread::{spawn_with_audio, Command, EmuHandle, Event};
use puce16_desktop::files;

/// LoROM 32 KiB with 8 KiB of SRAM (header per spec 08), `code` at $00:8000.
fn sram_rom(code: &[u8]) -> Vec<u8> {
    let mut r = vec![0u8; 0x8000];
    r[..code.len()].copy_from_slice(code);
    r[0x7FC0..0x7FC0 + 21].copy_from_slice(b"PUCE16 TEST          ");
    r[0x7FD5] = 0x20;
    r[0x7FD6] = 0x02;
    r[0x7FD7] = 0x05;
    r[0x7FD8] = 0x03;
    r[0x7FD9] = 0x01;
    r[0x7FDA] = 0x33;
    r[0x7FFC] = 0x00;
    r[0x7FFD] = 0x80;
    r[0x7FDC] = 0xFF;
    r[0x7FDD] = 0xFF;
    r[0x7FDE] = 0x00;
    r[0x7FDF] = 0x00;
    let sum: u32 = r.iter().map(|&b| b as u32).sum();
    let c = (sum & 0xFFFF) as u16;
    r[0x7FDE] = c as u8;
    r[0x7FDF] = (c >> 8) as u8;
    let n = !c;
    r[0x7FDC] = n as u8;
    r[0x7FDD] = (n >> 8) as u8;
    r
}

/// Writes 0x5A to SRAM[0], then loops.
const PROG_W: &[u8] = &[
    0x78, 0x18, 0xFB, // SEI CLC XCE
    0xA9, 0x5A, // LDA #$5A
    0x8F, 0x00, 0x00, 0x70, // STA $700000
    0x80, 0xFE, // BRA -2
];

/// Reads SRAM[0], adds one, writes SRAM[1], then loops.
const PROG_R: &[u8] = &[
    0x78, 0x18, 0xFB, // SEI CLC XCE
    0xAF, 0x00, 0x00, 0x70, // LDA $700000
    0x1A, // INC A
    0x8F, 0x01, 0x00, 0x70, // STA $700001
    0x80, 0xFE, // BRA -2
];

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("puce16_e08_22_{}_{}", tag, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Wait for the first event matching `pred`; other events are skipped.
fn wait_for<F: Fn(&Event) -> bool>(h: &EmuHandle, timeout: Duration, pred: F) -> Option<Event> {
    let deadline = Instant::now() + timeout;
    loop {
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return None;
        }
        match h.rx.recv_timeout(left) {
            Ok(ev) => {
                if pred(&ev) {
                    return Some(ev);
                }
            }
            Err(_) => return None,
        }
    }
}

fn is_status(ev: &Event, needle: &str) -> bool {
    matches!(ev, Event::Status(s) if s.contains(needle))
}

fn is_error(ev: &Event, needle: &str) -> bool {
    matches!(ev, Event::Error(s) if s.contains(needle))
}

fn frame_no(ev: &Event) -> Option<u64> {
    match ev {
        Event::Frame(f) => Some(f.frame_no),
        _ => None,
    }
}

/// Wait for a frame with this exact number.
fn wait_frame(h: &EmuHandle, n: u64) -> bool {
    wait_for(h, Duration::from_secs(3), |ev| frame_no(ev) == Some(n)).is_some()
}

/// Drop whatever is queued until the thread has been quiet for 150 ms.
fn drain(h: &EmuHandle) {
    while h.rx.recv_timeout(Duration::from_millis(150)).is_ok() {}
}

/// Paused console with the ROM loaded from `dir/game.sfc`, frame counter at 0.
fn start_paused(dir: &Path, rom: &[u8]) -> EmuHandle {
    let rom_path = dir.join("game.sfc");
    fs::write(&rom_path, rom).unwrap();
    let h = spawn_with_audio(Region::Ntsc, true, None);
    h.tx.send(Command::SetRomPath(rom_path)).unwrap();
    h.tx.send(Command::LoadRom(rom.to_vec(), "game.sfc".to_string())).unwrap();
    // LoadRom resumes emulation, so pause right after it, then restart the counter.
    h.tx.send(Command::Pause(true)).unwrap();
    h.tx.send(Command::Reset).unwrap();
    drain(&h);
    h
}

fn advance(h: &EmuHandle, count: u64, first_expected: u64) {
    for i in 0..count {
        h.tx.send(Command::FrameAdvance).unwrap();
        assert!(wait_frame(h, first_expected + i), "frame {} not received", first_expected + i);
    }
}

#[test]
fn e08_22_save_without_rom() {
    let h = spawn_with_audio(Region::Ntsc, true, None);
    h.tx.send(Command::SaveState(0)).unwrap();
    let ev = wait_for(&h, Duration::from_secs(2), |e| is_error(e, "no ROM loaded"));
    assert!(ev.is_some(), "expected Error(no ROM loaded)");
    h.quit();
}

#[test]
fn e08_22_save_creates_file() {
    let dir = temp_dir("save");
    let h = start_paused(&dir, &sram_rom(PROG_W));
    advance(&h, 2, 1);
    h.tx.send(Command::SaveState(3)).unwrap();
    assert!(
        wait_for(&h, Duration::from_secs(3), |e| is_status(e, "State saved to slot 3")).is_some(),
        "expected Status(State saved to slot 3)"
    );
    let data = fs::read(dir.join("game.3.p16s")).expect("state file");
    assert_eq!(&data[..4], b"P16S");
    assert!(data.len() > 1_000_000, "state file too small: {}", data.len());
    h.quit();
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn e08_22_load_restores_frame_counter() {
    let dir = temp_dir("counter");
    let h = start_paused(&dir, &sram_rom(PROG_W));
    advance(&h, 2, 1);
    h.tx.send(Command::SaveState(0)).unwrap();
    assert!(wait_for(&h, Duration::from_secs(3), |e| is_status(e, "State saved")).is_some());
    advance(&h, 3, 3); // frames 3, 4, 5
    h.tx.send(Command::LoadState(0)).unwrap();
    assert!(
        wait_for(&h, Duration::from_secs(3), |e| is_status(e, "State loaded from slot 0")).is_some(),
        "expected Status(State loaded from slot 0)"
    );
    let ev = wait_for(&h, Duration::from_secs(3), |e| frame_no(e).is_some()).expect("frame after load");
    assert_eq!(frame_no(&ev), Some(2), "the frame published after load is the saved one");
    h.tx.send(Command::FrameAdvance).unwrap();
    assert!(wait_frame(&h, 3), "next frame after load must be 3");
    h.quit();
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn e08_22_load_missing_slot() {
    let dir = temp_dir("missing");
    let h = start_paused(&dir, &sram_rom(PROG_W));
    h.tx.send(Command::LoadState(7)).unwrap();
    assert!(
        wait_for(&h, Duration::from_secs(2), |e| is_error(e, "no state in slot 7")).is_some(),
        "expected Error(no state in slot 7)"
    );
    h.quit();
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn e08_22_load_foreign_rom() {
    let dir = temp_dir("foreign");
    {
        let h = start_paused(&dir, &sram_rom(PROG_W));
        h.tx.send(Command::SaveState(0)).unwrap();
        assert!(wait_for(&h, Duration::from_secs(3), |e| is_status(e, "State saved")).is_some());
        h.quit();
    }
    // Another ROM at the same path: the state file is found but belongs to ROM A.
    let h = start_paused(&dir, &sram_rom(PROG_R));
    h.tx.send(Command::LoadState(0)).unwrap();
    assert!(
        wait_for(&h, Duration::from_secs(2), |e| is_error(e, "another ROM")).is_some(),
        "expected Error(... another ROM)"
    );
    h.quit();
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn e08_22_load_corrupt_blob() {
    let dir = temp_dir("corrupt");
    let rom = sram_rom(PROG_W);
    let h = start_paused(&dir, &rom);
    let data = files::encode_state(files::rom_hash(&rom), 0, &[1, 2, 3]);
    fs::write(dir.join("game.1.p16s"), data).unwrap();
    h.tx.send(Command::LoadState(1)).unwrap();
    assert!(
        wait_for(&h, Duration::from_secs(2), |e| is_error(e, "cannot load state")).is_some(),
        "expected Error(cannot load state ...)"
    );
    // The console was rebuilt: frames still advance (counter restarts from 0).
    h.tx.send(Command::FrameAdvance).unwrap();
    assert!(
        wait_for(&h, Duration::from_secs(3), |e| frame_no(e).is_some()).is_some(),
        "FrameAdvance must still work after a failed load"
    );
    h.quit();
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn e08_22_load_works_while_running() {
    let dir = temp_dir("running");
    let rom = sram_rom(PROG_W);
    let rom_path = dir.join("game.sfc");
    fs::write(&rom_path, &rom).unwrap();
    let h = spawn_with_audio(Region::Ntsc, false, None);
    h.tx.send(Command::SetRomPath(rom_path)).unwrap();
    h.tx.send(Command::LoadRom(rom, "game.sfc".to_string())).unwrap();
    h.tx.send(Command::SaveState(0)).unwrap();
    assert!(wait_for(&h, Duration::from_secs(3), |e| is_status(e, "State saved")).is_some());
    std::thread::sleep(Duration::from_millis(200));
    h.tx.send(Command::LoadState(0)).unwrap();
    assert!(wait_for(&h, Duration::from_secs(3), |e| is_status(e, "State loaded")).is_some());
    // Frames keep arriving after the load.
    let mut frames = 0;
    let deadline = Instant::now() + Duration::from_secs(2);
    while frames < 3 && Instant::now() < deadline {
        if let Ok(ev) = h.rx.recv_timeout(Duration::from_millis(200)) {
            if frame_no(&ev).is_some() {
                frames += 1;
            }
        }
    }
    assert!(frames >= 3, "frames must continue after load, got {}", frames);
    h.quit();
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn e08_22_pal_region_byte() {
    let dir = temp_dir("pal");
    let mut rom = sram_rom(PROG_W);
    rom[0x7FD9] = 0x02; // country: Europe -> PAL
    // Recompute the checksum after the header change.
    rom[0x7FDC] = 0xFF;
    rom[0x7FDD] = 0xFF;
    rom[0x7FDE] = 0x00;
    rom[0x7FDF] = 0x00;
    let sum: u32 = rom.iter().map(|&b| b as u32).sum();
    let c = (sum & 0xFFFF) as u16;
    rom[0x7FDE] = c as u8;
    rom[0x7FDF] = (c >> 8) as u8;
    rom[0x7FDC] = (!c) as u8;
    rom[0x7FDD] = ((!c) >> 8) as u8;

    let h = start_paused(&dir, &rom);
    h.tx.send(Command::SaveState(0)).unwrap();
    assert!(wait_for(&h, Duration::from_secs(3), |e| is_status(e, "State saved")).is_some());
    let data = fs::read(dir.join("game.0.p16s")).unwrap();
    assert_eq!(data[14], 1, "region byte must be 1 for a PAL ROM");
    h.quit();
    let _ = fs::remove_dir_all(&dir);
}
