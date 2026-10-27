use puce16_core::ppu::{Ppu, Pick};
use puce16_core::region::Region;
use puce16_core::ppu::colormath::{region, fixed_color, blend, math_layer_ok, color_math};

#[test]
fn e05_15_region_table() {
    assert_eq!(region(0, true), false);
    assert_eq!(region(0, false), false);
    assert_eq!(region(1, true), false);
    assert_eq!(region(1, false), true);
    assert_eq!(region(2, true), true);
    assert_eq!(region(2, false), false);
    assert_eq!(region(3, true), true);
    assert_eq!(region(3, false), true);
}

#[test]
fn e05_15_fixed_color_values() {
    assert_eq!(fixed_color(31, 0, 0), 0x001F);
    assert_eq!(fixed_color(0, 31, 0), 0x03E0);
    assert_eq!(fixed_color(0, 0, 31), 0x7C00);
    assert_eq!(fixed_color(1, 2, 3), 0x0C41);
    assert_eq!(fixed_color(31, 31, 31), 0x7FFF);
}

#[test]
fn e05_15_blend_add() {
    let base = 0x7A8A; // r=10, g=20, b=30
    let add = 0x29E5;  // r=5, g=15, b=10
    assert_eq!(blend(base, add, false, false), 0x7FEF); // (15, 35->31, 40->31)
    assert_eq!(blend(base, add, false, true), 0x5227);  // (7, 17, 20)
}

#[test]
fn e05_15_blend_sub() {
    let base = 0x7A8A; // r=10, g=20, b=30
    let add = 0x29E5;  // r=5, g=15, b=10
    assert_eq!(blend(base, add, true, false), 0x50A5);  // (5, 5, 20)
    assert_eq!(blend(base, add, true, true), 0x2842);   // (2, 2, 10)
    assert_eq!(blend(add, base, true, false), 0x0000);  // all negative -> 0
    assert_eq!(blend(add, base, true, true), 0x0000);
}

#[test]
fn e05_15_blend_neutral() {
    assert_eq!(blend(0x1234, 0, false, false), 0x1234);
    assert_eq!(blend(0x1234, 0, true, false), 0x1234);
    assert_eq!(blend(0x7FFF, 0x7FFF, false, false), 0x7FFF);
    assert_eq!(blend(0x7FFF, 0x7FFF, true, false), 0x0000);
    assert_eq!(blend(0x7FFF, 0x7FFF, false, true), 0x7FFF);
}

#[test]
fn e05_15_math_layer_ok_rules() {
    // BG layers: mask bit must be set
    assert_eq!(math_layer_ok(0, 0, 0x01), true);
    assert_eq!(math_layer_ok(1, 0, 0x01), false);
    assert_eq!(math_layer_ok(3, 0, 0x08), true);
    // OBJ: mask bit 4 AND palette >= 4
    assert_eq!(math_layer_ok(4, 4, 0x10), true);
    assert_eq!(math_layer_ok(4, 7, 0x10), true);
    assert_eq!(math_layer_ok(4, 3, 0x10), false);
    assert_eq!(math_layer_ok(4, 4, 0x00), false);
    // Backdrop: mask bit 5
    assert_eq!(math_layer_ok(5, 0, 0x20), true);
    assert_eq!(math_layer_ok(5, 0, 0x1F), false);
}

#[test]
fn e05_15_math_off_by_default() {
    let ppu = Ppu::new(Region::Ntsc);
    let main = Pick { rgb: 0x0001, layer: 0, obj_pal: 0 };
    let sub = Pick { rgb: 0, layer: 5, obj_pal: 0 };
    assert_eq!(color_math(&main, &sub, false, false, ppu.regs()), 0x0001);
}

#[test]
fn e05_15_math_fixed_add_half_sub() {
    let mut ppu = Ppu::new(Region::Ntsc);
    // Enable math on BG1 (mask bit 0)
    ppu.write(0x31, 0x01);
    // Fixed color: green=3
    ppu.write(0x32, 0x43);

    let main = Pick { rgb: 0x0001, layer: 0, obj_pal: 0 };
    let sub = Pick { rgb: 0, layer: 5, obj_pal: 0 };

    // Add fixed green=3 to pixel with r=1 -> (r=1, g=3)
    assert_eq!(color_math(&main, &sub, false, false, ppu.regs()), 0x0061);

    // With half: (1+0)>>1=0, (0+3)>>1=1 -> (r=0, g=1)
    ppu.write(0x31, 0x41);
    assert_eq!(color_math(&main, &sub, false, false, ppu.regs()), 0x0020);

    // With subtract: r=1-0=1, g=0-3->0 -> (r=1, g=0)
    ppu.write(0x31, 0x81);
    assert_eq!(color_math(&main, &sub, false, false, ppu.regs()), 0x0001);
}

