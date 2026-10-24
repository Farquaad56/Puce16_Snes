use crate::cpu::addr;
use crate::cpu::table::{Am, Mn};
use crate::cpu::{Bus, Cpu, FLAG_C, FLAG_N};

pub(super) fn exec<B: Bus>(cpu: &mut Cpu, bus: &mut B, mn: Mn, am: Am) -> bool {
    match mn {
        Mn::ASL => asl(cpu, bus, am),
        Mn::LSR => lsr(cpu, bus, am),
        Mn::ROL => rol(cpu, bus, am),
        Mn::ROR => ror(cpu, bus, am),
        _ => false,
    }
}

fn asl<B: Bus>(cpu: &mut Cpu, bus: &mut B, am: Am) -> bool {
    addr::rmw(cpu, bus, am, |c, v, wide| {
        let signbit = if wide { 0x8000 } else { 0x80 };
        let mask = if wide { 0xFFFF } else { 0xFF };
        c.set_flag(FLAG_C, (v & signbit) != 0);
        let r = (v << 1) & mask;
        c.set_nz(r, wide);
        r
    });
    true
}

fn lsr<B: Bus>(cpu: &mut Cpu, bus: &mut B, am: Am) -> bool {
    addr::rmw(cpu, bus, am, |c, v, wide| {
        c.set_flag(FLAG_C, (v & 1) != 0);
        let r = v >> 1;
        // LSR always clears N
        c.set_flag(FLAG_N, false);
        c.set_nz(r, wide);
        r
    });
    true
}

fn rol<B: Bus>(cpu: &mut Cpu, bus: &mut B, am: Am) -> bool {
    addr::rmw(cpu, bus, am, |c, v, wide| {
        let cin = if c.flag(FLAG_C) { 1 } else { 0 };
        let signbit = if wide { 0x8000 } else { 0x80 };
        let mask = if wide { 0xFFFF } else { 0xFF };
        c.set_flag(FLAG_C, (v & signbit) != 0);
        let r = ((v << 1) | cin) & mask;
        c.set_nz(r, wide);
        r
    });
    true
}

fn ror<B: Bus>(cpu: &mut Cpu, bus: &mut B, am: Am) -> bool {
    addr::rmw(cpu, bus, am, |c, v, wide| {
        let cin = if c.flag(FLAG_C) { 1 } else { 0 };
        let signbit = if wide { 0x8000 } else { 0x80 };
        c.set_flag(FLAG_C, (v & 1) != 0);
        let r = (v >> 1) | ((cin * signbit) as u16);
        c.set_nz(r, wide);
        r
    });
    true
}
