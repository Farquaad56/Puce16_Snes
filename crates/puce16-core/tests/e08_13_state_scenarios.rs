//! E08-13: Integration - save state roundtrip on real scenarios (NMI, DMA, HDMA, PPU, APU).
//! Proves that saving at frame K and loading into a fresh console produces identical future
//! behavior to uninterrupted execution.

mod common;
#[path = "common/prog.rs"]
mod prog;

use puce16_core::snes::Snes;
use puce16_core::region::Region;
use prog::build_rom;

/// Handler that increments $0100 and returns (from E03-37).
const HANDLER_INC0100: &[u8] = &[
    0xEE, 0x00, 0x01, // INC $0100
    0x40,             // RTI
];

/// PROG_A from E03-37: enable NMI and loop.
const PROG_NMI: &[u8] = &[
    0x78, 0x18, 0xFB, // SEI ; CLC ; XCE (native mode)
    0x9C, 0x00, 0x01, // STZ $0100
    0xA9, 0x80,       // LDA #$80
    0x8D, 0x00, 0x42, // STA $4200 (enable NMI)
    0x80, 0xFE,       // BRA -2 (loop at offset 0x0B = pc 0x800B)
];

/// DMA general ROM -> WRAM via $2180 from E03-39.
const PROG_DMA: &[u8] = &[
    0x78, 0x18, 0xFB, // SEI CLC XCE
    0xA9, 0x00, 0x8D, 0x81, 0x21, // WMADDL = 0
    0xA9, 0x02, 0x8D, 0x82, 0x21, // WMADDM = 2 (write address mid -> 0x200)
    0x9C, 0x83, 0x21, // STZ $2183
    0x9C, 0x00, 0x43, // DMAP0 = 0 (A->B direction)
    0xA9, 0x80, 0x8D, 0x01, 0x43, // BBAD0 = $80 ($2180 - write port for WRAM via DMA)
    0x9C, 0x02, 0x43, // A1T0L = 0 (source address low)
    0xA9, 0x82, 0x8D, 0x03, 0x43, // A1T0H = $82 (source address high -> 0x8200 in CPU space)
    0x9C, 0x04, 0x43, // A1B0 = 0 (bank)
    0xA9, 0x08, 0x8D, 0x05, 0x43, // DAS0L = 8 (transfer 8 bytes low)
    0x9C, 0x06, 0x43, // DAS0H = 0 (transfer 8 bytes high)
    0xA9, 0x01, 0x8D, 0x0B, 0x42, // MDMAEN = 1 (trigger DMA channel 0)
    0xEA,             // NOP - DMA starts after this instruction's opcode fetch
    0xA9, 0x7B, 0x8D, 0x10, 0x01, // Store marker $7B at $0110 to show CPU resumed after DMA
    0xA9, 0x00, 0x8D, 0x81, 0x21, // Reset WMADDL for readback
    0xA9, 0x02, 0x8D, 0x82, 0x21, // Reset WMADDM for readback
    0xAD, 0x80, 0x21, 0x8D, 0x11, 0x01, // Read back first byte via $2180 and store at $0111
    0xAD, 0x80, 0x21, 0x8D, 0x12, 0x01, // Read back second byte via $2180 and store at $0112
    0x80, 0xFE,       // Loop forever
];

/// HDMA to $2180 with direct table from E03-40.
fn build_hdma_prog(t_hi: u8, en: u8) -> Vec<u8> {
    vec![
        0x78, 0x18, 0xFB, // SEI CLC XCE
        0xA9, 0x00, 0x8D, 0x81, 0x21, // WMADDL = 0
        0xA9, 0x03, 0x8D, 0x82, 0x21, // WMADDM = 3 (write address mid -> 0x300)
        0x9C, 0x83, 0x21, // STZ $2183
        0x9C, 0x10, 0x43, // DMAP1 = 0 (A->B direction)
        0xA9, 0x80, 0x8D, 0x11, 0x43, // BBAD1 = $80 ($2180 - write port for WRAM via DMA)
        0x9C, 0x12, 0x43, // A1T1L = 0 (table address low)
        0xA9, t_hi, 0x8D, 0x13, 0x43, // A1T1H = T_HI (table address high)
        0x9C, 0x14, 0x43, // A1B1 = 0 (bank)
        0xA9, en, 0x8D, 0x0C, 0x42, // HDMAEN = EN (enable HDMA channel 1)
        0x80, 0xFE,       // Loop forever
    ]
}

