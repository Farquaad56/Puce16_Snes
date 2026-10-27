// E05-14 - ppu: pick_layers (priority order), windows per screen, pick_screen

use puce16_core::ppu::{Ppu, Pix, Pick};
use puce16_core::region::Region;
use puce16_core::ppu::bg::{pick_layers, priority_order, LAYER_BACKDROP};

// Helper: write a 16-bit word to VRAM at the given address.
fn put_vram(p: &mut Ppu, addr: u16, w: u16) {
    p.write(0x15, 0x80); // VMAIN = VRAM
    p.write(0x16, (addr & 0xFF) as u8);
    p.write(0x17, (addr >> 8) as u8);
    p.write(0x18, (w & 0xFF) as u8);
    p.write(0x19, (w >> 8) as u8);
}

// Helper: write a CGRAM entry.
fn put_cgram(p: &mut Ppu, i: u8, w: u16) {
    p.write(0x21, i);
    p.write(0x22, (w & 0xFF) as u8);
    p.write(0x22, (w >> 8) as u8);
}

// Helper: set scroll register.
fn set_scroll(p: &mut Ppu, reg: u8, v: u16) {
    p.write(reg, (v & 0xFF) as u8);
    p.write(reg, (v >> 8) as u8);
}

// Helper: load OAM with a single sprite at the given position.
fn spr(p: &mut Ppu, i: usize, x: i16, y: u8, tile: u8, attr: u8) {
    // OAMADD is a WORD address: sprite i starts at byte 4*i = word 2*i.
    let w = i * 2;
    p.write(0x02, (w & 0xFF) as u8);
    p.write(0x03, ((w >> 8) & 0x01) as u8);
    let x9 = if x < 0 { (x + 512) as u16 } else { x as u16 };
    p.write(0x04, (x9 & 0xFF) as u8);
    p.write(0x04, y);
    p.write(0x04, tile);
    p.write(0x04, attr);

    // High table: byte 0x200 + i/4, 2 bits per sprite (bit 0 = X bit 8, bit 1 = large).
    // High table writes are direct; the word address of byte b is 0x100 + b/2.
    let b = i / 4;
    let hw = 0x100 + b / 2;
    p.write(0x02, (hw & 0xFF) as u8);
    p.write(0x03, ((hw >> 8) & 0x01) as u8);
    if b % 2 == 1 {
        p.write(0x04, 0); // skip the even byte
    }
    let x8_bit = if x9 >= 256 { 1u8 } else { 0u8 };
    p.write(0x04, x8_bit << ((i % 4) * 2));
}

#[test]
fn e05_14_pick_priority_order_mode1() {
    // Mode 1 priority order: S3 1H 2H S2 1L 2L S1 3H S0 3L
    let order = priority_order(1, false, false);

    // (a) BG1 prio 1 + BG2 prio 1 + OBJ prio 2 -> 1H is before S2
    {
        let layers: [Option<Pix>; 5] = [
            Some(Pix { rgb: 1, prio: 1, obj_pal: 0 }), // BG1
            Some(Pix { rgb: 2, prio: 1, obj_pal: 0 }), // BG2
            None, None,
            Some(Pix { rgb: 3, prio: 2, obj_pal: 0 }), // OBJ
        ];
        let pick = pick_layers(order, &layers, 0x1F, [false; 5], 0x0BAD);
        assert_eq!(pick.rgb, 1);
        assert_eq!(pick.layer, 0);
    }

    // (b) BG1 prio 0 + OBJ prio 2 -> S2 is before 1L
    {
        let layers: [Option<Pix>; 5] = [
            Some(Pix { rgb: 1, prio: 0, obj_pal: 0 }), // BG1
            None, None, None,
            Some(Pix { rgb: 3, prio: 2, obj_pal: 0 }), // OBJ
        ];
        let pick = pick_layers(order, &layers, 0x1F, [false; 5], 0x0BAD);
        assert_eq!(pick.rgb, 3);
        assert_eq!(pick.layer, 4);
    }

    // (c) OBJ prio 0 + BG3 prio 0 -> S0 is before 3L
    {
        let layers: [Option<Pix>; 5] = [
            None, None,
            Some(Pix { rgb: 4, prio: 0, obj_pal: 0 }), // BG3
            None,
            Some(Pix { rgb: 3, prio: 0, obj_pal: 0 }), // OBJ
        ];
        let pick = pick_layers(order, &layers, 0x1F, [false; 5], 0x0BAD);
        assert_eq!(pick.rgb, 3);
    }

    // (d) OBJ prio 0 + BG3 prio 1 -> 3H is before S0
    {
        let layers: [Option<Pix>; 5] = [
            None, None,
            Some(Pix { rgb: 4, prio: 1, obj_pal: 0 }), // BG3
            None,
            Some(Pix { rgb: 3, prio: 0, obj_pal: 0 }), // OBJ
        ];
        let pick = pick_layers(order, &layers, 0x1F, [false; 5], 0x0BAD);
        assert_eq!(pick.rgb, 4);
        assert_eq!(pick.layer, 2);
    }

    // (e) BG3 prio 1 + OBJ prio 1 -> S1 is before 3H
    {
        let layers: [Option<Pix>; 5] = [
            None, None,
            Some(Pix { rgb: 4, prio: 1, obj_pal: 0 }), // BG3
            None,
            Some(Pix { rgb: 3, prio: 1, obj_pal: 0 }), // OBJ
        ];
        let pick = pick_layers(order, &layers, 0x1F, [false; 5], 0x0BAD);
        assert_eq!(pick.rgb, 3);
        assert_eq!(pick.layer, 4);
    }
}

