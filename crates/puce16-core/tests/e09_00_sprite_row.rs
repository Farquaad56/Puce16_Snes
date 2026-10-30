//! E09-00: sprite row formula `row = (r - y) & 0xFF` (regression: SMW hides sprites at Y=240).
use puce16_core::ppu::obj::sprite_row;
use puce16_core::ppu::Ppu;
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

fn load_oam(p: &mut Ppu, img: &[u8; 544]) {
    p.write(0x02, 0);
    p.write(0x03, 0);
    for b in img.iter() {
        p.write(0x04, *b);
    }
}

fn spr(img: &mut [u8; 544], i: usize, x: i16, y: u8, tile: u8, attr: u8, large: bool) {
    img[i * 4] = (x & 0xFF) as u8;
    img[i * 4 + 1] = y;
    img[i * 4 + 2] = tile;
    img[i * 4 + 3] = attr;
    let sh = (i % 4) * 2;
    let hb = 512 + i / 4;
    img[hb] &= !(3u8 << sh);
    let x8 = ((x >> 8) & 1) as u8;
    img[hb] |= (x8 | if large { 2 } else { 0 }) << sh;
}

fn hide_all(img: &mut [u8; 544]) {
    for i in 0..128 {
        spr(img, i, -200, 0, 0, 0, false);
    }
}

fn fill_tiles(p: &mut Ppu, words: u16) {
    for a in 0..words {
        put_vram(p, a, 0x0080);
    }
}

#[test]
fn e09_00_sprite_16x16_y240_hidden_on_all_rows() {
    let mut p = Ppu::new(Region::Ntsc);
    p.write(0x01, 0x60); // obj_size=3: small=16x16, large=32x32
    let mut img = [0u8; 544];
    hide_all(&mut img);
    spr(&mut img, 0, 10, 240, 0, 0, false);
    load_oam(&mut p, &img);
    fill_tiles(&mut p, 0x400);
    put_cgram(&mut p, 129, 0x1111);

    for r in 0..224u16 {
        let (list, _) = p.sprite_list(r);
        assert!(list.is_empty(), "row {r}: sprite list must be empty");
        p.render_obj_line(r);
        for x in 0..256usize {
            assert_eq!(p.obj_pixel(x), None, "row {r} x {x}");
        }
    }
}

#[test]
fn e09_00_sprite_y0_first_visible_row() {
    let mut p = Ppu::new(Region::Ntsc);
    let mut img = [0u8; 544];
    hide_all(&mut img);
    spr(&mut img, 0, 10, 0, 0, 0, false); // 8x8 at Y=0
    load_oam(&mut p, &img);
    for row in 0..8u16 {
        put_vram(&mut p, row, if row == 5 { 0x0000 } else { 0x0080 });
    }
    put_cgram(&mut p, 129, 0x1111);

    for r in 0..8u16 {
        assert_eq!(sprite_row(r, 0), r as i32);
        let (list, _) = p.sprite_list(r);
        assert_eq!(list, vec![0u8], "row {r}");
        p.render_obj_line(r);
        if r == 5 {
            assert_eq!(p.obj_pixel(10), None, "row 5 is empty in the tile");
        } else {
            assert!(p.obj_pixel(10).is_some(), "row {r}");
        }
    }
    let (list8, _) = p.sprite_list(8);
    assert!(list8.is_empty());
    p.render_obj_line(8);
    assert_eq!(p.obj_pixel(10), None);
}

#[test]
fn e09_00_sprite_32x32_y240_wraps_to_top() {
    let mut p = Ppu::new(Region::Ntsc);
    p.write(0x01, 0x60); // obj_size=3: large=32x32
    let mut img = [0u8; 544];
    hide_all(&mut img);
    spr(&mut img, 0, 10, 240, 0, 0, true);
    load_oam(&mut p, &img);
    // Only sprite rows 16..31 have data: tile 0x20 (rows 16..23), tile 0x30 (rows 24..31).
    for row in 0..8u16 {
        put_vram(&mut p, 0x200 + row, 0x0080);
        put_vram(&mut p, 0x300 + row, 0x0080);
    }
    put_cgram(&mut p, 129, 0x1111);

    for r in 0..16u16 {
        assert_eq!(sprite_row(r, 240), r as i32 + 16);
        let (list, _) = p.sprite_list(r);
        assert_eq!(list, vec![0u8], "row {r}");
        p.render_obj_line(r);
        assert!(p.obj_pixel(10).is_some(), "row {r}");
    }
    let (list16, _) = p.sprite_list(16);
    assert!(list16.is_empty());
    p.render_obj_line(16);
    assert_eq!(p.obj_pixel(10), None);
}

#[test]
fn e09_00_sprite_list_and_render_use_same_row() {
    let mut p = Ppu::new(Region::Ntsc);
    p.write(0x01, 0x60); // small=16x16
    fill_tiles(&mut p, 0x400);
    put_cgram(&mut p, 129, 0x1111);
    for y in 0..=255u8 {
        let mut img = [0u8; 544];
        hide_all(&mut img);
        spr(&mut img, 0, 10, y, 0, 0, false);
        load_oam(&mut p, &img);
        for r in 0..224u16 {
            let expect = sprite_row(r, y) < 16;
            let (list, _) = p.sprite_list(r);
            assert_eq!(list.contains(&0), expect, "y {y} row {r}");
            p.render_obj_line(r);
            assert_eq!(p.obj_pixel(10).is_some(), expect, "y {y} row {r}");
        }
    }
}