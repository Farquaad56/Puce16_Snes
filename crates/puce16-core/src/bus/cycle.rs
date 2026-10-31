//! Time advancement, catch_up, and cpu::Bus implementation for SysBus.

use crate::consts::DRAM_REFRESH_CLOCKS;
use crate::cpu::Bus;
use super::{SysBus, access_clocks};

impl SysBus {
    /// Advance master clock by `clocks` ticks and catch up PPU/APU.
    pub fn advance(&mut self, clocks: u32) {
        self.master += clocks as u64;
        self.catch_up();
    }

    /// Run PPU dots until ppu_t >= master, then step APU to master.
    pub fn catch_up(&mut self) {
        while self.ppu_t < self.master {
            let c = self.ppu.dot_clocks() as u64;
            self.ppu.step_dot();
            self.ppu_t += c;
            let out = self.timing.on_dot(&mut self.mmio, &mut self.joy, &self.ppu, self.ppu_t);
            if out.hdma_init || out.hdma_reset { self.pending_hdma_init = true; }
            if out.hdma_run { self.pending_hdma_run = true; }
            if out.vblank_start {
                self.vblank_flag = true;
                self.vblank_count += 1;
            }
        }
        self.apu.step_to(self.master);
    }

    /// Begin of CPU cycle: DRAM refresh and HDMA (E03-30, E03-33).
    fn begin_cycle(&mut self) {
        // DRAM refresh: if due this line, insert 40 clocks before the access.
        // Mark refreshed BEFORE advancing so that crossing a line boundary
        // (which resets the flag via on_dot at dot 0) doesn't undo it.
        if self.timing.refresh_due(self.master) {
            self.timing.mark_refreshed();
            self.advance(DRAM_REFRESH_CLOCKS);
        }

        // HDMA execution: init and/or run, triggered by Timing flags.
        if self.pending_hdma_init || self.pending_hdma_run {
            let (init, run) = (self.pending_hdma_init, self.pending_hdma_run);
            self.pending_hdma_init = false;
            self.pending_hdma_run = false;
            let m = self.mmio.hdmaen;
            let mut d = std::mem::take(&mut self.dma);
            if init { d.hdma_init(self, m); }
            if run  { d.hdma_run_line(self, m); }
            self.dma = d;
        }
    }

    /// End of CPU cycle: general DMA (E03-32).
    fn end_cycle(&mut self) {
        if self.pending_dma != 0 {
            if self.dma_wait {
                // Cycle immediately after MDMAEN write: just disarm the wait flag.
                self.dma_wait = false;
            } else {
                let m = self.pending_dma;
                self.pending_dma = 0;
                let mut d = std::mem::take(&mut self.dma);
                d.run_general(self, m);
                self.dma = d;
            }
        }
    }
}

impl Bus for SysBus {
    fn read(&mut self, addr: u32) -> u8 {
        let addr = addr & 0xFF_FFFF;
        self.begin_cycle();
        self.advance(access_clocks(addr, self.memsel));
        let v = self.read_inner(addr);
        self.mdr = v;
        self.end_cycle();
        v
    }

    fn write(&mut self, addr: u32, v: u8) {
        let addr = addr & 0xFF_FFFF;
        self.begin_cycle();
        let clocks = access_clocks(addr, self.memsel);
        if (addr >> 16) as u8 & 0x7F <= 0x3F && (0x2140..=0x217F).contains(&(addr & 0xFFFF)) {
            // Announce the APU port write before the APU catches up, so the SPC700 sees it at the right cycle.
            self.apu.queue_port_write(self.master + clocks as u64, (addr & 3) as usize, v);
        }
        self.advance(clocks);
        self.mdr = v;
        self.write_inner(addr, v);
        self.end_cycle();
    }

    fn idle(&mut self) {
        self.begin_cycle();
        self.advance(6);
        self.end_cycle();
    }

    fn nmi_pending(&mut self) -> bool {
        self.timing.take_nmi_edge()
    }

    fn irq_asserted(&mut self) -> bool {
        self.mmio.irq_line
    }
}
