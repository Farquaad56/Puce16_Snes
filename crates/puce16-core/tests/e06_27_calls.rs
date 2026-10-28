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
fn e06_27_jmp_abs() {
    // JMP !abs: jump to absolute address
    let (cpu, _, cyc) = run(|_, _| {}, &[0x5F, 0x00, 0x30]);
    assert_eq!(cpu.pc, 0x3000, "JMP !abs should set PC to target");
    assert_eq!(cyc, 3, "JMP !abs: 3 cycles");
}

#[test]
fn e06_27_jmp_abs_x_indirect() {
    // JMP [!abs+X]: indirect jump through pointer at abs+X (wraps modulo 65536)
    let (cpu1, _, cyc1) = run(|c, b| {
        c.x = 2;
        b.poke(0x3002, &[0x34, 0x12]);
    }, &[0x1F, 0x00, 0x30]);
    assert_eq!(cpu1.pc, 0x1234, "JMP [!abs+X] should follow pointer");
    assert_eq!(cyc1, 6, "JMP [!abs+X]: 6 cycles");

    // Wrap-around: abs=0xFFFF, X=1 -> ea=0x0000 (wraps)
    let (cpu2, _, _) = run(|c, b| {
        c.x = 1;
        b.poke(0x0000, &[0x78, 0x56]);
    }, &[0x1F, 0xFF, 0xFF]);
    assert_eq!(cpu2.pc, 0x5678, "JMP [!abs+X] should wrap pointer address");
}

#[test]
fn e06_27_call() {
    // CALL !abs: push return address (PC after instruction) and jump
    let (cpu, bus, cyc) = run(|c, _| { c.sp = 0xEF; }, &[0x3F, 0x00, 0x30]);
    assert_eq!(cpu.pc, 0x3000, "CALL !abs should jump to target");
    assert_eq!(cpu.sp, 0xED, "CALL should decrement SP by 2");
    // Return address = PC after CALL instruction (0x0203)
    assert_eq!(bus.peek(0x01EF), 0x02, "High byte of return address on stack");
    assert_eq!(bus.peek(0x01EE), 0x03, "Low byte of return address on stack");
    assert_eq!(cyc, 8, "CALL !abs: 8 cycles");
}

#[test]
fn e06_27_pcall() {
    // PCALL up: push return address and jump to $FF00|up
    let (cpu, bus, cyc) = run(|c, _| { c.sp = 0xEF; }, &[0x4F, 0x20]);
    assert_eq!(cpu.pc, 0xFF20, "PCALL should jump to $FF00|up");
    assert_eq!(cpu.sp, 0xED, "PCALL should decrement SP by 2");
    // Return address = PC after PCALL instruction (0x0202)
    assert_eq!(bus.peek(0x01EF), 0x02, "High byte of return address on stack");
    assert_eq!(bus.peek(0x01EE), 0x02, "Low byte of return address on stack");
    assert_eq!(cyc, 6, "PCALL: 6 cycles");
}

#[test]
fn e06_27_tcall_1() {
    // TCALL 1 (opcode 0x11): push PC and jump to vector at $FFDC
    let (cpu, bus, cyc) = run(|c, b| {
        c.sp = 0xEF;
        b.poke(0xFFDC, &[0x78, 0x56]);
    }, &[0x11]);
    assert_eq!(cpu.pc, 0x5678, "TCALL 1 should jump to vector at $FFDC");
    // Return address = PC after TCALL instruction (0x0201)
    assert_eq!(bus.peek(0x01EF), 0x02, "High byte of return address on stack");
    assert_eq!(bus.peek(0x01EE), 0x01, "Low byte of return address on stack");
    assert_eq!(cpu.sp, 0xED, "TCALL should decrement SP by 2");
    assert_eq!(cyc, 8, "TCALL: 8 cycles");
}

#[test]
fn e06_27_tcall_0_and_15() {
    // TCALL 0 (opcode 0x01): vector at $FFDE
    let (cpu1, _, _) = run(|c, b| {
        c.sp = 0xEF;
        b.poke(0xFFDE, &[0x34, 0x12]);
    }, &[0x01]);
    assert_eq!(cpu1.pc, 0x1234, "TCALL 0 should jump to vector at $FFDE");

    // TCALL 15 (opcode 0xF1): vector at $FFC0
    let (cpu2, _, _) = run(|c, b| {
        c.sp = 0xEF;
        b.poke(0xFFC0, &[0x21, 0x43]);
    }, &[0xF1]);
    assert_eq!(cpu2.pc, 0x4321, "TCALL 15 should jump to vector at $FFC0");
}

#[test]
fn e06_27_brk() {
    // BRK: push PC and PSW, set B=1 I=0, jump to vector at $FFDE
    let (cpu, bus, cyc) = run(|c, b| {
        c.sp = 0xEF;
        c.psw = 0x86;
        b.poke(0xFFDE, &[0x00, 0x40]);
    }, &[0x0F]);
    assert_eq!(cpu.pc, 0x4000, "BRK should jump to vector at $FFDE");
    // Stack: PC high (0x02), PC low (0x01), PSW (0x86)
    assert_eq!(bus.peek(0x01EF), 0x02, "PC high byte on stack");
    assert_eq!(bus.peek(0x01EE), 0x01, "PC low byte on stack");
    assert_eq!(bus.peek(0x01ED), 0x86, "PSW on stack");
    assert_eq!(cpu.sp, 0xEC, "BRK should decrement SP by 3");
    // PSW: B=1 (bit 4), I=0 (bit 2 cleared) -> 0x86 | 0x10 & ~0x04 = 0x92
    assert_eq!(cpu.psw, 0x92, "BRK should set B=1 and clear I");
    assert_eq!(cyc, 8, "BRK: 8 cycles");
}

#[test]
fn e06_27_ret() {
    // RET: pop return address and jump
    let (cpu, _, cyc) = run(|c, b| {
        c.sp = 0xED;
        b.poke(0x01EE, &[0x34, 0x12]);
    }, &[0x6F]);
    assert_eq!(cpu.pc, 0x1234, "RET should jump to popped address");
    assert_eq!(cpu.sp, 0xEF, "RET should increment SP by 2");
    assert_eq!(cyc, 5, "RET: 5 cycles");
}

#[test]
fn e06_27_reti() {
    // RETI: pop PSW then return address and jump
    let (cpu, _, cyc) = run(|c, b| {
        c.sp = 0xEC;
        b.poke(0x01ED, &[0xA5, 0x78, 0x56]);
    }, &[0x7F]);
    assert_eq!(cpu.psw, 0xA5, "RETI should restore PSW");
    assert_eq!(cpu.pc, 0x5678, "RETI should jump to popped address");
    assert_eq!(cpu.sp, 0xEF, "RETI should increment SP by 3");
    assert_eq!(cyc, 6, "RETI: 6 cycles");
}

#[test]
fn e06_27_call_ret_roundtrip() {
    // CALL !abs then RET should return to original PC
    let mut cpu = Spc700::new();
    let mut bus = FlatSmpBus::new();
    cpu.pc = 0x0200;
    cpu.sp = 0xEF;
    bus.poke(0x0200, &[0x3F, 0x00, 0x30]); // CALL $3000
    bus.poke(0x3000, &[0x6F]);            // RET

    let cyc1 = cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x3000, "CALL should jump to target");
    assert_eq!(cyc1, 8, "CALL: 8 cycles");

    let cyc2 = cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x0203, "RET should return to PC after CALL");
    assert_eq!(cpu.sp, 0xEF, "SP should be restored");
    assert_eq!(cyc2, 5, "RET: 5 cycles");
}
