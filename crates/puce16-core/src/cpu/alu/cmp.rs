use crate::cpu::addr;
use crate::cpu::table::{Am, Mn};
use crate::cpu::{Bus, Cpu, FLAG_C};

pub(super) fn exec<B: Bus>(cpu: &mut Cpu, bus: &mut B, mn: Mn, am: Am) -> bool {
    match mn {
        Mn::CMP => cmp_a(cpu, bus, am),
        Mn::CPX => cmp_x(cpu, bus, am),
        Mn::CPY => cmp_y(cpu, bus, am),
        _ => false,
    }
}

fn cmp_a<B: Bus>(cpu: &mut Cpu, bus: &mut B, am: Am) -> bool {
    let wide = !cpu.m8();
    let m = addr::read_operand(cpu, bus, am, wide);
    if wide {
        let a = cpu.a;
        let r = a.wrapping_sub(m);
        cpu.set_flag(FLAG_C, a >= m);
        cpu.set_nz(r, true);
    } else {
        let a_low = (cpu.a & 0xFF) as u8;
        let m_low = (m & 0xFF) as u8;
        let r = a_low.wrapping_sub(m_low);
        cpu.set_flag(FLAG_C, a_low >= m_low);
        cpu.set_nz(r as u16, false);
    }
    true
}

fn cmp_x<B: Bus>(cpu: &mut Cpu, bus: &mut B, am: Am) -> bool {
    let wide = !cpu.x8();
    let m = addr::read_operand(cpu, bus, am, wide);
    if wide {
        let x = cpu.x;
        let r = x.wrapping_sub(m);
        cpu.set_flag(FLAG_C, x >= m);
        cpu.set_nz(r, true);
    } else {
        let x_low = (cpu.x & 0xFF) as u8;
        let m_low = (m & 0xFF) as u8;
        let r = x_low.wrapping_sub(m_low);
        cpu.set_flag(FLAG_C, x_low >= m_low);
        cpu.set_nz(r as u16, false);
    }
    true
}

fn cmp_y<B: Bus>(cpu: &mut Cpu, bus: &mut B, am: Am) -> bool {
    let wide = !cpu.x8();
    let m = addr::read_operand(cpu, bus, am, wide);
    if wide {
        let y = cpu.y;
        let r = y.wrapping_sub(m);
        cpu.set_flag(FLAG_C, y >= m);
        cpu.set_nz(r, true);
    } else {
        let y_low = (cpu.y & 0xFF) as u8;
        let m_low = (m & 0xFF) as u8;
        let r = y_low.wrapping_sub(m_low);
        cpu.set_flag(FLAG_C, y_low >= m_low);
        cpu.set_nz(r as u16, false);
    }
    true
}
