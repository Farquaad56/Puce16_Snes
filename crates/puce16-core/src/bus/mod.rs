pub mod cycle;
pub mod route;
pub mod wram;
pub mod mmio;

use crate::cart::Cart;
use crate::dma::{Dma, DmaBus};
use crate::ppu::Ppu;
use crate::apu::Apu;
use crate::joypad::Joypads;
use crate::timing::Timing;
use crate::region::Region;
use wram::Wram;
use mmio::Mmio;

/// CPU memory access timing in master clocks, per spec 02 section 2.
/// addr = 24-bit address (bank << 16 | offset). memsel = MEMSEL bit0 of $420B.
pub fn access_clocks(addr: u32, memsel: bool) -> u32 {
    let a = (addr & 0xFFFF) as u16;
    if addr & 0x408000 != 0 {
        // ROM area: A15 or A22 set
        return if addr & 0x800000 != 0 && memsel { 6 } else { 8 };
    }
    if (a.wrapping_add(0x6000)) & 0x4000 != 0 {
        // $0000-$1FFF and $6000-$7FFF: WRAM/SRAM area
        return 8;
    }
    if (a.wrapping_sub(0x4000)) & 0x7E00 != 0 {
        // IO area except $4000-$41FF
        return 6;
    }
    // JOYSER area $4000-$41FF
    12
}

/// System bus: owns all SNES components (spec 09 section 1).
pub struct SysBus {
    pub master: u64,
    pub region: Region,
    pub cart: Cart,
    pub wram: Wram,
    pub mmio: Mmio,
    pub dma: Dma,
    pub ppu: Ppu,
    pub apu: Apu,
    pub joy: Joypads,
    pub timing: Timing,
    pub mdr: u8,
    pub memsel: bool,
    ppu_t: u64,
    pending_dma: u8,
    dma_wait: bool,
    pending_hdma_init: bool,
    pending_hdma_run: bool,
    vblank_flag: bool,
    vblank_count: u64,
}

impl SysBus {
    /// Create a new system bus in power-on state.
    pub fn new(cart: Cart, region: Region) -> SysBus {
        SysBus {
            master: 0,
            region,
            cart,
            wram: Wram::new(),
            mmio: Mmio::new(),
            dma: Dma::new(),
            ppu: Ppu::new(region),
            apu: Apu::new(),
            joy: Joypads::new(),
            timing: Timing::new(),
            mdr: 0,
            memsel: false,
            ppu_t: 0,
            pending_dma: 0,
            dma_wait: false,
            pending_hdma_init: false,
            pending_hdma_run: false,
            vblank_flag: false,
            vblank_count: 0,
        }
    }

    /// Reset button: reset specific components, preserve WRAM/cart SRAM/DMA/joy.
    pub fn reset(&mut self) {
        self.mmio.reset();
        self.ppu.reset();
        self.apu.reset();
        self.wram.reset_addr();
        self.timing = Timing::new();
        self.pending_dma = 0;
        self.dma_wait = false;
        self.pending_hdma_init = false;
        self.pending_hdma_run = false;
        self.master = 0;
        self.ppu_t = 0;
        self.mdr = 0;
        self.memsel = false;
        self.vblank_flag = false;
        self.vblank_count = 0;
    }

    /// Return and clear the VBlank flag.
    pub fn take_vblank_flag(&mut self) -> bool {
        let f = self.vblank_flag;
        self.vblank_flag = false;
        f
    }

    /// Get the VBlank count.
    pub fn vblank_count(&self) -> u64 {
        self.vblank_count
    }

    /// Get pending DMA channels mask.
    pub fn pending_dma(&self) -> u8 {
        self.pending_dma
    }

    /// Hash entire system state into the given hasher.
    pub fn hash_into(&self, h: &mut crate::hash::Fnv64) {
        h.write_u64(self.master);
        h.write_u8(self.mdr);
        if self.memsel {
            h.write_u8(1);
        } else {
            h.write_u8(0);
        }
        h.write_u64(self.ppu_t);
        h.write_u8(self.pending_dma);
        if self.dma_wait {
            h.write_u8(1);
        } else {
            h.write_u8(0);
        }
        if self.pending_hdma_init {
            h.write_u8(1);
        } else {
            h.write_u8(0);
        }
        if self.pending_hdma_run {
            h.write_u8(1);
        } else {
            h.write_u8(0);
        }
        if self.vblank_flag {
            h.write_u8(1);
        } else {
            h.write_u8(0);
        }
        h.write_u64(self.vblank_count);
        self.cart.hash_into(h);
        self.wram.hash_into(h);
        self.mmio.hash_into(h);
        self.dma.hash_into(h);
        self.ppu.hash_into(h);
        self.apu.hash_into(h);
        self.joy.hash_into(h);
        self.timing.hash_into(h);
    }
}

/// Check if an A-bus address is in a forbidden DMA zone (system registers).
fn dma_a_forbidden(addr: u32) -> bool {
    let bank = (addr >> 16) as u8;
    let a = (addr & 0xFFFF) as u16;
    if bank & 0x7F > 0x3F {
        return false;
    }
    matches!(a, 0x2100..=0x21FF | 0x4000..=0x41FF | 0x4200..=0x421F | 0x4300..=0x437F)
}

impl DmaBus for SysBus {
    fn a_read(&mut self, addr: u32) -> u8 {
        if dma_a_forbidden(addr) {
            return self.mdr;
        }
        let v = self.read_inner(addr);
        self.mdr = v;
        v
    }

    fn a_write(&mut self, addr: u32, v: u8) {
        if dma_a_forbidden(addr) {
            return;
        }
        self.mdr = v;
        self.write_inner(addr, v);
    }

    fn b_read(&mut self, reg: u8) -> u8 {
        let addr = 0x00_2100 | (reg as u32);
        let v = self.read_inner(addr);
        self.mdr = v;
        v
    }

    fn b_write(&mut self, reg: u8, v: u8) {
        let addr = 0x00_2100 | (reg as u32);
        self.mdr = v;
        self.write_inner(addr, v);
    }

    fn elapse(&mut self, clocks: u32) {
        self.advance(clocks);
    }

    fn master(&self) -> u64 {
        self.master
    }
}
