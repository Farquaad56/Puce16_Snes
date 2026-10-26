//! Timing: per-dot scheduling of NMI flag, IRQ, auto-read, HDMA triggers.
//!
//! `Timing::on_dot` is called once per dot (after the PPU position has been
//! advanced) and returns a `TimingOut` struct indicating which subsystems
//! should run this dot.

use crate::bus::mmio::Mmio;
use crate::hash::Fnv64;
use crate::joypad::Joypads;
use crate::ppu::Ppu;

/// Dot at which the RDNMI flag is raised (when entering VBlank).
pub const NMI_FLAG_DOT: u16 = 2;
/// Offset for H/V IRQ trigger relative to HTIME.
pub const IRQ_H_OFFSET_DOTS: u16 = 1;
/// Dot at which auto-read starts (if enabled).
pub const AUTOREAD_START_DOT: u16 = 33;
/// Duration of auto-read in master clocks.
pub const AUTOREAD_CLOCKS: u64 = 4224;
/// Dot at which HDMA init runs (first line of frame).
pub const HDMA_INIT_DOT: u16 = 3;
/// Dot at which per-line HDMA runs (visible lines only).
pub const HDMA_RUN_DOT: u16 = 276;

/// Outputs from `Timing::on_dot` indicating which subsystems should run.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct TimingOut {
    pub hdma_init: bool,
    pub hdma_run: bool,
    pub vblank_start: bool,
    pub frame_end: bool,
}

/// Per-frame timing state.
pub struct Timing {
    /// Master clock at the start of the current scanline (dot 0).
    line_start: u64,
    /// Whether DRAM refresh has been done this scanline.
    refreshed_this_line: bool,
    /// NMI edge detected on this dot (consumed by `take_nmi_edge`).
    nmi_edge: bool,
    /// Previous value of the computed "NMI line" condition for edge detection.
    nmi_line_prev: bool,
    /// Master clock at which auto-read starts.
    autoread_start: u64,
    /// Master clock at which auto-read ends (None if not active).
    autoread_end: Option<u64>,
}

impl Timing {
    pub fn new() -> Timing {
        Timing {
            line_start: 0,
            refreshed_this_line: false,
            nmi_edge: false,
            nmi_line_prev: false,
            autoread_start: 0,
            autoread_end: None,
        }
    }

    /// Called once per dot after the PPU position has been advanced.
    /// `t` is the master clock at the start of this dot (new position).
    pub fn on_dot(&mut self, mmio: &mut Mmio, joy: &mut Joypads, ppu: &Ppu, t: u64) -> TimingOut {
        let mut out = TimingOut::default();
        let v = ppu.v();
        let dot = ppu.dot();

        // 1. Line start (dot 0): reset line state; frame end at (0,0).
        if dot == 0 {
            self.line_start = t;
            self.refreshed_this_line = false;
            if v == 0 {
                out.frame_end = true;
                mmio.rdnmi_flag = false;
            }
        }

        // 2. VBlank start: raise RDNMI flag at (vblank_start, NMI_FLAG_DOT).
        if v == ppu.vblank_start() && dot == NMI_FLAG_DOT {
            mmio.rdnmi_flag = true;
            out.vblank_start = true;
        }

        // 3. NMI edge detection: rising edge of (rdnmi_flag && nmitimen bit7).
        let line = mmio.rdnmi_flag && (mmio.nmitimen & 0x80 != 0);
        if line && !self.nmi_line_prev {
            self.nmi_edge = true;
        }
        self.nmi_line_prev = line;

        // 4. IRQ H/V: evaluate timer condition each dot.
        let mode = (mmio.nmitimen >> 4) & 3;
        if mode != 0 {
            let htime_ok = mmio.htime <= 339;
            let vtime_ok = mmio.vtime <= ppu.last_line();
            let hit = match mode {
                1 => htime_ok && dot == mmio.htime + IRQ_H_OFFSET_DOTS,
                2 => vtime_ok && v == mmio.vtime && dot == IRQ_H_OFFSET_DOTS,
                3 => htime_ok && vtime_ok && v == mmio.vtime && dot == mmio.htime + IRQ_H_OFFSET_DOTS,
                _ => false,
            };
            if hit {
                mmio.timeup_flag = true;
                mmio.irq_line = true;
            }
        }

        // 5. Auto-read: start at (vblank_start, AUTOREAD_START_DOT) if enabled;
        //    update JOY registers when it ends.
        if mmio.nmitimen & 1 != 0 && v == ppu.vblank_start() && dot == AUTOREAD_START_DOT {
            self.autoread_start = t;
            self.autoread_end = Some(t + AUTOREAD_CLOCKS);
        }
        if let Some(e) = self.autoread_end {
            if t >= e {
                mmio.joy = joy.autoread_values();
                self.autoread_end = None;
            }
        }

        // 6. HDMA triggers: init at (0, HDMA_INIT_DOT), run per visible line.
        if mmio.hdmaen != 0 {
            if v == 0 && dot == HDMA_INIT_DOT {
                out.hdma_init = true;
            }
            let vis_last = ppu.vblank_start() - 1;
            if v <= vis_last && dot == HDMA_RUN_DOT {
                out.hdma_run = true;
            }
        }

        out
    }

    /// Consume the NMI edge flag (returns true if an edge was detected).
    pub fn take_nmi_edge(&mut self) -> bool {
        let e = self.nmi_edge;
        self.nmi_edge = false;
        e
    }

    /// Horizontal clock count since line start.
    pub fn hclock(&self, master: u64) -> u32 {
        (master - self.line_start) as u32
    }

    /// Whether DRAM refresh is due this scanline.
    pub fn refresh_due(&self, master: u64) -> bool {
        !self.refreshed_this_line && self.hclock(master) >= crate::consts::DRAM_REFRESH_HCLOCK
    }

    /// Mark DRAM refresh as done for this scanline.
    pub fn mark_refreshed(&mut self) {
        self.refreshed_this_line = true;
    }

    /// Whether auto-read is currently busy (E03-25).
    pub fn autoread_busy(&self, master: u64) -> bool {
        match self.autoread_end {
            Some(e) => master >= self.autoread_start && master < e,
            None => false,
        }
    }

    /// Hash timing state into the given hasher.
    pub fn hash_into(&self, h: &mut Fnv64) {
        h.write_u64(self.line_start);
        h.write_u8(if self.refreshed_this_line { 1 } else { 0 });
        h.write_u8(if self.nmi_edge { 1 } else { 0 });
        h.write_u8(if self.nmi_line_prev { 1 } else { 0 });
        h.write_u64(self.autoread_start);
        match self.autoread_end {
            Some(e) => {
                h.write_u8(1);
                h.write_u64(e);
            }
            None => h.write_u8(0),
        }
    }
}
