// Debugger: pure helper functions (tile decoding, colors, hex dump, flags) and the egui window.

use eframe::egui;

/// Convert SNES BGR555 color to RGBA8888.
pub fn bgr555_to_rgba(c: u16) -> [u8; 4] {
    let r5 = (c & 0x1F) as u32;
    let g5 = ((c >> 5) & 0x1F) as u32;
    let b5 = ((c >> 10) & 0x1F) as u32;
    // Expand 5-bit to 8-bit: (v << 3) | (v >> 2)
    let r8 = ((r5 << 3) | (r5 >> 2)) as u8;
    let g8 = ((g5 << 3) | (g5 >> 2)) as u8;
    let b8 = ((b5 << 3) | (b5 >> 2)) as u8;
    [r8, g8, b8, 255]
}

/// Number of VRAM words for a tile at the given bit depth.
pub fn tile_words(bpp: u8) -> usize {
    match bpp {
        2 => 8,
        4 => 16,
        8 => 32,
        _ => 8,
    }
}

/// Decode a single 8x8 tile from planar VRAM into color indices.
/// word_addr is the starting VRAM word address (modulo 0x8000).
/// Output: 64 bytes, row-major, left-to-right within each row.
pub fn decode_tile(vram: &[u16], word_addr: usize, bpp: u8, out: &mut [u8; 64]) {
    let words = tile_words(bpp);
    // For empty VRAM, fill with zeros (black)
    if vram.is_empty() {
        out.fill(0);
        return;
    }

    for row in 0..8 {
        let mut indices: [u8; 8] = [0; 8];
        match bpp {
            2 => {
                // Planar: word r has plan0=low byte, plan1=high byte
                let w = vram[(word_addr + row) & 0x7FFF];
                let p0 = (w & 0xFF) as u8;
                let p1 = ((w >> 8) & 0xFF) as u8;
                for col in 0..8 {
                    let bit = 7 - col;
                    let b0 = if p0 & (1 << bit) != 0 { 1 } else { 0 };
                    let b1 = if p1 & (1 << bit) != 0 { 2 } else { 0 };
                    indices[col] = b0 | b1;
                }
            }
            4 => {
                // Planar: words 0-7 have planes 0/1, words 8-15 have planes 2/3
                let w_low = vram[(word_addr + row) & 0x7FFF];
                let w_high = vram[(word_addr + 8 + row) & 0x7FFF];
                let p0 = (w_low & 0xFF) as u8;
                let p1 = ((w_low >> 8) & 0xFF) as u8;
                let p2 = (w_high & 0xFF) as u8;
                let p3 = ((w_high >> 8) & 0xFF) as u8;
                for col in 0..8 {
                    let bit = 7 - col;
                    let b0 = if p0 & (1 << bit) != 0 { 1 } else { 0 };
                    let b1 = if p1 & (1 << bit) != 0 { 2 } else { 0 };
                    let b2 = if p2 & (1 << bit) != 0 { 4 } else { 0 };
                    let b3 = if p3 & (1 << bit) != 0 { 8 } else { 0 };
                    indices[col] = b0 | b1 | b2 | b3;
                }
            }
            8 => {
                // Planar: 4 blocks of 8 words each for planes 0/1, 2/3, 4/5, 6/7
                let w0 = vram[(word_addr + row) & 0x7FFF];
                let w1 = vram[(word_addr + 8 + row) & 0x7FFF];
                let w2 = vram[(word_addr + 16 + row) & 0x7FFF];
                let w3 = vram[(word_addr + 24 + row) & 0x7FFF];
                for col in 0..8 {
                    let bit = 7 - col;
                    let b0 = if (w0 & 0xFF) & (1 << bit) != 0 { 1 } else { 0 };
                    let b1 = if ((w0 >> 8) & 0xFF) & (1 << bit) != 0 { 2 } else { 0 };
                    let b2 = if (w1 & 0xFF) & (1 << bit) != 0 { 4 } else { 0 };
                    let b3 = if ((w1 >> 8) & 0xFF) & (1 << bit) != 0 { 8 } else { 0 };
                    let b4 = if (w2 & 0xFF) & (1 << bit) != 0 { 16 } else { 0 };
                    let b5 = if ((w2 >> 8) & 0xFF) & (1 << bit) != 0 { 32 } else { 0 };
                    let b6 = if (w3 & 0xFF) & (1 << bit) != 0 { 64 } else { 0 };
                    let b7 = if ((w3 >> 8) & 0xFF) & (1 << bit) != 0 { 128 } else { 0 };
                    indices[col] = b0 | b1 | b2 | b3 | b4 | b5 | b6 | b7;
                }
            }
            _ => {}
        }
        out[row * 8..row * 8 + 8].copy_from_slice(&indices);
    }
}

