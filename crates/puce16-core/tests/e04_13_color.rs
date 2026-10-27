use puce16_core::ppu::output::{channel8, bgr555_to_xrgb, output_row};

#[test]
fn e04_13_channel8_extremes() {
    assert_eq!(channel8(31, 15), 255);
    assert_eq!(channel8(31, 0), 0);
    assert_eq!(channel8(0, 15), 0);
    assert_eq!(channel8(1, 15), 8);
    assert_eq!(channel8(16, 15), 132);
    assert_eq!(channel8(31, 1), 16);
    assert_eq!(channel8(31, 7), 115);
    assert_eq!(channel8(31, 8), 132);
    assert_eq!(channel8(10, 5), 24);
}

#[test]
fn e04_13_channel8_clamp() {
    // brightness > 15 is clamped to 15
    assert_eq!(channel8(31, 200), 255);
}

#[test]
fn e04_13_primary_colors() {
    // white (all channels max)
    assert_eq!(bgr555_to_xrgb(0x7FFF, 15), 0xFFFFFFFF);
    // red only
    assert_eq!(bgr555_to_xrgb(0x001F, 15), 0xFFFF0000);
    // green only
    assert_eq!(bgr555_to_xrgb(0x03E0, 15), 0xFF00FF00);
    // blue only
    assert_eq!(bgr555_to_xrgb(0x7C00, 15), 0xFF0000FF);
    // black (all channels zero)
    assert_eq!(bgr555_to_xrgb(0x0000, 15), 0xFF000000);
}

#[test]
fn e04_13_brightness_scaling() {
    // brightness 0 -> black regardless of color
    assert_eq!(bgr555_to_xrgb(0x7FFF, 0), 0xFF000000);
    // white at brightness 8: c5b = 31*8/15 = 16, (16<<3)|(16>>2) = 132 = 0x84
    assert_eq!(bgr555_to_xrgb(0x7FFF, 8), 0xFF848484);
    // r=g=b=1 at brightness 15: c5b = 1*15/15 = 1, (1<<3)|(1>>2) = 8 = 0x08
    assert_eq!(bgr555_to_xrgb(0x0421, 15), 0xFF080808);
}

#[test]
fn e04_13_bit15_ignored() {
    // bit 15 of rgb is ignored (it's the unused MSB in BGR555)
    assert_eq!(bgr555_to_xrgb(0x801F, 15), bgr555_to_xrgb(0x001F, 15));
}

#[test]
fn e04_13_output_row() {
    // non-interlaced: row maps directly
    assert_eq!(output_row(0, false, false), 0);
    assert_eq!(output_row(5, true, false), 5);
    // interlaced: field=false -> even rows, field=true -> odd rows
    assert_eq!(output_row(5, false, true), 10);
    assert_eq!(output_row(5, true, true), 11);
    // edge cases for full frame height
    assert_eq!(output_row(237, true, true), 475);
    assert_eq!(output_row(223, false, true), 446);
}
