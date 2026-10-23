//! Timing constants shared by the whole core (docs/specs/02_BUS_TIMING.md section 1).

pub const MASTER_HZ_NTSC: u64 = 21_477_272;
pub const MASTER_HZ_PAL:  u64 = 21_281_370;
pub const APU_RESONATOR_HZ: u64 = 24_606_720; // D-004
pub const CLOCKS_PER_LINE: u32 = 1364;        // normal line
pub const CLOCKS_SHORT_LINE: u32 = 1360;      // NTSC, interlace off, field=1, V=240
pub const CLOCKS_LONG_LINE: u32 = 1368;       // PAL, interlace on, field=1, V=311
pub const DOTS_PER_LINE: u32 = 340;           // H = 0..339 (long line: 341 dots, H = 0..340)
pub const LINES_NTSC: u32 = 262;              // interlace line count handled by the PPU (05_PPU.md section 5.2)
pub const LINES_PAL: u32 = 312;
pub const VBLANK_START_NORMAL: u32 = 225;
pub const VBLANK_START_OVERSCAN: u32 = 240;
pub const DRAM_REFRESH_CLOCKS: u32 = 40;
pub const DRAM_REFRESH_HCLOCK: u32 = 538;     // UNKNOWN - to confirm (D-018 provisional)
pub const VISIBLE_DOT_FIRST: u32 = 22;        // first pixel at clock 88 = dot 22
pub const VISIBLE_CLOCK_FIRST: u32 = 88;
pub const VISIBLE_CLOCK_END: u32 = 1112;      // exclusive: 256 pixels * 4 clocks

// PPU write phase relative to the CPU (05_PPU.md section 0).
pub const PPU_WRITE_PHASE: u32 = 0; // UNKNOWN - to confirm
// H/V latch offset applied to the dot counter (05_PPU.md section 4.1).
pub const HV_LATCH_DOT_OFFSET: u32 = 0; // UNKNOWN - to confirm

// Memory sizes.
pub const WRAM_SIZE: usize = 0x20000;
pub const VRAM_WORDS: usize = 0x8000;
pub const CGRAM_WORDS: usize = 256;
pub const OAM_BYTES: usize = 544;
pub const APU_RAM_SIZE: usize = 0x10000;

// Framebuffer (XRGB8888, 05_PPU.md section 6).
pub const FB_WIDTH: usize = 512;
pub const FB_HEIGHT: usize = 478;
