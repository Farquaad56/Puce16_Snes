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

    // Overflow: for subtraction, V is set when A and M have different signs
    // but the result has a different sign than A
    let v = ((a_op ^ m_op) & (a_op ^ r) & signbit) != 0;
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

// SBC decimal: same loop with m' = !m, in SIGNED arithmetic (a negative intermediate
// result after the correction means "no carry"). Returns (result, carry, overflow).
fn sbc_decimal_n(a: u32, m: u32, carry_in: bool, wide: bool) -> (u32, bool, bool) {
    let nibbles: usize = if wide { 4 } else { 2 };
    let a = a as i32;
    let mp = (!m) as i32;
    let mut r: i32 = 0;
    let mut c: i32 = carry_in as i32;
    let mut v = false;
    for i in 0..nibbles {
        let sh = 4 * i as u32;
        let low = (1i32 << sh) - 1;
        let mask = 0xFi32 << sh;
        r = (a & mask) + (mp & mask) + (c << sh) + (r & low);
        if i == nibbles - 1 {
            v = (!(a ^ mp) & (a ^ r) & (0x8i32 << sh)) != 0;
        }
        if r <= (1i32 << (sh + 4)) - 1 {
            r -= 6i32 << sh;
        }
        c = (r > (1i32 << (sh + 4)) - 1) as i32;
    }
    ((r as u32) & ((1u32 << (4 * nibbles as u32)) - 1), c != 0, v)
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
