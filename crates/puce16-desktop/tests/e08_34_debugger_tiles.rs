use puce16_desktop::debugger::*;

#[test]
fn e08_34_color_conversion() {
    assert_eq!(bgr555_to_rgba(0x0000), [0, 0, 0, 255]);
    assert_eq!(bgr555_to_rgba(0x7FFF), [255, 255, 255, 255]);
    assert_eq!(bgr555_to_rgba(0x001F), [255, 0, 0, 255]);
    assert_eq!(bgr555_to_rgba(0x03E0), [0, 255, 0, 255]);
    assert_eq!(bgr555_to_rgba(0x7C00), [0, 0, 255, 255]);
    // 16 << 3 | 16 >> 2 = 128 + 4 = 132
    assert_eq!(bgr555_to_rgba(0x0010), [132, 0, 0, 255]);
}

#[test]
fn e08_34_tile_words() {
    assert_eq!(tile_words(2), 8);
    assert_eq!(tile_words(4), 16);
    assert_eq!(tile_words(8), 32);
    assert_eq!(tile_words(3), 8);
}

#[test]
fn e08_34_decode_2bpp() {
    let mut vram = vec![0u16; 16];
    // Row 0: plan0=0xFF, plan1=0x00 -> all pixels index 1
    vram[0] = 0x00FF;
    // Row 1: plan0=0x00, plan1=0xFF -> all pixels index 2
    vram[1] = 0xFF00;

    let mut out = [0u8; 64];
    decode_tile(&vram, 0, 2, &mut out);

    // Row 0: eight 1s
    for i in 0..8 {
        assert_eq!(out[i], 1);
    }
    // Row 1: eight 2s
    for i in 8..16 {
        assert_eq!(out[i], 2);
    }
    // Rows 2-7: all zeros
    for i in 16..64 {
        assert_eq!(out[i], 0);
    }
}

#[test]
fn e08_34_decode_2bpp_bits() {
    let mut vram = vec![0u16; 8];
    // plan0=0x80 (bit7), plan1=0x01 (bit0)
    vram[0] = 0x0180;

    let mut out = [0u8; 64];
    decode_tile(&vram, 0, 2, &mut out);

    // Row 0: pixel 0 has bit7 of plan0 -> index 1
    //         pixels 1-6 have no bits set -> index 0
    //         pixel 7 has bit0 of plan1 -> index 2
    assert_eq!(&out[0..8], &[1, 0, 0, 0, 0, 0, 0, 2]);
}

#[test]
fn e08_34_decode_4bpp_planes() {
    let mut vram = vec![0u16; 16];
    // Planes 2/3 at words 8-15: plane2=0xFF, plane3=0x00 -> all pixels index 4
    vram[8] = 0x00FF;

    let mut out = [0u8; 64];
    decode_tile(&vram, 0, 4, &mut out);

    // Row 0: eight 4s (bit2 set)
    for i in 0..8 {
        assert_eq!(out[i], 4);
    }

    // Now add plane1=0xFF -> all pixels index 6 (bits 1 and 2 set)
    vram[0] = 0xFF00;
    decode_tile(&vram, 0, 4, &mut out);
    for i in 0..8 {
        assert_eq!(out[i], 6);
    }
}

#[test]
fn e08_34_decode_8bpp_planes() {
    let mut vram = vec![0u16; 32];
    // Plane 6 at word 24: bit0 set -> pixel 7 of row 0 gets index 64
    vram[24] = 0x0001;

    let mut out = [0u8; 64];
    decode_tile(&vram, 0, 8, &mut out);
    assert_eq!(out[7], 64); // row 0, pixel 7

    // Plane 4 at word 17 (block 2, row 1): bit7 set -> pixel 0 of row 1 gets index 16
    vram[17] = 0x0080;
    decode_tile(&vram, 0, 8, &mut out);
    assert_eq!(out[8], 16); // row 1, pixel 0
}

#[test]
fn e08_34_decode_wraps() {
    let mut vram = vec![0u16; 0x8000];
    // Row 0 at word 0x7FFF: plan0=0xFF -> all pixels index 1
    vram[0x7FFF] = 0x00FF;
    // Row 1 wraps to word 0: plan1=0xFF -> all pixels index 2
    vram[0] = 0xFF00;

    let mut out = [0u8; 64];
    decode_tile(&vram, 0x7FFF, 2, &mut out);

    // Row 0: eight 1s
    for i in 0..8 {
        assert_eq!(out[i], 1);
    }
    // Row 1: eight 2s (wrapped)
    for i in 8..16 {
        assert_eq!(out[i], 2);
    }
}

