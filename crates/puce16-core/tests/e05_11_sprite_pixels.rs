use puce16_core::ppu::{Ppu, Pix};
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

#[test]
fn e05_11_basic_pixel() {
    let mut p = Ppu::new(Region::Ntsc);
    // OBJSEL 0 (default): small=8x8, large=16x16
    let mut img = [0u8; 544];
    hide_all(&mut img);
    spr(&mut img, 0, 10, 0, 0, 0, false); // x=10, y=0, tile 0, attr 0, small
    load_oam(&mut p, &img);

    put_vram(&mut p, 0, 0x0080); // row 0, index 1 at pixel 0
    put_cgram(&mut p, 129, 0x1111);

    p.render_obj_line(0);
    assert_eq!(p.obj_pixel(10), Some(Pix { rgb: 0x1111, prio: 0, obj_pal: 0 }));
    assert_eq!(p.obj_pixel(9), None);
    assert_eq!(p.obj_pixel(11), None);
}

#[test]
fn e05_11_row_selects_data() {
    let mut p = Ppu::new(Region::Ntsc);
    let mut img = [0u8; 544];
    hide_all(&mut img);
    spr(&mut img, 0, 10, 0, 0, 0, false);
    load_oam(&mut p, &img);

    put_vram(&mut p, 0, 0x0080); // row 0: index 1
    put_cgram(&mut p, 129, 0x1111);
    put_vram(&mut p, 1, 0x8000); // row 1: index 2
    put_cgram(&mut p, 130, 0x2222);

    p.render_obj_line(1);
    assert_eq!(p.obj_pixel(10), Some(Pix { rgb: 0x2222, prio: 0, obj_pal: 0 }));

    p.render_obj_line(2);
    assert_eq!(p.obj_pixel(10), None); // row 2 is empty
}

#[test]
fn e05_11_hflip_8x8() {
    let mut p = Ppu::new(Region::Ntsc);
    let mut img = [0u8; 544];
    hide_all(&mut img);
    spr(&mut img, 0, 10, 0, 0, 0x40, false); // H flip
    load_oam(&mut p, &img);

    put_vram(&mut p, 0, 0x0080); // row 0, index 1 at pixel 0 (left edge)
    put_cgram(&mut p, 129, 0x1111);

    p.render_obj_line(0);
    // With H flip, left-edge data appears on right side of sprite
    assert_eq!(p.obj_pixel(17), Some(Pix { rgb: 0x1111, prio: 0, obj_pal: 0 }));
    assert_eq!(p.obj_pixel(10), None);
}

#[test]
fn e05_11_palette_and_priority() {
    let mut p = Ppu::new(Region::Ntsc);
    let mut img = [0u8; 544];
    hide_all(&mut img);
    spr(&mut img, 0, 10, 0, 0, 0x28, false); // prio=2, palette=4
    load_oam(&mut p, &img);

    put_vram(&mut p, 0, 0x0080);
    put_cgram(&mut p, 193, 0x3333); // 128 + 4*16 + 1 = 193

    p.render_obj_line(0);
    assert_eq!(p.obj_pixel(10), Some(Pix { rgb: 0x3333, prio: 2, obj_pal: 4 }));
}

#[test]
fn e05_11_lowest_index_wins() {
    let mut p = Ppu::new(Region::Ntsc);
    let mut img = [0u8; 544];
    hide_all(&mut img);
    spr(&mut img, 0, 10, 0, 0, 0, false); // sprite 0, palette 0
    spr(&mut img, 1, 10, 0, 0, 0x02, false); // sprite 1, palette 1 (higher index)
    load_oam(&mut p, &img);

    put_vram(&mut p, 0, 0x0080);
    put_cgram(&mut p, 129, 0x1111); // sprite 0 color
    put_cgram(&mut p, 145, 0x4444); // sprite 1 color (128 + 1*16 + 1)

    p.render_obj_line(0);
    // Sprite 0 wins because it has lower index
    assert_eq!(p.obj_pixel(10), Some(Pix { rgb: 0x1111, prio: 0, obj_pal: 0 }));
}

