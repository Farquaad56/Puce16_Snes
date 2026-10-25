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
    let mut r: u32 = 0;
    let mut c: u32 = carry_in as u32;
    let mut v = false;
    for i in 0..nibbles {
        let sh = 4 * i as u32;
        let low = (1u32 << sh) - 1;
        let mask = 0xFu32 << sh;
        r = (a & mask) + (m & mask) + (c << sh) + (r & low);
        if i == nibbles - 1 {
            v = (!(a ^ m) & (a ^ r) & (0x8u32 << sh)) != 0;
        }
        if r > ((9u32 << sh) | low) {
            r += 6u32 << sh;
        }
        c = (r > ((1u32 << (sh + 4)) - 1)) as u32;
    }
    (r & ((1u32 << (4 * nibbles as u32)) - 1), c != 0, v)
}

fn adc_decimal(cpu: &mut Cpu, m: u16, wide: bool) {
    if !wide {
        // 8-bit BCD addition
        let a_low = cpu.a as u32 & 0xFF;
        let m_low = m as u32 & 0xFF;
        let (result, carry_out, overflow) = adc_decimal_n(a_low, m_low, cpu.flag(FLAG_C), 2);

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