#[test]
fn e08_34_render_tiles_layout() {
    let mut vram = vec![0u16; 0x8000];
    // Tile at base+0: all pixels index 0 (black)
    // Tile at base+8: all pixels index 1 -> should be red from cgram[1]
    for row in 0..8 {
        vram[row + 8] = 0x00FF; // plan0=0xFF, plan1=0x00 -> index 1
    }

    let mut cgram = vec![0u16; 256];
    cgram[1] = 0x001F; // red in BGR555

    let (w, h, pixels) = render_tiles(&vram, &cgram, 0, 2, 0, 2, 1);
    assert_eq!(w, 16);
    assert_eq!(h, 8);
    assert_eq!(pixels.len(), 16 * 8 * 4);

    // First tile (x=0..7) should be black (index 0 -> cgram[0]=black)
    for y in 0..8 {
        for x in 0..8 {
            let offset = (y * w + x) * 4;
            assert_eq!(pixels[offset], 0);
            assert_eq!(pixels[offset + 1], 0);
            assert_eq!(pixels[offset + 2], 0);
        }
    }

    // Second tile (x=8..15) should be red (index 1 -> cgram[1]=red)
    for y in 0..8 {
        for x in 8..16 {
            let offset = (y * w + x) * 4;
            assert_eq!(pixels[offset], 255); // R
            assert_eq!(pixels[offset + 1], 0);   // G
            assert_eq!(pixels[offset + 2], 0);   // B
        }
    }
}

#[test]
fn e08_34_render_tiles_palette() {
    let mut vram = vec![0u16; 0x8000];
    // Tile with all pixels index 3 (binary 0011): plan0=0xFF, plan1=0xFF
    for row in 0..8 {
        vram[row] = 0xFFFF;   // plan0=0xFF, plan1=0xFF -> bits 0 and 1 set everywhere
    }

    let mut cgram = vec![0u16; 256];
    // Palette 2, index 3: color at cgram[2*16+3] = cgram[35]
    cgram[35] = 0x03E0; // green

    let (w, h, pixels) = render_tiles(&vram, &cgram, 0, 4, 2, 1, 1);
    assert_eq!(w, 8);
    assert_eq!(h, 8);

    // All pixels should be green (index 3 from palette 2 -> cgram[35])
    for y in 0..8 {
        for x in 0..8 {
            let offset = (y * w + x) * 4;
            assert_eq!(pixels[offset], 0);     // R
            assert_eq!(pixels[offset + 1], 255); // G
            assert_eq!(pixels[offset + 2], 0);   // B
        }
    }

    // For 8bpp, palette is ignored - use cgram[idx] directly
    let mut vram8 = vec![0u16; 0x8000];
    for row in 0..8 {
        vram8[row + 24] = 0x00FF; // plane6=0xFF -> index 64 everywhere
    }
    cgram[64] = 0x7C00; // blue

    let (w, _h, pixels) = render_tiles(&vram8, &cgram, 0, 8, 5, 1, 1);
    for y in 0..8 {
        for x in 0..8 {
            let offset = (y * w + x) * 4;
            assert_eq!(pixels[offset], 0);     // R
            assert_eq!(pixels[offset + 1], 0);   // G
            assert_eq!(pixels[offset + 2], 255); // B
        }
    }
}

#[test]
fn e08_34_render_tiles_empty_vram() {
    let (w, h, pixels) = render_tiles(&[], &[], 0, 4, 0, 4, 4);
    assert_eq!(w, 32);
    assert_eq!(h, 32);
    // All black opaque
    for p in pixels.chunks(4) {
        assert_eq!(p[0], 0);
        assert_eq!(p[1], 0);
        assert_eq!(p[2], 0);
        assert_eq!(p[3], 255);
    }
}

#[test]
fn e08_34_palette_rgba() {
    let mut cgram = vec![0u16; 256];
    cgram[5] = 0x03E0; // green

    let colors = palette_rgba(&cgram);
    assert_eq!(colors.len(), 256);
    assert_eq!(colors[5], [0, 255, 0, 255]);

    // Empty CGRAM -> all black
    let empty_colors = palette_rgba(&[]);
    assert_eq!(empty_colors.len(), 256);
    for c in &empty_colors {
        assert_eq!(*c, [0, 0, 0, 255]);
    }
}

#[test]
fn e08_34_hex_rows_format() {
    let data = vec![0x48, 0x69, 0x00, 0xFF];
    let lines = hex_rows(&data, 0x7E0000);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], "7E0000: 48 69 00 FF  Hi..");
}

#[test]
fn e08_34_hex_rows_multi() {
    let data: Vec<u8> = (0..20).collect();
    let lines = hex_rows(&data, 0x10);
    assert_eq!(lines.len(), 2);
    // Second line starts at offset 16 (0x10) from base 0x10 -> address 0x20
    assert!(lines[1].starts_with("000020: "));
    // Second line has 4 bytes
    let parts: Vec<&str> = lines[1].splitn(3, ": ").collect();
    let hex_and_ascii = parts[1];
    let hex_bytes = hex_and_ascii.split_whitespace().take_while(|s| s.len() == 2).count();
    assert_eq!(hex_bytes, 4);
}

#[test]
fn e08_34_flags_string() {
    // N=0,V=0,M=1,X=1,D=0,I=1,Z=0,C=0 -> nvMXdIzc
    assert_eq!(flags_string(0x34, true), "nvMXdIzc e=1");
    // All bits set -> NVMXDIZC
    assert_eq!(flags_string(0xFF, false), "NVMXDIZC e=0");
    // No bits set -> nvmxdizc
    assert_eq!(flags_string(0x00, false), "nvmxdizc e=0");
}