#[test]
fn e05_11_vflip_8x8() {
    let mut p = Ppu::new(Region::Ntsc);
    let mut img = [0u8; 544];
    hide_all(&mut img);
    spr(&mut img, 0, 10, 0, 0, 0x80, false); // V flip
    load_oam(&mut p, &img);

    put_vram(&mut p, 0, 0x0080); // row 0 has data
    put_cgram(&mut p, 129, 0x1111);

    // With V flip on 8x8: rv = 7 - row
    // render_obj_line(7): row=7 -> rv=0 (reads row 0 data)
    p.render_obj_line(7);
    assert!(p.obj_pixel(10).is_some());

    // render_obj_line(0): row=0 -> rv=7 (no data at row 7)
    p.render_obj_line(0);
    assert_eq!(p.obj_pixel(10), None);
}

#[test]
fn e05_11_large_two_tiles() {
    let mut p = Ppu::new(Region::Ntsc);
    // OBJSEL 0: large=16x16
    let mut img = [0u8; 544];
    hide_all(&mut img);
    spr(&mut img, 0, 20, 0, 0x01, 0, true); // large sprite, tile 0x01
    load_oam(&mut p, &img);

    put_vram(&mut p, 16, 0x0080);   // tile 1 row 0 index 1 (left column)
    put_vram(&mut p, 32, 0x8000);   // tile 2 row 0 index 2 (right column)
    put_vram(&mut p, 272, 0x0080);  // tile 17 (=1+16) row 0 index 1
    put_cgram(&mut p, 129, 0x1111);
    put_cgram(&mut p, 130, 0x2222);

    p.render_obj_line(0);
    assert_eq!(p.obj_pixel(20), Some(Pix { rgb: 0x1111, prio: 0, obj_pal: 0 })); // left tile
    assert_eq!(p.obj_pixel(28), Some(Pix { rgb: 0x2222, prio: 0, obj_pal: 0 })); // right tile

    p.render_obj_line(8);
    assert_eq!(p.obj_pixel(20), Some(Pix { rgb: 0x1111, prio: 0, obj_pal: 0 })); // tile 17 row 0
    assert_eq!(p.obj_pixel(28), None); // tile 18 is empty
}

#[test]
fn e05_11_large_hflip() {
    let mut p = Ppu::new(Region::Ntsc);
    let mut img = [0u8; 544];
    hide_all(&mut img);
    spr(&mut img, 0, 20, 0, 0x01, 0x40, true); // large sprite with H flip
    load_oam(&mut p, &img);

    put_vram(&mut p, 16, 0x0080);   // tile 1 row 0 index 1
    put_vram(&mut p, 32, 0x8000);   // tile 2 row 0 index 2
    put_cgram(&mut p, 129, 0x1111);
    put_cgram(&mut p, 130, 0x2222);

    p.render_obj_line(0);
    // With H flip on 16-wide sprite: c=0 reads tile 2 flipped, c=1 reads tile 1 flipped
    assert_eq!(p.obj_pixel(27), Some(Pix { rgb: 0x2222, prio: 0, obj_pal: 0 })); // c=0, pixel at dest 7
    assert_eq!(p.obj_pixel(35), Some(Pix { rgb: 0x1111, prio: 0, obj_pal: 0 })); // c=1, pixel at dest 7
}

