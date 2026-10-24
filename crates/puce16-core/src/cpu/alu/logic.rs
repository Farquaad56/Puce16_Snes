use crate::cpu::addr;
use crate::cpu::table::{Am, Mn};
use crate::cpu::{Bus, Cpu, FLAG_N, FLAG_V, FLAG_Z};

pub(super) fn exec<B: Bus>(cpu: &mut Cpu, bus: &mut B, mn: Mn, am: Am) -> bool {
    match mn {
        Mn::AND | Mn::ORA | Mn::EOR => logic_op(cpu, bus, mn, am),
        Mn::BIT => bit_op(cpu, bus, am),
        _ => false,
    }
}

fn logic_op<B: Bus>(cpu: &mut Cpu, bus: &mut B, mn: Mn, am: Am) -> bool {
    let wide = !cpu.m8();
    let m = addr::read_operand(cpu, bus, am, wide);
    let a = cpu.a;

    // In 8-bit mode, only use low byte of A for the operation
    let a_op: u16 = if wide { a } else { a & 0xFF };
    let m_op: u16 = if wide { m } else { m & 0xFF };

    let r = match mn {
        Mn::AND => a_op & m_op,
        Mn::ORA => a_op | m_op,
        Mn::EOR => a_op ^ m_op,
        _ => unreachable!(),
    };

    if wide {
        cpu.a = r;
    } else {
        // Preserve high byte in 8-bit mode
        cpu.a = (a & 0xFF00) | (r & 0xFF);
    }

    cpu.set_nz(r, wide);
    true
}

fn bit_op<B: Bus>(cpu: &mut Cpu, bus: &mut B, am: Am) -> bool {
    let wide = !cpu.m8();
    let m = addr::read_operand(cpu, bus, am, wide);
    let a = cpu.a;

    // In 8-bit mode, only use low byte of A for the operation
    let a_op: u16 = if wide { a } else { a & 0xFF };
    let m_op: u16 = if wide { m } else { m & 0xFF };

    // Z = (A & M) == 0
    cpu.set_flag(FLAG_Z, (a_op & m_op) == 0);

    if am == Am::Imm {
        // BIT #imm: only Z is affected; N and V unchanged
    } else {
        // BIT mem: N = bit7/15 of M, V = bit6/14 of M
        let n_bit = if wide { 0x8000 } else { 0x80 };
        let v_bit = if wide { 0x4000 } else { 0x40 };
        cpu.set_flag(FLAG_N, (m_op & n_bit) != 0);
        cpu.set_flag(FLAG_V, (m_op & v_bit) != 0);
    }

    true
}