#[test]
fn e05_14_pick_none_is_backdrop() {
    let order = priority_order(1, false, false);

    // No layers present -> backdrop
    {
        let layers: [Option<Pix>; 5] = [None; 5];
        let pick = pick_layers(order, &layers, 0x1F, [false; 5], 0x0BAD);
        assert_eq!(pick.rgb, 0x0BAD);
        assert_eq!(pick.layer, LAYER_BACKDROP);
    }

    // All layers present but en=0 -> backdrop
    {
        let layers: [Option<Pix>; 5] = [
            Some(Pix { rgb: 1, prio: 1, obj_pal: 0 }),
            Some(Pix { rgb: 2, prio: 1, obj_pal: 0 }),
            None, None,
            Some(Pix { rgb: 3, prio: 0, obj_pal: 0 }),
        ];
        let pick = pick_layers(order, &layers, 0x00, [false; 5], 0x0BAD);
        assert_eq!(pick.rgb, 0x0BAD);
    }
}

#[test]
fn e05_14_pick_level_must_match() {
    let order = priority_order(1, false, false);

    // BG1 with prio 1 is found at entry (0,1)
    {
        let layers: [Option<Pix>; 5] = [
            Some(Pix { rgb: 1, prio: 1, obj_pal: 0 }),
            None, None, None, None,
        ];
        let pick = pick_layers(order, &layers, 0x1F, [false; 5], 0x0BAD);
        assert_eq!(pick.rgb, 1);
    }

    // BG1 with invalid prio 5 is never chosen -> backdrop
    {
        let layers: [Option<Pix>; 5] = [
            Some(Pix { rgb: 1, prio: 5, obj_pal: 0 }),
            None, None, None, None,
        ];
        let pick = pick_layers(order, &layers, 0x1F, [false; 5], 0x0BAD);
        assert_eq!(pick.rgb, 0x0BAD);
    }
}

#[test]
fn e05_14_pick_en_mask() {
    let order = priority_order(1, false, false);

    // BG1 and BG2 both prio 1; en=0b11110 (BG1 disabled) -> BG2 wins
    {
        let layers: [Option<Pix>; 5] = [
            Some(Pix { rgb: 1, prio: 1, obj_pal: 0 }),
            Some(Pix { rgb: 2, prio: 1, obj_pal: 0 }),
            None, None, None,
        ];
        let pick = pick_layers(order, &layers, 0b11110, [false; 5], 0x0BAD);
        assert_eq!(pick.rgb, 2);
        assert_eq!(pick.layer, 1);
    }

    // en=0b11101 (BG2 disabled) -> BG1 wins
    {
        let layers: [Option<Pix>; 5] = [
            Some(Pix { rgb: 1, prio: 1, obj_pal: 0 }),
            Some(Pix { rgb: 2, prio: 1, obj_pal: 0 }),
            None, None, None,
        ];
        let pick = pick_layers(order, &layers, 0b11101, [false; 5], 0x0BAD);
        assert_eq!(pick.rgb, 1);
        assert_eq!(pick.layer, 0);
    }
}

#[test]
fn e05_14_pick_hidden() {
    let order = priority_order(1, false, false);

    // BG1 and BG2 both prio 1; BG1 hidden -> BG2 wins
    {
        let layers: [Option<Pix>; 5] = [
            Some(Pix { rgb: 1, prio: 1, obj_pal: 0 }),
            Some(Pix { rgb: 2, prio: 1, obj_pal: 0 }),
            None, None, None,
        ];
        let hidden = [true, false, false, false, false];
        let pick = pick_layers(order, &layers, 0x1F, hidden, 0x0BAD);
        assert_eq!(pick.rgb, 2);
    }

    // All layers hidden -> backdrop
    {
        let layers: [Option<Pix>; 5] = [
            Some(Pix { rgb: 1, prio: 1, obj_pal: 0 }),
            Some(Pix { rgb: 2, prio: 1, obj_pal: 0 }),
            None, None, None,
        ];
        let hidden = [true; 5];
        let pick = pick_layers(order, &layers, 0x1F, hidden, 0x0BAD);
        assert_eq!(pick.rgb, 0x0BAD);
    }
}

