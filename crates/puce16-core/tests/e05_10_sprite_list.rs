//! E05-10: Sprite dimensions, decoding, and per-row sprite list evaluation.

use puce16_core::ppu::obj::obj_dims;
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
    img[i * 4] = (x as u8);
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
fn e05_10_obj_dims_table() {
    assert_eq!(obj_dims(0, false), (8, 8));
    assert_eq!(obj_dims(0, true), (16, 16));
    assert_eq!(obj_dims(1, false), (8, 8));
    assert_eq!(obj_dims(1, true), (32, 32));
    assert_eq!(obj_dims(2, true), (64, 64));
    assert_eq!(obj_dims(3, false), (16, 16));
    assert_eq!(obj_dims(3, true), (32, 32));
    assert_eq!(obj_dims(4, true), (64, 64));
    assert_eq!(obj_dims(5, false), (32, 32));
    assert_eq!(obj_dims(5, true), (64, 64));
    assert_eq!(obj_dims(6, false), (16, 32));
    assert_eq!(obj_dims(6, true), (32, 64));
    assert_eq!(obj_dims(7, false), (16, 32));
    assert_eq!(obj_dims(7, true), (32, 32));
}

#[test]
fn e05_10_decode_sprite() {
    let mut p = Ppu::new(Region::Ntsc);
    let mut img = [0u8; 544];
    spr(&mut img, 5, -1, 0x12, 0x34, 0xE5, true);
    load_oam(&mut p, &img);

    let s = p.sprite(5);
    assert_eq!(s.x, -1);
    assert_eq!(s.y, 0x12);
    assert_eq!(s.tile, 0x34);
    assert_eq!(s.tbit, true);
    assert_eq!(s.pal, 2);
    assert_eq!(s.prio, 2);
    assert_eq!(s.hflip, true);
    assert_eq!(s.vflip, true);
    assert_eq!(s.large, true);

    // Other sprites in same high byte are unaffected
    let s4 = p.sprite(4);
    assert_eq!(s4.large, false);
}

#[test]
fn e05_10_decode_x_range() {
    let mut p = Ppu::new(Region::Ntsc);
    let mut img = [0u8; 544];

    spr(&mut img, 0, 255, 0, 0, 0, false);
    spr(&mut img, 1, -256, 0, 0, 0, false);
    spr(&mut img, 2, 0, 0, 0, 0, false);
    spr(&mut img, 3, -2, 0, 0, 0, false);
    spr(&mut img, 4, 100, 0, 0, 0, true);

    load_oam(&mut p, &img);

    assert_eq!(p.sprite(0).x, 255);
    assert_eq!(p.sprite(1).x, -256);
    assert_eq!(p.sprite(2).x, 0);
    assert_eq!(p.sprite(3).x, -2);
    assert_eq!(p.sprite(4).x, 100);
    assert_eq!(p.sprite(4).large, true);
}

#[test]
fn e05_10_row_visibility_y255() {
    let mut p = Ppu::new(Region::Ntsc);
    let mut img = [0u8; 544];
    hide_all(&mut img);
    spr(&mut img, 0, 10, 255, 0, 0, false); // 8x8 at y=255 -> visible on rows 0..7
    load_oam(&mut p, &img);

    let (list0, _) = p.sprite_list(0);
    assert!(list0.contains(&0));

    let (list7, _) = p.sprite_list(7);
    assert!(list7.contains(&0));

    let (list8, _) = p.sprite_list(8);
    assert!(!list8.contains(&0));
}

#[test]
fn e05_10_row_visibility_y0() {
    let mut p = Ppu::new(Region::Ntsc);
    let mut img = [0u8; 544];
    hide_all(&mut img);
    spr(&mut img, 0, 10, 0, 0, 0, false); // y=0 -> visible on rows 1..8
    load_oam(&mut p, &img);

    let (list0, _) = p.sprite_list(0);
    assert!(!list0.contains(&0));

    for r in 1..=8 {
        let (list, _) = p.sprite_list(r);
        assert!(list.contains(&0), "sprite should be on row {}", r);
    }

    let (list9, _) = p.sprite_list(9);
    assert!(!list9.contains(&0));

    // Second sprite at y=10 -> visible on rows 11..18
    let mut img2 = [0u8; 544];
    hide_all(&mut img2);
    spr(&mut img2, 0, 10, 10, 0, 0, false);
    load_oam(&mut p, &img2);

    for r in 0..11 {
        let (list, _) = p.sprite_list(r);
        assert!(!list.contains(&0), "sprite should not be on row {}", r);
    }
    for r in 11..=18 {
        let (list, _) = p.sprite_list(r);
        assert!(list.contains(&0), "sprite should be on row {}", r);
    }
}

#[test]
fn e05_10_large_height() {
    let mut p = Ppu::new(Region::Ntsc);
    // obj_size 0 -> small=8x8, large=16x16
    p.write(0x01, 0x00);

    let mut img = [0u8; 544];
    hide_all(&mut img);
    spr(&mut img, 0, 10, 255, 0, 0, true); // large -> 16x16 at y=255
    load_oam(&mut p, &img);

    for r in 0..=15 {
        let (list, _) = p.sprite_list(r);
        assert!(list.contains(&0), "large sprite should be on row {}", r);
    }
    let (list16, _) = p.sprite_list(16);
    assert!(!list16.contains(&0));
}

