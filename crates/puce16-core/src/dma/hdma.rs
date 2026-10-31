
use crate::dma::{pattern_offsets, Channel, Dma, DmaBus};

impl Dma {
    /// Frame-start HDMA reset (hardware `hdmaReset`): for ALL channels, enabled or not,
    /// the "completed" flag and the per-line transfer flag are cleared at V=0. A channel
    /// that the CPU enables later in the frame therefore runs again (E09-12: SFA2 enables
    /// channels 3-5 on line 21 after programming $43x8/$43x9/$43xA itself).
    pub fn hdma_reset(&mut self) {
        for ch in &mut self.ch {
            ch.done = false;
            ch.do_transfer = false;
        }
    }

    /// Load the next table entry of a channel (hardware `hdmaReload`): line byte, then the
    /// indirect address for indirect mode. $43xA (`nltr`) IS the line counter.
    fn hdma_reload<B: DmaBus>(ch: &mut Channel, bus: &mut B) {
        let table_addr = ((ch.a1b as u32) << 16) | (ch.a2a as u32);
        let b = bus.a_read(table_addr);
        ch.a2a = ch.a2a.wrapping_add(1);
        ch.nltr = b;

        if b == 0 {
            // Channel done for this frame
            ch.done = true;
            ch.do_transfer = false;
        } else {
            ch.done = false;
            ch.do_transfer = true;

            // Indirect mode: read DAS from table
            if (ch.dmap & 0x40) != 0 {
                let lo_addr = ((ch.a1b as u32) << 16) | (ch.a2a as u32);
                let lo = bus.a_read(lo_addr);
                ch.a2a = ch.a2a.wrapping_add(1);

                let hi_addr = ((ch.a1b as u32) << 16) | (ch.a2a as u32);
                let hi = bus.a_read(hi_addr);
                ch.a2a = ch.a2a.wrapping_add(1);

                ch.das = ((hi as u16) << 8) | (lo as u16);
            }
        }
    }

    /// HDMA frame start (V=0): always resets the per-channel flags, then, if any channel is
    /// enabled, initialises the enabled ones (A2A = A1T, first line byte).
    /// Does NOT perform the actual data transfer - that happens in hdma_run_line.
    pub fn hdma_init<B: DmaBus>(&mut self, bus: &mut B, mask: u8) {
        self.hdma_reset();
        if mask == 0 {
            return;
        }

        // One-time overhead for HDMA init (spec 09 section 5)
        bus.elapse(18);

        for n in 0..8 {
            let ch = &mut self.ch[n];

            // Hardware `hdmaSetup`: every channel gets its transfer flag set, enabled or not.
            ch.do_transfer = true;

            if (mask & (1 << n)) == 0 {
                continue;
            }

            // Per-channel overhead
            bus.elapse(8);

            ch.a2a = ch.a1t;
            ch.nltr = 0;
            Self::hdma_reload(ch, bus);
        }
    }

    /// Query HDMA state for channel n.
    /// Returns (a2a, remaining line count, do_transfer, done). The count is derived from the
    /// $43xA line counter: its low 7 bits, 128 when they are 0 but the counter is not 0.
    pub fn hdma_state(&self, n: usize) -> (u16, u8, bool, bool) {
        let ch = &self.ch[n];
        let count = if ch.nltr == 0 {
            0
        } else if ch.nltr & 0x7F == 0 {
            128
        } else {
            ch.nltr & 0x7F
        };
        (ch.a2a, count, ch.do_transfer, ch.done)
    }

    /// Execute HDMA transfers for one scanline.
    /// mask: which channels are active this line.
    pub fn hdma_run_line<B: DmaBus>(&mut self, bus: &mut B, mask: u8) {
        // Compute active channels (in mask and not done)
        let mut active = 0u8;
        for n in 0..8 {
            if (mask & (1 << n)) != 0 && !self.ch[n].done {
                active |= 1 << n;
            }
        }

        if active == 0 {
            return;
        }

        // One-time overhead for this scanline's HDMA
        bus.elapse(18);

        for n in 0..8 {
            if (active & (1 << n)) == 0 {
                continue;
            }

            let ch = &mut self.ch[n];

            // Per-channel overhead
            bus.elapse(8);

            // Perform transfer if needed
            if ch.do_transfer {
                let offs = pattern_offsets(ch.dmap & 7);
                for k in offs {
                    // Per-byte overhead before copy
                    bus.elapse(8);

                    let src;
                    if (ch.dmap & 0x40) == 0 {
                        // Direct mode: read from A1B:A2A, increment A2A
                        src = ((ch.a1b as u32) << 16) | (ch.a2a as u32);
                        ch.a2a = ch.a2a.wrapping_add(1);
                    } else {
                        // Indirect mode: read from DASB:DAS, increment DAS
                        src = ((ch.dasb as u32) << 16) | (ch.das as u32);
                        ch.das = ch.das.wrapping_add(1);
                    }

                    let v = bus.a_read(src);
                    bus.b_write(ch.bbad.wrapping_add(*k), v);
                }
            }

            // Hardware `hdmaAdvance`: the line counter ($43xA) counts down; bit 7 of the
            // result is the repeat flag for the next line; a new entry is loaded when the
            // 7-bit count reaches 0.
            ch.nltr = ch.nltr.wrapping_sub(1);
            ch.do_transfer = (ch.nltr & 0x80) != 0;
            if (ch.nltr & 0x7F) == 0 {
                Self::hdma_reload(ch, bus);
            }
        }
    }
}
