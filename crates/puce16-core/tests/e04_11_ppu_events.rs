use puce16_core::region::Region;
use puce16_core::ppu::Ppu;

fn at(p: &mut Ppu, v: u32, dot: u32) {
    for _ in 0..(v * 340 + dot) {
        p.step_dot();
    }
}

#[test]
fn e04_11_setini_bits() {
    let mut p = Ppu::new(Region::Ntsc);
    // write(0x33, 0x4A): bits 1,2,3,6 set -> obj_interlace, overscan, pseudo_hires, extbg
    p.write(0x33, 0x4A);
    assert!(p.obj_interlace());
    assert!(p.pseudo_hires());
    assert!(p.extbg());
    assert!(!p.interlace());
    assert!(!p.overscan());

    // write(0x33, 0x87): bits 0,1,2 set -> interlace, obj_interlace, overscan; bit7 ignored
    p.write(0x33, 0x87);
    assert!(p.interlace());
    assert!(p.obj_interlace());
    assert!(p.overscan());
    assert!(!p.pseudo_hires());
    assert!(!p.extbg());

    // write(0x33, 0x00): all clear
    p.write(0x33, 0x00);
    assert!(!p.interlace());
    assert!(!p.obj_interlace());
    assert!(!p.overscan());
    assert!(!p.pseudo_hires());
    assert!(!p.extbg());
}

#[test]
fn e04_11_oam_reload_at_vblank() {
    let mut p = Ppu::new(Region::Ntsc);
    // Forced blank: set OAM address to 7 (internal 14)
    p.write(0x02, 0x07);
    p.write(0x03, 0x00);
    assert_eq!(p.oam_internal_addr(), 14);

    // Write 3 bytes -> internal becomes 17
    p.write(0x04, 1);
    p.write(0x04, 1);
    p.write(0x04, 1);
    assert_eq!(p.oam_internal_addr(), 17);

    // Turn display on (clear forced blank)
    p.write(0x00, 0x0F);

    // Advance to VBlank start position (v=225, dot=0) - event hasn't fired yet
    at(&mut p, 225, 0);
    assert_eq!(p.oam_internal_addr(), 17);

    // One more step: now we process events at (225, 0), OAM reloads to 14
    p.step_dot();
    assert_eq!(p.oam_internal_addr(), 14);
}

#[test]
fn e04_11_oam_reload_skipped_in_forced_blank() {
    let mut p = Ppu::new(Region::Ntsc);
    // Forced blank (default): set OAM address to 7 (internal 14)
    p.write(0x02, 0x07);
    assert_eq!(p.oam_internal_addr(), 14);

    // Write 3 bytes -> internal becomes 17
    p.write(0x04, 1);
    p.write(0x04, 1);
    p.write(0x04, 1);
    assert_eq!(p.oam_internal_addr(), 17);

    // Advance to VBlank start (v=225) - still forced blank, no reload
    at(&mut p, 225, 1);
    assert_eq!(p.oam_internal_addr(), 17);
}

#[test]
fn e04_11_oam_write_in_vblank() {
    let mut p = Ppu::new(Region::Ntsc);
    // Turn display on
    p.write(0x00, 0x0F);

    // Advance to VBlank (v=226)
    at(&mut p, 226, 0);

    // Write OAM data in VBlank - should be allowed
    p.write(0x02, 0x00);
    p.write(0x04, 0xAA);
    p.write(0x04, 0xBB);

    assert_eq!(p.oam()[0], 0xAA);
    assert_eq!(p.oam()[1], 0xBB);
}

#[test]
fn e04_11_oam_reload_overscan() {
    let mut p = Ppu::new(Region::Ntsc);
    // Enable overscan -> VBlank starts at v=240
    p.write(0x33, 0x04);

    // Forced blank: set OAM address to 7 (internal 14)
    p.write(0x02, 0x07);
    assert_eq!(p.oam_internal_addr(), 14);

    // Write 3 bytes -> internal becomes 17
    p.write(0x04, 1);
    p.write(0x04, 1);
    p.write(0x04, 1);
    assert_eq!(p.oam_internal_addr(), 17);

    // Turn display on
    p.write(0x00, 0x0F);

    // Advance to v=225 (not VBlank yet with overscan) - no reload
    at(&mut p, 225, 1);
    assert_eq!(p.oam_internal_addr(), 17);

    // Advance to v=240 (VBlank start with overscan) -> reload happens
    for _ in 0..(15 * 340) {
        p.step_dot();
    }
    assert_eq!(p.oam_internal_addr(), 14);
}

#[test]
fn e04_11_flags_cleared_at_frame_start() {
    let mut p = Ppu::new(Region::Ntsc);
    // Set both over flags
    p.set_range_over(true);
    p.set_time_over(true);

    // One step at v=0, dot=0 -> frame start clears them
    p.step_dot();

    // Read STAT7E: bits 7 and 6 should be clear (only bit 0 set = 0x01)
    let stat = p.read(0x3E, 0, false);
    assert_eq!(stat & 0xC0, 0);
    assert_eq!(stat, 0x01);
}

#[test]
fn e04_11_flags_persist_mid_frame() {
    let mut p = Ppu::new(Region::Ntsc);

    // Advance to v=5, dot=0
    at(&mut p, 5, 0);

    // Set both over flags mid-frame
    p.set_range_over(true);
    p.set_time_over(true);

    // Advance 256 lines (to v=261) - flags should persist
    for _ in 0..(256 * 340) {
        p.step_dot();
    }
    let stat = p.read(0x3E, 0, false);
    assert_eq!(stat & 0xC0, 0xC0);

    // Advance to v=0 of next frame (but not yet at dot=0 event)
    for _ in 0..340 {
        p.step_dot();
    }
    let stat = p.read(0x3E, 0, false);
    assert_eq!(stat & 0xC0, 0xC0);

    // One more step: now at v=0, dot=0 -> frame start clears flags
    p.step_dot();
    let stat = p.read(0x3E, 0, false);
    assert_eq!(stat & 0xC0, 0);
}

#[test]
fn e04_11_hash() {
    use puce16_core::hash::Fnv64;

    let mut h1 = Fnv64::new();
    let p1 = Ppu::new(Region::Ntsc);
    p1.hash_into(&mut h1);
    let base = h1.finish();

    // obj_interlace (bit 1)
    let mut h2 = Fnv64::new();
    let mut p2 = Ppu::new(Region::Ntsc);
    p2.write(0x33, 0x02);
    p2.hash_into(&mut h2);
    assert_ne!(base, h2.finish());

    // pseudo_hires (bit 3)
    let mut h3 = Fnv64::new();
    let mut p3 = Ppu::new(Region::Ntsc);
    p3.write(0x33, 0x08);
    p3.hash_into(&mut h3);
    assert_ne!(base, h3.finish());

    // extbg (bit 6)
    let mut h4 = Fnv64::new();
    let mut p4 = Ppu::new(Region::Ntsc);
    p4.write(0x33, 0x40);
    p4.hash_into(&mut h4);
    assert_ne!(base, h4.finish());
}