#[test]
fn e05_14_pick_mode1_bg3_priority() {
    // With bg3_priority=true, BG3 prio 1 is at the front of the order
    let order_bg3 = priority_order(1, true, false);
    let order_normal = priority_order(1, false, false);

    let layers: [Option<Pix>; 5] = [
        None, None,
        Some(Pix { rgb: 4, prio: 1, obj_pal: 0 }), // BG3
        None,
        Some(Pix { rgb: 3, prio: 3, obj_pal: 0 }), // OBJ
    ];

    // With bg3_priority=true, BG3 wins (entry (2,1) is first)
    let pick_bg3 = pick_layers(order_bg3, &layers, 0x1F, [false; 5], 0x0BAD);
    assert_eq!(pick_bg3.rgb, 4);

    // With normal order, OBJ wins (S3 is before 3H)
    let pick_normal = pick_layers(order_normal, &layers, 0x1F, [false; 5], 0x0BAD);
    assert_eq!(pick_normal.rgb, 3);
}

#[test]
fn e05_14_pick_obj_pal_kept() {
    let order = priority_order(1, false, false);
    let layers: [Option<Pix>; 5] = [
        None, None, None, None,
        Some(Pix { rgb: 7, prio: 3, obj_pal: 5 }), // OBJ with pal 5
    ];
    let pick = pick_layers(order, &layers, 0x1F, [false; 5], 0x0BAD);
    assert_eq!(pick.rgb, 7);
    assert_eq!(pick.layer, 4);
    assert_eq!(pick.obj_pal, 5);
}

#[test]
fn e05_14_pick_screen_bg_and_backdrop() {
    let mut p = Ppu::new(Region::Ntsc);

    // Mode 1, BG1 enabled on main screen
    p.write(0x05, 0x01); // BGMODE = mode 1
    p.write(0x07, 0x04); // BG1SC: map at 0x400, 32x32
    p.write(0x0B, 0x01); // BG1ND: chr at 0x1000

    // Tile 2 data: row 0 has index 1 at pixel 0, row 1 has index 2 at pixel 0
    put_vram(&mut p, 0x1020, 0x0080);
    put_vram(&mut p, 0x1021, 0x8000);

    // Map entries: (0,0) = tile 2 pal 1 prio 0; (1,0) = tile 2 pal 1 prio 1
    put_vram(&mut p, 0x400, 0x0402);
    put_vram(&mut p, 0x401, 0x2402);

    // CGRAM: palette 1 colors and backdrop
    put_cgram(&mut p, 17, 0x1234);
    put_cgram(&mut p, 18, 0x2345);
    put_cgram(&mut p, 0, 0x7C00); // backdrop blue

    // Enable BG1 on main screen
    p.write(0x2C, 0x01); // TM = BG1

    // Pixel (0,0): tile 2 row 0 -> index 1 -> CGRAM[17] = 0x1234
    let pick = p.pick_screen(true, 0, 0);
    assert_eq!(pick.rgb, 0x1234);
    assert_eq!(pick.layer, 0);

    // Pixel (1,0): tile 2 row 0 -> index 0 at pixel 1 -> transparent -> backdrop
    let pick = p.pick_screen(true, 1, 0);
    assert_eq!(pick.rgb, 0x7C00);
    assert_eq!(pick.layer, LAYER_BACKDROP);

    // Pixel (8,0): tile 2 row 0 at map entry (1,0) with prio 1 -> still BG1
    let pick = p.pick_screen(true, 8, 0);
    assert_eq!(pick.rgb, 0x1234);
    assert_eq!(pick.layer, 0);

    // TM=0: no layers enabled -> backdrop
    p.write(0x2C, 0x00);
    let pick = p.pick_screen(true, 0, 0);
    assert_eq!(pick.rgb, 0x7C00);
}

