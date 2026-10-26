//! E03-25: timing/mod.rs - auto-read joypad and HDMA triggers.

#[path = "common/rig.rs"]
mod rig;

use puce16_core::region::Region;
use puce16_core::timing::{AUTOREAD_CLOCKS, AUTOREAD_START_DOT};
use rig::Rig;

#[test]
fn e03_25_autoread_updates_at_end() {
    let mut rig = Rig::new(Region::Ntsc);
    rig.mmio.nmitimen = 1;
    rig.joy.set_buttons(0, 0x8001);
    rig.joy.set_buttons(1, 0x0F00);

    // Run to just before auto-read starts (vblank_start=225, dot=33)
    rig.run_until(225, 33);
    assert_eq!(rig.mmio.joy, [0; 4]);
    assert!(rig.timing.autoread_busy(rig.t));

    let t0 = rig.t;

    // Step until joy registers are updated
    loop {
        rig.step();
        if rig.mmio.joy[0] != 0 {
            break;
        }
    }

    let dt = rig.t - t0;
    assert!(dt >= AUTOREAD_CLOCKS && dt < AUTOREAD_CLOCKS + 5, "dt={}", dt);
    assert_eq!(rig.mmio.joy, [0x8001, 0x0F00, 0, 0]);
    assert!(!rig.timing.autoread_busy(rig.t));
}

#[test]
fn e03_25_autoread_busy_window() {
    let mut rig = Rig::new(Region::Ntsc);
    rig.mmio.nmitimen = 1;

    // Run to just before auto-read starts
    rig.run_until(225, AUTOREAD_START_DOT - 1);
    assert!(!rig.timing.autoread_busy(rig.t));

    // Step into the start dot
    let t0 = rig.t + rig.ppu.dot_clocks() as u64;
    rig.step();
    assert!(rig.timing.autoread_busy(t0), "busy at start");
    assert!(rig.timing.autoread_busy(t0 + AUTOREAD_CLOCKS - 1), "busy near end");

    // Step until auto-read completes (joy registers updated)
    loop {
        rig.step();
        if !rig.timing.autoread_busy(rig.t) {
            break;
        }
    }
    assert!(!rig.timing.autoread_busy(rig.t), "not busy after completion");
}

#[test]
fn e03_25_shift_reset_after_autoread() {
    let mut rig = Rig::new(Region::Ntsc);
    rig.mmio.nmitimen = 1;
    rig.joy.set_buttons(0, 0x8001);

    // Run through auto-read completion
    loop {
        rig.step();
        if rig.mmio.joy[0] != 0 {
            break;
        }
    }

    // After autoread_values() is called, shift registers are reset to 0xFFFF
    assert_eq!(rig.joy.read_port(0), 1);
}

#[test]
fn e03_25_autoread_disabled() {
    let mut rig = Rig::new(Region::Ntsc);
    // nmitimen=0 means auto-read disabled
    rig.run_until(230, 0);

    for _ in 0..2000 {
        rig.step();
        assert!(!rig.timing.autoread_busy(rig.t));
    }
    assert_eq!(rig.mmio.joy, [0; 4]);
}

#[test]
fn e03_25_autoread_once_per_frame() {
    let mut rig = Rig::new(Region::Ntsc);
    rig.mmio.nmitimen = 1;
    rig.joy.set_buttons(0, 0x8001);

    // Run through first auto-read
    loop {
        rig.step();
        if rig.mmio.joy[0] != 0 {
            break;
        }
    }
    assert_eq!(rig.mmio.joy[0], 0x8001);

    // Change buttons mid-frame - should not update until next frame's auto-read
    rig.joy.set_buttons(0, 0x0001);
    let mut changed = false;
    for _ in 0..5000 {
        if rig.mmio.joy[0] == 0x0001 {
            changed = true;
            break;
        }
        rig.step();
    }
    assert!(!changed, "joy should not change mid-frame");

    // Continue to next frame's auto-read
    loop {
        rig.step();
        if rig.mmio.joy[0] == 0x0001 {
            break;
        }
    }
    assert_eq!(rig.mmio.joy[0], 0x0001);
}

