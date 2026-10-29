use puce16_desktop::debugger::*;

#[test]
fn e08_35_bg_map_size() {
    let vram = vec![0u16; 0x8000];
    let cgram = vec![0u16; 256];

    // map_size 0: 32x32 screens, tile16 false -> 256x256
    let (w, h, _) = render_bg_map(&vram, &cgram, 0, 0, 0, false, 2);
    assert_eq!((w, h), (256, 256));

    // map_size 1: 64x32 screens -> 512x256
    let (w, h, _) = render_bg_map(&vram, &cgram, 0, 0, 1, false, 2);
    assert_eq!((w, h), (512, 256));

    // map_size 2: 32x64 screens -> 256x512
    let (w, h, _) = render_bg_map(&vram, &cgram, 0, 0, 2, false, 2);
    assert_eq!((w, h), (256, 512));

    // map_size 3: 64x64 screens -> 512x512
    let (w, h, _) = render_bg_map(&vram, &cgram, 0, 0, 3, false, 2);
    assert_eq!((w, h), (512, 512));

    // tile16 true, map_size 0 -> 512x512
    let (w, h, _) = render_bg_map(&vram, &cgram, 0, 0, 0, true, 2);
    assert_eq!((w, h), (512, 512));
}

#[test]
fn e08_35_bg_map_tile_lookup() {
    let mut vram = vec![0u16; 0x8000];
    let mut cgram = vec![0u16; 256];

    // Map entry at 0x100: char 1, palette 0
    vram[0x100] = 0x0001;

    // Tile 1 at chr_base 0: row 0 all index 1 (plan0=0xFF)
    vram[8] = 0x00FF;

    // cgram[1] = red (BGR555: 0x001F)
    cgram[1] = 0x001F;

    let (_, _, pixels) = render_bg_map(&vram, &cgram, 0x100, 0, 0, false, 2);

    // Row 0 should be red (255,0,0,255)
    for x in 0..8 {
        let offset = x * 4;
        assert_eq!(pixels[offset], 255, "R at x={}", x);
        assert_eq!(pixels[offset + 1], 0, "G at x={}", x);
        assert_eq!(pixels[offset + 2], 0, "B at x={}", x);
        assert_eq!(pixels[offset + 3], 255, "A at x={}", x);
    }
}

#[test]
fn e08_35_bg_map_hflip() {
    let mut vram = vec![0u16; 0x8000];
    let mut cgram = vec![0u16; 256];

    // Map entry with hflip (bit 14)
    vram[0] = 0x4001;

    // Tile: only leftmost pixel set in row 0 (plan0=0x80)
    vram[8] = 0x0080;

    cgram[1] = 0x001F;

    let (_, _, pixels) = render_bg_map(&vram, &cgram, 0, 0, 0, false, 2);

    // With hflip, colored pixel should be at x=7
    let offset = 7 * 4;
    assert_eq!(pixels[offset], 255, "R at x=7");
}

#[test]
fn e08_35_bg_map_vflip() {
    let mut vram = vec![0u16; 0x8000];
    let mut cgram = vec![0u16; 256];

    // Map entry with vflip (bit 15)
    vram[0] = 0x8001;

    // Tile: only row 0 non-zero
    vram[8] = 0x00FF;

    cgram[1] = 0x001F;

    let (_, _, pixels) = render_bg_map(&vram, &cgram, 0, 0, 0, false, 2);

    // With vflip, colored row should be at y=7
    let offset = (7 * 256 + 0) * 4;
    assert_eq!(pixels[offset], 255, "R at y=7");
}

#[test]
fn e08_35_bg_map_palette_bits() {
    let mut vram = vec![0u16; 0x8000];
    let mut cgram = vec![0u16; 256];

    // Map entry: char 1, palette 3 (bits 10-12)
    vram[0] = 0x0C01;

    // Tile with index 1 in row 0
    vram[8] = 0x00FF;

    // cgram[(3 << 2) + 1] = cgram[13] = blue
    cgram[13] = 0x7C00;

    let (_, _, pixels) = render_bg_map(&vram, &cgram, 0, 0, 0, false, 2);

    // Row 0 should be blue (0,0,255,255)
    let offset = 0 * 4;
    assert_eq!(pixels[offset], 0, "R");
    assert_eq!(pixels[offset + 1], 0, "G");
    assert_eq!(pixels[offset + 2], 255, "B");
}

#[test]
fn e08_35_bg_map_screens_64x32() {
    let mut vram = vec![0u16; 0x8000];
    let mut cgram = vec![0u16; 256];

    // Map size 1 (64x32): screen 2 entry at map_base + 0x400
    vram[0x400] = 0x0001;

    // Tile 1: row 0 all index 1
    vram[8] = 0x00FF;

    cgram[1] = 0x001F;

    let (_, _, pixels) = render_bg_map(&vram, &cgram, 0, 0, 1, false, 2);

    // Screen 2 is at x=256..384, y=0. First tile of screen 2 at x=256
    let offset = (0 * 512 + 256) * 4;
    assert_eq!(pixels[offset], 255, "R at screen 2");
}

