//! E03-36: First integration test - synthetic ROM boots, CPU executes real code.

mod common;
#[path = "common/prog.rs"]
mod prog;

use puce16_core::snes::Snes;
use prog::build_rom;

/// BOOT program: SEI ; CLC ; XCE ; LDA #$42 ; STA $0010 ; LDA #$99 ; STA $7E0020 ; BRA -2
const PROG_BOOT: &[u8] = &[
    0x78, // SEI
    0x18, // CLC
    0xFB, // XCE (switch to native mode)
    0xA9, 0x42, // LDA #$42
    0x8D, 0x10, 0x00, // STA $0010
    0xA9, 0x99, // LDA #$99
    0x8F, 0x20, 0x00, 0x7E, // STA $7E0020 (WRAM)
    0x80, 0xFE, // BRA -2 (infinite loop at offset 0x0E = pc 0x800E)
];

fn snes(rom: Vec<u8>) -> Snes {
    Snes::new(&rom, None).ok().unwrap()
}

#[test]
fn e03_36_build_rom_layout() {
    // Basic layout: two blocks at different offsets
    let rom = build_rom(&[(0, &[1, 2]), (0x10, &[3])], &[]);
    assert_eq!(rom.len(), 32768);
    assert_eq!(&rom[0..2], &[1, 2]);
    assert_eq!(rom[0x10], 3);
    assert_eq!(rom[2], 0); // zero elsewhere
    assert_eq!(rom[0x0F], 0);

    // NMI vector at $FFEA (header offset 0x7FEA)
    let rom_nmi = build_rom(&[], &[("nmi", 0x8123)]);
    assert_eq!(&rom_nmi[0x7FEA..0x7FEC], &[0x23, 0x81]);

    // Reset vector at $FFFC (header offset 0x7FFC)
    let rom_reset = build_rom(&[], &[("reset", 0x8010)]);
    assert_eq!(&rom_reset[0x7FFC..0x7FFE], &[0x10, 0x80]);
}

#[test]
fn e03_36_build_rom_checksum() {
    let rom = build_rom(&[(0, &[1, 2, 3])], &[("nmi", 0x8100)]);

    // Copy rom and overwrite checksum/complement bytes with [0xFF, 0xFF, 0x00, 0x00]
    let mut copy = rom.clone();
    copy[0x7FDC] = 0xFF;
    copy[0x7FDD] = 0xFF;
    copy[0x7FDE] = 0x00;
    copy[0x7FDF] = 0x00;

    // Sum all bytes (u16 wrapping) of the modified copy
    let mut sum: u16 = 0;
    for &b in copy.iter() {
        sum = sum.wrapping_add(b as u16);
    }

    // This sum should equal the word read from 0x7FDE in the original rom (LE)
    let stored_checksum = rom[0x7FDE] as u16 | ((rom[0x7FDF] as u16) << 8);
    assert_eq!(stored_checksum, sum);

    // And the word at 0x7FDC should be !checksum (LE)
    let stored_complement = rom[0x7FDC] as u16 | ((rom[0x7FDD] as u16) << 8);
    assert_eq!(stored_complement, !sum);
}

#[test]
fn e03_36_boot_writes_wram() {
    let mut snes = snes(build_rom(&[(0, PROG_BOOT)], &[]));
    snes.run_frame();

    // Program wrote $42 to $0010 and $99 to WRAM $7E0020
    assert_eq!(snes.peek_wram(0x10), 0x42);
    assert_eq!(snes.peek_wram(0x20), 0x99);

    // CPU should be in the infinite loop at pc 0x800E (BRA -2)
    let dbg = snes.debug();
    assert_eq!(dbg.pc, 0x800E);
    assert_eq!(dbg.pb, 0);
    assert!(!dbg.e); // XCE switched to native mode
}

#[test]
fn e03_36_reset_vector_respected() {
    // Place code at offset 0x10 and set reset vector there
    let rom = build_rom(&[(0x10, PROG_BOOT)], &[("reset", 0x8010)]);
    let mut snes = snes(rom);

    // After Snes::new, PC should be at the custom reset vector
    assert_eq!(snes.debug().pc, 0x8010);

    // Run one frame: program executes and writes to WRAM
    snes.run_frame();
    assert_eq!(snes.peek_wram(0x10), 0x42);

    // PC should be in the loop at 0x801E (0x8010 + 0xE)
    assert_eq!(snes.debug().pc, 0x801E);
}

#[test]
fn e03_36_wram_initial_untouched() {
    let mut snes = snes(build_rom(&[(0, PROG_BOOT)], &[]));

    // Before any execution, WRAM should be untouched (initialized to 0x55)
    assert_eq!(snes.peek_wram(0x10), 0x55);
    assert_eq!(snes.peek_wram(0x20), 0x55);

    // After running the program, WRAM gets written
    snes.run_frame();
    assert_eq!(snes.peek_wram(0x10), 0x42);
    assert_eq!(snes.peek_wram(0x20), 0x99);

    // Adjacent byte should still be untouched
    assert_eq!(snes.peek_wram(0x11), 0x55);
}

#[test]
fn e03_36_starts_in_emulation_mode() {
    let mut snes = snes(build_rom(&[(0, PROG_BOOT)], &[]));

    // Just after new(), CPU is in emulation mode
    assert!(snes.debug().e);

    // After executing SEI, CLC, XCE (3 instructions), it switches to native mode
    snes.step_instruction(); // SEI
    snes.step_instruction(); // CLC
    snes.step_instruction(); // XCE
    assert!(!snes.debug().e);
}
