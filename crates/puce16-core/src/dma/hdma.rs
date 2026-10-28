use crate::dma::{pattern_offsets, Dma, DmaBus};

impl Dma {
    /// HDMA initialization at frame start (V=0).
    /// Reads the first line byte for each active channel and sets up transfer state.
    /// Does NOT perform the actual data transfer - that happens in hdma_run_line.
    pub fn hdma_init<B: DmaBus>(&mut self, bus: &mut B, mask: u8) {
        if mask == 0 {
            return;
        }

        // One-time overhead for HDMA init (spec 09 section 5)
        bus.elapse(18);

        for n in 0..8 {
            if (mask & (1 << n)) == 0 {
                continue;
            }

            let ch = &mut self.ch[n];

            // Per-channel overhead
            bus.elapse(8);

            // Step 1: A2A = A1T
            ch.a2a = ch.a1t;

            // Step 2: Read line byte NLTR from table at (A1B:A2A)
            let table_addr = ((ch.a1b as u32) << 16) | (ch.a2a as u32);
            let b = bus.a_read(table_addr);

            // Increment A2A after reading line byte
            ch.a2a = ch.a2a.wrapping_add(1);

            ch.nltr = b;

            if b == 0 {
                // Channel done for this frame
                ch.done = true;
                ch.do_transfer = false;
                ch.line_counter = 0;
            } else {
                ch.done = false;
                ch.do_transfer = true;
                // Line counter: low 7 bits, with $80 meaning 128
                if (b & 0x7F) == 0 {
                    ch.line_counter = 128;
                } else {
                    ch.line_counter = b & 0x7F;
                }

                // Step 4: Indirect mode - read DAS from table
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
    }

    /// Query HDMA state for channel n.
    /// Returns (a2a, line_counter, do_transfer, done).
    pub fn hdma_state(&self, n: usize) -> (u16, u8, bool, bool) {
        let ch = &self.ch[n];
        (ch.a2a, ch.line_counter, ch.do_transfer, ch.done)
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

            // Decrement line counter
            ch.line_counter -= 1;

            if ch.line_counter == 0 {
                // Read next line byte from table
                let table_addr = ((ch.a1b as u32) << 16) | (ch.a2a as u32);
                let b = bus.a_read(table_addr);
                ch.a2a = ch.a2a.wrapping_add(1);

                ch.nltr = b;

                if b == 0 {
                    // Channel done for this frame
                    ch.done = true;
                    ch.do_transfer = false;
                    ch.line_counter = 0;
                } else {
                    ch.done = false;
                    ch.do_transfer = true;
                    // Line counter: low 7 bits, with $80 meaning 128
                    if (b & 0x7F) == 0 {
                        ch.line_counter = 128;
                    } else {
                        ch.line_counter = b & 0x7F;
                    }

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
            } else {
                // Counter > 0: transfer only in repeat mode. NLTR $80 (count 0 = 128 lines)
                // is NOT repeat: $01-$80 = one write then wait, $81-$FF = repeat (spec 03).
                ch.do_transfer = (ch.nltr & 0x80) != 0 && (ch.nltr & 0x7F) != 0;
            }
        }
    }
}
