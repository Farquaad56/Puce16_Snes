//! E08-19: emu_thread audio sync, PAL/NTSC cadence, fast forward.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use puce16_core::Region;
use puce16_desktop::audio::{new_shared_ring, RING_WAIT_ABOVE};
use puce16_desktop::emu_thread::{Command, Event, frame_period_ns, spawn_with_audio};

/// Minimal LoROM (32 KiB) that loops forever at 0x8000.
fn tiny_rom() -> Vec<u8> {
    let mut r = vec![0u8; 0x8000];
    // BRA -2 at 0x8000 (infinite loop)
    r[0] = 0x80;
    r[1] = 0xFE;
    // RESET vector -> 0x8000
    r[0x7FFC] = 0x00;
    r[0x7FFD] = 0x80;
    r
}

#[test]
fn e08_19_frame_period() {
    assert_eq!(frame_period_ns(Region::Ntsc), 16_639_267);
    assert_eq!(frame_period_ns(Region::Pal), 19_997_200);
}

#[test]
fn e08_19_audio_flows_into_ring() {
    let ring = new_shared_ring();
    let handle = spawn_with_audio(Region::Ntsc, false, Some(ring.clone()));
    handle.tx.send(Command::LoadRom(tiny_rom(), "t".to_string())).unwrap();

    std::thread::sleep(Duration::from_millis(300));
    let len = ring.lock().unwrap().len();
    assert!(len > 500, "ring should have >500 frames after 300ms, got {}", len);
    handle.quit();
}

#[test]
fn e08_19_back_pressure() {
    let ring = new_shared_ring();
    let handle = spawn_with_audio(Region::Ntsc, false, Some(ring.clone()));
    handle.tx.send(Command::LoadRom(tiny_rom(), "t".to_string())).unwrap();

    // No consumer: emulation should back-pressure and wait.
    std::thread::sleep(Duration::from_millis(600));
    let len = ring.lock().unwrap().len();
    assert!(
        len <= RING_WAIT_ABOVE + 600,
        "ring should be bounded by back-pressure, got {}",
        len
    );
    assert!(
        len >= RING_WAIT_ABOVE - 600,
        "ring should be near RING_WAIT_ABOVE, got {}",
        len
    );
    handle.quit();
}

#[test]
fn e08_19_consumer_unblocks() {
    let ring = new_shared_ring();
    let handle = spawn_with_audio(Region::Ntsc, false, Some(ring.clone()));
    handle.tx.send(Command::LoadRom(tiny_rom(), "t".to_string())).unwrap();

    // Consumer thread: pop 256 frames every 5ms until told to stop.
    let consumer_ring = ring.clone();
    let should_stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stop_clone = should_stop.clone();
    let consumer = std::thread::spawn(move || {
        loop {
            if stop_clone.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }
            let mut buf = vec![[0i16; 2]; 256];
            match consumer_ring.lock() {
                Ok(mut r) => { r.pop(&mut buf); }
                Err(poisoned) => { poisoned.into_inner().pop(&mut buf); }
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    });

    // Count frames received in 2 seconds.
    let mut frame_count = 0;
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(2) {
        if handle.rx.try_recv().is_ok() {
            frame_count += 1;
        }
    }

    assert!(frame_count >= 20, "consumer should unblock producer: got {} frames", frame_count);
    should_stop.store(true, std::sync::atomic::Ordering::Relaxed);
    consumer.join().unwrap();
    handle.quit();
}

#[test]
fn e08_19_fast_forward_ignores_ring() {
    let ring = new_shared_ring();
    let handle = spawn_with_audio(Region::Ntsc, false, Some(ring.clone()));
    handle.tx.send(Command::LoadRom(tiny_rom(), "t".to_string())).unwrap();

    // Enable fast forward.
    handle.tx.send(Command::SetFastForward(true)).unwrap();

    // Count frames in 1 second (should be many).
    let mut ff_count = 0;
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(1) {
        if handle.rx.try_recv().is_ok() {
            ff_count += 1;
        }
    }
    assert!(ff_count >= 30, "fast forward should produce many frames: got {}", ff_count);

    // Disable fast forward and check rate drops.
    handle.tx.send(Command::SetFastForward(false)).unwrap();
    std::thread::sleep(Duration::from_millis(100));

    let mut normal_count = 0;
    let start2 = Instant::now();
    while start2.elapsed() < Duration::from_millis(300) {
        if handle.rx.try_recv().is_ok() {
            normal_count += 1;
        }
    }
    assert!(normal_count < 30, "rate should drop after fast forward off: got {}", normal_count);

    handle.quit();
}

#[test]
fn e08_19_timer_pacing_without_ring() {
    let handle = spawn_with_audio(Region::Ntsc, false, None);
    handle.tx.send(Command::LoadRom(tiny_rom(), "t".to_string())).unwrap();

    // Collect 12 frames and measure time.
    let start = Instant::now();
    for _ in 0..12 {
        handle.rx.recv_timeout(Duration::from_secs(5)).unwrap();
    }
    let elapsed = start.elapsed();

    assert!(
        elapsed > Duration::from_millis(150),
        "12 frames should take >150ms at ~60Hz, got {:?}",
        elapsed
    );
    assert!(
        elapsed < Duration::from_secs(1),
        "12 frames should take <1s at ~60Hz, got {:?}",
        elapsed
    );

    handle.quit();
}

#[test]
fn e08_19_pause_pushes_nothing() {
    let ring = new_shared_ring();
    let handle = spawn_with_audio(Region::Ntsc, false, Some(ring.clone()));
    handle.tx.send(Command::LoadRom(tiny_rom(), "t".to_string())).unwrap();

    // Let it run a bit to fill the ring.
    std::thread::sleep(Duration::from_millis(100));

    // Pause and clear the ring.
    handle.tx.send(Command::Pause(true)).unwrap();
    // The thread may be finishing the frame in flight (back-pressure wait + push); let it
    // reach the paused state before clearing, otherwise that last push races with the clear.
    std::thread::sleep(Duration::from_millis(60));
    {
        let mut r = ring.lock().unwrap();
        r.clear();
    }

    // Wait 200ms: nothing should be pushed while paused.
    std::thread::sleep(Duration::from_millis(200));
    let len = ring.lock().unwrap().len();
    assert_eq!(len, 0, "ring should be empty while paused, got {}", len);

    handle.quit();
}
