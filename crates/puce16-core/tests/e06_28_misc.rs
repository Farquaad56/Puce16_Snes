use puce16_core::apu::spc700::{Spc700, SmpBus, testbus::FlatSmpBus, F_C, F_Z, F_I, F_H, F_B, F_P, F_V, F_N};

fn run(setup: impl FnOnce(&mut Spc700, &mut FlatSmpBus), prog: &[u8]) -> (Spc700, FlatSmpBus, u32) {
    let mut cpu = Spc700::new();
    let mut bus = FlatSmpBus::new();
    cpu.pc = 0x0200;
    bus.poke(0x0200, prog);
    setup(&mut cpu, &mut bus);
    let cyc = cpu.step(&mut bus);
    (cpu, bus, cyc)
}

#[test]
fn e06_28_push_a() {
    // PUSH A: push accumulator onto stack
    let (cpu, bus, cyc) = run(|c, _| { c.sp = 0xEF; c.a = 0x5A; }, &[0x2D]);
    assert_eq!(bus.peek(0x01EF), 0x5A, "PUSH A should write value to stack");
    assert_eq!(cpu.sp, 0xEE, "PUSH A should decrement SP");
    assert_eq!(cyc, 4, "PUSH A: 4 cycles");
}

#[test]
fn e06_28_push_x_y_psw() {
    // PUSH X, Y, PSW
    let (cpu1, bus1, _) = run(|c, _| { c.sp = 0xEF; c.x = 0x11; }, &[0x4D]);
    assert_eq!(bus1.peek(0x01EF), 0x11, "PUSH X should write value to stack");

    let (cpu2, bus2, _) = run(|c, _| { c.sp = 0xEF; c.y = 0x22; }, &[0x6D]);
    assert_eq!(bus2.peek(0x01EF), 0x22, "PUSH Y should write value to stack");

    let (cpu3, bus3, _) = run(|c, _| { c.sp = 0xEF; c.psw = 0x86; }, &[0x0D]);
    assert_eq!(bus3.peek(0x01EF), 0x86, "PUSH PSW should write value to stack");
    assert_eq!(cpu3.sp, 0xEE, "PUSH PSW should decrement SP");
}

#[test]
fn e06_28_pop_a_no_flags() {
    // POP A: pop from stack into accumulator (no flag effects)
    let (cpu, _, cyc) = run(|c, b| { c.sp = 0xEE; c.psw = 0x00; b.poke(0x01EF, &[0x9C]); }, &[0xAE]);
    assert_eq!(cpu.a, 0x9C, "POP A should load value from stack");
    assert_eq!(cpu.sp, 0xEF, "POP A should increment SP");
    // PSW unchanged (no N/Z effects)
    assert!(!cpu.flag(F_N), "POP A should not set N flag");
    assert!(!cpu.flag(F_Z), "POP A should not set Z flag");
    assert_eq!(cyc, 4, "POP A: 4 cycles");
}

#[test]
fn e06_28_pop_x_y() {
    // POP X and POP Y
    let (cpu1, _, _) = run(|c, b| { c.sp = 0xEE; b.poke(0x01EF, &[0x33]); }, &[0xCE]);
    assert_eq!(cpu1.x, 0x33, "POP X should load value from stack");

    let (cpu2, _, _) = run(|c, b| { c.sp = 0xEE; b.poke(0x01EF, &[0x44]); }, &[0xEE]);
    assert_eq!(cpu2.y, 0x44, "POP Y should load value from stack");
}

#[test]
fn e06_28_pop_psw() {
    // POP PSW: pop from stack into PSW (restores all flags)
    let (cpu, _, _) = run(|c, b| { c.sp = 0xEE; b.poke(0x01EF, &[0xA5]); }, &[0x8E]);
    assert_eq!(cpu.psw, 0xA5, "POP PSW should restore all flags");
}

#[test]
fn e06_28_carry_ops() {
    // SETC, CLRC, NOTC
    let (cpu1, _, cyc1) = run(|c, _| { c.psw = 0; }, &[0x80]);
    assert_eq!(cpu1.psw & F_C, F_C, "SETC should set carry flag");
    assert_eq!(cyc1, 2, "SETC: 2 cycles");

    let (cpu2, _, cyc2) = run(|c, _| { c.psw = F_C; }, &[0x60]);
    assert_eq!(cpu2.psw & F_C, 0, "CLRC should clear carry flag");
    assert_eq!(cyc2, 2, "CLRC: 2 cycles");

    let (cpu3, _, cyc3) = run(|c, _| { c.psw = 0; }, &[0xED]);
    assert_eq!(cpu3.psw & F_C, F_C, "NOTC should toggle carry from 0 to 1");
    assert_eq!(cyc3, 3, "NOTC: 3 cycles");

    let (cpu4, _, _) = run(|c, _| { c.psw = F_C; }, &[0xED]);
    assert_eq!(cpu4.psw & F_C, 0, "NOTC should toggle carry from 1 to 0");
}

