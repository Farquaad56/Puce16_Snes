use crate::cpu::addr;
use crate::cpu::table::{Am, Mn};
use crate::cpu::{Bus, Cpu, FLAG_C, FLAG_D, FLAG_V};

pub(super) fn exec<B: Bus>(cpu: &mut Cpu, bus: &mut B, mn: Mn, am: Am) -> bool {
    if mn != Mn::SBC {
        return false;
    }
    let wide = !cpu.m8();
    let m = addr::read_operand(cpu, bus, am, wide);
    if cpu.flag(FLAG_D) {
        sbc_decimal(cpu, m, wide);
    } else {
        sbc_binary(cpu, m, wide);
    }
    true
}

fn sbc_binary(cpu: &mut Cpu, m: u16, wide: bool) {
    let a = cpu.a;
    let mask: u32 = if wide { 0xFFFF } else { 0xFF };
    let signbit: u32 = if wide { 0x8000 } else { 0x80 };

    // In 8-bit mode, only use low byte of A for the operation
    let a_op: u32 = if wide { a as u32 } else { (a & 0xFF) as u32 };
    let m_op: u32 = if wide { m as u32 } else { (m & 0xFF) as u32 };

    // SBC computes A + ~M + C (two's complement subtraction)
    let inv = (!m_op) & mask;
    let carry_in: u32 = if cpu.flag(FLAG_C) { 1 } else { 0 };
    let r = a_op + inv + carry_in;

    // Carry out (borrow cleared means no borrow needed)
    cpu.set_flag(FLAG_C, r > mask);

    // Overflow: sign of A and ~M differ from result's sign
    let v = ((a_op ^ r) & (inv ^ r) & signbit) != 0;
    cpu.set_flag(FLAG_V, v);

    // Truncate and store
    let result = (r & mask) as u16;
    if wide {
        cpu.a = result;
    } else {
        // Preserve high byte in 8-bit mode
        cpu.a = ((cpu.a & 0xFF00) | (result & 0xFF)) as u16;
    }

    cpu.set_nz(result, wide);
}

/// Internal helper for BCD subtraction using digit-by-digit algorithm.
/// Operates on u32 values to avoid overflow during intermediate calculations.
fn sbc_decimal_n(a: u32, m: u32, carry_in: bool, wide: bool) -> (u32, bool, bool) {
    // BCD subtraction: digit-by-digit with borrow propagation
    // C=1 means no previous borrow; C=0 means borrow needed
    
    let nibbles = if wide { 4 } else { 2 };
    let mut result = 0u32;
    let mut borrow = !carry_in; // invert: true = need to borrow
    
    for i in 0..nibbles {
        let a_digit = ((a >> (i * 4)) & 0x0F) as i8;
        let m_digit = ((m >> (i * 4)) & 0x0F) as i8;
        
        // Compute digit difference with borrow using signed arithmetic
        let mut diff = a_digit - m_digit - if borrow { 1 } else { 0 };
        borrow = false;
        if diff < 0 {
            diff += 10;
            borrow = true;
        }
        
        result |= (diff as u32) << (i * 4);
    }
    
    // Carry flag: C=1 means no final borrow (subtraction succeeded without underflow)
    let carry_out = !borrow;
    
    // Overflow for BCD: occurs when sign of result differs from expected
    // For subtraction, V is set if A and M have different signs but result has same sign as M
    // Simplified: check bit 7 (or 15) of the raw binary difference vs BCD result
    let raw_diff = a.wrapping_sub(m).wrapping_sub(if carry_in { 0 } else { 1 });
    let v_bit = if wide { 0x8000 } else { 0x80 };
    let overflow = ((raw_diff ^ result) & (raw_diff ^ a) & v_bit) != 0;
    
    (result, carry_out, overflow)
}

fn sbc_decimal(cpu: &mut Cpu, m: u16, wide: bool) {
    if !wide {
        // 8-bit BCD subtraction (2 digits)
        let a_low = cpu.a as u32 & 0xFF;
        let m_low = m as u32 & 0xFF;
        let (result, carry_out, overflow) = sbc_decimal_n(a_low, m_low, cpu.flag(FLAG_C), false);

        // Preserve high byte in 8-bit mode
        let result_u16 = ((cpu.a & 0xFF00) | (result as u16)) as u16;
        cpu.a = result_u16;
        cpu.set_flag(FLAG_C, carry_out);
        cpu.set_flag(FLAG_V, overflow);
        cpu.set_nz(result as u16, false);
    } else {
        // 16-bit BCD subtraction (4 digits)
        let a_val = cpu.a as u32;
        let m_val = m as u32;
        let (result, carry_out, overflow) = sbc_decimal_n(a_val, m_val, cpu.flag(FLAG_C), true);

        cpu.a = result as u16;
        cpu.set_flag(FLAG_C, carry_out);
        cpu.set_flag(FLAG_V, overflow);
        cpu.set_nz(result as u16, true);
    }
}