#[test]
fn e03_25_hdma_off() {
    let mut rig = Rig::new(Region::Ntsc);
    // hdmaen=0 means no HDMA triggers
    for _ in 0..4000 {
        let out = rig.step();
        assert!(!out.hdma_init, "no hdma_init when disabled");
        assert!(!out.hdma_run, "no hdma_run when disabled");
    }
}

#[test]
fn e03_25_hdma_init_once() {
    let mut rig = Rig::new(Region::Ntsc);
    rig.mmio.hdmaen = 1;

    let mut init_count = 0;
    let mut init_at_pos = None;
    for _ in 0..100000 {
        let out = rig.step();
        if out.hdma_init {
            init_count += 1;
            init_at_pos = Some(rig.pos());
        }
        // Stop after first frame (vblank_start)
        if rig.ppu.v() == 225 && rig.ppu.dot() == 0 {
            break;
        }
    }
    assert_eq!(init_count, 1);
    assert_eq!(init_at_pos, Some((0, 3)));
}

#[test]
fn e03_25_hdma_run_count() {
    let mut rig = Rig::new(Region::Ntsc);
    rig.mmio.hdmaen = 1;

    let mut run_count = 0;
    for _ in 0..100000 {
        let out = rig.step();
        if out.hdma_run {
            run_count += 1;
        }
        // Stop after first frame (vblank_start)
        if rig.ppu.v() == 225 && rig.ppu.dot() == 0 {
            break;
        }
    }
    assert_eq!(run_count, 225);

    // With overscan (vblank_start=240), should be 240 runs
    let mut rig2 = Rig::new(Region::Ntsc);
    rig2.mmio.hdmaen = 1;
    rig2.ppu.write(0x33, 0x04); // SETINI bit2=overscan

    let mut run_count2 = 0;
    for _ in 0..100000 {
        let out = rig2.step();
        if out.hdma_run {
            run_count2 += 1;
        }
        if rig2.ppu.v() == 240 && rig2.ppu.dot() == 0 {
            break;
        }
    }
    assert_eq!(run_count2, 240);
}

#[test]
fn e03_25_hdma_run_position() {
    let mut rig = Rig::new(Region::Ntsc);
    rig.mmio.hdmaen = 1;

    let mut first_run_pos = None;
    for _ in 0..100000 {
        let out = rig.step();
        if out.hdma_run && first_run_pos.is_none() {
            first_run_pos = Some(rig.pos());
        }
        // Check no runs after vblank_start
        if rig.ppu.v() >= 225 && out.hdma_run {
            panic!("hdma_run should not occur at v>=225");
        }
        if rig.ppu.v() == 225 && rig.ppu.dot() == 0 {
            break;
        }
    }
    assert_eq!(first_run_pos, Some((0, 276)));
}

#[test]
fn e03_25_hash_includes_autoread() {
    let mut rig = Rig::new(Region::Ntsc);
    rig.mmio.nmitimen = 1;

    // Hash before auto-read starts
    let mut h1 = puce16_core::hash::Fnv64::default();
    rig.timing.hash_into(&mut h1);
    let hash_before = h1.finish();

    // Run to during auto-read
    rig.run_until(225, AUTOREAD_START_DOT + 10);
    assert!(rig.timing.autoread_busy(rig.t));

    let mut h2 = puce16_core::hash::Fnv64::default();
    rig.timing.hash_into(&mut h2);
    let hash_during = h2.finish();

    // Hash after auto-read completes
    loop {
        rig.step();
        if !rig.timing.autoread_busy(rig.t) {
            break;
        }
    }

    let mut h3 = puce16_core::hash::Fnv64::default();
    rig.timing.hash_into(&mut h3);
    let hash_after = h3.finish();

    assert_ne!(hash_before, hash_during, "hash should differ during autoread");
    assert_ne!(hash_during, hash_after, "hash should differ after autoread");
}
