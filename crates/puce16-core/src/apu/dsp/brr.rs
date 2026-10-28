/// Clamp value to signed 16-bit range.
pub fn clamp16(v: i32) -> i32 {
    v.max(-32768).min(32767)
}

/// Decode a group of 4 BRR samples from 2 bytes.
/// b0 = high byte (nibbles 1-2), b1 = low byte (nibbles 3-4).
/// Returns (4 decoded samples at 16-bit scale, new p1, new p2).
pub fn decode_group(b0: u8, b1: u8, shift: u8, filter: u8, p1: i32, p2: i32) -> ([i32; 4], i32, i32) {
    let mut nyb = ((b0 as u16) << 8 | b1 as u16) as i16;
    let filter = filter & 3;
    let mut p1 = p1;
    let mut p2 = p2;
    let mut out = [0i32; 4];

    for i in 0..4 {
        // Extract signed nibble via arithmetic right shift
        let s = (nyb >> 12) as i32;
        nyb <<= 4;

        // Scale based on shift value
        let s = if shift > 12 {
            (s >> 4) << 11
        } else {
            (s << shift) >> 1
        };

        // Apply filter coefficients
        let s = match filter {
            0 => s,
            1 => s + (p1 >> 1) + ((-p1) >> 5),
            2 => s + p1 - p2 + (p2 >> 4) + ((p1 * -3) >> 6),
            3 => s + p1 - p2 + ((p1 * -13) >> 7) + ((p2 * 3) >> 4),
            _ => unreachable!(),
        };

        // Clamp to 16-bit range, then multiply by 2 with wraparound
        let s = clamp16(s);
        let s = (s * 2) as i16;
        out[i] = s as i32;

        // Update history for next sample
        p2 = p1 >> 1;
        p1 = s as i32;
    }

    (out, p1, p2)
}
