//! Deterministic FNV-1a 64-bit state hashing.
//!
//! Single implementation used for framebuffer hashes, full-state hashes and
//! save states (see docs/specs/06_TEST_ROMS.md section 6).

/// FNV-1a 64-bit hasher: offset basis `0xcbf29ce484222325`, prime `0x100000001b3`.
#[derive(Clone, Copy, Debug)]
pub struct Fnv64 {
    h: u64,
}

impl Default for Fnv64 {
    fn default() -> Self {
        Self::new()
    }
}

impl Fnv64 {
    /// FNV-1a 64 offset basis.
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    /// FNV-1a 64 prime.
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    /// Fresh hasher at the offset basis (hash of zero bytes).
    pub fn new() -> Self {
        Self { h: Self::OFFSET_BASIS }
    }

    /// Mix one byte: `h ^= b; h = h.wrapping_mul(PRIME)`.
    fn write_byte(&mut self, b: u8) {
        self.h ^= b as u64;
        self.h = self.h.wrapping_mul(Self::PRIME);
    }

    /// Mix a byte slice.
    pub fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.write_byte(b);
        }
    }

    /// Mix one `u8` (little-endian).
    pub fn write_u8(&mut self, v: u8) {
        self.write_byte(v);
    }

    /// Mix a `u16` in little-endian.
    pub fn write_u16(&mut self, v: u16) {
        self.write(&v.to_le_bytes());
    }

    /// Mix a `u32` in little-endian.
    pub fn write_u32(&mut self, v: u32) {
        self.write(&v.to_le_bytes());
    }

    /// Mix a `u64` in little-endian.
    pub fn write_u64(&mut self, v: u64) {
        self.write(&v.to_le_bytes());
    }

    /// Mix each element of the slice as little-endian `u16`.
    pub fn write_u16_slice(&mut self, s: &[u16]) {
        for &v in s {
            self.write_u16(v);
        }
    }

    /// Mix each element of the slice as little-endian `u32`.
    pub fn write_u32_slice(&mut self, s: &[u32]) {
        for &v in s {
            self.write_u32(v);
        }
    }

    /// Final hash value (FNV-1a 64 has no finalization step).
    pub fn finish(&self) -> u64 {
        self.h
    }
}