/// Render a grid of tiles to RGBA pixels.
/// Returns (width, height, rgba_data).
pub fn render_tiles(
    vram: &[u16],
    cgram: &[u16],
    base_word: usize,
    bpp: u8,
    palette: u8,
    cols: usize,
    rows: usize,
) -> (usize, usize, Vec<u8>) {
    let width = cols * 8;
    let height = rows * 8;
    let mut pixels = vec![0u8; width * height * 4];

    // Fill with black if VRAM is empty
    if vram.is_empty() {
        for p in pixels.chunks_mut(4) {
            p[3] = 255;
        }
        return (width, height, pixels);
    }

    let words_per_tile = tile_words(bpp);
    let palette_size = if bpp == 8 { 1 } else { 1 << bpp };

    for ty in 0..rows {
        for tx in 0..cols {
            let tile_addr = base_word + (ty * cols + tx) * words_per_tile;
            let mut indices = [0u8; 64];
            decode_tile(vram, tile_addr, bpp, &mut indices);

            for row in 0..8 {
                for col in 0..8 {
                    let idx = indices[row * 8 + col];
                    let color_idx = if bpp == 8 {
                        idx as usize
                    } else {
                        ((palette as usize) * palette_size + idx as usize) & 0xFF
                    };
                    let cgram_word = if color_idx < cgram.len() {
                        cgram[color_idx]
                    } else {
                        0x0000
                    };
                    let rgba = bgr555_to_rgba(cgram_word);

                    let px = tx * 8 + col;
                    let py = ty * 8 + row;
                    let offset = (py * width + px) * 4;
                    pixels[offset] = rgba[0];
                    pixels[offset + 1] = rgba[1];
                    pixels[offset + 2] = rgba[2];
                    pixels[offset + 3] = rgba[3];
                }
            }
        }
    }

    (width, height, pixels)
}

/// Get all 256 CGRAM colors as RGBA.
pub fn palette_rgba(cgram: &[u16]) -> Vec<[u8; 4]> {
    let mut result = Vec::with_capacity(256);
    for i in 0..256 {
        if i < cgram.len() {
            result.push(bgr555_to_rgba(cgram[i]));
        } else {
            result.push([0, 0, 0, 255]);
        }
    }
    result
}

/// Format memory as hex dump rows (16 bytes per line).
pub fn hex_rows(data: &[u8], base: u32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut offset = 0;
    while offset < data.len() {
        let chunk = &data[offset..std::cmp::min(offset + 16, data.len())];
        let addr_str = format!("{:06X}:", base + offset as u32);

        let hex_part: String = chunk
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");

        let mut ascii_part = String::new();
        for b in chunk {
            if *b >= 0x20 && *b <= 0x7E {
                ascii_part.push(*b as char);
            } else {
                ascii_part.push('.');
            }
        }

        lines.push(format!("{} {}  {}", addr_str, hex_part, ascii_part));
        offset += chunk.len();
    }
    lines
}

/// Format CPU flags register as a string.
pub fn flags_string(p: u8, e: bool) -> String {
    let bits = [7, 6, 5, 4, 3, 2, 1, 0]; // N V M X D I Z C
    let names = ['N', 'V', 'M', 'X', 'D', 'I', 'Z', 'C'];
    let mut s = String::new();
    for (i, bit) in bits.iter().enumerate() {
        if p & (1 << bit) != 0 {
            s.push(names[i]);
        } else {
            s.push(names[i].to_ascii_lowercase());
        }
    }
    if e {
        s.push_str(" e=1");
    } else {
        s.push_str(" e=0");
    }
    s
}

/// Sprite information decoded from OAM.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SpriteInfo {
    pub index: u8,
    pub x: i16,
    pub y: u8,
    pub tile: u16,
    pub palette: u8,
    pub priority: u8,
    pub hflip: bool,
    pub vflip: bool,
    pub large: bool,
}

