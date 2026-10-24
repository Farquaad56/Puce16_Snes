//! E01-10 tests for the pure video helpers (XRGB -> RGBA, display size).

use puce16_desktop::video::*;

#[test]
fn e01_10_rgba_basic() {
    let rgba = xrgb_to_rgba(&[0xFF102030, 0x00FFFFFF]);
    assert_eq!(rgba, [0x10, 0x20, 0x30, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
}

#[test]
fn e01_10_display_square() {
    assert_eq!(display_size(256, 224, 3, Ratio::Square), (768, 672));
}

#[test]
fn e01_10_display_87() {
    // 768 * 8 / 7 = 877 (integer division).
    assert_eq!(display_size(256, 224, 3, Ratio::Par87), (877, 672));
}

#[test]
fn e01_10_fit_scale() {
    assert_eq!(fit_scale(256, 224, 1000, 700), 3);
    assert_eq!(fit_scale(256, 224, 100, 100), 1);
    assert_eq!(fit_scale(512, 448, 2048, 896), 2);
}
