use super::addr;
use super::{F_C, SmpBus, Spc700};

fn inc_reg(cpu: &mut Spc700, v: u8) -> u8 {
    let r = v.wrapping_add(1);
    cpu.set_nz(r);
    r
}

fn dec_reg(cpu: &mut Spc700, v: u8) -> u8 {
    let r = v.wrapping_sub(1);
    cpu.set_nz(r);
    r
}

fn asl8(cpu: &mut Spc700, v: u8) -> u8 {
    let c = (v >> 7) != 0;
    let r = v << 1;
    cpu.set_flag(F_C, c);
    cpu.set_nz(r);
    r
}

fn lsr8(cpu: &mut Spc700, v: u8) -> u8 {
    let c = (v & 1) != 0;
    let r = v >> 1;
    cpu.set_flag(F_C, c);
    cpu.set_nz(r);
    r
}

fn rol8(cpu: &mut Spc700, v: u8) -> u8 {
    let cin = cpu.flag(F_C);
    let c = (v >> 7) != 0;
    let mut r = v << 1;
    if cin {
        r |= 1;
    }
    cpu.set_flag(F_C, c);
    cpu.set_nz(r);
    r
}

fn ror8(cpu: &mut Spc700, v: u8) -> u8 {
    let cin = cpu.flag(F_C);
    let c = (v & 1) != 0;
    let mut r = v >> 1;
    if cin {
        r |= 0x80;
    }
    cpu.set_flag(F_C, c);
    cpu.set_nz(r);
    r
}

fn inc_mem<B: SmpBus>(cpu: &mut Spc700, bus: &mut B, ea: u16) {
    let v = cpu.rd(bus, ea);
    let r = inc_reg(cpu, v);
    cpu.wr(bus, ea, r);
}

fn dec_mem<B: SmpBus>(cpu: &mut Spc700, bus: &mut B, ea: u16) {
    let v = cpu.rd(bus, ea);
    let r = dec_reg(cpu, v);
    cpu.wr(bus, ea, r);
}

fn asl_mem<B: SmpBus>(cpu: &mut Spc700, bus: &mut B, ea: u16) {
    let v = cpu.rd(bus, ea);
    let r = asl8(cpu, v);
    cpu.wr(bus, ea, r);
}

fn lsr_mem<B: SmpBus>(cpu: &mut Spc700, bus: &mut B, ea: u16) {
    let v = cpu.rd(bus, ea);
    let r = lsr8(cpu, v);
    cpu.wr(bus, ea, r);
}

fn rol_mem<B: SmpBus>(cpu: &mut Spc700, bus: &mut B, ea: u16) {
    let v = cpu.rd(bus, ea);
    let r = rol8(cpu, v);
    cpu.wr(bus, ea, r);
}

fn ror_mem<B: SmpBus>(cpu: &mut Spc700, bus: &mut B, ea: u16) {
    let v = cpu.rd(bus, ea);
    let r = ror8(cpu, v);
    cpu.wr(bus, ea, r);
}

pub(super) fn exec<B: SmpBus>(cpu: &mut Spc700, bus: &mut B, op: u8) -> bool {
    match op {
        // INC A
        0xBC => {
            cpu.a = inc_reg(cpu, cpu.a);
        }
        // INC X
        0x3D => {
            cpu.x = inc_reg(cpu, cpu.x);
        }
        // INC Y
        0xFC => {
            cpu.y = inc_reg(cpu, cpu.y);
        }
        // INC dp
        0xAB => {
            let ea = addr::ea_dp(cpu, bus);
            inc_mem(cpu, bus, ea);
        }
        // INC dp+X
        0xBB => {
            let ea = addr::ea_dpx(cpu, bus);
            inc_mem(cpu, bus, ea);
        }
        // INC !abs
        0xAC => {
            let ea = addr::ea_abs(cpu, bus);
            inc_mem(cpu, bus, ea);
        }
        // DEC A
        0x9C => {
            cpu.a = dec_reg(cpu, cpu.a);
        }
        // DEC X
        0x1D => {
            cpu.x = dec_reg(cpu, cpu.x);
        }
        // DEC Y
        0xDC => {
            cpu.y = dec_reg(cpu, cpu.y);
        }
        // DEC dp
        0x8B => {
            let ea = addr::ea_dp(cpu, bus);
            dec_mem(cpu, bus, ea);
        }
        // DEC dp+X
        0x9B => {
            let ea = addr::ea_dpx(cpu, bus);
            dec_mem(cpu, bus, ea);
        }
        // DEC !abs
        0x8C => {
            let ea = addr::ea_abs(cpu, bus);
            dec_mem(cpu, bus, ea);
        }
        // ASL A
        0x1C => {
            cpu.a = asl8(cpu, cpu.a);
        }
        // ASL dp
        0x0B => {
            let ea = addr::ea_dp(cpu, bus);
            asl_mem(cpu, bus, ea);
        }
        // ASL dp+X
        0x1B => {
            let ea = addr::ea_dpx(cpu, bus);
            asl_mem(cpu, bus, ea);
        }
        // ASL !abs
        0x0C => {
            let ea = addr::ea_abs(cpu, bus);
            asl_mem(cpu, bus, ea);
        }
        // LSR A
        0x5C => {
            cpu.a = lsr8(cpu, cpu.a);
        }
        // LSR dp
        0x4B => {
            let ea = addr::ea_dp(cpu, bus);
            lsr_mem(cpu, bus, ea);
        }
        // LSR dp+X
        0x5B => {
            let ea = addr::ea_dpx(cpu, bus);
            lsr_mem(cpu, bus, ea);
        }
        // LSR !abs
        0x4C => {
            let ea = addr::ea_abs(cpu, bus);
            lsr_mem(cpu, bus, ea);
        }
        // ROL A
        0x3C => {
            cpu.a = rol8(cpu, cpu.a);
        }
        // ROL dp
        0x2B => {
            let ea = addr::ea_dp(cpu, bus);
            rol_mem(cpu, bus, ea);
        }
        // ROL dp+X
        0x3B => {
            let ea = addr::ea_dpx(cpu, bus);
            rol_mem(cpu, bus, ea);
        }
        // ROL !abs
        0x2C => {
            let ea = addr::ea_abs(cpu, bus);
            rol_mem(cpu, bus, ea);
        }
        // ROR A
        0x7C => {
            cpu.a = ror8(cpu, cpu.a);
        }
        // ROR dp
        0x6B => {
            let ea = addr::ea_dp(cpu, bus);
            ror_mem(cpu, bus, ea);
        }
        // ROR dp+X
        0x7B => {
            let ea = addr::ea_dpx(cpu, bus);
            ror_mem(cpu, bus, ea);
        }
        // ROR !abs
        0x6C => {
            let ea = addr::ea_abs(cpu, bus);
            ror_mem(cpu, bus, ea);
        }
        // XCN A
        0x9F => {
            cpu.a = (cpu.a << 4) | (cpu.a >> 4);
            cpu.set_nz(cpu.a);
        }
        _ => return false,
    }
    true
}

