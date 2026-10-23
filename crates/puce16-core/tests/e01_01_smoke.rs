#[test]
fn e01_01_lib_exports() {
    assert!(!puce16_core::VERSION.is_empty());
    assert_ne!(puce16_core::Region::Ntsc, puce16_core::Region::Pal);
    let fv = puce16_core::FrameView { width: 1, height: 1, pixels: &[0] };
    assert_eq!(fv.width, 1);
    assert_eq!(fv.height, 1);
    assert_eq!(fv.pixels.len(), 1);
}
