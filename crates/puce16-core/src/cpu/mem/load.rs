use crate::cpu::addr;
use crate::cpu::table::{Am, Mn};
use crate::cpu::{Bus, Cpu};

pub(super) fn exec<B: Bus>(cpu: &mut Cpu, bus: &mut B, mn: Mn, am: Am) -> bool {
    match mn {
        Mn::LDA => {
            let wide = !cpu.m8();
            let v = addr::read_operand(cpu, bus, am, wide);
            if wide {
                cpu.a = v;
            } else {
                // Preserve high byte B in 8-bit mode
                cpu.a = ((cpu.a & 0xFF00) | (v & 0xFF)) as u16;
            }
            cpu.set_nz(v, wide);
        }
        Mn::LDX => {
            let wide = !cpu.x8();
            let v = addr::read_operand(cpu, bus, am, wide);
            if wide {
                cpu.x = v;
            } else {
                // Zero-extend in 8-bit mode (high byte becomes 0)
                cpu.x = v & 0xFF;
            }
            cpu.set_nz(v, wide);
        }
        Mn::LDY => {
            let wide = !cpu.x8();
            let v = addr::read_operand(cpu, bus, am, wide);
            if wide {
                cpu.y = v;
            } else {
                // Zero-extend in 8-bit mode (high byte becomes 0)
                cpu.y = v & 0xFF;
            }
            cpu.set_nz(v, wide);
        }
        _ => return false,
    }
    true
}