/// Get sprite dimensions based on size code and large flag.
pub fn sprite_dims(obj_size: u8, large: bool) -> (u8, u8) {
    if !large {
        match obj_size {
            0 => (8, 8),
            1 => (16, 16),
            2 => (32, 32),
            3 => (64, 64),
            4 => (32, 32),
            5 => (32, 32),
            6 => (16, 32),
            7 => (16, 32),
            _ => (8, 8),
        }
    } else {
        match obj_size {
            0 => (16, 16),
            1 => (32, 32),
            2 => (64, 64),
            3 => (32, 32),
            4 => (32, 32),
            5 => (64, 64),
            6 => (32, 64),
            7 => (32, 32),
            _ => (16, 16),
        }
    }
}

/// Decode OAM data into sprite information.
pub fn decode_oam(oam: &[u8]) -> Vec<SpriteInfo> {
    let mut sprites = Vec::with_capacity(128);
    for i in 0..128 {
        let base = i * 4;
        let x_low = if base < oam.len() { oam[base] } else { 0 };
        let y = if base + 1 < oam.len() { oam[base + 1] } else { 0 };
        let tile_byte = if base + 2 < oam.len() { oam[base + 2] } else { 0 };
        let attrs = if base + 3 < oam.len() { oam[base + 3] } else { 0 };

        // High table: bits for x bit 8 and large flag
        let high_byte_idx = 512 + i / 4;
        let shift = 2 * (i % 4);
        let high_bits = if high_byte_idx < oam.len() {
            (oam[high_byte_idx] >> shift) & 0x3
        } else {
            0
        };

        // x is 9-bit signed (-256..255)
        let mut x_val = x_low as i16;
        if high_bits & 0x1 != 0 {
            x_val |= 0x100;
        }
        // Sign-extend: if bit 8 is set, it's a negative 9-bit value
        if x_val >= 256 {
            x_val -= 512;
        }

        let large = high_bits & 0x2 != 0;
        let tile_n = attrs & 0x1;
        let tile = (tile_byte as u16) | ((tile_n as u16) << 8);
        let palette = (attrs >> 1) & 0x7;
        let priority = (attrs >> 4) & 0x3;
        let hflip = attrs & 0x40 != 0;
        let vflip = attrs & 0x80 != 0;

        sprites.push(SpriteInfo {
            index: i as u8,
            x: x_val,
            y,
            tile,
            palette,
            priority,
            hflip,
            vflip,
            large,
        });
    }
    sprites
}

