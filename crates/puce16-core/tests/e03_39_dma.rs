//! E03-39: Integration - DMA general ROM -> WRAM via $2180 and readback.

mod common;
#[path = "common/prog.rs"]
mod prog;

use puce16_core::snes::Snes;
use prog::build_rom;

/// PROG_DMA: configure channel 0 for A->B transfer from ROM (0x8200) to WRAM via $2180,
/// trigger DMA, then read back the transferred data.
const PROG_DMA: &[u8] = &[
    // SEI CLC XCE - disable interrupts, clear carry, switch to native mode
    0x78, 0x18, 0xFB,
    // WMADDL = 0 (write address low)
    0xA9, 0x00, 0x8D, 0x81, 0x21,
    // WMADDM = 2 (write address mid -> 0x200)
    0xA9, 0x02, 0x8D, 0x82, 0x21,
    // STZ $2183 (clear some register)
    0x9C, 0x83, 0x21,
    // DMAP0 = 0 (A->B direction, pattern 0, auto-increment source)
    0x9C, 0x00, 0x43,
    // BBAD0 = $80 ($2180 - write port for WRAM via DMA)
    0xA9, 0x80, 0x8D, 0x01, 0x43,
    // A1T0L = 0 (source address low)
    0x9C, 0x02, 0x43,
    // A1T0H = $82 (source address high -> 0x8200 in CPU space)
    0xA9, 0x82, 0x8D, 0x03, 0x43,
    // A1B0 = 0 (bank)
    0x9C, 0x04, 0x43,
    // DAS0L = 8 (transfer 8 bytes low)
    0xA9, 0x08, 0x8D, 0x05, 0x43,
    // DAS0H = 0 (transfer 8 bytes high)
    0x9C, 0x06, 0x43,
    // MDMAEN = 1 (trigger DMA channel 0)
    0xA9, 0x01, 0x8D, 0x0B, 0x42,
    // NOP - DMA starts after this instruction's opcode fetch
    0xEA,
    // Store marker $7B at $0110 to show CPU resumed after DMA
    0xA9, 0x7B, 0x8D, 0x10, 0x01,
    // Reset WMADD to 0x200 for readback
    0xA9, 0x00, 0x8D, 0x81, 0x21,
    0xA9, 0x02, 0x8D, 0x82, 0x21,
    // Read back first byte via $2180 and store at $0111
    0xAD, 0x80, 0x21, 0x8D, 0x11, 0x01,
    // Read back second byte via $2180 and store at $0112
    0xAD, 0x80, 0x21, 0x8D, 0x12, 0x01,
    // Loop forever
    0x80, 0xFE,
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

#[test]
fn e03_39_data_copied() {
    let rom = build_rom(
        &[
            (0, PROG_DMA),
            // Data block at offset 0x200 (CPU address 0x8200)
            (0x200, &[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88]),
        ],
        &[],
    );
    let mut s = snes(rom);

    // WRAM powers up as 0x55 (verified in e03_26_sysbus.rs)
    assert_eq!(s.peek_wram(0x208), 0x55, "WRAM power-on default should be 0x55");

    s.run_frame();
    settle(&mut s);

    let expected = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
    for i in 0..8 {
        assert_eq!(s.peek_wram(0x200 + i), expected[i as usize], "byte {} not copied", i);
    }
    // Byte at 0x208 should be unchanged (still 0x55)
    assert_eq!(s.peek_wram(0x208), 0x55, "byte past transfer range changed");
}

#[test]
fn e03_39_cpu_resumes() {
    let rom = build_rom(
        &[
            (0, PROG_DMA),
            (0x200, &[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88]),
        ],
        &[],
    );
    let mut s = snes(rom);

    s.run_frame();
    settle(&mut s);

    // CPU should have resumed and stored the marker $7B at $0110
    assert_eq!(s.peek_wram(0x110), 0x7B, "CPU did not resume after DMA");
}

#[test]
fn e03_39_wmdata_readback() {
    let rom = build_rom(
        &[
            (0, PROG_DMA),
            (0x200, &[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88]),
        ],
        &[],
    );
    let mut s = snes(rom);

    s.run_frame();
    settle(&mut s);

    // Readback via $2180 with WMADD reset to 0x200 should return the transferred data
    assert_eq!(s.peek_wram(0x111), 0x11, "first readback byte wrong");
    assert_eq!(s.peek_wram(0x112), 0x22, "second readback byte wrong");
}

#[test]
fn e03_39_dma_takes_time() {
    // Build ROM with DMA trigger (MDMAEN = 1)
    let rom_dma = build_rom(
        &[
            (0, PROG_DMA),
            (0x200, &[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88]),
        ],
        &[],
    );

    // Build ROM without DMA trigger (MDMAEN = 0) - same instruction timing otherwise
    let mut prog_nodma = PROG_DMA.to_vec();
    // Offset of MDMAEN value byte: find the A9 01 8D 0B 42 sequence
    // The program is at offset 0, so we need to calculate the exact position
    // SEI CLC XCE (3) + WMADDL (5) + WMADDM (5) + STZ $2183 (3) + DMAP0 (3)
    // + BBAD0 (5) + A1T0L (3) + A1T0H (5) + A1B0 (3) + DAS0L (5) + DAS0H (3) = 43 bytes
    // Then MDMAEN instruction starts at offset 43: A9 xx 8D 0B 42
    prog_nodma[44] = 0x00; // Change MDMAEN value from 1 to 0

    let rom_nodma = build_rom(
        &[
            (0, &prog_nodma),
            (0x200, &[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88]),
        ],
        &[],
    );

    let mut s_dma = snes(rom_dma);
    let mut s_nodma = snes(rom_nodma);

    // Run exactly 22 instructions (up to and including the NOP after MDMAEN write)
    for _ in 0..22 {
        s_dma.step_instruction();
    }
    for _ in 0..22 {
        s_nodma.step_instruction();
    }

    let master_dma = s_dma.debug().master;
    let master_nodma = s_nodma.debug().master;
    let diff = master_dma - master_nodma;

    // Expected: 8 (start) + 8 (channel overhead) + 8*8 (bytes) = 80 clocks minimum
    // Plus alignment (0-7) and possibly a DRAM refresh (40 clocks) delayed by DMA
    assert!(diff >= 80 && diff <= 130, "DMA timing off: expected 80-130, got {}", diff);
}

#[test]
fn e03_39_dma_data_not_before_trigger() {
    let rom = build_rom(
        &[
            (0, PROG_DMA),
            (0x200, &[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88]),
        ],
        &[],
    );
    let mut s = snes(rom);

    // WRAM powers up as 0x55 (verified in e03_26_sysbus.rs)
    assert_eq!(s.peek_wram(0x200), 0x55, "WRAM power-on default should be 0x55");

    // Step instruction by instruction until we reach the MDMAEN write (STA $420B)
    // The STA $420B is at program offset 41-45 (A9 01 8D 0B 42)
    // CPU address = 0x8000 + 41 = 0x8029 for the LDA, 0x802C for STA $420B
    loop {
        let dbg = s.debug();
        if dbg.pc == 0x8030 {
            // We've just executed STA $420B (MDMAEN write), NOP is next
            break;
        }
        s.step_instruction();
    }

    // DMA should not have started yet - data still at original value
    assert_eq!(s.peek_wram(0x200), 0x55, "DMA started before trigger");

    // Execute one more instruction (the NOP) - DMA starts after opcode fetch of this
    s.step_instruction();

    // Now DMA should have completed
    assert_eq!(s.peek_wram(0x200), 0x11, "first byte not copied after trigger");
    assert_eq!(s.peek_wram(0x207), 0x88, "last byte not copied after trigger");
}

#[test]
fn e03_39_deterministic() {
    let rom = build_rom(
        &[
            (0, PROG_DMA),
            (0x200, &[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88]),
        ],
        &[],
    );

    let mut s1 = snes(rom.clone());
    let mut s2 = snes(rom);

    for _ in 0..2 {
        s1.run_frame();
        settle(&mut s1);
        s2.run_frame();
        settle(&mut s2);
    }

    assert_eq!(s1.state_hash(), s2.state_hash(), "DMA execution not deterministic");
}

#[test]
fn e03_39_mdmaen_zero_no_dma() {
    // Build ROM with MDMAEN = 0 (no DMA trigger)
    let mut prog_nodma = PROG_DMA.to_vec();
    prog_nodma[44] = 0x00; // Change MDMAEN value from 1 to 0

    let rom = build_rom(
        &[
            (0, &prog_nodma),
            (0x200, &[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88]),
        ],
        &[],
    );
    let mut s = snes(rom);

    // WRAM powers up as 0x55 (verified in e03_26_sysbus.rs)
    assert_eq!(s.peek_wram(0x200), 0x55, "WRAM power-on default should be 0x55");

    s.run_frame();
    settle(&mut s);

    // No DMA should have occurred - data still at original fill value
    assert_eq!(s.peek_wram(0x200), 0x55, "DMA occurred with MDMAEN=0");
}