#[test]
fn e08_35_bg_map_tile16_layout() {
    let mut vram = vec![0u16; 0x8000];
    let mut cgram = vec![0u16; 256];

    // Map entry at 0x100: char 0, no flip
    vram[0x100] = 0x0000;

    // Tile data for char 0 (top-left sub-tile): row 0 all index 1 (red)
    vram[0] = 0x00FF;
    // Tile data for char 1 (top-right sub-tile): row 0 all index 2 (green)
    vram[8] = 0xFF00;
    // Tile data for char 16 (bottom-left sub-tile): row 0 all index 3 (blue)
    vram[128] = 0xFFFF;

    cgram[1] = 0x001F; // red
    cgram[2] = 0x03E0; // green
    cgram[3] = 0x7C00; // blue

    let (_, _, pixels) = render_bg_map(&vram, &cgram, 0x100, 0, 0, true, 2);

    // Top-left (char 0): red at x=0
    let offset_tl = 0 * 4;
    assert_eq!(pixels[offset_tl], 255, "TL R");

    // Top-right (char 1): green at x=8
    let offset_tr = 8 * 4;
    assert_eq!(pixels[offset_tr + 1], 255, "TR G");

    // Bottom-left (char 16): blue at y=8
    let offset_bl = (8 * 512) * 4;
    assert_eq!(pixels[offset_bl + 2], 255, "BL B");
}

#[test]
fn e08_35_bg_map_empty_vram() {
    let (w, h, pixels) = render_bg_map(&[], &[], 0, 0, 0, false, 2);
    assert_eq!((w, h), (256, 256));

    // All black opaque
    for p in pixels.chunks(4) {
        assert_eq!(p[0], 0);
        assert_eq!(p[1], 0);
        assert_eq!(p[2], 0);
        assert_eq!(p[3], 255);
    }
}

#[test]
fn e08_35_mode7_map() {
    let mut vram = vec![0u16; 0x8000];
    let mut cgram = vec![0u16; 256];

    // Map entry at (0,0): tile 5
    vram[0] = 0x0005;

    // Tile 5 pixel (0,0): index 9 (high byte)
    vram[5 * 64 + 0] = 0x0900;

    cgram[9] = 0x7C00; // blue

    let (w, h, pixels) = render_mode7_map(&vram, &cgram);
    assert_eq!((w, h), (1024, 1024));

    // Pixel (0,0) should be blue
    let offset = 0 * 4;
    assert_eq!(pixels[offset], 0, "R");
    assert_eq!(pixels[offset + 1], 0, "G");
    assert_eq!(pixels[offset + 2], 255, "B");
}

#[test]
fn e08_35_oam_decode_basic() {
    let mut oam = vec![0u8; 544];
    // Sprite 0: x=16, y=0x20, tile=0x05, attrs=0x31 (N=1, pp=01, oo=11)
    oam[0] = 0x10;
    oam[1] = 0x20;
    oam[2] = 0x05;
    oam[3] = 0x31;

    let sprites = decode_oam(&oam);
    assert_eq!(sprites.len(), 128);

    let s = sprites[0];
    assert_eq!(s.index, 0);
    assert_eq!(s.x, 16);
    assert_eq!(s.y, 0x20);
    assert_eq!(s.tile, 0x105);
    assert_eq!(s.palette, 0);
    assert_eq!(s.priority, 3);
    assert!(!s.hflip);
    assert!(!s.vflip);
    assert!(!s.large);
}

#[test]
fn e08_35_oam_decode_high_table() {
    let mut oam = vec![0u8; 544];
    // Sprite 1: x low=0xFF, high table bit for x8=1 -> x=-1
    oam[4] = 0xFF;
    // High table byte at 512: sprite 1 bits are shift 2 (bits 2-3)
    // bit 0 of pair = x8, bit 1 = large. Set both to 1 -> 0b11 << 2 = 0xC
    oam[512] = 0x0C;

    let sprites = decode_oam(&oam);

    let s1 = sprites[1];
    assert_eq!(s1.x, -1);
    assert!(s1.large);

    // Sprite 0 unchanged
    let s0 = sprites[0];
    assert_eq!(s0.x, 0);
    assert!(!s0.large);
}

#[test]
fn e08_35_oam_decode_flips_palette() {
    let mut oam = vec![0u8; 544];
    // Sprite 0: attrs=0xE6 (v=1,h=1,oo=2,ppp=3,N=0)
    oam[3] = 0xE6;

    let sprites = decode_oam(&oam);
    let s = sprites[0];

    assert!(s.vflip);
    assert!(s.hflip);
    assert_eq!(s.priority, 2);
    assert_eq!(s.palette, 3);
}

#[test]
fn e08_35_oam_short_input() {
    let sprites = decode_oam(&[]);
    assert_eq!(sprites.len(), 128);
    for s in &sprites {
        assert_eq!(s.x, 0);
        assert_eq!(s.y, 0);
        assert_eq!(s.tile, 0);
    }

    let sprites = decode_oam(&[0; 544]);
    assert_eq!(sprites.len(), 128);
}

#[test]
fn e08_35_sprite_dims() {
    assert_eq!(sprite_dims(0, false), (8, 8));
    assert_eq!(sprite_dims(0, true), (16, 16));
    assert_eq!(sprite_dims(3, true), (32, 32));
    assert_eq!(sprite_dims(5, false), (32, 32));
    assert_eq!(sprite_dims(5, true), (64, 64));
    assert_eq!(sprite_dims(6, false), (16, 32));
    assert_eq!(sprite_dims(6, true), (32, 64));
    assert_eq!(sprite_dims(7, false), (16, 32));
    assert_eq!(sprite_dims(7, true), (32, 32));
}