/// PPU red screen program from E04-15.
const PROG_RED: &[u8] = &[
    0x78, 0x18, 0xFB, // SEI ; CLC ; XCE
    0xA9, 0x00, 0x8D, 0x21, 0x21, // LDA #$00 ; STA $2121 (CGADD=0)
    0xA9, 0x1F, 0x8D, 0x22, 0x21, // LDA #$1F ; STA $2122 (color low byte = red)
    0xA9, 0x00, 0x8D, 0x22, 0x21, // LDA #$00 ; STA $2122 (color high byte = 0)
    0xA9, 0x0F, 0x8D, 0x00, 0x21, // LDA #$0F ; STA $2100 (display on, brightness 15)
    0x80, 0xFE,       // BRA -2 (loop)
];

/// APU handshake program from E06-37.
const PROG_APU: &[u8] = &[
    0x78, 0x18, 0xFB, // SEI ; CLC ; XCE
    0xAD, 0x40, 0x21, // LDA $2140 (wait for APU handshake byte 1)
    0xC9, 0xAA,       // CMP #$AA
    0xD0, 0xF9,       // BNE -3 (loop back to LDA $2140)
    0xAD, 0x41, 0x21, // LDA $2141 (wait for APU handshake byte 2)
    0xC9, 0xBB,       // CMP #$BB
    0xD0, 0xF2,       // BNE -8 (loop back to LDA $2140)
    0xA9, 0x00, 0x8D, 0x21, 0x21, // Set CGADD=0
    0xA9, 0x1F, 0x8D, 0x22, 0x21, // Set color 0 to red (low byte = $1F)
    0xA9, 0x00, 0x8D, 0x22, 0x21, // Set color 0 high byte = 0
    0xA9, 0x0F, 0x8D, 0x00, 0x21, // Display on, brightness 15
    0x80, 0xFE,       // Loop forever
];

fn snes(rom: Vec<u8>) -> Snes {
    Snes::new(&rom, None).ok().unwrap()
}

/// Let the CPU take any pending interrupt after run_frame.
fn settle(s: &mut Snes) {
    for _ in 0..12 {
        s.step_instruction();
    }
}

/// Compare uninterrupted execution (A) vs save/load at frame K (C).
/// A runs k+n frames; B saves at k; C loads and runs n frames.
/// At each frame after K, state_hash, pixels, and audio must match between A and C.
fn compare(rom: &[u8], k: u64, n: u64) {
    // Console A: uninterrupted execution for k+n frames
    let mut a = snes(rom.to_vec());
    let mut a_hashes = Vec::new();
    let mut a_frames = Vec::new();
    let mut a_audio = Vec::new();
    for i in 0..k + n {
        a.run_frame();
        settle(&mut a);
        if i >= k - 1 {
            a_hashes.push(a.state_hash());
            a_frames.push(a.frame().pixels.to_vec());
            let mut buf = Vec::new();
            a.take_audio(&mut buf);
            a_audio.push(buf);
        }
    }

    // Console B: run k frames, save state
    let mut b = snes(rom.to_vec());
    for _ in 0..k {
        b.run_frame();
        settle(&mut b);
    }
    // A frontend drains the audio every frame, so drain before saving (as A does at k-1).
    b.take_audio(&mut Vec::new());
    let blob = b.save_state();

    // Console C: load saved state, run n frames
    let mut c = snes(rom.to_vec());
    c.load_state(&blob).expect("load_state failed");
    for i in 0..n {
        c.run_frame();
        settle(&mut c);
        let idx = (i + 1) as usize; // frame after K is index 0 in our arrays

        assert_eq!(c.state_hash(), a_hashes[idx], "state_hash mismatch at frame {}", i + k + 1);
        assert_eq!(c.frame().pixels, &a_frames[idx], "pixel mismatch at frame {}", i + k + 1);
        let mut buf = Vec::new();
        c.take_audio(&mut buf);
        assert_eq!(buf, a_audio[idx], "audio mismatch at frame {}", i + k + 1);
    }
}

#[test]
fn e08_13_nmi_counter() {
    let rom = build_rom(
        &[
            (0, PROG_NMI),
            (0x100, HANDLER_INC0100),
        ],
        &[("nmi", 0x8100)],
    );
    compare(&rom, 3, 5);
}

