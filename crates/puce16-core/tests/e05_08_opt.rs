// E05-08: offset-per-tile (modes 2 and 4)
use puce16_core::ppu::{bg::Pix, Ppu};
use puce16_core::region::Region;

fn put_vram(p: &mut Ppu, addr: u16, w: u16) {
    p.write(0x15, 0x80);
    p.write(0x16, (addr & 0xFF) as u8);
    p.write(0x17, (addr >> 8) as u8);
    p.write(0x18, (w & 0xFF) as u8);
    p.write(0x19, (w >> 8) as u8);
}

fn put_cgram(p: &mut Ppu, i: u8, w: u16) {
    p.write(0x21, i);
    p.write(0x22, (w & 0xFF) as u8);
    p.write(0x22, (w >> 8) as u8);
}

fn set_scroll(p: &mut Ppu, reg: u8, v: u16) {
    p.write(reg, (v & 0xFF) as u8);
    p.write(reg, ((v >> 8) & 0x3F) as u8);
}

// Scene E: mode 2, BG1 4bpp, BG3 map at 0x800
fn scene_e() -> Ppu {
    let mut p = Ppu::new(Region::Ntsc);
    p.write(0x05, 0x02); // mode 2
    p.write(0x07, 0x04); // BG1 map at 0x400 (32x32)
    p.write(0x0B, 0x01); // chr BG1 at 0x1000
    p.write(0x09, 0x08); // BG3 map at 0x800 (32x32)
    put_vram(&mut p, 0x1020, 0x0080); // tile 2 row 0: pixel 7 = index 1
    put_cgram(&mut p, 17, 0x1234);
    p
}

#[test]
fn e05_08_mode2_both_offsets() {
    let mut p = scene_e();
    // BG1 entry at (3,1) -> tile 2, pal 0
    put_vram(&mut p, 0x423, 0x0402);
    // BG3 entries: horizontal override at col 0 row 0, vertical at col 0 row 1
    put_vram(&mut p, 0x800, 0x2010); // bit13=BG1 enable, value 0x10 (hofs)
    put_vram(&mut p, 0x820, 0x2005); // bit13=BG1 enable, value 5 (vofs)

    // At x=8: n_col = (8+0)>>3 = 1 -> ex=0, ey=0
    // hofs_use = 0x10 | 0 = 16, vofs_use = 5
    // px = 8 + 16 = 24, py = 3 + 5 = 8 -> tile (3,1)
    let pix = p.bg_pixel(0, 8, 3);
    assert_eq!(pix, Some(Pix { rgb: 0x1234, prio: 0, obj_pal: 0 }));
}

#[test]
fn e05_08_mode2_first_column_unmodified() {
    let mut p = scene_e();
    put_vram(&mut p, 0x423, 0x0402);
    put_vram(&mut p, 0x800, 0x2010);
    put_vram(&mut p, 0x820, 0x2005);

    // x=0: n_col = (0+0)>>3 = 0 -> no override
    assert_eq!(p.bg_pixel(0, 0, 3), None);
    // x=7: n_col = (7+0)>>3 = 0 -> no override
    assert_eq!(p.bg_pixel(0, 7, 3), None);
}

#[test]
fn e05_08_mode2_horizontal_only() {
    let mut p = scene_e();
    put_vram(&mut p, 0x423, 0x0402);
    // BG3 entry without bit13: horizontal override ignored
    put_vram(&mut p, 0x800, 0x0010);

    // No override applied -> px=8, py=3+5=8 -> tile (1,1) which is empty
    assert_eq!(p.bg_pixel(0, 8, 3), None);
}

#[test]
fn e05_08_mode2_wrong_enable_bit() {
    let mut p = scene_e();
    put_vram(&mut p, 0x423, 0x0402);
    // BG3 entries with bit14 (BG2) instead of bit13 (BG1)
    put_vram(&mut p, 0x800, 0x4010);
    put_vram(&mut p, 0x820, 0x4005);

    // BG1 not affected -> px=8, py=3+5=8 -> tile (1,1) empty
    assert_eq!(p.bg_pixel(0, 8, 3), None);
}

