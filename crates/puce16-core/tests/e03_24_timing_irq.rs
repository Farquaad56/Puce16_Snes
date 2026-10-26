//! E03-24: Timing IRQ H/V (point 4 of on_dot).

#[path = "common/rig.rs"]
mod rig;

use puce16_core::region::Region;
use rig::Rig;

#[test]
fn e03_24_mode1() {
    let mut rig = Rig::new(Region::Ntsc);
    rig.mmio.nmitimen = 0x10; // mode 1 (H only)
    rig.mmio.htime = 100;

    // Step until just before the trigger point.
    rig.run_until(0, 100);
    assert!(!rig.mmio.timeup_flag, "timeup should not be set at dot 100");
    assert!(!rig.mmio.irq_line, "irq_line should not be set at dot 100");

    // The step that arrives at (0, 101) triggers the IRQ.
    let out = rig.step();
    assert!(rig.mmio.timeup_flag, "timeup should be set after stepping to dot 101");
    assert!(rig.mmio.irq_line, "irq_line should be set after stepping to dot 101");
}

#[test]
fn e03_24_mode1_every_line() {
    let mut rig = Rig::new(Region::Ntsc);
    rig.mmio.nmitimen = 0x10; // mode 1 (H only)
    rig.mmio.htime = 100;

    // First trigger on line 0.
    rig.run_until(0, 101);
    assert!(rig.mmio.timeup_flag, "first trigger should occur");

    // Clear and verify no re-trigger until next line's dot 101.
    rig.mmio.timeup_flag = false;
    rig.mmio.irq_line = false;
    rig.run_until(1, 100);
    assert!(!rig.mmio.timeup_flag, "should not trigger at (1,100)");

    // Trigger again on line 1.
    let out = rig.step();
    assert!(rig.mmio.timeup_flag, "should trigger again at (1,101)");
}

#[test]
fn e03_24_mode2() {
    let mut rig = Rig::new(Region::Ntsc);
    rig.mmio.nmitimen = 0x20; // mode 2 (V only)
    rig.mmio.vtime = 10;

    // No trigger before line 10.
    for v in 0..10 {
        rig.run_until(v, 339);
        assert!(!rig.mmio.timeup_flag, "should not trigger on line {}", v);
    }

    // Trigger at (10, 1).
    rig.run_until(10, 0);
    let out = rig.step();
    assert!(rig.mmio.timeup_flag, "should trigger at (10,1)");
    assert!(rig.mmio.irq_line, "irq_line should be set");

    // No re-trigger on line 11.
    rig.mmio.timeup_flag = false;
    rig.mmio.irq_line = false;
    rig.run_until(11, 339);
    assert!(!rig.mmio.timeup_flag, "should not trigger again on line 11");
}

#[test]
fn e03_24_mode3() {
    let mut rig = Rig::new(Region::Ntsc);
    rig.mmio.nmitimen = 0x30; // mode 3 (H and V)
    rig.mmio.htime = 50;
    rig.mmio.vtime = 10;

    // No trigger before line 10.
    for v in 0..10 {
        rig.run_until(v, 339);
        assert!(!rig.mmio.timeup_flag, "should not trigger on line {}", v);
    }

    // On line 10, no trigger until dot 51.
    rig.run_until(10, 50);
    assert!(!rig.mmio.timeup_flag, "should not trigger at (10,50)");

    let out = rig.step();
    assert!(rig.mmio.timeup_flag, "should trigger at (10,51)");

    // No re-trigger on line 11.
    rig.mmio.timeup_flag = false;
    rig.mmio.irq_line = false;
    rig.run_until(11, 339);
    assert!(!rig.mmio.timeup_flag, "should not trigger again on line 11");
}

#[test]
fn e03_24_mode0_never() {
    let mut rig = Rig::new(Region::Ntsc);
    rig.mmio.nmitimen = 0x00; // mode 0 (disabled)
    rig.mmio.htime = 100;

    // Run two full lines.
    for v in 0..2 {
        rig.run_until(v, 339);
        assert!(!rig.mmio.timeup_flag, "mode 0 should never trigger on line {}", v);
    }
}