#[test]
fn e08_13_dma_general() {
    let rom = build_rom(
        &[
            (0, PROG_DMA),
            // Data block at offset 0x200 (CPU address 0x8200)
            (0x200, &[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88]),
        ],
        &[],
    );
    compare(&rom, 2, 3);
}

#[test]
fn e08_13_hdma_table() {
    // Table at offset 0x300 (CPU address 0x8300): non-repeated entries
    let table = [0x01, 0xA1, 0x01, 0xA2, 0x03, 0xA3, 0x00];
    let rom = build_rom(
        &[
            (0, &build_hdma_prog(0x83, 0x02)),
            (0x300, &table),
        ],
        &[],
    );
    compare(&rom, 2, 3);
}

#[test]
fn e08_13_ppu_scene_mode1() {
    let rom = build_rom(&[(0, PROG_RED)], &[]);
    compare(&rom, 2, 3);

    // Also verify pixel (0,0) is red at the end
    let mut s = snes(rom.to_vec());
    for _ in 0..5 {
        s.run_frame();
        settle(&mut s);
    }
    let view = s.frame();
    assert_eq!(view.pixels[0], 0xFFFF_0000, "pixel (0,0) should be red");
}

#[test]
fn e08_13_apu_handshake_red() {
    let rom = build_rom(&[(0, PROG_APU)], &[]);
    // K=1: before APU handshake completes (screen still black or transitioning)
    compare(&rom, 1, 2);
    // K=3: after APU handshake completes (screen should be red)
    compare(&rom, 3, 2);
}

#[test]
fn e08_13_long_dma() {
    // Build ROM with DMA trigger but DAS = 0x0000 (65536 bytes)
    let mut prog_long = PROG_DMA.to_vec();
    // Offset of DAS0L value byte: find the A9 08 8D 05 42 sequence
    // SEI CLC XCE (3) + WMADDL (5) + WMADDM (5) + STZ $2183 (3) + DMAP0 (3)
    // + BBAD0 (5) + A1T0L (3) + A1T0H (5) + A1B0 (3) = 35 bytes
    // Then DAS0L instruction starts at offset 35: A9 xx 8D 05 42
    prog_long[36] = 0x00; // Change DAS0L from 8 to 0 (low byte of 65536)
    // DAS0H is already 0 at offset 39

    let rom = build_rom(
        &[
            (0, &prog_long),
            // Data block at offset 0x200 (CPU address 0x8200). The default ROM is 32 KiB, so the block
            // must fit in it; the DMA still moves 65536 bytes (DAS=0) and reads the rest from the
            // bank mirror, which is fine because only the copied bytes' identity matters here.
            (0x200, &[0xAA; 0x7000]),
        ],
        &[],
    );

    // K=1: save before DMA starts
    compare(&rom, 1, 3);
    // K=2: save during/after DMA (DMA takes multiple frames for 64KB)
    compare(&rom, 2, 3);
}

#[test]
fn e08_13_pal_console() {
    let rom = build_rom(&[(0, PROG_RED)], &[]);

    // Console A: PAL, uninterrupted execution
    let mut a = Snes::new(&rom, Some(Region::Pal)).ok().unwrap();
    let mut a_hashes = Vec::new();
    let mut a_frames = Vec::new();
    for i in 0..5 {
        a.run_frame();
        settle(&mut a);
        if i >= 1 {
            a_hashes.push(a.state_hash());
            a_frames.push(a.frame().pixels.to_vec());
        }
    }

    // Console B: PAL, run k frames, save state
    let mut b = Snes::new(&rom, Some(Region::Pal)).ok().unwrap();
    for _ in 0..2 {
        b.run_frame();
        settle(&mut b);
    }
    let blob = b.save_state();

    // Console C: PAL, load saved state, run n frames
    let mut c = Snes::new(&rom, Some(Region::Pal)).ok().unwrap();
    c.load_state(&blob).expect("load_state failed");
    for i in 0..3 {
        c.run_frame();
        settle(&mut c);
        let idx = (i + 1) as usize;

        assert_eq!(c.state_hash(), a_hashes[idx], "PAL state_hash mismatch at frame {}", i + 3);
        assert_eq!(c.frame().pixels, &a_frames[idx], "PAL pixel mismatch at frame {}", i + 3);
    }
}