#[test]
fn e05_11_name_table_bit() {
    let mut p = Ppu::new(Region::Ntsc);
    // Write OBJSEL to set obj_name_gap (default 0x00 -> gap_words=1 -> 0x1000)
    p.write(0x01, 0x00);

    let mut img = [0u8; 544];
    hide_all(&mut img);
    spr(&mut img, 0, 10, 0, 0, 1, false); // tbit=1 (attr bit 0)
    load_oam(&mut p, &img);

    put_vram(&mut p, 0x1000, 0x0080); // obj_name_gap = 0x1000 with OBJSEL 0
    put_cgram(&mut p, 129, 0x1111);

    p.render_obj_line(0);
    assert!(p.obj_pixel(10).is_some());

    // Now test with custom obj_base
    let mut p2 = Ppu::new(Region::Ntsc);
    p2.write(0x01, 0x02); // obj_base = 2 << 13 = 0x4000
    let mut img2 = [0u8; 544];
    hide_all(&mut img2);
    spr(&mut img2, 0, 10, 0, 0, 0, false); // tbit=0
    load_oam(&mut p2, &img2);

    put_vram(&mut p2, 0x4000, 0x0080);
    put_cgram(&mut p2, 129, 0x1111);

    p2.render_obj_line(0);
    assert!(p2.obj_pixel(10).is_some());
}

#[test]
fn e05_11_clipping_left_right() {
    let mut p = Ppu::new(Region::Ntsc);
    let mut img = [0u8; 544];
    hide_all(&mut img);
    spr(&mut img, 0, -4, 0, 0, 0, false); // x=-4
    load_oam(&mut p, &img);

    put_vram(&mut p, 0, 0x0001); // pixel at data position 7 (rightmost)
    put_cgram(&mut p, 129, 0x1111);

    p.render_obj_line(0);
    // dest = -4 + 7 = 3
    assert!(p.obj_pixel(3).is_some());
    assert_eq!(p.obj_pixel(0), None);

    // Test right clipping: sprite at x=250 with pixel at data position 7 -> dest 257 (out of range)
    let mut p2 = Ppu::new(Region::Ntsc);
    let mut img2 = [0u8; 544];
    hide_all(&mut img2);
    spr(&mut img2, 0, 250, 0, 0, 0, false);
    load_oam(&mut p2, &img2);

    put_vram(&mut p2, 0, 0x0001);
    put_cgram(&mut p2, 129, 0x1111);

    p2.render_obj_line(0);
    assert_eq!(p2.obj_pixel(255), None); // no panic, just clipped
}

#[test]
fn e05_11_time_over_35_slivers() {
    let mut p = Ppu::new(Region::Ntsc);
    p.write(0x01, 0x40); // obj_size=2: small=8x8, large=64x64

    let mut img = [0u8; 544];
    hide_all(&mut img);
    // 4 large sprites (64x64) at x=0,64,128,192 -> 32 slivers
    for i in 0..4 {
        spr(&mut img, i, (i * 64) as i16, 0, 0, 0, true);
    }
    // 3 small sprites at x=100,110,120 -> 3 slivers = total 35
    spr(&mut img, 4, 100, 0, 7, 0, false);
    spr(&mut img, 5, 110, 0, 7, 0, false);
    spr(&mut img, 6, 120, 0, 7, 0, false);
    load_oam(&mut p, &img);

    put_vram(&mut p, 0, 0x0080);   // tile 0 row 0 index 1
    put_vram(&mut p, 112, 0x0080); // tile 7 row 0 index 1
    put_cgram(&mut p, 129, 0x1111);

    p.render_obj_line(0);
    assert!(p.time_over());
    assert!(p.obj_pixel(0).is_some()); // first sliver rendered
    assert_eq!(p.obj_pixel(56), None); // 35th sliver (c=7 of sprite 0) was dropped
}

#[test]
fn e05_11_no_time_over_34() {
    let mut p = Ppu::new(Region::Ntsc);
    p.write(0x01, 0x40); // obj_size=2: large=64x64

    let mut img = [0u8; 544];
    hide_all(&mut img);
    for i in 0..4 {
        spr(&mut img, i, (i * 64) as i16, 0, 0, 0, true);
    }
    // Only 2 small sprites -> 34 slivers total
    spr(&mut img, 4, 100, 0, 7, 0, false);
    spr(&mut img, 5, 110, 0, 7, 0, false);
    load_oam(&mut p, &img);

    put_vram(&mut p, 0, 0x0080);
    put_vram(&mut p, 112, 0x0080);
    put_cgram(&mut p, 129, 0x1111);

    p.render_obj_line(0);
    assert!(!p.time_over());
    assert!(p.obj_pixel(56).is_some()); // tile 7 of sprite 0 rendered
}