#[test]
fn e03_24_htime_too_big() {
    let mut rig = Rig::new(Region::Ntsc);
    rig.mmio.nmitimen = 0x10; // mode 1 (H only)

    // htime=340: never triggers (dot 341 doesn't exist on normal lines).
    rig.mmio.htime = 340;
    for v in 0..262 {
        rig.run_until(v, 339);
        assert!(!rig.mmio.timeup_flag, "htime=340 should never trigger");
    }

    // Reset and try htime=338: triggers at dot 339.
    let mut rig2 = Rig::new(Region::Ntsc);
    rig2.mmio.nmitimen = 0x10;
    rig2.mmio.htime = 338;
    rig2.run_until(0, 338);
    assert!(!rig2.mmio.timeup_flag, "should not trigger at dot 338");
    let out = rig2.step();
    assert!(rig2.mmio.timeup_flag, "htime=338 should trigger at dot 339");
}

#[test]
fn e03_24_vtime_too_big() {
    // NTSC: vtime=262 > last_line (261), never triggers.
    let mut rig_ntsc = Rig::new(Region::Ntsc);
    rig_ntsc.mmio.nmitimen = 0x20; // mode 2 (V only)
    rig_ntsc.mmio.vtime = 262;

    for v in 0..262 {
        rig_ntsc.run_until(v, 339);
        assert!(!rig_ntsc.mmio.timeup_flag, "NTSC vtime=262 should never trigger");
    }

    // PAL: last_line is higher (311), so vtime=262 triggers at line 262.
    let mut rig_pal = Rig::new(Region::Pal);
    rig_pal.mmio.nmitimen = 0x20;
    rig_pal.mmio.vtime = 262;

    for v in 0..262 {
        rig_pal.run_until(v, 339);
        assert!(!rig_pal.mmio.timeup_flag, "PAL should not trigger before line 262");
    }

    // Trigger at (262, 1).
    rig_pal.run_until(262, 0);
    let out = rig_pal.step();
    assert!(rig_pal.mmio.timeup_flag, "PAL vtime=262 should trigger at line 262");
}

#[test]
fn e03_24_clear_by_4211() {
    let mut rig = Rig::new(Region::Ntsc);
    rig.mmio.nmitimen = 0x10; // mode 1 (H only)
    rig.mmio.htime = 100;

    // Trigger the IRQ.
    rig.run_until(0, 101);
    assert!(rig.mmio.timeup_flag, "IRQ should be triggered");

    // Read $4211 to clear it.
    let val = rig.mmio.read(0x4211, 0, rig.t, None);
    assert_eq!(val & 0x80, 0x80, "$4211 read should have bit 7 set");
    assert!(!rig.mmio.timeup_flag, "timeup_flag should be cleared after reading $4211");
    assert!(!rig.mmio.irq_line, "irq_line should be cleared after reading $4211");
}

#[test]
fn e03_24_clear_by_nmitimen() {
    let mut rig = Rig::new(Region::Ntsc);
    rig.mmio.nmitimen = 0x10; // mode 1 (H only)
    rig.mmio.htime = 100;

    // Trigger the IRQ.
    rig.run_until(0, 101);
    assert!(rig.mmio.timeup_flag, "IRQ should be triggered");

    // Write NMITIMEN with V/H=00 to clear.
    rig.mmio.write(0x4200, 0x00, rig.t);
    assert!(!rig.mmio.timeup_flag, "timeup_flag should be cleared by writing NMITIMEN=0");
    assert!(!rig.mmio.irq_line, "irq_line should be cleared by writing NMITIMEN=0");

    // Re-trigger and verify that writing V/H!=00 does NOT clear.
    rig.mmio.nmitimen = 0x10;
    rig.run_until(1, 101);
    assert!(rig.mmio.timeup_flag, "IRQ should be triggered again");

    rig.mmio.write(0x4200, 0x10, rig.t);
    assert!(rig.mmio.timeup_flag, "timeup_flag should NOT be cleared by writing NMITIMEN=0x10");
}

#[test]
fn e03_24_vtime_zero_mode3() {
    let mut rig = Rig::new(Region::Ntsc);
    rig.mmio.nmitimen = 0x30; // mode 3 (H and V)
    rig.mmio.htime = 0;
    rig.mmio.vtime = 0;

    // Should trigger at (0, 1).
    let out = rig.step();
    assert!(rig.mmio.timeup_flag, "mode 3 with vtime=0, htime=0 should trigger at (0,1)");
}
