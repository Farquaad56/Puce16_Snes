use crate::cpu::addr;
use crate::cpu::table::{Am, Mn};
use crate::cpu::{Bus, Cpu};

pub(super) fn exec<B: Bus>(cpu: &mut Cpu, bus: &mut B, mn: Mn, am: Am) -> bool {
    match mn {
        Mn::INC => inc(cpu, bus, am),
        Mn::DEC => dec(cpu, bus, am),
        Mn::INX => inx(cpu, bus),
        Mn::DEX => dex(cpu, bus),
        Mn::INY => iny(cpu, bus),
        Mn::DEY => dey(cpu, bus),
        _ => false,
    }
}

fn inc<B: Bus>(cpu: &mut Cpu, bus: &mut B, am: Am) -> bool {
    addr::rmw(cpu, bus, am, |c, v, wide| {
        let r = if wide {
            v.wrapping_add(1)
        } else {
            (v as u8).wrapping_add(1) as u16
        };
        c.set_nz(r, wide);
        r
    });
    true
}

fn dec<B: Bus>(cpu: &mut Cpu, bus: &mut B, am: Am) -> bool {
    addr::rmw(cpu, bus, am, |c, v, wide| {
        let r = if wide {
            v.wrapping_sub(1)
        } else {
            (v as u8).wrapping_sub(1) as u16
        };
        c.set_nz(r, wide);
        r
    });
    true
}

fn inx<B: Bus>(cpu: &mut Cpu, bus: &mut B) -> bool {
    bus.idle();
    let wide = !cpu.x8();
    let r = if wide {
        cpu.x.wrapping_add(1)
    } else {
        (cpu.x as u8).wrapping_add(1) as u16
    };
    cpu.x = r;
    cpu.set_nz(r, wide);
    true
}

fn dex<B: Bus>(cpu: &mut Cpu, bus: &mut B) -> bool {
    bus.idle();
    let wide = !cpu.x8();
    let r = if wide {
        cpu.x.wrapping_sub(1)
    } else {
        (cpu.x as u8).wrapping_sub(1) as u16
    };
    cpu.x = r;
    cpu.set_nz(r, wide);
    true
}

fn iny<B: Bus>(cpu: &mut Cpu, bus: &mut B) -> bool {
    bus.idle();
    let wide = !cpu.x8();
    let r = if wide {
        cpu.y.wrapping_add(1)
    } else {
        (cpu.y as u8).wrapping_add(1) as u16
    };
    cpu.y = r;
    cpu.set_nz(r, wide);
    true
}

fn dey<B: Bus>(cpu: &mut Cpu, bus: &mut B) -> bool {
    bus.idle();
    let wide = !cpu.x8();
    let r = if wide {
        cpu.y.wrapping_sub(1)
    } else {
        (cpu.y as u8).wrapping_sub(1) as u16
    };
    cpu.y = r;
    cpu.set_nz(r, wide);
    true
}