#[test]
fn e05_11_x_minus_256_counts_slivers() {
    let mut p = Ppu::new(Region::Ntsc);
    p.write(0x01, 0x40); // obj_size=2: large=64x64

    let mut img = [0u8; 544];
    hide_all(&mut img);
    // 5 large sprites at x=-256 -> all count as on-screen (hardware bug)
    for i in 0..5 {
        spr(&mut img, i, -256, 0, 0, 0, true);
    }
    load_oam(&mut p, &img);

    put_vram(&mut p, 0, 0x0080);
    put_cgram(&mut p, 129, 0x1111);

    p.render_obj_line(0);
    assert!(p.time_over()); // 5*8 = 40 slivers counted
    assert_eq!(p.obj_pixel(0), None);   // no pixels written (all at x=-256)
    assert_eq!(p.obj_pixel(100), None);
    assert_eq!(p.obj_pixel(255), None);
    assert!(!p.range_over());
}

#[test]
fn e05_11_range_over_flag() {
    let mut p = Ppu::new(Region::Ntsc);
    let mut img = [0u8; 544];
    hide_all(&mut img);
    // 33 visible sprites -> overflow
    for i in 0..33 {
        spr(&mut img, i, (i * 2) as i16, 0, 0, 0, false);
    }
    load_oam(&mut p, &img);

    put_vram(&mut p, 0, 0x0080);
    put_cgram(&mut p, 129, 0x1111);

    p.render_obj_line(1);
    assert!(p.range_over());
    // STAT77 bit 7 should be set (range_over)
    let stat = p.read(0x3E, 0, false);
    assert_eq!(stat & 0x81, 0x81);

    // Next row without overflow does NOT clear the flag
    p.render_obj_line(2);
    assert!(p.range_over());
}

#[test]
fn e05_11_erratum_tall_vflip() {
    let mut p = Ppu::new(Region::Ntsc);
    p.write(0x01, 0xC0); // obj_size=6: small=16x32

    let mut img = [0u8; 544];
    hide_all(&mut img);
    spr(&mut img, 0, 10, 0, 0, 0x80, false); // V flip on 16x32 sprite
    load_oam(&mut p, &img);

    put_vram(&mut p, 263, 0x0080);   // tile 16 row 7 index 1 (top half)
    put_vram(&mut p, 775, 0x8000);   // tile 48 row 7 index 2 (bottom half)
    put_cgram(&mut p, 129, 0x1111);
    put_cgram(&mut p, 130, 0x2222);

    // render_obj_line(0): row=0 -> rv = (0/16)*16 + (15-0) = 15 -> tile_row=1, fine_y=7
    p.render_obj_line(0);
    assert_eq!(p.obj_pixel(10), Some(Pix { rgb: 0x1111, prio: 0, obj_pal: 0 }));

    // render_obj_line(16): row=16 -> rv = 16 + (15-0) = 31 -> tile_row=3, fine_y=7
    p.render_obj_line(16);
    assert_eq!(p.obj_pixel(10), Some(Pix { rgb: 0x2222, prio: 0, obj_pal: 0 }));
}

#[test]
fn e05_11_square_vflip_full() {
    let mut p = Ppu::new(Region::Ntsc);
    // OBJSEL 0: large=16x16 (square)
    let mut img = [0u8; 544];
    hide_all(&mut img);
    spr(&mut img, 0, 10, 0, 0x01, 0x80, true); // V flip on 16x16 sprite
    load_oam(&mut p, &img);

    put_vram(&mut p, 279, 0x0080); // tile 17 (=1+16) row 7 index 1
    put_cgram(&mut p, 129, 0x1111);

    // render_obj_line(0): row=0 -> rv = 15-0 = 15 -> tile_row=1, fine_y=7
    p.render_obj_line(0);
    assert!(p.obj_pixel(10).is_some());
}
