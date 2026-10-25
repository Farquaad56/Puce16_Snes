use crate::cpu::addr;
use crate::cpu::table::{Am, Mn};
use crate::cpu::{Bus, Cpu};

pub(super) fn exec<B: Bus>(cpu: &mut Cpu, bus: &mut B, mn: Mn, _am: Am) -> bool {
    match mn {
        // Old stack pushes (PHA/PHX/PHY/PHP/PHB/PHK)
        Mn::PHA => {
            bus.idle();
            let wide = !cpu.m8();
            if wide {
                cpu.push8(bus, ((cpu.a >> 8) & 0xFF) as u8);
                cpu.push8(bus, (cpu.a & 0xFF) as u8);
            } else {
                cpu.push8(bus, (cpu.a & 0xFF) as u8);
            }
        }
        Mn::PHX => {
            bus.idle();
            let wide = !cpu.x8();
            if wide {
                cpu.push8(bus, ((cpu.x >> 8) & 0xFF) as u8);
                cpu.push8(bus, (cpu.x & 0xFF) as u8);
            } else {
                cpu.push8(bus, (cpu.x & 0xFF) as u8);
            }
        }
        Mn::PHY => {
            bus.idle();
            let wide = !cpu.x8();
            if wide {
                cpu.push8(bus, ((cpu.y >> 8) & 0xFF) as u8);
                cpu.push8(bus, (cpu.y & 0xFF) as u8);
            } else {
                cpu.push8(bus, (cpu.y & 0xFF) as u8);
            }
        }
        Mn::PHP => {
            bus.idle();
            cpu.push8(bus, cpu.p);
        }
        Mn::PHB => {
            bus.idle();
            cpu.push8(bus, cpu.db);
        }
        Mn::PHK => {
            bus.idle();
            cpu.push8(bus, cpu.pb);
        }
        // New stack pushes (PHD/PEA/PEI/PER)
        Mn::PHD => {
            bus.idle();
            cpu.push8n(bus, ((cpu.d >> 8) & 0xFF) as u8);
            cpu.push8n(bus, (cpu.d & 0xFF) as u8);
            cpu.fix_stack_e();
        }
        Mn::PEA => {
            let lo = cpu.fetch8(bus);
            let hi = cpu.fetch8(bus);
            cpu.push8n(bus, hi);
            cpu.push8n(bus, lo);
            cpu.fix_stack_e();
        }
        Mn::PEI => {
            let e = addr::ea(cpu, bus, Am::Dp, false);
            let v = addr::read_data(cpu, bus, &e, true);
            cpu.push8n(bus, ((v >> 8) & 0xFF) as u8);
            cpu.push8n(bus, (v & 0xFF) as u8);
            cpu.fix_stack_e();
        }
        Mn::PER => {
            let lo = cpu.fetch8(bus);
            let hi = cpu.fetch8(bus);
            bus.idle();
            let off16: i16 = ((hi as i16) << 8) | (lo as i16);
            let target = cpu.pc.wrapping_add(off16 as u16);
            cpu.push8n(bus, ((target >> 8) & 0xFF) as u8);
            cpu.push8n(bus, (target & 0xFF) as u8);
            cpu.fix_stack_e();
        }
        _ => return false,
    }
    true
}
