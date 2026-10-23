//! Top-level emulator state (spec 01_ARCHITECTURE section 5).
//!
//! E01-06 adds a temporary test-pattern constructor for milestone D0; it is
//! removed/replaced when the real PPU renders an image.

use crate::hash::Fnv64;
use crate::ppu::output::{make_view, FrameBuf, FrameView};
use crate::region::Region;

/// Test pattern width in columns (E01-06).
const TEST_W: usize = 256;
/// Test pattern height in rows, NTSC and PAL alike (E01-06).
const TEST_H: usize = 224;

pub struct Snes {
    region: Region,
    frame: u64,
    fb: FrameBuf,
    joy: [u16; 2],
    test_pattern: bool,
    /// Compact copy of the useful area (stride `view_w`), refreshed by run_frame.
    compact: Vec<u32>,
}

impl Snes {
    /// Temporary constructor for milestone D0: frame 0, black 256x224 view,
    /// test pattern enabled.
    pub fn new_test_pattern(region: Region) -> Snes {
        let fb = FrameBuf::new(); // black buffer, default view 256x224
        Self {
            region,
            frame: 0,
            compact: fb.compact(),
            fb,
            joy: [0; 2],
            test_pattern: true,
        }
    }

    /// Advance one frame: `frame += 1`, then draw the test pattern if enabled.
    pub fn run_frame(&mut self) {
        self.frame += 1;
        if self.test_pattern {
            let f = (self.frame % 256) as u32;
            for y in 0..TEST_H {
                for x in 0..TEST_W {
                    let argb = 0xFF00_0000
                        | (((x as u32 + f) & 0xFF) << 16)
                        | ((y as u32 & 0xFF) << 8)
                        | 0x40;
                    self.fb.set(x, y, argb);
                }
            }
            if self.joy[0] != 0 {
                self.fb.set(0, 0, 0xFFFF_FFFF);
            }
        }
        self.compact = self.fb.compact();
    }

    /// Compact view of the current frame (256x224 for the test pattern).
    pub fn frame(&self) -> FrameView<'_> {
        make_view(self.fb.view_w, self.fb.view_h, &self.compact)
    }

    /// Number of frames run so far.
    pub fn frame_count(&self) -> u64 {
        self.frame
    }

    /// Set the buttons for port 0 or 1; other ports are ignored.
    pub fn set_joypad(&mut self, port: usize, buttons: u16) {
        if port < self.joy.len() {
            self.joy[port] = buttons;
        }
    }

    /// Buttons for port 0 or 1; other ports read as no button (0).
    pub fn joypad(&self, port: usize) -> u16 {
        if port < self.joy.len() {
            self.joy[port]
        } else {
            0
        }
    }

    /// Audio samples for the frame; does nothing for now.
    pub fn take_audio(&mut self, _out: &mut Vec<[i16; 2]>) {}

    /// Deterministic state hash: frame (u64), joy[0], joy[1], FrameBuf.
    pub fn state_hash(&self) -> u64 {
        let mut h = Fnv64::new();
        h.write_u64(self.frame);
        h.write_u16(self.joy[0]);
        h.write_u16(self.joy[1]);
        self.fb.hash(&mut h);
        h.finish()
    }

    /// Region of this console.
    pub fn region(&self) -> Region {
        self.region
    }
}
