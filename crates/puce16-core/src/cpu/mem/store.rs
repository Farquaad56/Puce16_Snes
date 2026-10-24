use crate::cpu::addr;
use crate::cpu::table::{Am, Mn};
use crate::cpu::{Bus, Cpu};

pub(super) fn exec<B: Bus>(cpu: &mut Cpu, bus: &mut B, mn: Mn, am: Am) -> bool {
    let (v, wide) = match mn {
        Mn::STA => (cpu.a, !cpu.m8()),
        Mn::STX => (cpu.x, !cpu.x8()),
        Mn::STY => (cpu.y, !cpu.x8()),
        Mn::STZ => (0, !cpu.m8()),
        _ => return false,
    };

    let e = addr::ea(cpu, bus, am, true);
    addr::write_data(cpu, bus, &e, v, wide);
    // No flags modified by store instructions.
    true
}
