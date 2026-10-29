use puce16_core::{Snes, Region};

mod common;
use common::{make_rom, Map};

#[test]
fn e08_32_pattern_variant_empty() {
    let snes = Snes::new_test_pattern(Region::Ntsc);
    assert!(snes.debug_vram().is_empty());
    assert!(snes.debug_cgram().is_empty());
    assert!(snes.debug_oam().is_empty());
    assert!(snes.debug_ppu_regs().is_none());
    assert!(snes.debug_smp().is_none());
    assert!(snes.debug_dma().is_none());
    assert_eq!(snes.debug_wram_page(0), vec![0u8; 256]);
}

#[test]
fn e08_32_real_sizes() {
    let rom = make_rom(&[0x80, 0xFE], Map::LoRom);
    let snes = Snes::new(&rom, None).unwrap();
    assert_eq!(snes.debug_vram().len(), 32768);
    assert_eq!(snes.debug_cgram().len(), 256);
    assert_eq!(snes.debug_oam().len(), 544);
    assert!(snes.debug_ppu_regs().is_some());
    assert!(snes.debug_smp().is_some());
    assert!(snes.debug_dma().is_some());
}

#[test]
fn e08_32_wram_page_content() {
    // Program: SEI CLC XCE LDA #$42 STA $0010 BRA -2 (infinite loop writing 0x42 to $0010)
    let rom = make_rom(&[0x78, 0x18, 0xFB, 0xA9, 0x42, 0x8D, 0x10, 0x00, 0x80, 0xFE], Map::LoRom);
    let mut snes = Snes::new(&rom, None).unwrap();
    snes.run_frame();

    // Page 0 should have 0x42 at offset 0x10
    let page0 = snes.debug_wram_page(0);
    assert_eq!(page0[0x10], 0x42);

    // Page 1 should not have it (different page)
    let page1 = snes.debug_wram_page(1);
    assert_eq!(page1[0x10], 0x55); // power-on WRAM pattern (D-011), untouched by the program

    // Each page is exactly 256 bytes
    assert_eq!(page0.len(), 256);
    assert_eq!(page1.len(), 256);

    // Page 512 wraps to page 0 (mask 0x1FF)
    let page512 = snes.debug_wram_page(512);
    assert_eq!(page512, page0);
}

#[test]
fn e08_32_run_until_pc_reaches() {
    // Same program as above - runs to $8008 (the LDA #$42 instruction)
    let rom = make_rom(&[0x78, 0x18, 0xFB, 0xA9, 0x42, 0x8D, 0x10, 0x00, 0x80, 0xFE], Map::LoRom);
    let mut snes = Snes::new(&rom, None).unwrap();

    // Run until we reach the LDA #$42 at $8008 (after SEI CLC XCE)
    let reached = snes.debug_run_until_pc(0, 0x8008, 100);
    assert!(reached);
    assert_eq!(snes.debug().pc, 0x8008);
    assert_eq!(snes.debug().pb, 0);

    // Execute a few more instructions to get past the LDA/STA
    snes.step_instruction(); // LDA #$42
    snes.step_instruction(); // STA $0010
    snes.step_instruction(); // BRA -2 (loops back)
    assert_eq!(snes.peek_wram(0x10), 0x42);
}

#[test]
fn e08_32_run_until_pc_already_there() {
    let rom = make_rom(&[0x78, 0x18, 0xFB], Map::LoRom);
    let mut snes = Snes::new(&rom, None).unwrap();

    // At reset, PC is at $8000. Running until $8000 should return true immediately.
    let reached = snes.debug_run_until_pc(0, 0x8000, 100);
    assert!(reached);
    assert_eq!(snes.debug().pc, 0x8000);
}

#[test]
fn e08_32_run_until_pc_limit() {
    let rom = make_rom(&[0x78, 0x18, 0xFB], Map::LoRom);
    let mut snes = Snes::new(&rom, None).unwrap();

    // Try to reach an address that's never executed (infinite loop at $8000-$8002)
    let reached = snes.debug_run_until_pc(0, 0x9000, 50);
    assert!(!reached);

    // CPU should have advanced but not panicked - it's still in the reset code area
    let pc = snes.debug().pc;
    assert!(pc >= 0x8000 && pc <= 0x8009, "PC should be in reset code range, got {:#x}", pc);
}

#[test]
fn e08_32_smp_snapshot_after_boot() {
    let rom = make_rom(&[0x80, 0xFE], Map::LoRom);
    let mut snes = Snes::new(&rom, None).unwrap();

    // Run two frames to let SMP boot and do IPL handshake
    snes.run_frame();
    snes.run_frame();

    // debug_smp should return Some after SMP has had time to run
    let smp = snes.debug_smp();
    assert!(smp.is_some(), "SMP snapshot should be available after 2 frames");
}
