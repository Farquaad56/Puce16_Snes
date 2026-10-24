use crate::cpu::addr;
use crate::cpu::table::{Am, Mn};
use crate::cpu::{Bus, Cpu, FLAG_Z};

pub(super) fn exec<B: Bus>(cpu: &mut Cpu, bus: &mut B, mn: Mn, am: Am) -> bool {
    match mn {
        Mn::TSB => tsb(cpu, bus, am),
        Mn::TRB => trb(cpu, bus, am),
        _ => false,
    }
}

fn tsb<B: Bus>(cpu: &mut Cpu, bus: &mut B, am: Am) -> bool {
    addr::rmw(cpu, bus, am, |c, v, wide| {
        let a = if wide { c.a } else { (c.a & 0xFF) as u16 };
        // Z = (A & M) == 0
        c.set_flag(FLAG_Z, (a & v) == 0);
        // Set bits: M = M | A
        v | a
    });
    true
}

fn trb<B: Bus>(cpu: &mut Cpu, bus: &mut B, am: Am) -> bool {
    addr::rmw(cpu, bus, am, |c, v, wide| {
        let a = if wide { c.a } else { (c.a & 0xFF) as u16 };
        // Z = (A & M) == 0
        c.set_flag(FLAG_Z, (a & v) == 0);
        // Reset bits: M = M & ~A
        v & !a
    });
    true
}
