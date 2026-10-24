// Absolute addressing modes: Abs, AbsX, AbsY, Long, LongX

use super::super::{Bus, Cpu};
use crate::cpu::table::Am;
use crate::cpu::addr::Ea;

pub(super) fn ea<B: Bus>(cpu: &mut Cpu, bus: &mut B, am: Am, store: bool) -> Ea {
    match am {
        Am::Abs => {
            let aa = cpu.fetch16(bus);
            let base = ((cpu.db as u32) << 16) | (aa as u32);
            Ea { lo: base, hi: (base + 1) & 0xFFFFFF }
        }
        Am::AbsX => {
            let aa = cpu.fetch16(bus);
            let base = ((cpu.db as u32) << 16) | (aa as u32);
            let idx = cpu.x as u32;
            let addr = (base + idx) & 0xFFFFFF;
            let cross = (base >> 8) != (addr >> 8);
            if store || !cpu.x8() || cross {
                bus.idle();
            }
            Ea { lo: addr, hi: (addr + 1) & 0xFFFFFF }
        }
        Am::AbsY => {
            let aa = cpu.fetch16(bus);
            let base = ((cpu.db as u32) << 16) | (aa as u32);
            let idx = cpu.y as u32;
            let addr = (base + idx) & 0xFFFFFF;
            let cross = (base >> 8) != (addr >> 8);
            if store || !cpu.x8() || cross {
                bus.idle();
            }
            Ea { lo: addr, hi: (addr + 1) & 0xFFFFFF }
        }
        Am::Long => {
            let v = cpu.fetch24(bus);
            Ea { lo: v, hi: (v + 1) & 0xFFFFFF }
        }
        Am::LongX => {
            let v = cpu.fetch24(bus);
            let addr = (v + (cpu.x as u32)) & 0xFFFFFF;
            Ea { lo: addr, hi: (addr + 1) & 0xFFFFFF }
        }
        _ => unreachable!("absolute::ea called with non-absolute mode"),
    }
}
