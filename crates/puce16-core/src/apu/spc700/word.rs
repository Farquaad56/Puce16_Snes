use super::{F_C, F_N, F_Z, SmpBus, Spc700};

/// ADDW YA,dp: 16-bit add with carry chain (CLRC implicit)
fn addw<B: SmpBus>(cpu: &mut Spc700, bus: &mut B, b: u8) {
    let a0 = cpu.dp() | b as u16;
    let a1 = cpu.dp() | ((b.wrapping_add(1)) as u16);

    let lo = cpu.rd(bus, a0);
    let hi = cpu.rd(bus, a1);

    // CLRC implicit - ignore incoming carry
    cpu.set_flag(F_C, false);

    // Low byte: A + lo (no carry in)
    cpu.a = cpu.adc_op(cpu.a, lo);

    // High byte: Y + hi + carry from low
    cpu.y = cpu.adc_op(cpu.y, hi);

    // Z flag based on full 16-bit result
    cpu.set_flag(F_N, (cpu.y & 0x80) != 0);
    cpu.set_flag(F_Z, cpu.a == 0 && cpu.y == 0);
}

/// SUBW YA,dp: 16-bit subtract with borrow chain (SETC implicit)
fn subw<B: SmpBus>(cpu: &mut Spc700, bus: &mut B, b: u8) {
    let a0 = cpu.dp() | b as u16;
    let a1 = cpu.dp() | ((b.wrapping_add(1)) as u16);

    let lo = cpu.rd(bus, a0);
    let hi = cpu.rd(bus, a1);

    // SETC implicit - no borrow in
    cpu.set_flag(F_C, true);

    // Low byte: A - lo (no borrow in)
    cpu.a = cpu.sbc_op(cpu.a, lo);

    // High byte: Y - hi - borrow from low
    cpu.y = cpu.sbc_op(cpu.y, hi);

    // Z flag based on full 16-bit result
    cpu.set_flag(F_N, (cpu.y & 0x80) != 0);
    cpu.set_flag(F_Z, cpu.a == 0 && cpu.y == 0);
}

/// CMPW YA,dp: compare 16-bit values, set flags only
fn cmpw<B: SmpBus>(cpu: &mut Spc700, bus: &mut B, b: u8) {
    let a0 = cpu.dp() | b as u16;
    let a1 = cpu.dp() | ((b.wrapping_add(1)) as u16);

    let lo = cpu.rd(bus, a0);
    let hi = cpu.rd(bus, a1);

    let word = ((hi as u16) << 8) | (lo as u16);
    let ya = cpu.ya();

    // Compute difference for flags (don't store result)
    let diff = ya.wrapping_sub(word);

    cpu.set_flag(F_C, ya >= word);
    cpu.set_flag(F_N, (diff & 0x8000) != 0);
    cpu.set_flag(F_Z, diff == 0);
    // V and H unchanged per spec
}

/// INCW dp: increment 16-bit value in memory
fn incw<B: SmpBus>(cpu: &mut Spc700, bus: &mut B, b: u8) {
    let a0 = cpu.dp() | b as u16;
    let a1 = cpu.dp() | ((b.wrapping_add(1)) as u16);

    // Read low byte
    let lo = cpu.rd(bus, a0);
    let new_lo = lo.wrapping_add(1);
    cpu.wr(bus, a0, new_lo);

    // Read high byte, increment if low wrapped
    let hi = cpu.rd(bus, a1);
    let new_hi = if new_lo == 0 {
        hi.wrapping_add(1)
    } else {
        hi
    };
    cpu.wr(bus, a1, new_hi);

    // Set flags based on result
    cpu.set_flag(F_N, (new_hi & 0x80) != 0);
    cpu.set_flag(F_Z, new_lo == 0 && new_hi == 0);
}

/// DECW dp: decrement 16-bit value in memory
fn decw<B: SmpBus>(cpu: &mut Spc700, bus: &mut B, b: u8) {
    let a0 = cpu.dp() | b as u16;
    let a1 = cpu.dp() | ((b.wrapping_add(1)) as u16);

    // Read low byte
    let lo = cpu.rd(bus, a0);
    let new_lo = lo.wrapping_sub(1);
    cpu.wr(bus, a0, new_lo);

    // Read high byte, decrement if low wrapped (borrowed)
    let hi = cpu.rd(bus, a1);
    let new_hi = if lo == 0 {
        hi.wrapping_sub(1)
    } else {
        hi
    };
    cpu.wr(bus, a1, new_hi);

    // Set flags based on result
    cpu.set_flag(F_N, (new_hi & 0x80) != 0);
    cpu.set_flag(F_Z, new_lo == 0 && new_hi == 0);
}

pub(super) fn exec<B: SmpBus>(cpu: &mut Spc700, bus: &mut B, op: u8) -> bool {
    match op {
        // ADDW YA,dp (opcode 7A)
        0x7A => {
            let b = cpu.fetch8(bus);
            addw(cpu, bus, b);
        }
        // SUBW YA,dp (opcode 9A)
        0x9A => {
            let b = cpu.fetch8(bus);
            subw(cpu, bus, b);
        }
        // CMPW YA,dp (opcode 5A)
        0x5A => {
            let b = cpu.fetch8(bus);
            cmpw(cpu, bus, b);
        }
        // INCW dp (opcode 3A)
        0x3A => {
            let b = cpu.fetch8(bus);
            incw(cpu, bus, b);
        }
        // DECW dp (opcode 1A)
        0x1A => {
            let b = cpu.fetch8(bus);
            decw(cpu, bus, b);
        }
        _ => return false,
    }
    true
}
