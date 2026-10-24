// Direct page addressing modes: Dp, DpX, DpY, Sr (stack relative)

use super::super::{Bus, Cpu};
use crate::cpu::table::Am;
use crate::cpu::addr::Ea;

pub(super) fn ea<B: Bus>(cpu: &mut Cpu, bus: &mut B, am: Am, store: bool) -> Ea {
    match am {
        Am::Dp => {
            let dl = cpu.fetch8(bus);
            if (cpu.d & 0xFF) != 0 {
                bus.idle();
            }
            let lo = ((cpu.d as u32) + (dl as u32)) & 0xFFFF;
            let hi = (lo + 1) & 0xFFFF;
            Ea { lo, hi }
        }
        Am::DpX => {
            let dl = cpu.fetch8(bus);
            if (cpu.d & 0xFF) != 0 {
                bus.idle();
            }
            bus.idle();
            let idx = cpu.x as u32;
            let d = cpu.d as u32;
            let lo;
            let hi;
            if cpu.e && (d & 0xFF) == 0 {
                // Emulation mode page wrap: stay within direct page
                let wrapped = ((dl as u32) + idx) & 0xFF;
                lo = (d & 0xFF00) | wrapped;
                hi = (d & 0xFF00) | ((wrapped + 1) & 0xFF);
            } else {
                // Native mode or DL != 0: wrap at 64KB boundary of bank 0
                lo = (d + (dl as u32) + idx) & 0xFFFF;
                hi = (lo + 1) & 0xFFFF;
            }
            Ea { lo, hi }
        }
        Am::DpY => {
            let dl = cpu.fetch8(bus);
            if (cpu.d & 0xFF) != 0 {
                bus.idle();
            }
            bus.idle();
            let idx = cpu.y as u32;
            let d = cpu.d as u32;
            let lo;
            let hi;
            if cpu.e && (d & 0xFF) == 0 {
                // Emulation mode page wrap: stay within direct page
                let wrapped = ((dl as u32) + idx) & 0xFF;
                lo = (d & 0xFF00) | wrapped;
                hi = (d & 0xFF00) | ((wrapped + 1) & 0xFF);
            } else {
                // Native mode or DL != 0: wrap at 64KB boundary of bank 0
                lo = (d + (dl as u32) + idx) & 0xFFFF;
                hi = (lo + 1) & 0xFFFF;
            }
            Ea { lo, hi }
        }
        Am::Sr => {
            let sr = cpu.fetch8(bus);
            bus.idle();
            let lo = ((cpu.s as u32) + (sr as u32)) & 0xFFFF;
            let hi = (lo + 1) & 0xFFFF;
            Ea { lo, hi }
        }
        _ => unreachable!("direct::ea called with non-direct mode"),
    }
}
