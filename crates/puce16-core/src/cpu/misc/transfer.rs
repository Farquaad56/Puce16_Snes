use crate::cpu::table::{Am, Mn};
use crate::cpu::{Bus, Cpu};

pub(super) fn exec<B: Bus>(cpu: &mut Cpu, bus: &mut B, mn: Mn, _am: Am) -> bool {
    match mn {
        // TAX: A -> X (width = dest size = !x8())
        Mn::TAX => {
            let wide = !cpu.x8();
            if wide {
                cpu.x = cpu.a;
            } else {
                cpu.x = cpu.a & 0xFF;
            }
            cpu.set_nz(cpu.x, wide);
        }
        // TAY: A -> Y (width = dest size = !x8())
        Mn::TAY => {
            let wide = !cpu.x8();
            if wide {
                cpu.y = cpu.a;
            } else {
                cpu.y = cpu.a & 0xFF;
            }
            cpu.set_nz(cpu.y, wide);
        }
        // TXA: X -> A (width = dest size = !m8())
        Mn::TXA => {
            let wide = !cpu.m8();
            if wide {
                cpu.a = cpu.x;
            } else {
                cpu.a = ((cpu.a & 0xFF00) | (cpu.x & 0xFF)) as u16;
            }
            cpu.set_nz(cpu.a, wide);
        }
        // TYA: Y -> A (width = dest size = !m8())
        Mn::TYA => {
            let wide = !cpu.m8();
            if wide {
                cpu.a = cpu.y;
            } else {
                cpu.a = ((cpu.a & 0xFF00) | (cpu.y & 0xFF)) as u16;
            }
            cpu.set_nz(cpu.a, wide);
        }
        // TSX: S -> X (width = dest size = !x8())
        Mn::TSX => {
            let wide = !cpu.x8();
            if wide {
                cpu.x = cpu.s;
            } else {
                cpu.x = cpu.s & 0xFF;
            }
            cpu.set_nz(cpu.x, wide);
        }
        // TXS: X -> S (no flags)
        Mn::TXS => {
            if cpu.e {
                cpu.s = 0x0100 | (cpu.x & 0xFF);
            } else {
                cpu.s = cpu.x;
            }
        }
        // TXY: X -> Y (width = dest size = !x8())
        Mn::TXY => {
            let wide = !cpu.x8();
            if wide {
                cpu.y = cpu.x;
            } else {
                cpu.y = cpu.x & 0xFF;
            }
            cpu.set_nz(cpu.y, wide);
        }
        // TYX: Y -> X (width = dest size = !x8())
        Mn::TYX => {
            let wide = !cpu.x8();
            if wide {
                cpu.x = cpu.y;
            } else {
                cpu.x = cpu.y & 0xFF;
            }
            cpu.set_nz(cpu.x, wide);
        }
        // TCD: A -> D (always 16 bits)
        Mn::TCD => {
            cpu.d = cpu.a;
            cpu.set_nz16(cpu.d);
        }
        // TDC: D -> A (always 16 bits)
        Mn::TDC => {
            cpu.a = cpu.d;
            cpu.set_nz16(cpu.a);
        }
        // TCS: A -> S (no flags)
        Mn::TCS => {
            if cpu.e {
                cpu.s = 0x0100 | (cpu.a & 0xFF);
            } else {
                cpu.s = cpu.a;
            }
        }
        // TSC: S -> A (always 16 bits)
        Mn::TSC => {
            cpu.a = cpu.s;
            cpu.set_nz16(cpu.a);
        }
        _ => return false,
    }
    bus.idle();
    true
}