/// Render a background map (modes 0-6).
pub fn render_bg_map(
    vram: &[u16],
    cgram: &[u16],
    map_base_word: usize,
    chr_base_word: usize,
    map_size: u8,
    tile16: bool,
    bpp: u8,
) -> (usize, usize, Vec<u8>) {
    let screens_h = if map_size == 1 || map_size == 3 { 2 } else { 1 };
    let screens_v = if map_size == 2 || map_size == 3 { 2 } else { 1 };
    let tile_px = if tile16 { 16 } else { 8 };
    let width = (screens_h * 32) as usize * tile_px;
    let height = (screens_v * 32) as usize * tile_px;

    let mut pixels = vec![0u8; width * height * 4];
    // Fill with black opaque if VRAM is empty
    if vram.is_empty() {
        for p in pixels.chunks_mut(4) {
            p[3] = 255;
        }
        return (width, height, pixels);
    }

    let words_per_tile = tile_words(bpp);
    let palette_size = if bpp == 8 { 1 } else { 1 << bpp };

    for screen_y in 0..screens_v {
        for screen_x in 0..screens_h {
            // Calculate map base offset for this screen quadrant
            let map_offset = match (screen_x, screen_y) {
                (0, 0) => 0,
                (1, 0) => 0x400,
                (0, 1) => 0x400,
                (1, 1) => 0xC00,
                _ => 0,
            };

            let map_addr = map_base_word + map_offset;

            for ty in 0..32 {
                for tx in 0..32 {
                    // Read map entry
                    let entry_idx = map_addr + ty * 32 + tx;
                    if entry_idx >= vram.len() {
                        continue;
                    }
                    let entry = vram[entry_idx];

                    let char_num = (entry & 0x3FF) as usize;
                    let palette = ((entry >> 10) & 0x7) as u8;
                    let hflip = entry & 0x4000 != 0;
                    let vflip = entry & 0x8000 != 0;

                    // Calculate tile VRAM address
                    let tile_addr = (chr_base_word + char_num * words_per_tile) & 0x7FFF;

                    // Decode the tile(s) and render
                    if !tile16 {
                        // Single 8x8 tile
                        let mut indices = [0u8; 64];
                        decode_tile(vram, tile_addr, bpp, &mut indices);

                        for row in 0..8 {
                            for col in 0..8 {
                                let src_row = if vflip { 7 - row } else { row };
                                let src_col = if hflip { 7 - col } else { col };
                                let idx = indices[src_row * 8 + src_col];

                                let color_idx = if bpp == 8 {
                                    idx as usize
                                } else {
                                    ((palette as usize) << bpp) | (idx as usize)
                                };
                                let cgram_word = if color_idx < cgram.len() {
                                    cgram[color_idx]
                                } else {
                                    0x0000
                                };
                                let rgba = bgr555_to_rgba(cgram_word);

                                let px = (screen_x * 32 + tx) as usize * tile_px + col;
                                let py = (screen_y * 32 + ty) as usize * tile_px + row;
                                let offset = (py * width + px) * 4;
                                pixels[offset] = rgba[0];
                                pixels[offset + 1] = rgba[1];
                                pixels[offset + 2] = rgba[2];
                                pixels[offset + 3] = rgba[3];
                            }
                        }
                    } else {
                        // 16x16 tile composed of four 8x8 sub-tiles
                        // Top-left: char_num, top-right: char_num+1
                        // Bottom-left: char_num+16, bottom-right: char_num+17
                        let tl_addr = (chr_base_word + char_num * words_per_tile) & 0x7FFF;
                        let tr_addr = (chr_base_word + (char_num + 1) * words_per_tile) & 0x7FFF;
                        let bl_addr = (chr_base_word + (char_num + 16) * words_per_tile) & 0x7FFF;
                        let br_addr = (chr_base_word + (char_num + 17) * words_per_tile) & 0x7FFF;

                        for row in 0..16 {
                            for col in 0..16 {
                                // Determine which sub-tile and local coordinates
                                let sub_row = if vflip { 15 - row } else { row };
                                let sub_col = if hflip { 15 - col } else { col };

                                let tile_addr;
                                let local_row = sub_row & 7;
                                let local_col = sub_col & 7;

                                if sub_row < 8 {
                                    if sub_col < 8 {
                                        tile_addr = tl_addr;
                                    } else {
                                        tile_addr = tr_addr;
                                    }
                                } else {
                                    if sub_col < 8 {
                                        tile_addr = bl_addr;
                                    } else {
                                        tile_addr = br_addr;
                                    }
                                }

                                let mut indices = [0u8; 64];
                                decode_tile(vram, tile_addr, bpp, &mut indices);
                                let idx = indices[local_row * 8 + local_col];

                                let color_idx = if bpp == 8 {
                                    idx as usize
                                } else {
                                    ((palette as usize) << bpp) | (idx as usize)
                                };
                                let cgram_word = if color_idx < cgram.len() {
                                    cgram[color_idx]
                                } else {
                                    0x0000
                                };
                                let rgba = bgr555_to_rgba(cgram_word);

                                let px = (screen_x * 32 + tx) as usize * tile_px + col;
                                let py = (screen_y * 32 + ty) as usize * tile_px + row;
                                let offset = (py * width + px) * 4;
                                pixels[offset] = rgba[0];
                                pixels[offset + 1] = rgba[1];
                                pixels[offset + 2] = rgba[2];
                                pixels[offset + 3] = rgba[3];
                            }
                        }
                    }
                }
            }
        }
    }

    (width, height, pixels)
}

/// Parse a PC address from text. Accepts "BBPPPP" (6 hex digits) or "PPPP" (4 hex digits = bank 00).
pub fn parse_pc(text: &str) -> Option<u32> {
    let s = text.trim();
    if s.is_empty() {
        return None;
    }
    // Try 6-digit form first (bank + offset).
    if s.len() == 6 {
        if let Ok(val) = u32::from_str_radix(s, 16) {
            return Some(val);
        }
    }
    // Try 4-digit form (offset only, bank 00).
    if s.len() == 4 {
        if let Ok(offset) = u16::from_str_radix(s, 16) {
            return Some(offset as u32);
        }
    }
    None
}

