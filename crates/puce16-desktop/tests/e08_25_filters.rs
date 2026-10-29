//! E08-25 tests for video filters, ratio names, and scanlines.

use puce16_desktop::video::*;

#[test]
fn e08_25_filter_names() {
    assert_eq!(Filter::from_name("linear"), Filter::Linear);
    assert_eq!(Filter::from_name("scanlines"), Filter::Scanlines);
    assert_eq!(Filter::from_name("nearest"), Filter::Nearest);
    assert_eq!(Filter::from_name(""), Filter::Nearest);
    assert_eq!(Filter::from_name("weird"), Filter::Nearest);
    assert_eq!(Filter::Linear.name(), "linear");
}

#[test]
fn e08_25_filter_cycle() {
    assert_eq!(Filter::Nearest.next(), Filter::Linear);
    assert_eq!(Filter::Linear.next(), Filter::Scanlines);
    assert_eq!(Filter::Scanlines.next(), Filter::Nearest);
}

#[test]
fn e08_25_ratio_names() {
    assert_eq!(ratio_from_name("par87"), Ratio::Par87);
    assert_eq!(ratio_from_name("square"), Ratio::Square);
    assert_eq!(ratio_from_name("x"), Ratio::Square);
    assert_eq!(ratio_name(Ratio::Par87), "par87");
}

#[test]
fn e08_25_scanlines_active() {
    assert!(!scanlines_active(Filter::Scanlines, 1));
    assert!(scanlines_active(Filter::Scanlines, 2));
    assert!(scanlines_active(Filter::Scanlines, 3));
    assert!(!scanlines_active(Filter::Nearest, 3));
    assert!(!scanlines_active(Filter::Linear, 5));
}

#[test]
fn e08_25_apply_scanlines_values() {
    // 2x4 image (width=2), all pixels [200,100,40,255]
    let mut rgba = vec![0u8; 2 * 4 * 4];
    for i in 0..rgba.len() / 4 {
        rgba[i * 4 + 0] = 200; // R
        rgba[i * 4 + 1] = 100; // G
        rgba[i * 4 + 2] = 40; // B
        rgba[i * 4 + 3] = 255; // A
    }
    apply_scanlines(&mut rgba, 2);

    // Row 0: unchanged
    assert_eq!(rgba[0], 200);
    assert_eq!(rgba[1], 100);
    assert_eq!(rgba[2], 40);
    // Row 1: darkened (200*3/4=150, 100*3/4=75, 40*3/4=30)
    let row1 = &rgba[8..16];
    assert_eq!(row1[0], 150);
    assert_eq!(row1[1], 75);
    assert_eq!(row1[2], 30);
    // Row 2: unchanged
    let row2 = &rgba[16..24];
    assert_eq!(row2[0], 200);
    assert_eq!(row2[1], 100);
    assert_eq!(row2[2], 40);
    // Row 3: darkened
    let row3 = &rgba[24..32];
    assert_eq!(row3[0], 150);
    assert_eq!(row3[1], 75);
    assert_eq!(row3[2], 30);
}

#[test]
fn e08_25_apply_scanlines_alpha() {
    // Alpha should remain unchanged on all rows
    let mut rgba = vec![0u8; 2 * 4 * 4];
    for i in 0..rgba.len() / 4 {
        rgba[i * 4 + 0] = 200;
        rgba[i * 4 + 1] = 100;
        rgba[i * 4 + 2] = 40;
        rgba[i * 4 + 3] = 255;
    }
    apply_scanlines(&mut rgba, 2);
    for i in (0..rgba.len()).step_by(4) {
        assert_eq!(rgba[i + 3], 255);
    }

    // Test with non-255 alpha
    let mut rgba2 = vec![0u8; 2 * 4 * 4];
    for i in 0..rgba2.len() / 4 {
        rgba2[i * 4 + 0] = 200;
        rgba2[i * 4 + 1] = 100;
        rgba2[i * 4 + 2] = 40;
        rgba2[i * 4 + 3] = 7;
    }
    apply_scanlines(&mut rgba2, 2);
    for i in (0..rgba2.len()).step_by(4) {
        assert_eq!(rgba2[i + 3], 7);
    }
}

#[test]
fn e08_25_apply_scanlines_odd_height() {
    // 3 rows: only row 1 should be modified, no panic
    let mut rgba = vec![0u8; 2 * 3 * 4];
    for i in 0..rgba.len() / 4 {
        rgba[i * 4 + 0] = 200;
        rgba[i * 4 + 1] = 100;
        rgba[i * 4 + 2] = 40;
        rgba[i * 4 + 3] = 255;
    }
    apply_scanlines(&mut rgba, 2);

    // Row 0: unchanged
    assert_eq!(rgba[0], 200);
    // Row 1: darkened
    let row1 = &rgba[8..16];
    assert_eq!(row1[0], 150);
    // Row 2: unchanged
    let row2 = &rgba[16..24];
    assert_eq!(row2[0], 200);
}

#[test]
fn e08_25_apply_scanlines_empty() {
    apply_scanlines(&mut [], 256); // no panic
    let mut empty = vec![0u8; 0];
    apply_scanlines(&mut empty, 0); // width == 0, no panic
}
