//! E03-38: Integration - IRQ H/V (modes 1, 2, 3, mask I, acknowledgment).

mod common;
#[path = "common/prog.rs"]
mod prog;

use puce16_core::snes::Snes;
use prog::build_rom;

/// Handler that acknowledges IRQ ($4211), increments $0103, and returns.
const HANDLER_IRQ: &[u8] = &[
    0xAD, 0x11, 0x42, // LDA $4211 (acknowledge)
    0xEE, 0x03, 0x01, // INC $0103
    0x40,             // RTI
];

/// PROG_H: mode 1 (H only), HTIME=100. IRQ every line.
const PROG_H: &[u8] = &[
    0x78, 0x18, 0xFB, // SEI ; CLC ; XCE (native mode)
    0x9C, 0x03, 0x01, // STZ $0103 (counter)
    0xA9, 0x64,       // LDA #$64 (=100)
    0x8D, 0x07, 0x42, // STA $4207 (HTIME)
    0x9C, 0x08, 0x42, // STZ $4208 (VTIME=0 -> mode 1: H only)
    0xA9, 0x10,       // LDA #$10 (=16 = bit 4 = enable H IRQ)
    0x8D, 0x00, 0x42, // STA $4200 (NMITIMEN: enable H IRQ)
    0x58,             // CLI (unmask interrupts)
    0x80, 0xFE,       // BRA -2 (loop at offset 0x13 = pc 0x8013)
];

/// PROG_V: mode 2 (V only), VTIME=100. IRQ once per frame at line 100.
const PROG_V: &[u8] = &[
    0x78, 0x18, 0xFB, // SEI ; CLC ; XCE
    0x9C, 0x03, 0x01, // STZ $0103 (counter)
    0xA9, 0x64,       // LDA #$64 (=100)
    0x8D, 0x09, 0x42, // STA $4209 (VTIME)
    0x9C, 0x0A, 0x42, // STZ $420A (HTIME=0 -> mode 2: V only)
    0xA9, 0x20,       // LDA #$20 (=32 = bit 5 = enable V IRQ)
    0x8D, 0x00, 0x42, // STA $4200 (NMITIMEN: enable V IRQ)
    0x58,             // CLI
    0x80, 0xFE,       // BRA -2 (loop at offset 0x13 = pc 0x8013)
];

/// PROG_HV: mode 3 (H and V), HTIME=50, VTIME=100. IRQ once per frame.
const PROG_HV: &[u8] = &[
    0x78, 0x18, 0xFB, // SEI ; CLC ; XCE
    0x9C, 0x03, 0x01, // STZ $0103 (counter)
    0xA9, 0x32,       // LDA #$32 (=50)
    0x8D, 0x07, 0x42, // STA $4207 (HTIME)
    0x9C, 0x08, 0x42, // STZ $4208 (VTIME=0 -> mode 1: H only... wait no)
    0xA9, 0x64,       // LDA #$64 (=100)
    0x8D, 0x09, 0x42, // STA $4209 (VTIME)
    0x9C, 0x0A, 0x42, // STZ $420A (HTIME=0 -> mode 2: V only... wait no)
    0xA9, 0x30,       // LDA #$30 (=48 = bits 4+5 = enable H and V IRQ)
    0x8D, 0x00, 0x42, // STA $4200 (NMITIMEN: enable both)
    0x58,             // CLI
    0x80, 0xFE,       // BRA -2 (loop at offset 0x1B = pc 0x801B)
];

