//! E10-03: Cheat application on console (Snes::set_cheats, silent writes each frame).

use puce16_core::cheat::ParCode;
use puce16_core::snes::Snes;

mod common;
#[path = "common/prog.rs"]
mod prog;
use prog::build_rom_with;

/// Build a simple INC $00 loop program.
fn inc_loop_program() -> Vec<u8> {
    // E6 00 = INC $00 (absolute)
    // FC FF FD = BRA -4 (loop forever, relative offset -4 from next instruction)
    vec![0xE6, 0x00, 0x00, 0xFC, 0xFF, 0xFD]
}

/// Build a ROM with the INC loop program.
fn build_inc_rom() -> Vec<u8> {
    let prog = inc_loop_program();
    build_rom_with(&[(0, &prog)], &[("reset", 0x8000)], 0, 0x00)
}

/// Build a ROM with SRAM (8 KiB).
fn build_sram_rom() -> Vec<u8> {
    let prog = inc_loop_program();
    // sram_log2=3 means 1024 << 3 = 8 KiB of SRAM, chipset 0x02 = ROM + RAM + battery
    build_rom_with(&[(0, &prog)], &[("reset", 0x8000)], 3, 0x02)
}

#[test]
fn e10_03_wram_bank7e_written_each_frame() {
    let rom = build_inc_rom();
    let mut snes = Snes::new(&rom, None).expect("ROM should load");

    // Cheat: write 0x42 to bank 7E addr 0 each frame
    let cheat = ParCode { addr: 0x7E0000, value: 0x42 };
    snes.set_cheats(&[cheat]);

    for _ in 0..3 {
        snes.run_frame();
        // Program increments $00 each instruction, but cheat overwrites it back to 0x42
        assert_eq!(snes.peek_wram(0), 0x42);
    }
}

#[test]
fn e10_03_wram_low_mirror_and_bank7f() {
    let rom = build_inc_rom();
    let mut snes = Snes::new(&rom, None).expect("ROM should load");

    // Cheats at different WRAM locations:
    // 00:0100 -> bank 00 addr 0x100 (low mirror)
    // 7F:0010 -> bank 7F addr 0x10 (upper WRAM)
    // 7E:0200 -> bank 7E addr 0x200
    let cheats = vec![
        ParCode { addr: 0x000100, value: 0x11 },
        ParCode { addr: 0x7F0010, value: 0x22 },
        ParCode { addr: 0x7E0200, value: 0x33 },
    ];
    snes.set_cheats(&cheats);

    snes.run_frame();

    assert_eq!(snes.peek_wram(0x100), 0x11);
    // Bank 7F maps to upper WRAM at offset 0x10000
    assert_eq!(snes.peek_wram(0x10010), 0x22);
    assert_eq!(snes.peek_wram(0x200), 0x33);
}

#[test]
fn e10_03_sram_written_through_cart() {
    let rom = build_sram_rom();
    let mut snes = Snes::new(&rom, None).expect("ROM should load");

    // Cheat: write 0x5A to bank 70 addr 1 (SRAM area)
    let cheat = ParCode { addr: 0x700001, value: 0x5A };
    snes.set_cheats(&[cheat]);

    snes.run_frame();

    let sram = snes.sram().expect("ROM should have SRAM");
    assert_eq!(sram[1], 0x5A);
}

#[test]
fn e10_03_rom_and_io_ignored() {
    let rom = build_inc_rom();
    let mut snes = Snes::new(&rom, None).expect("ROM should load");

    // Cheats at ROM and IO addresses (should be ignored):
    // 00:8000 -> ROM area (bank 00 addr >= 0x6000) - goes to cart.write which ignores ROM writes
    // 00:2100 -> PPU register area (ignored by cheat_write)
    // 00:4200 -> MMIO area (ignored by cheat_write)
    // Plus one valid WRAM write at 7E:0300 to verify it still works
    let cheats = vec![
        ParCode { addr: 0x008000, value: 0xEE },
        ParCode { addr: 0x002100, value: 0x0F },
        ParCode { addr: 0x004200, value: 0x81 },
        ParCode { addr: 0x7E0300, value: 0x01 },
    ];
    snes.set_cheats(&cheats);

    snes.run_frame();

    // Valid WRAM write should have taken effect
    assert_eq!(snes.peek_wram(0x300), 0x01);

    // ROM not modified: program still runs (no crash, state_hash computable)
    let _hash = snes.state_hash();

    // Run another frame to confirm program is still executing
    snes.run_frame();
}

#[test]
fn e10_03_order_last_wins() {
    let rom = build_inc_rom();
    let mut snes = Snes::new(&rom, None).expect("ROM should load");

    // Two cheats at same address: last one wins
    let cheats = vec![
        ParCode { addr: 0x7E0400, value: 0x01 },
        ParCode { addr: 0x7E0400, value: 0x02 },
    ];
    snes.set_cheats(&cheats);

    snes.run_frame();

    assert_eq!(snes.peek_wram(0x400), 0x02);
}

#[test]
fn e10_03_no_cheat_hash_unchanged() {
    let rom = build_inc_rom();

    // Console with empty cheat list
    let mut snes_with_empty = Snes::new(&rom, None).expect("ROM should load");
    snes_with_empty.set_cheats(&[]);
    snes_with_empty.run_frame();
    snes_with_empty.run_frame();

    // Console without calling set_cheats at all
    let mut snes_without = Snes::new(&rom, None).expect("ROM should load");
    snes_without.run_frame();
    snes_without.run_frame();

    assert_eq!(snes_with_empty.state_hash(), snes_without.state_hash());
    assert_eq!(snes_with_empty.cheats().len(), 0);
}

#[test]
fn e10_03_cheats_survive_reset_and_load_state() {
    let rom = build_inc_rom();
    let mut snes = Snes::new(&rom, None).expect("ROM should load");

    // Set a cheat
    let cheat = ParCode { addr: 0x7E0500, value: 0x07 };
    snes.set_cheats(&[cheat]);

    // Reset should not clear cheats
    snes.reset();
    assert_eq!(snes.cheats().len(), 1);

    // Run a frame to apply the cheat
    snes.run_frame();
    assert_eq!(snes.peek_wram(0x500), 0x07);

    // Save state, load it back - cheats should survive
    let saved = snes.save_state();
    snes.load_state(&saved).expect("load_state should succeed");
    assert_eq!(snes.cheats().len(), 1);

    // Cheat still applied on next frame
    snes.run_frame();
    assert_eq!(snes.peek_wram(0x500), 0x07);
}