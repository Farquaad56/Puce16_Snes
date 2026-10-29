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
use crate::savestate::{State, StateWriter, StateReader, STATE_VERSION, StateError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SmpSnapshot {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    pub psw: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DmaChanSnapshot {
    pub dmap: u8,
    pub bbad: u8,
    pub a1t: u16,
    pub a1b: u8,
    pub das: u16,
    pub dasb: u8,
}

/// Audio sample rate (D-004): APU_RESONATOR_HZ / 768 = 32040 Hz.
pub const AUDIO_SAMPLE_RATE: u32 = 32_040;

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
    Real { cpu: Cpu, sys: Box<SysBus> },
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
        Ok(Self {
            region,
            frame: 0,
            joy: [0; 2],
            inner: Inner::Real { cpu, sys },
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
            Inner::Real { cpu, sys } => {
                sys.reset();
                cpu.reset(&mut **sys);
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
            Inner::Real { sys, .. } => sys.ppu.frame(),
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

    /// Take accumulated audio samples and append them to `out`.
    /// Pattern variant does nothing (no audio). Real variant drains the DSP buffer.
    pub fn take_audio(&mut self, out: &mut Vec<[i16; 2]>) {
        match &mut self.inner {
            Inner::Pattern { .. } => {}
            Inner::Real { sys, .. } => {
                let flat = sys.apu.take_audio();
                for i in (0..flat.len()).step_by(2) {
                    out.push([flat[i], flat[i + 1]]);
                }
            }
        }
    }

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

    /// VRAM contents for debugger (32768 words). Empty slice for test pattern variant.
    pub fn debug_vram(&self) -> &[u16] {
        match &self.inner {
            Inner::Pattern { .. } => &[],
            Inner::Real { sys, .. } => sys.ppu.vram(),
        }
    }

    /// CGRAM contents for debugger (256 words). Empty slice for test pattern variant.
    pub fn debug_cgram(&self) -> &[u16] {
        match &self.inner {
            Inner::Pattern { .. } => &[],
            Inner::Real { sys, .. } => sys.ppu.cgram(),
        }
    }

    /// OAM contents for debugger (544 bytes). Empty slice for test pattern variant.
    pub fn debug_oam(&self) -> &[u8] {
        match &self.inner {
            Inner::Pattern { .. } => &[],
            Inner::Real { sys, .. } => sys.ppu.oam(),
        }
    }

    /// PPU register snapshot for debugger. None for test pattern variant.
    pub fn debug_ppu_regs(&self) -> Option<crate::ppu::PpuRegsSnapshot> {
        match &self.inner {
            Inner::Pattern { .. } => None,
            Inner::Real { sys, .. } => Some(sys.ppu.regs_snapshot()),
        }
    }

    /// SMP register snapshot for debugger. None for test pattern variant.
    pub fn debug_smp(&self) -> Option<SmpSnapshot> {
        match &self.inner {
            Inner::Pattern { .. } => None,
            Inner::Real { sys, .. } => {
                let (a, x, y, sp, pc, psw) = sys.apu.smp_regs();
                Some(SmpSnapshot { a, x, y, sp, pc, psw })
            }
        }
    }

    /// DMA channel snapshots for debugger. None for test pattern variant.
    pub fn debug_dma(&self) -> Option<[DmaChanSnapshot; 8]> {
        match &self.inner {
            Inner::Pattern { .. } => None,
            Inner::Real { sys, .. } => {
                let snaps = core::array::from_fn(|i| {
                    let ch = &sys.dma.ch[i];
                    DmaChanSnapshot {
                        dmap: ch.dmap,
                        bbad: ch.bbad,
                        a1t: ch.a1t,
                        a1b: ch.a1b,
                        das: ch.das,
                        dasb: ch.dasb,
                    }
                });
                Some(snaps)
            }
        }
    }

    /// Read 256 bytes of WRAM at the given page (0..=511). Returns zeros for test pattern variant.
    pub fn debug_wram_page(&self, page: u16) -> Vec<u8> {
        let masked = page & 0x1FF;
        match &self.inner {
            Inner::Pattern { .. } => vec![0u8; 256],
            Inner::Real { sys, .. } => {
                let mut out = Vec::with_capacity(256);
                for i in 0..256 {
                    let addr = (masked as u32) * 256 + i as u32;
                    out.push(sys.wram.read(addr));
                }
                out
            }
        }
    }

    /// Execute instructions until PB:PC matches the target, or max_instructions is reached.
    /// Returns true if the address was reached (or already at it). Pattern variant returns false.
    pub fn debug_run_until_pc(&mut self, pb: u8, pc: u16, max_instructions: u32) -> bool {
        match &mut self.inner {
            Inner::Pattern { .. } => false,
            Inner::Real { cpu, sys, .. } => {
                if cpu.pb == pb && cpu.pc == pc {
                    return true;
                }
                for _ in 0..max_instructions {
                    cpu.step(&mut **sys);
                    if cpu.pb == pb && cpu.pc == pc {
                        return true;
                    }
                }
                false
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

    /// Copy of the PPU OAM (544 bytes; real variant only; zeros for pattern). Debug aid.
    pub fn peek_oam(&self) -> [u8; 544] {
        match &self.inner {
            Inner::Pattern { .. } => [0; 544],
            Inner::Real { sys, .. } => *sys.ppu.oam(),
        }
    }

    /// Read APU port $2140-$2143 (mirrored every 4). Returns byte written by SPC700.
    pub fn apu_port(&self, i: usize) -> u8 {
        match &self.inner {
            Inner::Pattern { .. } => 0,
            Inner::Real { sys, .. } => sys.apu.port_read(i),
        }
    }

    /// Save the full emulator state to a binary blob.
    pub fn save_state(&self) -> Vec<u8> {
        let mut w = StateWriter::new();
        // Header: version, region, frame count, master clock
        w.u32(STATE_VERSION);
        if self.region == Region::Ntsc {
            w.u8(0);
        } else {
            w.u8(1);
        }
        match &self.inner {
            Inner::Pattern { .. } => {
                w.u64(self.frame);
                w.u64(0); // no master clock for pattern variant
            }
            Inner::Real { cpu, sys, .. } => {
                w.u64(sys.vblank_count());
                w.u64(sys.master);
                cpu.save(&mut w);
                sys.save(&mut w);
            }
        }
        w.finish()
    }

    /// Load emulator state from a binary blob.
    pub fn load_state(&mut self, data: &[u8]) -> Result<(), StateError> {
        let mut r = StateReader::new(data);
        // Read and validate header first (Snes not modified until body is read)
        let version = r.u32()?;
        if version != STATE_VERSION {
            return Err(StateError::Version(version));
        }
        let region_byte = r.u8()?;
        let loaded_region = if region_byte == 0 { Region::Ntsc } else { Region::Pal };
        if loaded_region != self.region {
            return Err(StateError::Mismatch("region"));
        }
        let frame_count = r.u64()?;
        let _master_clocks = r.u64()?; // informational: restored from the CPU/bus state

        match &mut self.inner {
            Inner::Pattern { .. } => {
                // Pattern variant: just restore frame count, no body to read
                self.frame = frame_count;
                // Consume any remaining data (should be none for pattern saves)
                r.finish()?;
            }
            Inner::Real { cpu, sys, .. } => {
                // Load CPU and bus state. If the body is corrupt, the emulator
                // state may become inconsistent - the desktop should reload the ROM.
                cpu.load(&mut r)?;
                sys.load(&mut r)?;
                self.frame = frame_count;
                // Verify no trailing data
                r.finish()?;
            }
        }
        Ok(())
    }
}