#[test]
fn e05_10_x_range_small() {
    let mut p = Ppu::new(Region::Ntsc);
    // obj_size 0 -> small=8x8
    p.write(0x01, 0x00);

    let positions = [-7i16, -8, -9, -256, 255, 0];
    for (idx, x) in positions.iter().enumerate() {
        let mut img = [0u8; 544];
        hide_all(&mut img);
        spr(&mut img, idx, *x, 0, 0, 0, false); // y=0 -> visible on row 1
        load_oam(&mut p, &img);

        let (list, _) = p.sprite_list(1);
        if *x == -7 || *x == -256 || *x == 255 || *x == 0 {
            assert!(list.contains(&(idx as u8)), "x={} should be visible", x);
        } else {
            assert!(!list.contains(&(idx as u8)), "x={} should not be visible", x);
        }
    }
}

#[test]
fn e05_10_x_range_large() {
    let mut p = Ppu::new(Region::Ntsc);
    // obj_size 0 -> large=16x16
    p.write(0x01, 0x00);

    for (idx, x) in [-15i16, -16].iter().enumerate() {
        let mut img = [0u8; 544];
        hide_all(&mut img);
        spr(&mut img, idx, *x, 0, 0, 0, true); // large -> 16x16 at y=0
        load_oam(&mut p, &img);

        let (list, _) = p.sprite_list(1);
        if *x == -15 {
            assert!(list.contains(&(idx as u8)), "large x=-15 should be visible");
        } else {
            assert!(!list.contains(&(idx as u8)), "large x=-16 should not be visible");
        }
    }
}

#[test]
fn e05_10_limit_32() {
    let mut p = Ppu::new(Region::Ntsc);
    // obj_size 0 -> small=8x8
    p.write(0x01, 0x00);

    // 33 sprites all on row 1 (y=0)
    let mut img = [0u8; 544];
    hide_all(&mut img);
    for i in 0..=32 {
        spr(&mut img, i, 10 + i as i16, 0, 0, 0, false);
    }
    load_oam(&mut p, &img);

    let (list, overflow) = p.sprite_list(1);
    assert_eq!(list.len(), 32);
    assert!(overflow);
    // First 32 in evaluation order: indices 0..31
    for i in 0..32 {
        assert!(list.contains(&(i as u8)));
    }

    // Now only 32 sprites -> no overflow
    let mut img2 = [0u8; 544];
    hide_all(&mut img2);
    for i in 0..=31 {
        spr(&mut img2, i, 10 + i as i16, 0, 0, 0, false);
    }
    load_oam(&mut p, &img2);

    let (list2, overflow2) = p.sprite_list(1);
    assert_eq!(list2.len(), 32);
    assert!(!overflow2);
}

#[test]
fn e05_10_invisible_do_not_count() {
    let mut p = Ppu::new(Region::Ntsc);
    // obj_size 0 -> small=8x8
    p.write(0x01, 0x00);

    // 40 sprites: even indices on row 1 (y=0), odd indices at y=100 (not visible)
    let mut img = [0u8; 544];
    hide_all(&mut img);
    for i in 0..40 {
        if i % 2 == 0 {
            spr(&mut img, i, 10 + i as i16, 0, 0, 0, false); // visible on row 1
        } else {
            spr(&mut img, i, 10 + i as i16, 100, 0, 0, false); // not visible
        }
    }
    load_oam(&mut p, &img);

    let (list, overflow) = p.sprite_list(1);
    assert_eq!(list.len(), 20);
    assert!(!overflow);
    // Only even indices should be in the list
    for i in 0..40 {
        if i % 2 == 0 {
            assert!(list.contains(&(i as u8)), "even sprite {} should be visible", i);
        } else {
            assert!(!list.contains(&(i as u8)), "odd sprite {} should not be visible", i);
        }
    }
}

#[test]
fn e05_10_rotation_first_sprite() {
    let mut p = Ppu::new(Region::Ntsc);
    // obj_size 0 -> small=8x8
    p.write(0x01, 0x00);

    // Sprites 0 and 5 visible on row 1 (y=0), others hidden
    let mut img = [0u8; 544];
    hide_all(&mut img);
    spr(&mut img, 0, 10, 0, 0, 0, false);
    spr(&mut img, 5, 20, 0, 0, 0, false);
    load_oam(&mut p, &img);

    // Without rotation: evaluation starts at sprite 0 -> [0, 5]
    let (list1, _) = p.sprite_list(1);
    assert_eq!(list1, vec![0u8, 5]);

    // With rotation: OAMADD=10 -> internal=20 -> first=(20>>2)&0x7F=5
    // Evaluation order: 5,6,...,127,0,1,... -> [5, 0]
    p.write(0x02, 10);
    p.write(0x03, 0x80); // rotation bit set
    let (list2, _) = p.sprite_list(1);
    assert_eq!(list2, vec![5u8, 0]);

    // OAMADD=12 -> internal=24 -> first=(24>>2)&0x7F=6
    // Evaluation order: 6,...,127,0,1,...,5 -> [0, 5] (both after wrap)
    p.write(0x02, 12);
    let (list3, _) = p.sprite_list(1);
    assert_eq!(list3, vec![0u8, 5]);
}