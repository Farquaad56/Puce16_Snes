//! Top-level emulator state (spec 01_ARCHITECTURE section 5).
//!
//! E03-34: Snes enum with Pattern and Real variants. The test-pattern variant
//! preserves the E01-06 API; the real variant loads a cartridge and runs CPU/bus.

use crate::cart::{Cart, CartError, MapMode};
use crate::cpu::Cpu;
use crate::hash::Fnv64;
use crate::ppu::output::{make_view, FrameBuf, FrameView};
use crate::region::Region;
use crate::bus::SysBus;

/// Test pattern width in columns (E01-06).
const TEST_W: usize = 256;
/// Test pattern height in rows, NTSC and PAL alike (E01-06).
const TEST_H: usize = 224;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LoadError { Cart(CartError) }

impl From<CartError> for LoadError {
    fn from(e: CartError) -> Self { LoadError::Cart(e) }
}

enum Inner {
    Pattern { fb: FrameBuf, compact: Vec<u32> },
    Real { cpu: Cpu, sys: Box<SysBus>, fb: FrameBuf, compact: Vec<u32> },
}

pub struct Snes {
    region: Region,
    frame: u64,
    joy: [u16; 2],
    inner: Inner,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DebugSnapshot {
    pub pc: u16,
    pub pb: u8,
    pub a: u16,
    pub x: u16,
    pub y: u16,
    pub s: u16,
    pub d: u16,
    pub db: u8,
    pub p: u8,
    pub e: bool,
    pub v: u16,
    pub dot: u16,
    pub master: u64,
    pub frame: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CartInfo {
    pub title: String,
    pub map: MapMode,
    pub region: Region,
    pub rom_len: usize,
    pub sram_len: usize,
    pub battery: bool,
}

impl Snes {
    /// Temporary constructor for milestone D0: frame 0, black 256x224 view,
    /// test pattern enabled.
    pub fn new_test_pattern(region: Region) -> Snes {
        let fb = FrameBuf::new(); // black buffer, default view 256x224
        Self {
            region,
            frame: 0,
            joy: [0; 2],
            inner: Inner::Pattern { compact: fb.compact(), fb },
        }
    }

    /// Load a cartridge and create a real SNES. Region is auto-detected from header.
    pub fn new(rom: &[u8], region: Option<Region>) -> Result<Snes, LoadError> {
        Self::new_with(rom, region, None)
    }

    /// Load with optional forced mapper mode and/or region.
    pub fn new_with(
        rom: &[u8],
        region: Option<Region>,
        map: Option<MapMode>,
    ) -> Result<Snes, LoadError> {
        let cart = Cart::from_bytes_forced(rom, map, region)?;
        let region = cart.region();
        let mut sys = Box::new(SysBus::new(cart, region));
        let mut cpu = Cpu::new();
        cpu.reset(&mut *sys);
        let fb = FrameBuf::new();
        Ok(Self {
            region,
            frame: 0,
            joy: [0; 2],
            inner: Inner::Real { cpu, sys, compact: fb.compact(), fb },
        })
    }

    /// Reset the console. Pattern variant resets to black buffer; real variant
    /// re-runs CPU reset vector fetch. Joypad state is preserved in both cases.
    pub fn reset(&mut self) {
        match &mut self.inner {
            Inner::Pattern { fb, .. } => {
                *fb = FrameBuf::new();
                self.frame = 0;
            }
            Inner::Real { cpu, sys, fb, .. } => {
                sys.reset();
                cpu.reset(&mut **sys);
                *fb = FrameBuf::new();
                self.frame = 0;
            }
        }
    }

    /// Advance one frame. Pattern variant draws the test pattern; real variant
    /// runs CPU until VBlank (E03-35).
    pub fn run_frame(&mut self) {
        match &mut self.inner {
            Inner::Pattern { fb, compact } => {
                self.frame += 1;
                let f = (self.frame % 256) as u32;
                for y in 0..TEST_H {
                    for x in 0..TEST_W {
                        let argb = 0xFF00_0000
                            | (((x as u32 + f) & 0xFF) << 16)
                            | ((y as u32 & 0xFF) << 8)
                            | 0x40;
                        fb.set(x, y, argb);
                    }
                }
                if self.joy[0] != 0 {
                    fb.set(0, 0, 0xFFFF_FFFF);
                }
                *compact = fb.compact();
            }
            Inner::Real { cpu, sys, .. } => {
                let mut count = 0u64;
                loop {
                    cpu.step(&mut **sys);
                    if sys.take_vblank_flag() {
                        break;
                    }
                    count += 1;
                    if count >= 2_000_000 {
                        panic!("run_frame: no vblank");
                    }
                }
                self.frame = sys.vblank_count();
            }
        }
    }

    /// Compact view of the current frame.
    pub fn frame(&self) -> FrameView<'_> {
        match &self.inner {
            Inner::Pattern { fb, compact } => make_view(fb.view_w, fb.view_h, compact),
            Inner::Real { fb, compact, .. } => make_view(fb.view_w, fb.view_h, compact),
        }
    }

    /// Run for a specific number of master clocks (for timing tests).
    pub fn run_master(&mut self, master_clocks: u64) {
        match &mut self.inner {
            Inner::Pattern { .. } => {}
            Inner::Real { cpu, sys, .. } => {
                let end = sys.master + master_clocks;
                while sys.master < end {
                    cpu.step(&mut **sys);
                }
            }
        }
    }

    /// Execute one CPU instruction (for debugger). Pattern variant: no-op.
    pub fn step_instruction(&mut self) {
        match &mut self.inner {
            Inner::Pattern { .. } => {}
            Inner::Real { cpu, sys, .. } => {
                cpu.step(&mut **sys);
            }
        }
    }

    /// Number of frames run so far.
    pub fn frame_count(&self) -> u64 {
        match &self.inner {
            Inner::Pattern { .. } => self.frame,
            Inner::Real { sys, .. } => sys.vblank_count(),
        }
    }

    /// Set the buttons for port 0 or 1; other ports are ignored.
    pub fn set_joypad(&mut self, port: usize, buttons: u16) {
        if port < self.joy.len() {
            self.joy[port] = buttons;
        }
        match &mut self.inner {
            Inner::Pattern { .. } => {}
            Inner::Real { sys, .. } => {
                if port < 2 {
                    sys.joy.set_buttons(port, buttons);
                }
            }
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

    /// Deterministic state hash: region, frame, CPU state, system state.
    pub fn state_hash(&self) -> u64 {
        let mut h = Fnv64::new();
        if self.region == Region::Ntsc {
            h.write_u8(0);
        } else {
            h.write_u8(1);
        }
        match &self.inner {
            Inner::Pattern { fb, .. } => {
                h.write_u64(self.frame);
                h.write_u16(self.joy[0]);
                h.write_u16(self.joy[1]);
                fb.hash(&mut h);
            }
            Inner::Real { cpu, sys, .. } => {
                h.write_u64(sys.vblank_count());
                // CPU state hashed manually into Fnv64 (cpu.hash_into uses std Hasher)
                h.write_u16(cpu.a);
                h.write_u16(cpu.x);
                h.write_u16(cpu.y);
                h.write_u16(cpu.s);
                h.write_u16(cpu.d);
                h.write_u8(cpu.db);
                h.write_u8(cpu.pb);
                h.write_u16(cpu.pc);
                h.write_u8(cpu.p);
                if cpu.e { h.write_u8(1); } else { h.write_u8(0); }
                sys.hash_into(&mut h);
            }
        }
        h.finish()
    }

    /// Region of this console.
    pub fn region(&self) -> Region {
        self.region
    }

    /// SRAM contents if the cartridge has SRAM; None otherwise.
    pub fn sram(&self) -> Option<&[u8]> {
        match &self.inner {
            Inner::Pattern { .. } => None,
            Inner::Real { sys, .. } => {
                let cart = &sys.cart;
                if cart.sram().is_empty() {
                    None
                } else {
                    Some(cart.sram())
                }
            }
        }
    }

    /// Load SRAM contents from a byte slice.
    pub fn load_sram(&mut self, data: &[u8]) {
        match &mut self.inner {
            Inner::Pattern { .. } => {}
            Inner::Real { sys, .. } => {
                sys.cart.load_sram(data);
            }
        }
    }

    /// Debug snapshot of CPU and system state.
    pub fn debug(&self) -> DebugSnapshot {
        match &self.inner {
            Inner::Pattern { .. } => {
                DebugSnapshot {
                    pc: 0, pb: 0, a: 0, x: 0, y: 0, s: 0, d: 0, db: 0, p: 0,
                    e: false, v: 0, dot: 0, master: 0, frame: self.frame,
                }
            }
            Inner::Real { cpu, sys, .. } => {
                DebugSnapshot {
                    pc: cpu.pc,
                    pb: cpu.pb,
                    a: cpu.a,
                    x: cpu.x,
                    y: cpu.y,
                    s: cpu.s,
                    d: cpu.d,
                    db: cpu.db,
                    p: cpu.p,
                    e: cpu.e,
                    v: sys.ppu.v(),
                    dot: sys.ppu.dot(),
                    master: sys.master,
                    frame: self.frame,
                }
            }
        }
    }

    /// Cartridge info (real variant only).
    pub fn cart_info(&self) -> Option<CartInfo> {
        match &self.inner {
            Inner::Pattern { .. } => None,
            Inner::Real { sys, .. } => {
                let cart = &sys.cart;
                Some(CartInfo {
                    title: cart.header().title.to_string(),
                    map: cart.map_mode(),
                    region: cart.region(),
                    rom_len: cart.rom_len(),
                    sram_len: cart.sram().len(),
                    battery: cart.has_battery(),
                })
            }
        }
    }

    /// Current master clock count (real variant only; 0 for pattern).
    pub fn master(&self) -> u64 {
        match &self.inner {
            Inner::Pattern { .. } => 0,
            Inner::Real { sys, .. } => sys.master,
        }
    }

    /// Read a byte from WRAM (real variant only; 0 for pattern).
    pub fn peek_wram(&self, idx: u32) -> u8 {
        match &self.inner {
            Inner::Pattern { .. } => 0,
            Inner::Real { sys, .. } => sys.wram.read(idx),
        }
    }
}
