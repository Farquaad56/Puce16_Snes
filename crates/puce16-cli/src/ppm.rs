//! PPM (P6) encoding of XRGB8888 frames (E01-07).

/// Encode a frame as binary PPM (`P6`).
///
/// The header is the ASCII string `P6\n<w> <h>\n255\n`, followed by one R, G,
/// B byte per pixel: R = bits 23-16, G = bits 15-8, B = bits 7-0 (the X/alpha
/// byte is discarded). Panics if `pixels.len() != width * height`.
pub fn encode_ppm(width: usize, height: usize, pixels: &[u32]) -> Vec<u8> {
    if pixels.len() != width * height {
        panic!(
            "encode_ppm: expected {} pixels ({}x{}), got {}",
            width * height,
            width,
            height,
            pixels.len()
        );
    }
    let mut out = Vec::with_capacity(15 + pixels.len() * 3);
    out.extend_from_slice(format!("P6\n{width} {height}\n255\n").as_bytes());
    for &p in pixels {
        out.push((p >> 16) as u8); // R = bits 23-16
        out.push((p >> 8) as u8); // G = bits 15-8
        out.push(p as u8); // B = bits 7-0
    }
    out
}
