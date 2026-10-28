// Period counter tables shared by envelopes and noise (spec 07b section 2).

pub const PERIOD: [u16; 32] = [
    0, 2048, 1536, 1280, 1024, 768, 640, 512, 384, 320, 256, 192, 160, 128, 96, 80,
    64, 48, 40, 32, 24, 20, 16, 12, 10, 8, 6, 5, 4, 3, 2, 1,
];

/// Offset applied to the counter before modulo (spec 07b section 2).
pub fn offset(rate: u8) -> u16 {
    if rate == 0 { return 0; }
    match rate % 3 {
        1 => 0,
        2 => 1040,
        _ => 536, // rate % 3 == 0
    }
}

/// Returns true when the period counter has reached its trigger point.
pub fn poll(counter: u16, rate: u8) -> bool {
    let r = (rate & 31) as usize;
    if r == 0 { return false; }
    ((counter as u32 + offset(rate) as u32) % PERIOD[r] as u32) == 0
}
