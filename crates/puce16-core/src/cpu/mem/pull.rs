use crate::cpu::table::{Am, Mn};
use crate::cpu::{Bus, Cpu};

pub(super) fn exec<B: Bus>(cpu: &mut Cpu, bus: &mut B, mn: Mn, _am: Am) -> bool {
    match mn {
        // PLA - Pull accumulator
        Mn::PLA => {
            bus.idle();
            bus.idle();
            let wide = !cpu.m8();
            if wide {
                let lo = cpu.pull8(bus);
                let hi = cpu.pull8(bus);
                cpu.a = ((hi as u16) << 8) | (lo as u16);
                cpu.set_nz(cpu.a, true);
            } else {
                let v = cpu.pull8(bus);
                cpu.a = (cpu.a & 0xFF00) | (v as u16);
                cpu.set_nz(v as u16, false);
            }
        }
        // PLX - Pull X register
        Mn::PLX => {
            bus.idle();
            bus.idle();
            let wide = !cpu.x8();
            if wide {
                let lo = cpu.pull8(bus);
                let hi = cpu.pull8(bus);
                cpu.x = ((hi as u16) << 8) | (lo as u16);
                cpu.set_nz(cpu.x, true);
            } else {
                let v = cpu.pull8(bus);
                cpu.x = v as u16;
                cpu.set_nz(v as u16, false);
            }
        }
        // PLY - Pull Y register
        Mn::PLY => {
            bus.idle();
            bus.idle();
            let wide = !cpu.x8();
            if wide {
                let lo = cpu.pull8(bus);
                let hi = cpu.pull8(bus);
                cpu.y = ((hi as u16) << 8) | (lo as u16);
                cpu.set_nz(cpu.y, true);
            } else {
                let v = cpu.pull8(bus);
                cpu.y = v as u16;
                cpu.set_nz(v as u16, false);
            }
        }
        // PLP - Pull processor status
        Mn::PLP => {
            bus.idle();
            bus.idle();
            let v = cpu.pull8(bus);
            cpu.set_p(v);
        }
        // PLB - Pull data bank register (new stack: no page-1 wrap in emulation, S high forced to 1)
        Mn::PLB => {
            bus.idle();
            bus.idle();
            let v = cpu.pull8n(bus);
            cpu.db = v;
            cpu.set_nz8(v);
            cpu.fix_stack_e();
        }
        // PLD - Pull direct page register (new stack)
        Mn::PLD => {
            bus.idle();
            bus.idle();
            let lo = cpu.pull8n(bus);
            let hi = cpu.pull8n(bus);
            cpu.d = ((hi as u16) << 8) | (lo as u16);
            cpu.set_nz16(cpu.d);
            cpu.fix_stack_e();
        }
        _ => return false,
    }
    true
}
