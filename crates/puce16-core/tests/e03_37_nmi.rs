//! E03-37: Integration - NMI (frame counter, RDNMI polling, enable in VBlank).

mod common;
#[path = "common/prog.rs"]
mod prog;

use puce16_core::snes::Snes;
use prog::build_rom;

/// Handler that increments $0100 and returns.
const HANDLER_INC0100: &[u8] = &[
    0xEE, 0x00, 0x01, // INC $0100
    0x40,             // RTI
];

/// PROG_A: SEI ; CLC ; XCE ; STZ $0100 ; LDA #$80 ; STA $4200 ; BRA -2
const PROG_A: &[u8] = &[
    0x78, 0x18, 0xFB, // SEI ; CLC ; XCE (native mode)
    0x9C, 0x00, 0x01, // STZ $0100
    0xA9, 0x80,       // LDA #$80
    0x8D, 0x00, 0x42, // STA $4200 (enable NMI)
    0x80, 0xFE,       // BRA -2 (loop at offset 0x0B = pc 0x800B)
];

/// PROG_B: SEI ; CLC ; XCE ; STZ $0100 ; BRA -2 (NMI never enabled)
const PROG_B: &[u8] = &[
    0x78, 0x18, 0xFB, // SEI ; CLC ; XCE
    0x9C, 0x00, 0x01, // STZ $0100
    0x80, 0xFE,       // BRA -2 (loop at offset 0x07 = pc 0x8007)
];

/// PROG_C: SEI ; CLC ; XCE ; STZ $0102 ; L: LDA $4210 ; BPL L ; INC $0102 ; BRA L
const PROG_C: &[u8] = &[
    0x78, 0x18, 0xFB, // SEI ; CLC ; XCE
    0x9C, 0x02, 0x01, // STZ $0102 (counter)
    0xAD, 0x10, 0x42, // L: LDA $4210 (read RDNMI flag)
    0x10, 0xFB,       // BPL L (bit 7 not set -> loop)
    0xEE, 0x02, 0x01, // INC $0102 (count this vblank)
    0x80, 0xF6,       // BRA L (loop at offset 0x06 = pc 0x8006)
];

/// PROG_G: SEI ; CLC ; XCE ; STZ $0100 ; L: LDA $4212 ; BPL L ; enable NMI ; loop
const PROG_G: &[u8] = &[
    0x78, 0x18, 0xFB, // SEI ; CLC ; XCE
    0x9C, 0x00, 0x01, // STZ $0100 (counter)
    0xAD, 0x12, 0x42, // L: LDA $4212 (read VTIME to detect vblank start)
    0x10, 0xFB,       // BPL L (not in vblank yet -> loop)
    0xA9, 0x80,       // LDA #$80
    0x8D, 0x00, 0x42, // STA $4200 (enable NMI during vblank)
    0x80, 0xFE,       // BRA -2 (loop at offset 0x0B = pc 0x800B)
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
fn e03_37_nmi_counts_frames() {
    let rom = build_rom(
        &[
            (0, PROG_A),
            (0x100, HANDLER_INC0100),
        ],
        &[("nmi", 0x8100)],
    );
    let mut s = snes(rom);

    for k in 1..=5 {
        s.run_frame();
        settle(&mut s);
        assert_eq!(s.peek_wram(0x100), k, "frame {}", k);
    }
}

#[test]
fn e03_37_nmi_not_before_vblank() {
    let rom = build_rom(
        &[(0, PROG_A)],
        &[("nmi", 0x8100)],
    );
    let mut s = snes(rom);

    // Run for ~1/3 of a frame (before vblank at ~306900 clocks)
    s.run_master(100_000);
    assert_eq!(s.peek_wram(0x100), 0, "NMI should not fire before vblank");
    assert_eq!(s.frame_count(), 0, "no frame completed yet");
}

#[test]
fn e03_37_nmi_disabled() {
    let rom = build_rom(
        &[(0, PROG_B)],
        &[("nmi", 0x8100)],
    );
    let mut s = snes(rom);

    for _ in 0..3 {
        s.run_frame();
        settle(&mut s);
    }
    assert_eq!(s.peek_wram(0x100), 0, "NMI should never fire when disabled");
}

#[test]
fn e03_37_rdnmi_polling() {
    let rom = build_rom(&[(0, PROG_C)], &[]);
    let mut s = snes(rom);

    for k in 1..=4 {
        s.run_frame();
        settle(&mut s);
        assert_eq!(s.peek_wram(0x102), k, "RDNMI should be seen once per frame (frame {})", k);
    }
}

#[test]
fn e03_37_enable_in_vblank_fires() {
    let rom = build_rom(
        &[
            (0, PROG_G),
            (0x100, HANDLER_INC0100),
        ],
        &[("nmi", 0x8100)],
    );
    let mut s = snes(rom);

    // First frame: enable NMI during vblank -> should fire immediately
    s.run_frame();
    settle(&mut s);
    assert_eq!(s.peek_wram(0x100), 1, "NMI should fire when enabled in vblank");

    // Second frame: normal NMI at start of vblank
    s.run_frame();
    settle(&mut s);
    assert_eq!(s.peek_wram(0x100), 2, "second NMI should also fire");
}

#[test]
fn e03_37_nmi_handler_returns() {
    let rom = build_rom(
        &[(0, PROG_A)],
        &[("nmi", 0x8100)],
    );
    let mut s = snes(rom);

    s.run_frame();
    settle(&mut s);

    // CPU should be back in the main loop at pc 0x800B (BRA -2)
    let dbg = s.debug();
    assert_eq!(dbg.pc, 0x800B, "CPU should return to main loop after NMI handler");
    assert!(!dbg.e, "should be in native mode");
}

#[test]
fn e03_37_nmi_native_vs_emulation() {
    // Same as PROG_A but without CLC;XCE (stays in emulation mode)
    let prog_a_emu: &[u8] = &[
        0x78,             // SEI
        0x9C, 0x00, 0x01, // STZ $0100
        0xA9, 0x80,       // LDA #$80
        0x8D, 0x00, 0x42, // STA $4200 (enable NMI)
        0x80, 0xFE,       // BRA -2 (loop at offset 0x07 = pc 0x8007)
    ];

    let rom = build_rom(
        &[
            (0, prog_a_emu),
            (0x100, HANDLER_INC0100),
        ],
        &[("nmi", 0x8100)],
    );
    let mut s = snes(rom);

    s.run_frame();
    settle(&mut s);

    // NMI should still fire (uses emulation vector $FFFA which we set to same address)
    assert_eq!(s.peek_wram(0x100), 1, "NMI should work in emulation mode");
    let dbg = s.debug();
    assert!(dbg.e, "should be in emulation mode after handler returns");
}
