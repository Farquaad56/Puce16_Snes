//! E08-21: SRAM loading, periodic saving (every 5s), and save on close.

use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::RecvTimeoutError;
use std::time::{Duration, Instant};

use puce16_core::Region;
use puce16_desktop::emu_thread::{Command, Event, SramSync, SRAM_SAVE_INTERVAL};

fn sram_rom(code: &[u8]) -> Vec<u8> {
    let mut r = vec![0u8; 0x8000];
    r[..code.len()].copy_from_slice(code);
    // SNES header at 0x7FC0 (LoROM)
    r[0x7FC0..0x7FC0 + 21].copy_from_slice(b"PUCE16 TEST          ");
    r[0x7FD5] = 0x20; // mapper type: LoROM with SRAM
    r[0x7FD6] = 0x02; // ROM size: 32 KiB (1 bank)
    r[0x7FD7] = 0x05; // RAM size: 8 KiB
    r[0x7FD8] = 0x03; // region: NTSC
    r[0x7FD9] = 0x01; // company code
    r[0x7FDA] = 0x33; // version
    // Reset vector at 0x7FFC (points to 0x8000)
    r[0x7FFC] = 0x00;
    r[0x7FFD] = 0x80;
    // Checksums
    let sum: u32 = r.iter().map(|&b| b as u32).sum();
    let c = (sum & 0xFFFF) as u16;
    r[0x7FDE] = c as u8;
    r[0x7FDF] = (c >> 8) as u8;
    let n = !c;
    r[0x7FDC] = n as u8;
    r[0x7FDD] = (n >> 8) as u8;
    r
}

fn tiny_rom() -> Vec<u8> {
    // Minimal ROM without SRAM (RAM size byte = 0)
    let mut r = vec![0u8; 0x8000];
    r[0x7FC0..0x7FC0 + 21].copy_from_slice(b"TINY ROM TEST        ");
    r[0x7FD5] = 0x20; // map mode: LoROM
    r[0x7FD6] = 0x00; // cartridge type: ROM only
    r[0x7FD7] = 0x05; // ROM size: 1 KiB << 5 = 32 KiB
    r[0x7FD8] = 0x00; // RAM size: none
    r[0x7FD9] = 0x01; // region: USA (NTSC)
    let sum: u32 = r.iter().map(|&b| b as u32).sum();
    let c = (sum & 0xFFFF) as u16;
    r[0x7FDE] = c as u8;
    r[0x7FDF] = (c >> 8) as u8;
    let n = !c;
    r[0x7FDC] = n as u8;
    r[0x7FDD] = (n >> 8) as u8;
    r
}

fn wait_event(rx: &std::sync::mpsc::Receiver<Event>, timeout: Duration) -> Option<Event> {
    match rx.recv_timeout(timeout) {
        Ok(e) => Some(e),
        Err(RecvTimeoutError::Timeout) => None,
        Err(RecvTimeoutError::Disconnected) => None,
    }
}

#[test]
fn e08_21_sync_due() {
    let t0 = Instant::now();
    let mut sync = SramSync::new(t0);

    // 4s later: not due
    assert!(!sync.due(t0 + Duration::from_secs(4)));
    // 5s later: due, resets last_check to t0+5s
    assert!(sync.due(t0 + Duration::from_secs(5)));
    // 6s later (1s after reset): not due
    assert!(!sync.due(t0 + Duration::from_secs(6)));
    // 10s later (5s after reset): due again
    assert!(sync.due(t0 + Duration::from_secs(10)));
}

#[test]
fn e08_21_sync_changed() {
    let t0 = Instant::now();
    let mut sync = SramSync::new(t0);

    // Nothing saved yet: changed is true for any data
    assert!(sync.changed(&[1]));
    // After marking as saved, unchanged data reports false
    sync.mark_saved(&[1]);
    assert!(!sync.changed(&[1]));
    // Different data reports true
    assert!(sync.changed(&[2]));
    // Reset with known state: that exact data is not changed
    sync.reset(t0, Some(&[7]));
    assert!(!sync.changed(&[7]));
}

