use crate::cpu::addr;
use crate::cpu::table::{Am, Mn};
use crate::cpu::{Bus, Cpu, FLAG_C, FLAG_D, FLAG_V};

pub(super) fn exec<B: Bus>(cpu: &mut Cpu, bus: &mut B, mn: Mn, am: Am) -> bool {
    if mn != Mn::ADC {
        return false;
    }
    let wide = !cpu.m8();
    let m = addr::read_operand(cpu, bus, am, wide);
    if cpu.flag(FLAG_D) {
        adc_decimal(cpu, m, wide);
    } else {
        adc_binary(cpu, m, wide);
    }
    true
}

fn adc_binary(cpu: &mut Cpu, m: u16, wide: bool) {
    let a = cpu.a;
    let mask: u32 = if wide { 0xFFFF } else { 0xFF };
    let signbit: u32 = if wide { 0x8000 } else { 0x80 };

    // In 8-bit mode, only use low byte of A for the operation
    let a_op: u32 = if wide { a as u32 } else { (a & 0xFF) as u32 };
    let m_op: u32 = if wide { m as u32 } else { (m & 0xFF) as u32 };

    let carry_in: u32 = if cpu.flag(FLAG_C) { 1 } else { 0 };
    let r = a_op + m_op + carry_in;

    // Carry out
    cpu.set_flag(FLAG_C, r > mask);

    // Overflow: sign of operands same but different from result
    let v = ((a_op ^ r) & (m_op ^ r) & signbit) != 0;
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

/// Internal helper for BCD addition following the reference algorithm from the spec.
/// Operates on u32 values to avoid overflow during intermediate calculations.
fn adc_decimal_n(a: u32, m: u32, carry_in: bool, nibbles: usize) -> (u32, bool, bool) {
    if nibbles == 1 {
        // ADC 8 bits algorithm from spec
        let mut r = (a & 0x0F) + (m & 0x0F) + if carry_in { 1 } else { 0 };
        if r > 9 {
            r += 6;
        }
        let c_low = r > 15;
        r = (a & 0xF0) + (m & 0xF0) + if c_low { 0x10 } else { 0 } + (r & 0x0F);

        // V calculated before final correction
        let v = ((!(a ^ m)) & (a ^ r) & 0x80) != 0;

        if r > 0x9F {
            r += 0x60;
        }
        let c_final = r > 255;
        let result = r & 0xFF;
        (result, c_final, v)
    } else {
        // ADC 16 bits: add as binary then apply iterative BCD corrections
        // Same principle as 8-bit but on 4 nibbles with cumulative thresholds
        let mut result = a + m + if carry_in { 1u32 } else { 0 };

        // Correct units digit (position 0): threshold 9, correction 6
        if (result & 0x0F) > 9 {
            result += 0x6;
        }
        // Correct tens digit (position 1): check after units correction propagated
        if ((result >> 4) & 0x0F) > 9 {
            result += 0x60;
        }
        // Correct hundreds digit (position 2)
        if ((result >> 8) & 0x0F) > 9 {
            result += 0x600;
        }
        // V calculated BEFORE the correction of the last quartet (thousands digit)
        let v = ((!(a ^ m)) & (a ^ result) & 0x8000) != 0;
        // Correct thousands digit (position 3)
        if ((result >> 12) & 0x0F) > 9 {
            result += 0x6000;
        }

        let carry_out = result > 0xFFFF;
        (result & 0xFFFF, carry_out, v)
    }
}

fn adc_decimal(cpu: &mut Cpu, m: u16, wide: bool) {
    if !wide {
        // 8-bit BCD addition
        let a_low = cpu.a as u32 & 0xFF;
        let m_low = m as u32 & 0xFF;
        let (result, carry_out, overflow) = adc_decimal_n(a_low, m_low, cpu.flag(FLAG_C), 1);

        // Preserve high byte in 8-bit mode
        let result_u16 = ((cpu.a & 0xFF00) | (result as u16)) as u16;
        cpu.a = result_u16;
        cpu.set_flag(FLAG_C, carry_out);
        cpu.set_flag(FLAG_V, overflow);
        cpu.set_nz(result as u16, false);
    } else {
        // 16-bit BCD addition
        let a_val = cpu.a as u32;
        let m_val = m as u32;
        let (result, carry_out, overflow) = adc_decimal_n(a_val, m_val, cpu.flag(FLAG_C), 4);

        cpu.a = result as u16;
        cpu.set_flag(FLAG_C, carry_out);
        cpu.set_flag(FLAG_V, overflow);
        cpu.set_nz(result as u16, true);
    }
}
