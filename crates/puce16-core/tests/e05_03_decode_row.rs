use puce16_core::ppu::bg::decode_row;

fn make_vram() -> Vec<u16> {
    vec![0; 0x8000]
}

#[test]
fn e05_03_bpp2_pattern() {
    let mut vram = make_vram();
    vram[0] = 0xCCAA; // plane0=0xAA, plane1=0xCC
    assert_eq!(decode_row(&vram, 0, 2), [3, 2, 1, 0, 3, 2, 1, 0]);
}

#[test]
fn e05_03_bpp2_zero_and_full() {
    let mut vram = make_vram();
    vram[0] = 0x0000;
    assert_eq!(decode_row(&vram, 0, 2), [0, 0, 0, 0, 0, 0, 0, 0]);

    vram[0] = 0xFFFF;
    assert_eq!(decode_row(&vram, 0, 2), [3, 3, 3, 3, 3, 3, 3, 3]);

    vram[0] = 0x0080;
    assert_eq!(decode_row(&vram, 0, 2), [1, 0, 0, 0, 0, 0, 0, 0]);

    vram[0] = 0x0001;
    assert_eq!(decode_row(&vram, 0, 2), [0, 0, 0, 0, 0, 0, 0, 1]);

    vram[0] = 0x8000;
    assert_eq!(decode_row(&vram, 0, 2), [2, 0, 0, 0, 0, 0, 0, 0]);
}

#[test]
fn e05_03_bpp4_planes() {
    let mut vram = make_vram();
    vram[0] = 0xCCAA; // plane0=0xAA, plane1=0xCC
    vram[8] = 0x00F0; // plane2=0xF0, plane3=0
    assert_eq!(decode_row(&vram, 0, 4), [7, 6, 5, 4, 3, 2, 1, 0]);

    vram[8] = 0xFFF0; // plane2=0xF0, plane3=0xFF
    assert_eq!(decode_row(&vram, 0, 4), [15, 14, 13, 12, 11, 10, 9, 8]);
}

#[test]
fn e05_03_bpp8_planes() {
    let mut vram = make_vram();
    // All zeros except specific planes set
    vram[0] = 0x0000;   // plane0=0, plane1=0
    vram[8] = 0x0000;   // plane2=0, plane3=0
    vram[16] = 0x0180;  // plane4=0x80, plane5=0x01
    vram[24] = 0x0040;  // plane6=0x40, plane7=0
    assert_eq!(decode_row(&vram, 0, 8), [16, 64, 0, 0, 0, 0, 0, 32]);
}

#[test]
fn e05_03_row_offset() {
    let mut vram = make_vram();
    // Test row offset in 2bpp
    vram[0x0100] = 0x0080;
    assert_eq!(decode_row(&vram, 0x0100, 2)[0], 1);
    assert_eq!(decode_row(&vram, 0x0101, 2), [0, 0, 0, 0, 0, 0, 0, 0]);

    // Test row offset in 4bpp (planes 2/3 at +8)
    vram[0x0100] = 0;
    vram[0x0108] = 0x0080;
    assert_eq!(decode_row(&vram, 0x0100, 4)[0], 4);
}

#[test]
fn e05_03_wrap_at_end() {
    let mut vram = make_vram();
    // Test VRAM wrap: a=0x7FFF in 4bpp -> planes 2/3 at (0x7FFF+8)&0x7FFF = 7
    vram[7] = 0x0080;
    assert_eq!(decode_row(&vram, 0x7FFF, 4)[0], 4);
}

#[test]
#[should_panic(expected = "invalid bpp")]
fn e05_03_bad_bpp() {
    let vram = make_vram();
    decode_row(&vram, 0, 3);
}