#[test]
fn e08_21_flush_writes_file() {
    let dir = std::env::temp_dir().join(format!("puce16_e08_21_flush_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();

    // Program that writes 0x5A to SRAM address 0
    let prog_w: &[u8] = &[
        0x78, 0x18, 0xFB, // SEI CLC XCE
        0xA9, 0x5A,       // LDA #$5A
        0x8F, 0x00, 0x00, 0x70, 0x80, // STA $700000 (SRAM bank)
        0xFE,             // BRA -2 (loop)
    ];

    let rom = sram_rom(prog_w);
    let rom_path = dir.join("game.sfc");
    fs::write(&rom_path, &rom).unwrap();

    let handle = puce16_desktop::emu_thread::spawn_with_audio(Region::Ntsc, true, None);

    // Send ROM path and load
    handle.tx.send(Command::SetRomPath(rom_path.clone())).unwrap();
    handle.tx.send(Command::LoadRom(rom, "game.sfc".to_string())).unwrap();

    // Wait for the program to write to SRAM (300ms)
    std::thread::sleep(Duration::from_millis(300));

    // Flush SRAM
    handle.tx.send(Command::FlushSram).unwrap();

    // Expect SramSaved event
    let mut saved_path = None;
    for _ in 0..20 {
        match wait_event(&handle.rx, Duration::from_millis(100)) {
            Some(Event::SramSaved(p)) => { saved_path = Some(p); break; }
            Some(_) => {} // Ignore other events
            None => break,
        }
    }
    assert!(saved_path.is_some(), "Expected SramSaved event");

    let srm_path = dir.join("game.srm");
    assert!(srm_path.exists(), "SRAM file should exist");
    let data = fs::read(&srm_path).unwrap();
    assert_eq!(data.len(), 8192, "SRAM should be 8 KiB");
    assert_eq!(data[0], 0x5A, "SRAM byte 0 should be 0x5A");

    handle.quit();
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn e08_21_sram_loaded_on_open() {
    let dir = std::env::temp_dir().join(format!("puce16_e08_21_load_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();

    // First: write SRAM with value 0x5A at address 0
    let prog_w: &[u8] = &[
        0x78, 0x18, 0xFB,
        0xA9, 0x5A,
        0x8F, 0x00, 0x00, 0x70, 0x80,
        0xFE,
    ];

    let rom = sram_rom(prog_w);
    let rom_path = dir.join("game.sfc");
    fs::write(&rom_path, &rom).unwrap();

    {
        let handle = puce16_desktop::emu_thread::spawn_with_audio(Region::Ntsc, true, None);
        handle.tx.send(Command::SetRomPath(rom_path.clone())).unwrap();
        handle.tx.send(Command::LoadRom(rom.clone(), "game.sfc".to_string())).unwrap();
        std::thread::sleep(Duration::from_millis(300));
        handle.tx.send(Command::FlushSram).unwrap();

        // Wait for save event
        let mut got_save = false;
        for _ in 0..20 {
            match wait_event(&handle.rx, Duration::from_millis(100)) {
                Some(Event::SramSaved(_)) => { got_save = true; break; }
                Some(_) => {}
                None => break,
            }
        }
        assert!(got_save);
        handle.quit();
    }

    // Verify SRAM was saved with 0x5A at address 0
    let srm_path = dir.join("game.srm");
    let data = fs::read(&srm_path).unwrap();
    assert_eq!(data[0], 0x5A);

    // Now: load ROM that reads SRAM[0] and writes SRAM[1] = SRAM[0]+1
    let prog_r: &[u8] = &[
        0x78, 0x18, 0xFB, // SEI CLC XCE
        0xAF, 0x00, 0x00, 0x70, // LDA $700000 (read SRAM[0])
        0x1A,             // INC A
        0x8F, 0x01, 0x00, 0x70, 0x80, // STA $700001 (write to SRAM[1])
        0xFE,             // BRA -2
    ];

    let rom2 = sram_rom(prog_r);
    fs::write(&rom_path, &rom2).unwrap();

    {
        let handle = puce16_desktop::emu_thread::spawn_with_audio(Region::Ntsc, true, None);
        handle.tx.send(Command::SetRomPath(rom_path.clone())).unwrap();
        handle.tx.send(Command::LoadRom(rom2, "game.sfc".to_string())).unwrap();
        std::thread::sleep(Duration::from_millis(300));
        handle.tx.send(Command::FlushSram).unwrap();

        let mut got_save = false;
        for _ in 0..20 {
            match wait_event(&handle.rx, Duration::from_millis(100)) {
                Some(Event::SramSaved(_)) => { got_save = true; break; }
                Some(_) => {}
                None => break,
            }
        }
        assert!(got_save);
        handle.quit();
    }

    // Verify: SRAM[0] should still be 0x5A (loaded from file), SRAM[1] should be 0x5B
    let data = fs::read(&srm_path).unwrap();
    assert_eq!(data[0], 0x5A, "SRAM[0] should be preserved from loaded file");
    assert_eq!(data[1], 0x5B, "SRAM[1] should be SRAM[0]+1 (proof of loading)");

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn e08_21_quit_flushes() {
    let dir = std::env::temp_dir().join(format!("puce16_e08_21_quit_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();

    let prog_w: &[u8] = &[
        0x78, 0x18, 0xFB,
        0xA9, 0x5A,
        0x8F, 0x00, 0x00, 0x70, 0x80,
        0xFE,
    ];

    let rom = sram_rom(prog_w);
    let rom_path = dir.join("game.sfc");
    fs::write(&rom_path, &rom).unwrap();

    let handle = puce16_desktop::emu_thread::spawn_with_audio(Region::Ntsc, true, None);
    handle.tx.send(Command::SetRomPath(rom_path.clone())).unwrap();
    handle.tx.send(Command::LoadRom(rom, "game.sfc".to_string())).unwrap();

    std::thread::sleep(Duration::from_millis(300));

    // Quit without explicit FlushSram - should still save on exit
    handle.quit();

    let srm_path = dir.join("game.srm");
    assert!(srm_path.exists(), "SRAM file should exist after quit");
    let data = fs::read(&srm_path).unwrap();
    assert_eq!(data[0], 0x5A);

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn e08_21_no_sram_no_file() {
    let dir = std::env::temp_dir().join(format!("puce16_e08_21_nosram_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();

    let rom = tiny_rom();
    let rom_path = dir.join("game.sfc");
    fs::write(&rom_path, &rom).unwrap();

    let handle = puce16_desktop::emu_thread::spawn_with_audio(Region::Ntsc, true, None);
    handle.tx.send(Command::SetRomPath(rom_path.clone())).unwrap();
    handle.tx.send(Command::LoadRom(rom, "game.sfc".to_string())).unwrap();

    std::thread::sleep(Duration::from_millis(100));
    handle.tx.send(Command::FlushSram).unwrap();
    handle.quit();

    let srm_path = dir.join("game.srm");
    assert!(!srm_path.exists(), "No SRAM file should exist for ROM without SRAM");

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn e08_21_unchanged_not_rewritten() {
    let dir = std::env::temp_dir().join(format!("puce16_e08_21_unch_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();

    let prog_w: &[u8] = &[
        0x78, 0x18, 0xFB,
        0xA9, 0x5A,
        0x8F, 0x00, 0x00, 0x70, 0x80,
        0xFE,
    ];

    let rom = sram_rom(prog_w);
    let rom_path = dir.join("game.sfc");
    fs::write(&rom_path, &rom).unwrap();

    let handle = puce16_desktop::emu_thread::spawn_with_audio(Region::Ntsc, true, None);
    handle.tx.send(Command::SetRomPath(rom_path.clone())).unwrap();
    handle.tx.send(Command::LoadRom(rom, "game.sfc".to_string())).unwrap();

    std::thread::sleep(Duration::from_millis(300));

    // First flush: writes file
    handle.tx.send(Command::FlushSram).unwrap();
    let mut got_save = false;
    for _ in 0..20 {
        match wait_event(&handle.rx, Duration::from_millis(100)) {
            Some(Event::SramSaved(_)) => { got_save = true; break; }
            Some(_) => {}
            None => break,
        }
    }
    assert!(got_save);

    let srm_path = dir.join("game.srm");
    assert!(srm_path.exists());

    // Delete the file manually
    fs::remove_file(&srm_path).unwrap();
    assert!(!srm_path.exists());

    // Second flush: SRAM unchanged, should NOT recreate file
    handle.tx.send(Command::FlushSram).unwrap();
    std::thread::sleep(Duration::from_millis(200));

    assert!(!srm_path.exists(), "Unchanged SRAM should not be rewritten");

    handle.quit();
    fs::remove_dir_all(&dir).ok();
}