/// Debugger UI state.
pub struct DebuggerUi {
    pub open: bool,
    pub last: Option<Box<crate::emu_thread::DebugData>>,
    pub wram_page: u16,
    pub pc_text: String,
    /// Selected tab: 0 Registres, 1 Tuiles, 2 Cartes, 3 Palettes, 4 OAM, 5 Memoire.
    pub tab: usize,
    /// Pause state requested by the window; the `App` applies it (it owns the `paused` flag).
    pub pause_request: Option<bool>,
    // Future panel fields (E08-37 / E08-38).
}

impl DebuggerUi {
    /// Create a new debugger UI state.
    pub fn new() -> DebuggerUi {
        DebuggerUi {
            open: false,
            last: None,
            wram_page: 0,
            pc_text: String::new(),
            tab: 0,
            pause_request: None,
        }
    }

    /// Show the debugger window with tabs for Registers, Tiles, Maps, Palettes, OAM, Memory.
    pub fn show(
        &mut self,
        ctx: &eframe::egui::Context,
        tx: &std::sync::mpsc::Sender<crate::emu_thread::Command>,
        paused: bool,
    ) {
        if !self.open {
            return;
        }

        use crate::emu_thread::Command;
        // Take the data out of `self` so the panels can borrow `self` mutably.
        let last = self.last.take();
        let mut open = self.open;
        egui::Window::new("Puce16 - Debugger")
            .open(&mut open)
            .resizable(true)
            .default_size([640.0, 520.0])
            .show(ctx, |ui| {
                // Execution control.
                ui.horizontal(|ui| {
                    let label = if paused { "Reprendre" } else { "Pause" };
                    if ui.button(label).clicked() {
                        self.pause_request = Some(!paused);
                    }
                    if ui.add_enabled(paused, egui::Button::new("Image")).clicked() {
                        let _ = tx.send(Command::FrameAdvance);
                    }
                    if ui.add_enabled(paused, egui::Button::new("Instruction")).clicked() {
                        let _ = tx.send(Command::StepInstruction);
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("PC (hex):");
                    ui.add(egui::TextEdit::singleline(&mut self.pc_text).desired_width(80.0).font(egui::TextStyle::Monospace));
                    let parsed = parse_pc(&self.pc_text);
                    if ui.add_enabled(paused, egui::Button::new("Executer jusqu'a")).clicked() {
                        if let Some(addr) = parsed {
                            let _ = tx.send(Command::RunToPc(addr));
                        }
                    }
                    if parsed.is_none() && !self.pc_text.is_empty() {
                        ui.colored_label(egui::Color32::RED, "adresse invalide (BBPPPP ou PPPP)");
                    }
                });
                ui.separator();

                ui.horizontal(|ui| {
                    for (i, name) in ["Registres", "Tuiles", "Cartes", "Palettes", "OAM", "Memoire"].iter().enumerate() {
                        ui.selectable_value(&mut self.tab, i, *name);
                    }
                });
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| match self.tab {
                    0 => self.show_registers(ui, last.as_ref()),
                    5 => self.show_memory(ui, tx, last.as_ref()),
                    _ => {
                        ui.label("(a venir)");
                    }
                });
            });
        self.last = last;
        if !open {
            // Closed with the window's close button.
            self.open = false;
            let _ = tx.send(Command::RequestDebug(false));
        }
    }