#[test]
fn e05_14_pick_screen_sub() {
    let mut p = Ppu::new(Region::Ntsc);

    // Same setup as above but on sub screen
    p.write(0x05, 0x01); // BGMODE = mode 1
    p.write(0x07, 0x04); // BG1SC: map at 0x400, 32x32
    p.write(0x0B, 0x01); // BG1ND: chr at 0x1000

    put_vram(&mut p, 0x1020, 0x0080);
    put_vram(&mut p, 0x1021, 0x8000);
    put_vram(&mut p, 0x400, 0x0402);
    put_vram(&mut p, 0x401, 0x2402);

    put_cgram(&mut p, 17, 0x1234);
    put_cgram(&mut p, 18, 0x2345);
    put_cgram(&mut p, 0, 0x7C00); // main backdrop

    // Enable BG1 on sub screen only
    p.write(0x2D, 0x01); // TS = BG1

    // Sub screen pixel (0,0): tile 2 row 0 -> index 1 -> CGRAM[17] = 0x1234
    let pick = p.pick_screen(false, 0, 0);
    assert_eq!(pick.rgb, 0x1234);
    assert_eq!(pick.layer, 0);

    // Sub screen pixel (1,0): transparent -> sub backdrop = fixed color (default 0)
    let pick = p.pick_screen(false, 1, 0);
    assert_eq!(pick.rgb, 0x0000);
    assert_eq!(pick.layer, LAYER_BACKDROP);

    // Set fixed color to red (R=10)
    p.write(0x32, 0x2A); // COLUPRI: fixed R=10
    let pick = p.pick_screen(false, 1, 0);
    assert_eq!(pick.rgb, 0x000A);

    // Main screen with TM=0 -> main backdrop
    let pick = p.pick_screen(true, 0, 0);
    assert_eq!(pick.rgb, 0x7C00);
}

#[test]
fn e05_14_pick_screen_window() {
    let mut p = Ppu::new(Region::Ntsc);

    // Same setup as above with window hiding BG1
    p.write(0x05, 0x01); // BGMODE = mode 1
    p.write(0x07, 0x04); // BG1SC: map at 0x400, 32x32
    p.write(0x0B, 0x01); // BG1ND: chr at 0x1000

    put_vram(&mut p, 0x1020, 0x0080);
    put_vram(&mut p, 0x1021, 0x8000);
    put_vram(&mut p, 0x400, 0x0402);
    put_vram(&mut p, 0x401, 0x2402);

    put_cgram(&mut p, 17, 0x1234);
    put_cgram(&mut p, 18, 0x2345);
    put_cgram(&mut p, 0, 0x7C00); // backdrop blue

    // Enable BG1 on main screen and window for BG1
    p.write(0x2C, 0x01); // TM = BG1
    p.write(0x2E, 0x01); // TMW = BG1
    p.write(0x23, 0x02); // W1SEL: window 1 on BG1

    // Window 1: x=0 to x=7
    p.write(0x26, 0);   // W1L = 0
    p.write(0x27, 7);   // W1R = 7

    // Pixel (0,0): inside window -> BG1 hidden -> backdrop
    let pick = p.pick_screen(true, 0, 0);
    assert_eq!(pick.rgb, 0x7C00);

    // Pixel (8,0): outside window -> BG1 visible
    let pick = p.pick_screen(true, 8, 0);
    assert_eq!(pick.rgb, 0x1234);

    // layer_hidden checks
    assert!(p.layer_hidden(true, 0, 3));   // main, BG1, inside window
    assert!(!p.layer_hidden(true, 0, 8));  // main, BG1, outside window
    assert!(!p.layer_hidden(false, 0, 3)); // sub, TSW=0 -> not hidden
    assert!(!p.layer_hidden(true, 1, 3));  // main, BG2: TMW bit 1 = 0
}

#[test]
fn e05_14_pick_screen_obj_over_bg() {
    let mut p = Ppu::new(Region::Ntsc);

    // Same BG setup as above
    p.write(0x05, 0x01); // BGMODE = mode 1
    p.write(0x07, 0x04); // BG1SC: map at 0x400, 32x32
    p.write(0x0B, 0x01); // BG1ND: chr at 0x1000

    put_vram(&mut p, 0x1020, 0x0080);
    put_vram(&mut p, 0x1021, 0x8000);
    put_vram(&mut p, 0x400, 0x0402);
    put_vram(&mut p, 0x401, 0x2402);

    put_cgram(&mut p, 17, 0x1234);
    put_cgram(&mut p, 18, 0x2345);
    put_cgram(&mut p, 0, 0x7C00); // backdrop blue

    // Sprite: x=0, y=255 (first visible row), tile 0, prio 3, pal 0
    spr(&mut p, 0, 0, 255, 0, 0x30);

    // Tile 0 data: index 1 at pixel 0
    put_vram(&mut p, 0, 0x0080);

    // Sprite palette color
    put_cgram(&mut p, 129, 0x03E0);

    // Enable BG1 and OBJ on main screen
    p.write(0x2C, 0x11); // TM = BG1 + OBJ

    // Render sprite line
    p.render_obj_line(0);

    // Pixel (0,0): OBJ prio 3 is in front of BG1 prio 0
    let pick = p.pick_screen(true, 0, 0);
    assert_eq!(pick.rgb, 0x03E0);
    assert_eq!(pick.layer, 4);

    // Pixel (8,0): no sprite -> BG1 visible
    let pick = p.pick_screen(true, 8, 0);
    assert_eq!(pick.rgb, 0x1234);
}
