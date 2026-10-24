// Effective address calculation and data access layer shared by all instructions.
// Three addressing mode groups are in submodules: absolute, direct, indirect.

use super::{Bus, Cpu};
use crate::cpu::table::Am;

mod absolute;
mod direct;
mod indirect;

/// Effective address for a memory operand (24-bit low and high bytes).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ea {
    pub lo: u32,
    pub hi: u32,
}

/// Calculate the effective address for an addressing mode.
/// `store` = true if the instruction writes (STA/STX/STY/STZ and all RMW): forces
/// the extra IO cycle of indexed modes.
pub fn ea<B: Bus>(cpu: &mut Cpu, bus: &mut B, am: Am, store: bool) -> Ea {
    match am {
        // Absolute group
        Am::Abs | Am::AbsX | Am::AbsY | Am::Long | Am::LongX => {
            absolute::ea(cpu, bus, am, store)
        }
        // Direct page group
        Am::Dp | Am::DpX | Am::DpY | Am::Sr => direct::ea(cpu, bus, am, store),
        // Indirect group
        Am::DpInd | Am::DpIndLong | Am::DpXInd | Am::DpIndY | Am::DpIndLongY | Am::SrIndY => {
            indirect::ea(cpu, bus, am, store)
        }
        _ => panic!("ea: mode {:?} has no memory operand", am),
    }
}

/// Read data at the effective address. `wide` = true for 16-bit access.
pub fn read_data<B: Bus>(cpu: &mut Cpu, bus: &mut B, ea: &Ea, wide: bool) -> u16 {
    let lo = bus.read(ea.lo);
    if wide {
        let hi = bus.read(ea.hi);
        (hi as u16) << 8 | (lo as u16)
    } else {
        lo as u16
    }
}

/// Write data at the effective address. `wide` = true for 16-bit access.
pub fn write_data<B: Bus>(cpu: &mut Cpu, bus: &mut B, ea: &Ea, v: u16, wide: bool) {
    bus.write(ea.lo, (v & 0xFF) as u8);
    if wide {
        bus.write(ea.hi, ((v >> 8) & 0xFF) as u8);
    }
}

/// Read an operand value. For immediate mode, fetch from PC; otherwise calculate EA and read memory.
pub fn read_operand<B: Bus>(cpu: &mut Cpu, bus: &mut B, am: Am, wide: bool) -> u16 {
    if am == Am::Imm {
        if wide {
            cpu.fetch16(bus)
        } else {
            cpu.fetch8(bus) as u16
        }
    } else {
        let e = ea(cpu, bus, am, false);
        read_data(cpu, bus, &e, wide)
    }
}

/// Read-modify-write operation. `f` receives (cpu, old_value, wide) and returns the new value.
pub fn rmw<B: Bus, F: FnOnce(&mut Cpu, u16, bool) -> u16>(
    cpu: &mut Cpu,
    bus: &mut B,
    am: Am,
    f: F,
) {
    let wide = !cpu.m8();

    if am == Am::Acc {
        // Accumulator RMW: IO cycle only, no memory access
        bus.idle();
        let old = if wide { cpu.a } else { (cpu.a & 0xFF) as u16 };
        let new = f(cpu, old, wide);
        if wide {
            cpu.a = new;
        } else {
            // Preserve high byte B in 8-bit mode
            cpu.a = ((cpu.a & 0xFF00) | (new & 0xFF)) as u16;
        }
    } else {
        // Memory RMW: read, IO cycle, write back
        let e = ea(cpu, bus, am, true);
        let lo = bus.read(e.lo);
        let old = if wide {
            let hi = bus.read(e.hi);
            ((hi as u16) << 8) | (lo as u16)
        } else {
            lo as u16
        };

        bus.idle(); // IO cycle for modification

        let new = f(cpu, old, wide);

        if wide {
            // Write high byte first in 16-bit mode
            bus.write(e.hi, ((new >> 8) & 0xFF) as u8);
            bus.write(e.lo, (new & 0xFF) as u8);
        } else {
            bus.write(e.lo, (new & 0xFF) as u8);
        }
    }
}
