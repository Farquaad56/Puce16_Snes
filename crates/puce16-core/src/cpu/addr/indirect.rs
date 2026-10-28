// Indirect addressing modes: DpInd, DpIndLong, DpXInd, DpIndY, DpIndLongY, SrIndY

use super::super::{Bus, Cpu};
use crate::cpu::table::Am;
use crate::cpu::addr::Ea;

/// Compute pointer byte addresses for indirect modes.
/// Returns (p0, p1, p2) where p0/p1 are always used and p2 only for long modes.
fn ptr_addrs(cpu: &Cpu, off: u32, long: bool) -> (u32, u32, u32) {
    let d = cpu.d as u32;
    if !long && cpu.e && (d & 0xFF) == 0 {
        // Emulation mode page wrap for non-long indirect modes
        let p0 = (d & 0xFF00) | (off & 0xFF);
        let p1 = (d & 0xFF00) | ((off + 1) & 0xFF);
        (p0, p1, 0)
    } else {
        // Normal wrap at 64KB boundary of bank 0
        let p0 = (d + off) & 0xFFFF;
        let p1 = (p0 + 1) & 0xFFFF;
        let p2 = (p0 + 2) & 0xFFFF;
        (p0, p1, p2)
    }
}

pub(super) fn ea<B: Bus>(cpu: &mut Cpu, bus: &mut B, am: Am, store: bool) -> Ea {
    match am {
        Am::DpInd => {
            let dl = cpu.fetch8(bus);
            if (cpu.d & 0xFF) != 0 {
                bus.idle();
            }
            let (p0, p1, _) = ptr_addrs(cpu, dl as u32, false);
            let lo = bus.read(p0);
            let hi = bus.read(p1);
            let base = ((cpu.db as u32) << 16) | ((hi as u32) << 8) | (lo as u32);
            Ea { lo: base, hi: (base + 1) & 0xFFFFFF }
        }
        Am::DpIndLong => {
            let dl = cpu.fetch8(bus);
            if (cpu.d & 0xFF) != 0 {
                bus.idle();
            }
            let (p0, p1, p2) = ptr_addrs(cpu, dl as u32, true);
            let lo = bus.read(p0);
            let hi = bus.read(p1);
            let bank = bus.read(p2);
            let v = ((bank as u32) << 16) | ((hi as u32) << 8) | (lo as u32);
            Ea { lo: v, hi: (v + 1) & 0xFFFFFF }
        }
        Am::DpXInd => {
            let dl = cpu.fetch8(bus);
            if (cpu.d & 0xFF) != 0 {
                bus.idle();
            }
            bus.idle();
            let off = (dl as u32) + (cpu.x as u32);
            let (mut p0, mut p1, _) = ptr_addrs(cpu, off, false);
            if cpu.e && (cpu.d & 0xFF) != 0 {
                // E=1, DL!=0 (undocumented, verified by gilyon cputest 0027): the dp+D+X sum
                // does not wrap, but the +1 for the pointer high byte wraps inside the page.
                p0 = ((cpu.d as u32) + off) & 0xFFFF;
                p1 = (p0 & 0xFF00) | ((p0 + 1) & 0xFF);
            }
            let lo = bus.read(p0);
            let hi = bus.read(p1);
            let base = ((cpu.db as u32) << 16) | ((hi as u32) << 8) | (lo as u32);
            Ea { lo: base, hi: (base + 1) & 0xFFFFFF }
        }
        Am::DpIndY => {
            let dl = cpu.fetch8(bus);
            if (cpu.d & 0xFF) != 0 {
                bus.idle();
            }
            let (p0, p1, _) = ptr_addrs(cpu, dl as u32, false);
            let lo = bus.read(p0);
            let hi = bus.read(p1);
            let base = ((cpu.db as u32) << 16) | ((hi as u32) << 8) | (lo as u32);
            let addr = (base + (cpu.y as u32)) & 0xFFFFFF;
            let cross = (base >> 8) != (addr >> 8);
            if store || !cpu.x8() || cross {
                bus.idle();
            }
            Ea { lo: addr, hi: (addr + 1) & 0xFFFFFF }
        }
        Am::DpIndLongY => {
            let dl = cpu.fetch8(bus);
            if (cpu.d & 0xFF) != 0 {
                bus.idle();
            }
            let (p0, p1, p2) = ptr_addrs(cpu, dl as u32, true);
            let lo = bus.read(p0);
            let hi = bus.read(p1);
            let bank = bus.read(p2);
            let v = ((bank as u32) << 16) | ((hi as u32) << 8) | (lo as u32);
            let addr = (v + (cpu.y as u32)) & 0xFFFFFF;
            Ea { lo: addr, hi: (addr + 1) & 0xFFFFFF }
        }
        Am::SrIndY => {
            let sr = cpu.fetch8(bus);
            bus.idle();
            let p0 = ((cpu.s as u32) + (sr as u32)) & 0xFFFF;
            let p1 = (p0 + 1) & 0xFFFF;
            let lo = bus.read(p0);
            let hi = bus.read(p1);
            bus.idle();
            let base = ((cpu.db as u32) << 16) | ((hi as u32) << 8) | (lo as u32);
            let addr = (base + (cpu.y as u32)) & 0xFFFFFF;
            Ea { lo: addr, hi: (addr + 1) & 0xFFFFFF }
        }
        _ => unreachable!("indirect::ea called with non-indirect mode"),
    }
}
