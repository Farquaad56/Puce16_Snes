use super::addr;
use super::{SmpBus, Spc700};

pub(super) fn exec<B: SmpBus>(cpu: &mut Spc700, bus: &mut B, op: u8) -> bool {
    match op {
        // MOV A, #imm
        0xE8 => {
            let v = cpu.fetch8(bus);
            cpu.a = v;
            cpu.set_nz(v);
        }
        // MOV A, dp
        0xE4 => {
            let ea = addr::ea_dp(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.a = v;
            cpu.set_nz(v);
        }
        // MOV A, dp+X
        0xF4 => {
            let ea = addr::ea_dpx(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.a = v;
            cpu.set_nz(v);
        }
        // MOV A, !abs
        0xE5 => {
            let ea = addr::ea_abs(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.a = v;
            cpu.set_nz(v);
        }
        // MOV A, !abs+X
        0xF5 => {
            let ea = addr::ea_absx(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.a = v;
            cpu.set_nz(v);
        }
        // MOV A, !abs+Y
        0xF6 => {
            let ea = addr::ea_absy(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.a = v;
            cpu.set_nz(v);
        }
        // MOV A, (X)
        0xE6 => {
            let ea = addr::ea_x(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.a = v;
            cpu.set_nz(v);
        }
        // MOV A, [dp+X]
        0xE7 => {
            let ea = addr::ea_dpx_ind(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.a = v;
            cpu.set_nz(v);
        }
        // MOV A, [dp]+Y
        0xF7 => {
            let ea = addr::ea_dp_ind_y(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.a = v;
            cpu.set_nz(v);
        }
        // MOV X, #imm
        0xCD => {
            let v = cpu.fetch8(bus);
            cpu.x = v;
            cpu.set_nz(v);
        }
        // MOV X, dp
        0xF8 => {
            let ea = addr::ea_dp(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.x = v;
            cpu.set_nz(v);
        }
        // MOV X, dp+Y
        0xF9 => {
            let ea = addr::ea_dpy(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.x = v;
            cpu.set_nz(v);
        }
        // MOV X, !abs
        0xE9 => {
            let ea = addr::ea_abs(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.x = v;
            cpu.set_nz(v);
        }
        // MOV Y, #imm
        0x8D => {
            let v = cpu.fetch8(bus);
            cpu.y = v;
            cpu.set_nz(v);
        }
        // MOV Y, dp
        0xEB => {
            let ea = addr::ea_dp(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.y = v;
            cpu.set_nz(v);
        }
        // MOV Y, dp+X
        0xFB => {
            let ea = addr::ea_dpx(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.y = v;
            cpu.set_nz(v);
        }
        // MOV Y, !abs
        0xEC => {
            let ea = addr::ea_abs(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.y = v;
            cpu.set_nz(v);
        }

        // === MOV stores (dummy read of destination before write) ===

        // MOV dp, A
        0xC4 => {
            let ea = addr::ea_dp(cpu, bus);
            let _ = cpu.rd(bus, ea);
            cpu.wr(bus, ea, cpu.a);
        }
        // MOV !abs, A
        0xC5 => {
            let ea = addr::ea_abs(cpu, bus);
            let _ = cpu.rd(bus, ea);
            cpu.wr(bus, ea, cpu.a);
        }
        // MOV (X), A
        0xC6 => {
            let ea = addr::ea_x(cpu, bus);
            let _ = cpu.rd(bus, ea);
            cpu.wr(bus, ea, cpu.a);
        }
        // MOV [dp+X], A
        0xC7 => {
            let ea = addr::ea_dpx_ind(cpu, bus);
            let _ = cpu.rd(bus, ea);
            cpu.wr(bus, ea, cpu.a);
        }
        // MOV dp+X, A
        0xD4 => {
            let ea = addr::ea_dpx(cpu, bus);
            let _ = cpu.rd(bus, ea);
            cpu.wr(bus, ea, cpu.a);
        }
        // MOV !abs+X, A
        0xD5 => {
            let ea = addr::ea_absx(cpu, bus);
            let _ = cpu.rd(bus, ea);
            cpu.wr(bus, ea, cpu.a);
        }
        // MOV !abs+Y, A
        0xD6 => {
            let ea = addr::ea_absy(cpu, bus);
            let _ = cpu.rd(bus, ea);
            cpu.wr(bus, ea, cpu.a);
        }
        // MOV [dp]+Y, A
        0xD7 => {
            // store: pointer reads first, then internal + dummy cycle (pattern aligned by wr)
            let b = cpu.fetch8(bus);
            let lo_addr = cpu.dp() | (b as u16);
            let hi_addr = cpu.dp() | ((b.wrapping_add(1)) as u16);
            let ptr = cpu.rd16(bus, lo_addr, hi_addr);
            let ea = (ptr as i32 + cpu.y as i32) as u16;
            cpu.wr(bus, ea, cpu.a);
        }

        // MOV !abs, X
        0xC9 => {
            let ea = addr::ea_abs(cpu, bus);
            let _ = cpu.rd(bus, ea);
            cpu.wr(bus, ea, cpu.x);
        }
        // MOV dp, X
        0xD8 => {
            let ea = addr::ea_dp(cpu, bus);
            let _ = cpu.rd(bus, ea);
            cpu.wr(bus, ea, cpu.x);
        }
        // MOV dp+Y, X
        0xD9 => {
            let ea = addr::ea_dpy(cpu, bus);
            let _ = cpu.rd(bus, ea);
            cpu.wr(bus, ea, cpu.x);
        }

        // MOV dp, Y
        0xCB => {
            let ea = addr::ea_dp(cpu, bus);
            let _ = cpu.rd(bus, ea);
            cpu.wr(bus, ea, cpu.y);
        }
        // MOV !abs, Y
        0xCC => {
            let ea = addr::ea_abs(cpu, bus);
            let _ = cpu.rd(bus, ea);
            cpu.wr(bus, ea, cpu.y);
        }
        // MOV dp+X, Y
        0xDB => {
            let ea = addr::ea_dpx(cpu, bus);
            let _ = cpu.rd(bus, ea);
            cpu.wr(bus, ea, cpu.y);
        }

        // === Register transfers with N/Z flags ===

        // MOV X,A
        0x5D => {
            cpu.x = cpu.a;
            cpu.set_nz(cpu.a);
        }
        // MOV A,X
        0x7D => {
            cpu.a = cpu.x;
            cpu.set_nz(cpu.x);
        }
        // MOV A,Y
        0xDD => {
            cpu.a = cpu.y;
            cpu.set_nz(cpu.y);
        }
        // MOV Y,A
        0xFD => {
            cpu.y = cpu.a;
            cpu.set_nz(cpu.a);
        }
        // MOV X,SP (sets N/Z)
        0x9D => {
            cpu.x = cpu.sp;
            cpu.set_nz(cpu.sp);
        }

        // === Register transfers without flags ===

        // MOV SP,X (no flags)
        0xBD => {
            cpu.sp = cpu.x;
        }

        // === MOV dp,#imm (dummy read of destination before write) ===

        // MOV dp,#imm
        0x8F => {
            let imm = cpu.fetch8(bus);
            let dp = cpu.fetch8(bus);
            let ea = cpu.dp() | dp as u16;
            let _ = cpu.rd(bus, ea);
            cpu.wr(bus, ea, imm);
        }

        // === MOV dp,dp (read source, write destination - no dummy read) ===

        // MOV dp,dp
        0xFA => {
            let src = cpu.fetch8(bus);
            let dst = cpu.fetch8(bus);
            let src_ea = cpu.dp() | src as u16;
            let dst_ea = cpu.dp() | dst as u16;
            let v = cpu.rd(bus, src_ea);
            cpu.wr(bus, dst_ea, v);
        }

        // === MOV A,(X)+ and MOV (X)+,A (post-increment X) ===

        // MOV A,(X)+
        0xBF => {
            let ea = addr::ea_x(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.a = v;
            cpu.x = cpu.x.wrapping_add(1);
            cpu.set_nz(v);
        }
        // MOV (X)+,A
        0xAF => {
            let ea = addr::ea_x(cpu, bus);
            cpu.wr(bus, ea, cpu.a);
            cpu.x = cpu.x.wrapping_add(1);
        }

        // === MOVW YA,dp and MOVW dp,YA (16-bit transfers) ===

        // MOVW YA,dp (load 16-bit word into Y:A)
        0xBA => {
            let b = cpu.fetch8(bus);
            let ea = cpu.dp() | b as u16;
            let lo = cpu.rd(bus, ea);
            let hi_ea = cpu.dp() | ((b.wrapping_add(1)) as u16);
            let hi = cpu.rd(bus, hi_ea);
            let word = ((hi as u16) << 8) | (lo as u16);
            cpu.set_ya(word);
            // N = bit 15 of the word, Z = (word == 0)
            cpu.set_flag(0x80, hi & 0x80 != 0);
            cpu.set_flag(0x02, word == 0);
        }
        // MOVW dp,YA (store 16-bit word from Y:A)
        0xDA => {
            let b = cpu.fetch8(bus);
            let ea = cpu.dp() | b as u16;
            let _ = cpu.rd(bus, ea);
            cpu.wr(bus, ea, cpu.a);
            let hi_ea = cpu.dp() | ((b.wrapping_add(1)) as u16);
            cpu.wr(bus, hi_ea, cpu.y);
        }

        _ => return false,
    }
    true
}