#[test]
fn e05_08_mode2_fine_hofs_kept() {
    let mut p = scene_e();
    put_vram(&mut p, 0x423, 0x0402);
    put_vram(&mut p, 0x800, 0x2010);
    put_vram(&mut p, 0x820, 0x2005);
    set_scroll(&mut p, 0x0D, 3); // HOFS BG1 = 3

    // x=5: n_col = (5+3)>>3 = 1 -> ex=0, ey=0
    // hofs_use = 0x10 | 3 = 19, vofs_use = 5
    // px = 5 + 19 = 24, py = 3 + 5 = 8 -> tile (3,1)
    let pix = p.bg_pixel(0, 5, 3);
    assert_eq!(pix, Some(Pix { rgb: 0x1234, prio: 0, obj_pal: 0 }));
}

#[test]
fn e05_08_mode2_bg3_hofs_shifts_column() {
    let mut p = scene_e();
    put_vram(&mut p, 0x423, 0x0402);
    // BG3 entries at column 1 instead of 0
    put_vram(&mut p, 0x800, 0x0000);
    put_vram(&mut p, 0x801, 0x2010);
    put_vram(&mut p, 0x821, 0x2005);

    // Without BG3 scroll: ex = (1-1) + 0 = 0 -> no override at col 0
    assert_eq!(p.bg_pixel(0, 8, 3), None);

    // With BG3 HOFS=8: ex = (1-1) + 1 = 1 -> override found
    set_scroll(&mut p, 0x11, 8);
    let pix = p.bg_pixel(0, 8, 3);
    assert_eq!(pix, Some(Pix { rgb: 0x1234, prio: 0, obj_pal: 0 }));
}

// Scene F: mode 4, BG2 2bpp
fn scene_f() -> Ppu {
    let mut p = Ppu::new(Region::Ntsc);
    p.write(0x05, 0x04); // mode 4
    p.write(0x08, 0x0C); // BG2 map at 0xC00 (32x32)
    p.write(0x0B, 0x20); // chr BG2 at 0x2000
    p.write(0x09, 0x08); // BG3 map at 0x800 (32x32)
    put_vram(&mut p, 0x2008, 0x0080); // tile 1 row 0: pixel 7 = index 1
    put_cgram(&mut p, 1, 0x0AAA);
    p
}

#[test]
fn e05_08_mode4_horizontal() {
    let mut p = scene_f();
    // BG2 entry at (3,1) -> tile 1, pal 0
    put_vram(&mut p, 0xC23, 0x0001);
    // BG3: bit15=0 -> horizontal override, bit14=BG2 enable, value 0x10
    put_vram(&mut p, 0x800, 0x4010);

    // x=8: n_col = (8+0)>>3 = 1 -> ex=0, ey=0
    // hofs_use = 0x10 | 0 = 16, vofs_use unchanged
    // px = 8 + 16 = 24, py = 8 -> tile (3,1)
    let pix = p.bg_pixel(1, 8, 8);
    assert_eq!(pix, Some(Pix { rgb: 0x0AAA, prio: 0, obj_pal: 0 }));
}

#[test]
fn e05_08_mode4_vertical() {
    let mut p = scene_f();
    // BG2 entry at (1,1) -> tile 1, pal 0
    put_vram(&mut p, 0xC21, 0x0001);
    // BG3: bit15=1 -> vertical override, bit14=BG2 enable, value 5
    put_vram(&mut p, 0x800, 0xC005);

    // x=8: n_col = (8+0)>>3 = 1 -> ex=0, ey=0
    // hofs_use unchanged, vofs_use = 5
    // px = 8, py = 3 + 5 = 8 -> tile (1,1)
    let pix = p.bg_pixel(1, 8, 3);
    assert_eq!(pix, Some(Pix { rgb: 0x0AAA, prio: 0, obj_pal: 0 }));

    // Without BG3 entry: no override -> py=3 -> tile (1,0) empty
    put_vram(&mut p, 0x800, 0);
    assert_eq!(p.bg_pixel(1, 8, 3), None);
}

#[test]
fn e05_08_non_opt_modes() {
    // Mode 1: offset-per-tile not active
    let mut p = Ppu::new(Region::Ntsc);
    p.write(0x05, 0x01); // mode 1
    p.write(0x07, 0x04); // BG1 map at 0x400 (32x32)
    p.write(0x0B, 0x01); // chr BG1 at 0x1000
    p.write(0x09, 0x08); // BG3 map at 0x800

    put_vram(&mut p, 0x423, 0x0402);
    put_cgram(&mut p, 17, 0x1234);
    put_vram(&mut p, 0x800, 0x2010);
    put_vram(&mut p, 0x820, 0x2005);

    // No override in mode 1 -> px=8, py=3 -> tile (1,0) empty
    assert_eq!(p.bg_pixel(0, 8, 3), None);
}