/// PROG_V_MASKED: same as PROG_V but with NOP instead of CLI. IRQ stays masked.
const PROG_V_MASKED: &[u8] = &[
    0x78, 0x18, 0xFB, // SEI ; CLC ; XCE
    0x9C, 0x03, 0x01, // STZ $0103 (counter)
    0xA9, 0x64,       // LDA #$64 (=100)
    0x8D, 0x09, 0x42, // STA $4209 (VTIME)
    0x9C, 0x0A, 0x42, // STZ $420A (HTIME=0 -> mode 2: V only)
    0xA9, 0x20,       // LDA #$20 (=32 = bit 5 = enable V IRQ)
    0x8D, 0x00, 0x42, // STA $4200 (NMITIMEN: enable V IRQ)
    0xEA,             // NOP (instead of CLI - interrupts stay masked)
    0x80, 0xFE,       // BRA -2 (loop at offset 0x13 = pc 0x8013)
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
fn e03_38_irq_h_every_line() {
    let rom = build_rom(
        &[
            (0, PROG_H),
            (0x100, HANDLER_IRQ),
        ],
        &[("irq", 0x8100)],
    );
    let mut s = snes(rom);

    // One frame: IRQ should fire on every line from 0 to 224 (visible lines)
    s.run_frame();
    settle(&mut s);
    let count = s.peek_wram(0x103);
    assert!(count >= 224 && count <= 225, "expected ~224 IRQs per frame, got {}", count);
}

#[test]
fn e03_38_irq_v_once_per_frame() {
    let rom = build_rom(
        &[
            (0, PROG_V),
            (0x100, HANDLER_IRQ),
        ],
        &[("irq", 0x8100)],
    );
    let mut s = snes(rom);

    for k in 1..=3 {
        s.run_frame();
        settle(&mut s);
        assert_eq!(s.peek_wram(0x103), k, "frame {}", k);
    }
}

#[test]
fn e03_38_irq_hv_once_per_frame() {
    let rom = build_rom(
        &[
            (0, PROG_HV),
            (0x100, HANDLER_IRQ),
        ],
        &[("irq", 0x8100)],
    );
    let mut s = snes(rom);

    for k in 1..=3 {
        s.run_frame();
        settle(&mut s);
        assert_eq!(s.peek_wram(0x103), k, "frame {}", k);
    }
}

#[test]
fn e03_38_irq_masked_by_i() {
    let rom = build_rom(
        &[
            (0, PROG_V_MASKED),
            (0x100, HANDLER_IRQ),
        ],
        &[("irq", 0x8100)],
    );
    let mut s = snes(rom);

    // Run 2 frames; IRQ level is asserted but I=1 so CPU never takes it
    for _ in 0..2 {
        s.run_frame();
        settle(&mut s);
    }
    assert_eq!(s.peek_wram(0x103), 0, "IRQ should be masked by I flag");
}

#[test]
fn e03_38_irq_not_before_line_100() {
    let rom = build_rom(
        &[
            (0, PROG_V),
            (0x100, HANDLER_IRQ),
        ],
        &[("irq", 0x8100)],
    );
    let mut s = snes(rom);

    // Run for 99 lines (before VTIME=100)
    s.run_master(1364 * 99);
    assert_eq!(s.peek_wram(0x103), 0, "IRQ should not fire before line 100");

    // Now run a few more lines past line 100
    s.run_master(1364 * 3);
    settle(&mut s);
    assert_eq!(s.peek_wram(0x103), 1, "IRQ should fire after line 100");
}

#[test]
fn e03_38_irq_position() {
    let rom = build_rom(
        &[
            (0, PROG_V),
            (0x100, HANDLER_IRQ),
        ],
        &[("irq", 0x8100)],
    );
    let mut s = snes(rom);

    // Run to line 100 + some extra clocks
    s.run_master(1364 * 100 + 400);
    settle(&mut s);

    assert_eq!(s.peek_wram(0x103), 1, "IRQ should have fired at line 100");
    let dbg = s.debug();
    assert_eq!(dbg.v, 100, "should be at or past line 100");
}

#[test]
fn e03_38_irq_ack_prevents_retrigger() {
    let rom = build_rom(
        &[
            (0, PROG_V),
            (0x100, HANDLER_IRQ),
        ],
        &[("irq", 0x8100)],
    );
    let mut s = snes(rom);

    // One frame: exactly one IRQ (acknowledged by handler)
    s.run_frame();
    settle(&mut s);
    assert_eq!(s.peek_wram(0x103), 1, "exactly one IRQ per frame when acknowledged");
}

#[test]
fn e03_38_irq_without_ack_loops() {
    // Handler without acknowledgment: just INC and RTI
    let handler_no_ack: &[u8] = &[
        0xEE, 0x03, 0x01, // INC $0103
        0x40,             // RTI
    ];

    let rom = build_rom(
        &[
            (0, PROG_V),
            (0x100, handler_no_ack),
        ],
        &[("irq", 0x8100)],
    );
    let mut s = snes(rom);

    // Without ack, IRQ level stays asserted -> CPU re-enters handler immediately after RTI
    s.run_master(1364 * 103);
    settle(&mut s);

    let count = s.peek_wram(0x103);
    assert!(count > 10, "IRQ should loop without acknowledgment (got {})", count);
}