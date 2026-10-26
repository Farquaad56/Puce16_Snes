//! E03-40: Integration - HDMA to $2180 (direct table, repeat mode).

mod common;
#[path = "common/prog.rs"]
mod prog;

use puce16_core::snes::Snes;
use prog::build_rom;

/// PROG_HDMA: configure channel 1 for HDMA from ROM to WRAM via $2180.
/// T_HI is the high byte of A1T1H (table address). EN is the HDMAEN value.
fn build_hdma_prog(t_hi: u8, en: u8) -> Vec<u8> {
    vec![
        // SEI CLC XCE - disable interrupts, clear carry, switch to native mode
        0x78, 0x18, 0xFB,
        // WMADDL = 0 (write address low)
        0xA9, 0x00, 0x8D, 0x81, 0x21,
        // WMADDM = 3 (write address mid -> 0x300)
        0xA9, 0x03, 0x8D, 0x82, 0x21,
        // STZ $2183 (clear some register)
        0x9C, 0x83, 0x21,
        // DMAP1 = 0 (A->B direction, pattern 0, auto-increment source)
        0x9C, 0x10, 0x43,
        // BBAD1 = $80 ($2180 - write port for WRAM via DMA)
        0xA9, 0x80, 0x8D, 0x11, 0x43,
        // A1T1L = 0 (table address low)
        0x9C, 0x12, 0x43,
        // A1T1H = T_HI (table address high)
        0xA9, t_hi, 0x8D, 0x13, 0x43,
        // A1B1 = 0 (bank)
        0x9C, 0x14, 0x43,
        // HDMAEN = EN (enable HDMA channel 1)
        0xA9, en, 0x8D, 0x0C, 0x42,
        // Loop forever
        0x80, 0xFE,
    ]
}

fn snes(rom: Vec<u8>) -> Snes {
    Snes::new(&rom, None).ok().unwrap()
}

/// Let the CPU take any pending interrupt after run_frame.
fn settle(s: &mut Snes) {
    for _ in 0..12 {
        s.step_instruction();
    }
}

#[test]
fn e03_40_frame0_no_hdma() {
    // Table 1 at offset 0x300 (CPU address 0x8300): non-repeated entries
    let table1 = [0x01, 0xA1, 0x01, 0xA2, 0x03, 0xA3, 0x00];
    let rom = build_rom(
        &[
            (0, &build_hdma_prog(0x83, 0x02)),
            (0x300, &table1),
        ],
        &[],
    );
    let mut s = snes(rom);

    // Run frame 0 - HDMA init happens at (0,3) before program enables HDMAEN
    s.run_frame();
    settle(&mut s);

    // No transfer should have occurred yet
    for i in 0..6 {
        assert_eq!(s.peek_wram(0x300 + i), 0x55, "frame 0: byte {} transferred too early", i);
    }
}

#[test]
fn e03_40_frame1_three_bytes() {
    // Table 1 at offset 0x300 (CPU address 0x8300): non-repeated entries
    let table1 = [0x01, 0xA1, 0x01, 0xA2, 0x03, 0xA3, 0x00];
    let rom = build_rom(
        &[
            (0, &build_hdma_prog(0x83, 0x02)),
            (0x300, &table1),
        ],
        &[],
    );
    let mut s = snes(rom);

    // Frame 0: init happens before HDMAEN is set
    s.run_frame();
    settle(&mut s);

    // Frame 1: first actual HDMA transfer (3 bytes from table)
    s.run_frame();
    settle(&mut s);

    assert_eq!(s.peek_wram(0x300), 0xA1, "frame 1: byte 0 wrong");
    assert_eq!(s.peek_wram(0x301), 0xA2, "frame 1: byte 1 wrong");
    assert_eq!(s.peek_wram(0x302), 0xA3, "frame 1: byte 2 wrong");
    assert_eq!(s.peek_wram(0x303), 0x55, "frame 1: byte 3 should be unchanged");
}