    /// Show the Registers tab.
    fn show_registers(&mut self, ui: &mut egui::Ui, data: Option<&Box<crate::emu_thread::DebugData>>) {
        if let Some(d) = data {
            let snap = &d.snapshot;

            // CPU registers.
            ui.heading("CPU");
            ui.monospace(format!(
                "A={:04X} X={:04X} Y={:04X} S={:02X} D={:02X} DB={:02X} PB={:02X} PC={:04X}",
                snap.a, snap.x, snap.y, snap.s, snap.d, snap.db, snap.pb, snap.pc
            ));
            ui.monospace(format!("P={} ({})", flags_string(snap.p, snap.e), snap.p));

            // Position info.
            ui.heading("Position");
            ui.monospace(format!(
                "frame={} V={} dot={} master={}",
                snap.frame, snap.v, snap.dot, snap.master
            ));

            // SMP registers (if present).
            if let Some(smp) = &d.smp {
                ui.heading("SMP");
                ui.monospace(format!(
                    "A={:02X} X={:02X} Y={:02X} SP={:02X} PC={:04X} PSW={:02X}",
                    smp.a, smp.x, smp.y, smp.sp, smp.pc, smp.psw
                ));
            }

            // PPU main registers.
            if let Some(ppu) = &d.ppu_regs {
                ui.heading("PPU");
                ui.monospace(format!(
                    "brightness={} bg_mode={} tm={} ts={} forced_blank={}",
                    ppu.brightness, ppu.bg_mode, ppu.tm, ppu.ts, ppu.forced_blank
                ));
            }

            // DMA channels.
            if let Some(dma) = &d.dma {
                ui.heading("DMA");
                for (i, ch) in dma.iter().enumerate() {
                    ui.monospace(format!(
                        "ch {}: dmap={:02X} bbad={:02X} a1t={:04X} a1b={:02X} das={:04X}",
                        i, ch.dmap, ch.bbad, ch.a1t, ch.a1b, ch.das
                    ));
                }
            }
        } else {
            ui.label("No debug data (open debugger with F10)");
        }
    }

    /// Show the Memory tab (WRAM page viewer).
    fn show_memory(&mut self, ui: &mut egui::Ui, tx: &std::sync::mpsc::Sender<crate::emu_thread::Command>, data: Option<&Box<crate::emu_thread::DebugData>>) {
        if let Some(d) = data {
            // Page navigation.
            ui.horizontal(|ui| {
                if ui.button("Prev").clicked() {
                    self.wram_page = (self.wram_page + 511) % 512;
                    let _ = tx.send(crate::emu_thread::Command::SetWramPage(self.wram_page));
                }
                ui.label(format!("WRAM page {:03X}", self.wram_page));
                if ui.button("Next").clicked() {
                    self.wram_page = (self.wram_page + 1) % 512;
                    let _ = tx.send(crate::emu_thread::Command::SetWramPage(self.wram_page));
                }
            });

            // Hex dump of the page.
            if !d.wram_page.is_empty() {
                let rows = hex_rows(&d.wram_page, (self.wram_page as u32) << 8);
                for row in rows {
                    ui.monospace(row);
                }
            } else {
                ui.label("No WRAM data");
            }
        } else {
            ui.label("No debug data");
        }
    }
}

/// Render mode 7 map (128x128 tiles -> 1024x1024 image).
pub fn render_mode7_map(vram: &[u16], cgram: &[u16]) -> (usize, usize, Vec<u8>) {
    let width = 1024;
    let height = 1024;
    let mut pixels = vec![0u8; width * height * 4];

    // Fill with black opaque if VRAM is empty
    if vram.is_empty() {
        for p in pixels.chunks_mut(4) {
            p[3] = 255;
        }
        return (width, height, pixels);
    }

    for y in 0..128 {
        for x in 0..128 {
            // Map entry: low byte of word at y*128+x is tile number
            let map_idx = y * 128 + x;
            if map_idx >= vram.len() {
                continue;
            }
            let map_entry = vram[map_idx];
            let tile_num = (map_entry & 0xFF) as usize;

            // Tile data: high byte of word at tile*64 + py*8 + px is pixel index
            for py in 0..8 {
                for px in 0..8 {
                    let tile_idx = tile_num * 64 + py * 8 + px;
                    if tile_idx >= vram.len() {
                        continue;
                    }
                    let tile_word = vram[tile_idx];
                    let pixel_idx = ((tile_word >> 8) & 0xFF) as usize;

                    let cgram_word = if pixel_idx < cgram.len() {
                        cgram[pixel_idx]
                    } else {
                        0x0000
                    };
                    let rgba = bgr555_to_rgba(cgram_word);

                    let out_x = x * 8 + px;
                    let out_y = y * 8 + py;
                    let offset = (out_y * width + out_x) * 4;
                    pixels[offset] = rgba[0];
                    pixels[offset + 1] = rgba[1];
                    pixels[offset + 2] = rgba[2];
                    pixels[offset + 3] = rgba[3];
                }
            }
        }
    }

    (width, height, pixels)
}

