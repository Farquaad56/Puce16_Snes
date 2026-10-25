use puce16_core::ppu::{Ppu, DotEvents};
use puce16_core::region::Region;

#[test]
fn e03_08_power_on() {
    let mut p = Ppu::new(Region::Ntsc);
    assert_eq!(p.v(), 0);
    assert_eq!(p.dot(), 0);
    assert!(!p.field());
    assert_eq!(p.dot_clocks(), 4);
    assert_eq!(p.line_dots(), 340);
    assert_eq!(p.last_line(), 261);
    assert_eq!(p.vblank_start(), 225);
    assert!(!p.in_vblank());
    assert!(p.in_hblank()); // dot 0 < 1

    let p_pal = Ppu::new(Region::Pal);
    assert_eq!(p_pal.last_line(), 311);
}

#[test]
fn e03_08_dot_clocks_walk() {
    let mut p = Ppu::new(Region::Ntsc);
    for _ in 0..323 {
        p.step_dot();
    }
    assert_eq!(p.dot(), 323);
    assert_eq!(p.dot_clocks(), 5);

    p.step_dot(); // now at dot 324
    assert_eq!(p.dot_clocks(), 5);

    p.step_dot(); // now at dot 325
    assert_eq!(p.dot_clocks(), 4);

    for _ in 0..2 {
        p.step_dot();
    }
    assert_eq!(p.dot(), 327);
    assert_eq!(p.dot_clocks(), 5);

    for _ in 0..2 {
        p.step_dot();
    }
    assert_eq!(p.dot(), 329);
    assert_eq!(p.dot_clocks(), 4);
}

#[test]
fn e03_08_line_sum() {
    let mut p = Ppu::new(Region::Ntsc);
    let mut sum = 0u32;
    for _ in 0..340 {
        sum += p.dot_clocks();
        p.step_dot();
    }
    assert_eq!(sum, 1364);
    assert_eq!(p.v(), 1);
    assert_eq!(p.dot(), 0);
}

#[test]
fn e03_08_events_line0() {
    let mut p = Ppu::new(Region::Ntsc);

    // First step: dot=0, v=0 -> VBLANK_END | LINE_START = 17
    let ev = p.step_dot();
    assert_eq!(ev.0, 17);

    // Second step: dot=1 -> HBLANK_END = 4
    let ev = p.step_dot();
    assert_eq!(ev.0, 4);

    // Steps 2..=273: no events
    for _ in 0..272 {
        let ev = p.step_dot();
        assert_eq!(ev.0, 0);
    }

    // Step at dot=274 -> HBLANK_START = 2
    let ev = p.step_dot();
    assert_eq!(ev.0, 2);
}

#[test]
fn e03_08_vblank_event() {
    let mut p = Ppu::new(Region::Ntsc);

    // Advance to v=225 (vblank start)
    for _ in 0..225 * 340 {
        let ev = p.step_dot();
        assert!(!ev.has(DotEvents::VBLANK_START));
    }

    assert_eq!(p.v(), 225);
    assert_eq!(p.dot(), 0);
    assert!(p.in_vblank());

    // Next step triggers VBLANK_START | LINE_START = 9
    let ev = p.step_dot();
    assert_eq!(ev.0, 9);
}

#[test]
fn e03_08_frame_ntsc_field0() {
    let mut p = Ppu::new(Region::Ntsc);
    let mut sum = 0u32;

    for _ in 0..262 * 340 {
        sum += p.dot_clocks();
        p.step_dot();
    }

    assert_eq!(sum, 357368);
    assert_eq!(p.v(), 0);
    assert_eq!(p.dot(), 0);
    assert!(p.field());
}

#[test]
fn e03_08_frame_ntsc_field1_short() {
    let mut p = Ppu::new(Region::Ntsc);

    // First frame (field 0)
    for _ in 0..262 * 340 {
        p.step_dot();
    }

    // Second frame (field 1) - line 240 is short
    let mut sum = 0u32;
    for _ in 0..262 * 340 {
        sum += p.dot_clocks();
        p.step_dot();
    }

    assert_eq!(sum, 357364); // 1360 less due to short line
    assert_eq!(p.v(), 0);
    assert!(!p.field());
}

#[test]
fn e03_08_frame_pal() {
    let mut p = Ppu::new(Region::Pal);

    // Frame 0 (field false)
    let mut sum0 = 0u32;
    for _ in 0..312 * 340 {
        sum0 += p.dot_clocks();
        p.step_dot();
    }
    assert_eq!(sum0, 425568);

    // Frame 1 (field true) - no interlace so same length
    let mut sum1 = 0u32;
    for _ in 0..312 * 340 {
        sum1 += p.dot_clocks();
        p.step_dot();
    }
    assert_eq!(sum1, 425568);
}

#[test]
fn e03_08_vblank_end_event() {
    let mut p = Ppu::new(Region::Ntsc);

    // Advance through one full frame
    for _ in 0..262 * 340 {
        p.step_dot();
    }

    // Next step: v=0 again -> VBLANK_END | LINE_START = 17
    let ev = p.step_dot();
    assert_eq!(ev.0, 17);
}