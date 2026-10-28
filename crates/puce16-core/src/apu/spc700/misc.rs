use super::{F_C, F_H, F_I, F_P, F_V, SmpBus, Spc700};

fn daa(cpu: &mut Spc700) {
    let mut a = cpu.a;
    if cpu.flag(F_C) || a > 0x99 {
        a = a.wrapping_add(0x60);
        cpu.set_flag(F_C, true);
    }
    if cpu.flag(F_H) || (a & 0xF) > 9 {
        a = a.wrapping_add(6);
    }
    cpu.a = a;
    cpu.set_nz(a);
}

fn das(cpu: &mut Spc700) {
    let mut a = cpu.a;
    if !cpu.flag(F_C) || a > 0x99 {
        a = a.wrapping_sub(0x60);
        cpu.set_flag(F_C, false);
    }
    if !cpu.flag(F_H) || (a & 0xF) > 9 {
        a = a.wrapping_sub(6);
    }
    cpu.a = a;
    cpu.set_nz(a);
}

fn mul(cpu: &mut Spc700) {
    let r = (cpu.y as u16) * (cpu.a as u16);
    cpu.a = r as u8;
    cpu.y = (r >> 8) as u8;
    cpu.set_nz(cpu.y);
}

fn div_op(cpu: &mut Spc700) {
    let ya = ((cpu.y as u32) << 8) | (cpu.a as u32);
    let x = cpu.x as u32;

    // Set V and H flags based on comparison of Y with X
    cpu.set_flag(F_V, cpu.y >= cpu.x);
    cpu.set_flag(F_H, (cpu.y & 0xF) >= (cpu.x & 0xF));

    if x == 0 {
        // Division by zero: use second branch formula
        let a_val = 255u32.wrapping_sub(ya / 256);
        let y_val = ya % 256;
        cpu.a = a_val as u8;
        cpu.y = y_val as u8;
    } else if (cpu.y as u32) < (x << 1) {
        // Normal division branch
        let q = ya / x;
        let r = ya % x;
        cpu.a = q as u8;
        cpu.y = r as u8;
    } else {
        // Overflow branch: use complement arithmetic
        let diff = ya.wrapping_sub(x << 9);
        let denom = 256u32 - x;
        let a_val = 255u32.wrapping_sub(diff / denom);
        let y_val = x + (diff % denom);
        cpu.a = a_val as u8;
        cpu.y = y_val as u8;
    }

    cpu.set_nz(cpu.a);
}

fn push_a<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    cpu.push8(bus, cpu.a);
}

fn push_x<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    cpu.push8(bus, cpu.x);
}

fn push_y<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    cpu.push8(bus, cpu.y);
}

fn push_psw<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    cpu.push8(bus, cpu.psw);
}

fn pop_a<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    cpu.a = cpu.pull8(bus);
}

fn pop_x<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    cpu.x = cpu.pull8(bus);
}

fn pop_y<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    cpu.y = cpu.pull8(bus);
}

fn pop_psw<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    cpu.psw = cpu.pull8(bus);
}

fn clrc(cpu: &mut Spc700) {
    cpu.set_flag(F_C, false);
}

fn setc(cpu: &mut Spc700) {
    cpu.set_flag(F_C, true);
}

fn notc(cpu: &mut Spc700) {
    let c = cpu.flag(F_C);
    cpu.set_flag(F_C, !c);
}

fn ei(cpu: &mut Spc700) {
    cpu.set_flag(F_I, true);
}

fn di(cpu: &mut Spc700) {
    cpu.set_flag(F_I, false);
}

fn clrv(cpu: &mut Spc700) {
    cpu.set_flag(F_V, false);
    cpu.set_flag(F_H, false);
}

fn clrp(cpu: &mut Spc700) {
    cpu.set_flag(F_P, false);
}

fn setp(cpu: &mut Spc700) {
    cpu.set_flag(F_P, true);
}

fn nop(_cpu: &mut Spc700) {}

fn sleep(cpu: &mut Spc700) {
    cpu.halted = true;
}

fn stop(cpu: &mut Spc700) {
    cpu.halted = true;
}

pub(super) fn exec<B: SmpBus>(cpu: &mut Spc700, bus: &mut B, op: u8) -> bool {
    match op {
        0xDF => daa(cpu),
        0xBE => das(cpu),
        0xCF => mul(cpu),
        0x9E => div_op(cpu),
        // PUSH instructions (4 cycles each)
        0x2D => push_a(cpu, bus),
        0x4D => push_x(cpu, bus),
        0x6D => push_y(cpu, bus),
        0x0D => push_psw(cpu, bus),
        // POP instructions (4 cycles each)
        0xAE => pop_a(cpu, bus),
        0xCE => pop_x(cpu, bus),
        0xEE => pop_y(cpu, bus),
        0x8E => pop_psw(cpu, bus),
        // Flag operations
        0x60 => clrc(cpu),   // CLRC - clear carry (2 cycles)
        0x80 => setc(cpu),   // SETC - set carry (2 cycles)
        0xED => notc(cpu),   // NOTC - toggle carry (3 cycles)
        0xA0 => ei(cpu),     // EI - enable interrupts (3 cycles)
        0xC0 => di(cpu),     // DI - disable interrupts (3 cycles)
        0xE0 => clrv(cpu),   // CLRV - clear V and H (2 cycles)
        0x20 => clrp(cpu),   // CLRP - clear P flag (2 cycles)
        0x40 => setp(cpu),   // SETP - set P flag (2 cycles)
        // NOP, SLEEP, STOP
        0x00 => nop(cpu),    // NOP (2 cycles)
        0xEF => sleep(cpu),  // SLEEP (3 cycles)
        0xFF => stop(cpu),   // STOP (2 cycles)
        _ => return false,
    }
    true
}