#[test]
fn e05_15_math_subscreen() {
    let mut ppu = Ppu::new(Region::Ntsc);
    // Addend from subscreen
    ppu.write(0x30, 0x02);
    // Enable math on BG1
    ppu.write(0x31, 0x01);

    let main = Pick { rgb: 0x0001, layer: 0, obj_pal: 0 };
    let sub = Pick { rgb: 0x0002, layer: 1, obj_pal: 0 };

    // Add subscreen pixel (r=2) to main (r=1) -> r=3
    assert_eq!(color_math(&main, &sub, false, false, ppu.regs()), 0x0003);

    // With half: (1+2)>>1 = 1
    ppu.write(0x31, 0x41);
    assert_eq!(color_math(&main, &sub, false, false, ppu.regs()), 0x0001);

    // Subscreen backdrop with half: no division
    let sub_back = Pick { rgb: 0x0002, layer: 5, obj_pal: 0 };
    assert_eq!(color_math(&main, &sub_back, false, false, ppu.regs()), 0x0003);
}

#[test]
fn e05_15_math_prevent_black() {
    let mut ppu = Ppu::new(Region::Ntsc);
    // Enable math on BG1
    ppu.write(0x31, 0x01);
    // Fixed color: red=10
    ppu.write(0x32, 0x2A);

    let main = Pick { rgb: 0x0001, layer: 0, obj_pal: 0 };
    let sub = Pick { rgb: 0, layer: 5, obj_pal: 0 };

    // Prevent overrides math
    assert_eq!(color_math(&main, &sub, false, true, ppu.regs()), 0x0001);

    // Black replaces base with 0, then add fixed red=10 -> r=10
    assert_eq!(color_math(&main, &sub, true, false, ppu.regs()), 0x000A);

    // Black with no math enabled -> just black (r=0)
    ppu.write(0x31, 0x00);
    assert_eq!(color_math(&main, &sub, true, false, ppu.regs()), 0x0000);
}

#[test]
fn e05_15_math_obj_palettes() {
    let mut ppu = Ppu::new(Region::Ntsc);
    // Enable math on OBJ (mask bit 4)
    ppu.write(0x31, 0x10);
    // Fixed color: red=10
    ppu.write(0x32, 0x2A);

    let main_low = Pick { rgb: 0x0005, layer: 4, obj_pal: 3 };
    let sub = Pick { rgb: 0, layer: 5, obj_pal: 0 };

    // Palette < 4: no math applied
    assert_eq!(color_math(&main_low, &sub, false, false, ppu.regs()), 0x0005);

    let main_high = Pick { rgb: 0x0005, layer: 4, obj_pal: 4 };
    // Palette >= 4: math applied (r=5+10=15)
    assert_eq!(color_math(&main_high, &sub, false, false, ppu.regs()), 0x000F);
}

#[test]
fn e05_15_math_backdrop_layer() {
    let mut ppu = Ppu::new(Region::Ntsc);
    // Fixed color: red=10
    ppu.write(0x32, 0x2A);

    let main = Pick { rgb: 0x0001, layer: 5, obj_pal: 0 };
    let sub = Pick { rgb: 0, layer: 5, obj_pal: 0 };

    // Enable math on backdrop (mask bit 5)
    ppu.write(0x31, 0x20);
    assert_eq!(color_math(&main, &sub, false, false, ppu.regs()), 0x000B);

    // Math not enabled for backdrop -> no change
    ppu.write(0x31, 0x01);
    assert_eq!(color_math(&main, &sub, false, false, ppu.regs()), 0x0001);
}

#[test]
fn e05_15_math_clamp_white() {
    let mut ppu = Ppu::new(Region::Ntsc);
    // Enable math on BG1
    ppu.write(0x31, 0x01);
    // Fixed color: white (31,31,31)
    ppu.write(0x32, 0xFF);

    let main = Pick { rgb: 0x7FFF, layer: 0, obj_pal: 0 };
    let sub = Pick { rgb: 0, layer: 5, obj_pal: 0 };

    // White + white clamps to white
    assert_eq!(color_math(&main, &sub, false, false, ppu.regs()), 0x7FFF);
}

#[test]
fn e05_15_color_window_flags() {
    let mut ppu = Ppu::new(Region::Ntsc);
    // Color window: group 2 (window 1 active)
    ppu.write(0x25, 0x20);
    // Window 1 bounds: x=10 to x=20
    ppu.write(0x26, 10);
    ppu.write(0x27, 20);
    // black_region=1 (outside), math_region=2 (inside)
    ppu.write(0x30, 0x60);

    // Inside window: prevent=true, black=false
    assert_eq!(ppu.color_window_flags(15), (false, true));
    // Outside window: black=true, prevent=false
    assert_eq!(ppu.color_window_flags(5), (true, false));

    // Both regions disabled -> always false
    ppu.write(0x30, 0x00);
    assert_eq!(ppu.color_window_flags(15), (false, false));
    assert_eq!(ppu.color_window_flags(5), (false, false));

    // Both regions set to "always" -> always true
    ppu.write(0x30, 0xF0);
    assert_eq!(ppu.color_window_flags(15), (true, true));
    assert_eq!(ppu.color_window_flags(5), (true, true));

    // Color window inactive: inside=false everywhere
    ppu.write(0x25, 0x00);
    ppu.write(0x30, 0x60);
    // black_region=1 with inside=false -> true; math_region=2 with inside=false -> false
    assert_eq!(ppu.color_window_flags(15), (true, false));
}