#[test]
fn e06_28_i_flag() {
    // EI and DI
    let (cpu1, _, cyc1) = run(|c, _| { c.psw = 0; }, &[0xA0]);
    assert_eq!(cpu1.psw & F_I, F_I, "EI should set interrupt flag");
    assert_eq!(cyc1, 3, "EI: 3 cycles");

    let (cpu2, _, cyc2) = run(|c, _| { c.psw = F_I; }, &[0xC0]);
    assert_eq!(cpu2.psw & F_I, 0, "DI should clear interrupt flag");
    assert_eq!(cyc2, 3, "DI: 3 cycles");
}

#[test]
fn e06_28_clrv_clears_h() {
    // CLRV clears both V and H flags
    let (cpu1, _, cyc) = run(|c, _| { c.psw = F_V | F_H; }, &[0xE0]);
    assert_eq!(cpu1.psw & (F_V | F_H), 0, "CLRV should clear both V and H");
    assert_eq!(cyc, 2, "CLRV: 2 cycles");

    // Other flags preserved
    let (cpu2, _, _) = run(|c, _| { c.psw = F_N | F_V | F_H | F_C; }, &[0xE0]);
    assert_eq!(cpu2.psw & (F_V | F_H), 0, "CLRV should clear V and H");
    assert_eq!(cpu2.psw & (F_N | F_C), F_N | F_C, "CLRV should preserve other flags");
}

#[test]
fn e06_28_p_flag() {
    // SETP and CLRP
    let (cpu1, _, cyc) = run(|c, _| { c.psw = 0; }, &[0x40]);
    assert_eq!(cpu1.psw & F_P, F_P, "SETP should set P flag");
    assert_eq!(cyc, 2, "SETP: 2 cycles");

    let (cpu2, _, _) = run(|c, _| { c.psw = F_P; }, &[0x20]);
    assert_eq!(cpu2.psw & F_P, 0, "CLRP should clear P flag");
}

#[test]
fn e06_28_nop() {
    // NOP: no operation
    let (cpu, _, cyc) = run(|c, _| { c.psw = 0x5A; }, &[0x00]);
    assert_eq!(cpu.psw, 0x5A, "NOP should not change PSW");
    assert_eq!(cpu.pc, 0x0201, "NOP should advance PC by 1");
    assert_eq!(cyc, 2, "NOP: 2 cycles");
}

#[test]
fn e06_28_sleep_halts() {
    // SLEEP and STOP halt the CPU
    let (cpu1, _, cyc) = run(|_, _| {}, &[0xEF]);
    assert!(cpu1.halted, "SLEEP should set halted flag");
    assert_eq!(cyc, 7, "SLEEP: 7 cycles (oracle)");
    assert_eq!(cpu1.pc, 0x0201, "SLEEP should advance PC by 1");

    // Subsequent step returns 1 cycle and doesn't change PC
    let mut cpu = Spc700::new();
    let mut bus = FlatSmpBus::new();
    cpu.pc = 0x0200;
    cpu.halted = true;
    let cyc2 = cpu.step(&mut bus);
    assert_eq!(cyc2, 1, "Halted CPU step should return 1 cycle");
    assert_eq!(cpu.pc, 0x0200, "Halted CPU step should not change PC");

    // STOP also halts
    let (cpu3, _, cyc3) = run(|_, _| {}, &[0xFF]);
    assert!(cpu3.halted, "STOP should set halted flag");
    assert_eq!(cyc3, 7, "STOP: 7 cycles (oracle)");
}

#[test]
fn e06_28_all_opcodes_implemented() {
    // All 256 opcodes must be implemented (no panic on any opcode)
    for op in 0..=255u8 {
        let mut cpu = Spc700::new();
        let mut bus = FlatSmpBus::new();
        cpu.pc = 0x0200;
        // Write the opcode and some operand bytes (all zeros)
        bus.poke(0x0200, &[op, 0x00, 0x00]);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            cpu.step(&mut bus);
        }));

        assert!(result.is_ok(), "Opcode {:02X} should not panic", op);
    }
}