#[test]
fn e03_40_frame2_repeats() {
    // Table 1 at offset 0x300 (CPU address 0x8300): non-repeated entries
    let table1 = [0x01, 0xA1, 0x01, 0xA2, 0x03, 0xA3, 0x00];
    let rom = build_rom(
        &[
            (0, &build_hdma_prog(0x83, 0x02)),
            (0x300, &table1),
        ],
        &[],
    );
    let mut s = snes(rom);

    // Frame 0: init happens before HDMAEN is set
    s.run_frame();
    settle(&mut s);

    // Frame 1: first transfer (3 bytes)
    s.run_frame();
    settle(&mut s);

    // Frame 2: table reinitialized, another 3 bytes at next WMADD position
    s.run_frame();
    settle(&mut s);

    let expected = [0xA1, 0xA2, 0xA3, 0xA1, 0xA2, 0xA3];
    for i in 0..6 {
        assert_eq!(s.peek_wram(0x300 + i), expected[i as usize], "frame 2: byte {} wrong", i);
    }
    assert_eq!(s.peek_wram(0x306), 0x55, "frame 2: byte 6 should be unchanged");
}

#[test]
fn e03_40_repeat_mode_table() {
    // Table 2 at offset 0x400 (CPU address 0x8400): repeated entry
    let table2 = [0x83, 0xB1, 0xB2, 0xB3, 0x00];
    let rom = build_rom(
        &[
            (0, &build_hdma_prog(0x84, 0x02)),
            (0x400, &table2),
        ],
        &[],
    );
    let mut s = snes(rom);

    // Frame 0: init happens before HDMAEN is set
    s.run_frame();
    settle(&mut s);

    // Frame 1: repeat mode writes B1,B2,B3 every line for 3 lines
    s.run_frame();
    settle(&mut s);

    assert_eq!(s.peek_wram(0x300), 0xB1, "repeat: byte 0 wrong");
    assert_eq!(s.peek_wram(0x301), 0xB2, "repeat: byte 1 wrong");
    assert_eq!(s.peek_wram(0x302), 0xB3, "repeat: byte 2 wrong");
    assert_eq!(s.peek_wram(0x303), 0x55, "repeat: byte 3 should be unchanged");
}

#[test]
fn e03_40_hdma_disabled() {
    // Same program but HDMAEN = 0 (disabled)
    let table1 = [0x01, 0xA1, 0x01, 0xA2, 0x03, 0xA3, 0x00];
    let rom = build_rom(
        &[
            (0, &build_hdma_prog(0x83, 0x00)),
            (0x300, &table1),
        ],
        &[],
    );
    let mut s = snes(rom);

    for _ in 0..3 {
        s.run_frame();
        settle(&mut s);
    }

    assert_eq!(s.peek_wram(0x300), 0x55, "HDMA occurred with HDMAEN=0");
}

#[test]
fn e03_40_other_channel_bit() {
    // Enable channel 0 instead of channel 1 (channel 0 not configured)
    let table1 = [0x01, 0xA1, 0x01, 0xA2, 0x03, 0xA3, 0x00];
    let rom = build_rom(
        &[
            (0, &build_hdma_prog(0x83, 0x01)),
            (0x300, &table1),
        ],
        &[],
    );
    let mut s = snes(rom);

    for _ in 0..3 {
        s.run_frame();
        settle(&mut s);
    }

    assert_eq!(s.peek_wram(0x300), 0x55, "HDMA occurred on wrong channel");
}

#[test]
fn e03_40_deterministic() {
    let table1 = [0x01, 0xA1, 0x01, 0xA2, 0x03, 0xA3, 0x00];
    let rom = build_rom(
        &[
            (0, &build_hdma_prog(0x83, 0x02)),
            (0x300, &table1),
        ],
        &[],
    );

    let mut s1 = snes(rom.clone());
    let mut s2 = snes(rom);

    for _ in 0..3 {
        s1.run_frame();
        settle(&mut s1);
        s2.run_frame();
        settle(&mut s2);
    }

    assert_eq!(s1.state_hash(), s2.state_hash(), "HDMA execution not deterministic");
}