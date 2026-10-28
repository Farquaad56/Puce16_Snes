use super::addr;
use super::{F_C, SmpBus, Spc700};

impl Spc700 {
    pub fn or_op(&mut self, a: u8, b: u8) -> u8 {
        let r = a | b;
        self.set_nz(r);
        r
    }

    pub fn and_op(&mut self, a: u8, b: u8) -> u8 {
        let r = a & b;
        self.set_nz(r);
        r
    }

    pub fn eor_op(&mut self, a: u8, b: u8) -> u8 {
        let r = a ^ b;
        self.set_nz(r);
        r
    }

    pub fn adc_op(&mut self, a: u8, b: u8) -> u8 {
        use super::{F_C, F_H, F_V};
        let cin = if self.flag(F_C) { 1u32 } else { 0 };
        let r = a as u32 + b as u32 + cin;
        self.set_flag(F_C, r > 0xFF);
        let h_in = (a & 0xF) as u32 + (b & 0xF) as u32 + cin;
        self.set_flag(F_H, h_in > 0xF);
        let v_mask = (!(a as u32 ^ b as u32)) & ((a as u32) ^ (r & 0xFF));
        self.set_flag(F_V, (v_mask & 0x80) != 0);
        self.set_nz(r as u8);
        r as u8
    }

    pub fn sbc_op(&mut self, a: u8, b: u8) -> u8 {
        // SBC is exactly ADC with complemented operand (spec 6.2)
        self.adc_op(a, !b)
    }

    pub fn cmp_op(&mut self, a: u8, b: u8) -> u8 {
        let r = a.wrapping_sub(b);
        self.set_flag(F_C, a >= b);
        self.set_nz(r);
        r
    }
}

fn alu_form<B: SmpBus>(
    cpu: &mut Spc700,
    bus: &mut B,
    form: u8,
    f: fn(&mut Spc700, u8, u8) -> u8,
) -> bool {
    match form {
        // A,dp
        0x04 => {
            let ea = addr::ea_dp(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.a = f(cpu, cpu.a, v);
        }
        // A,!abs
        0x05 => {
            let ea = addr::ea_abs(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.a = f(cpu, cpu.a, v);
        }
        // A,(X)
        0x06 => {
            let ea = addr::ea_x(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.a = f(cpu, cpu.a, v);
        }
        // A,[dp+X]
        0x07 => {
            let ea = addr::ea_dpx_ind(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.a = f(cpu, cpu.a, v);
        }
        // A,#imm
        0x08 => {
            let v = cpu.fetch8(bus);
            cpu.a = f(cpu, cpu.a, v);
        }
        // dp,dp (read source first, then destination)
        0x09 => {
            let src = cpu.fetch8(bus);
            let dst = cpu.fetch8(bus);
            let src_ea = cpu.dp() | src as u16;
            let dst_ea = cpu.dp() | dst as u16;
            let sv = cpu.rd(bus, src_ea);
            let dv = cpu.rd(bus, dst_ea);
            let r = f(cpu, dv, sv);
            cpu.wr(bus, dst_ea, r);
        }
        // A,dp+X
        0x14 => {
            let ea = addr::ea_dpx(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.a = f(cpu, cpu.a, v);
        }
        // A,!abs+X
        0x15 => {
            let ea = addr::ea_absx(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.a = f(cpu, cpu.a, v);
        }
        // A,!abs+Y
        0x16 => {
            let ea = addr::ea_absy(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.a = f(cpu, cpu.a, v);
        }
        // A,[dp]+Y
        0x17 => {
            let ea = addr::ea_dp_ind_y(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.a = f(cpu, cpu.a, v);
        }
        // dp,#imm (read imm first, then dp)
        0x18 => {
            let imm = cpu.fetch8(bus);
            let dst = cpu.fetch8(bus);
            let dst_ea = cpu.dp() | dst as u16;
            let dv = cpu.rd(bus, dst_ea);
            let r = f(cpu, dv, imm);
            cpu.wr(bus, dst_ea, r);
        }
        // (X),(Y) (read Y location first, then X location)
        0x19 => {
            cpu.idle(bus);
            let ea_y = cpu.dp() | cpu.y as u16;
            let yv = cpu.rd(bus, ea_y);
            let ea_x = cpu.dp() | cpu.x as u16;
            let xv = cpu.rd(bus, ea_x);
            let r = f(cpu, xv, yv);
            cpu.wr(bus, ea_x, r);
        }
        _ => return false,
    }
    true
}

fn cmp_form<B: SmpBus>(
    cpu: &mut Spc700,
    bus: &mut B,
    form: u8,
) -> bool {
    match form {
        // A,dp (no write-back to A)
        0x04 => {
            let ea = addr::ea_dp(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.cmp_op(cpu.a, v);
        }
        // A,!abs (no write-back to A)
        0x05 => {
            let ea = addr::ea_abs(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.cmp_op(cpu.a, v);
        }
        // A,(X) (no write-back to A)
        0x06 => {
            let ea = addr::ea_x(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.cmp_op(cpu.a, v);
        }
        // A,[dp+X] (no write-back to A)
        0x07 => {
            let ea = addr::ea_dpx_ind(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.cmp_op(cpu.a, v);
        }
        // A,#imm (no write-back to A)
        0x08 => {
            let v = cpu.fetch8(bus);
            cpu.cmp_op(cpu.a, v);
        }
        // dp,dp (read source first, then destination; no write-back)
        0x09 => {
            let src = cpu.fetch8(bus);
            let dst = cpu.fetch8(bus);
            let src_ea = cpu.dp() | src as u16;
            let dst_ea = cpu.dp() | dst as u16;
            let sv = cpu.rd(bus, src_ea);
            let dv = cpu.rd(bus, dst_ea);
            cpu.cmp_op(dv, sv);
        }
        // A,dp+X (no write-back to A)
        0x14 => {
            let ea = addr::ea_dpx(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.cmp_op(cpu.a, v);
        }
        // A,!abs+X (no write-back to A)
        0x15 => {
            let ea = addr::ea_absx(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.cmp_op(cpu.a, v);
        }
        // A,!abs+Y (no write-back to A)
        0x16 => {
            let ea = addr::ea_absy(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.cmp_op(cpu.a, v);
        }
        // A,[dp]+Y (no write-back to A)
        0x17 => {
            let ea = addr::ea_dp_ind_y(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.cmp_op(cpu.a, v);
        }
        // dp,#imm (read imm first, then dp; no write-back)
        0x18 => {
            let imm = cpu.fetch8(bus);
            let dst = cpu.fetch8(bus);
            let dst_ea = cpu.dp() | dst as u16;
            let dv = cpu.rd(bus, dst_ea);
            cpu.cmp_op(dv, imm);
        }
        // (X),(Y) (read Y location first, then X location; no write-back)
        0x19 => {
            cpu.idle(bus);
            let ea_y = cpu.dp() | cpu.y as u16;
            let yv = cpu.rd(bus, ea_y);
            let ea_x = cpu.dp() | cpu.x as u16;
            let xv = cpu.rd(bus, ea_x);
            cpu.cmp_op(xv, yv);
        }
        _ => return false,
    }
    true
}

pub(super) fn exec<B: SmpBus>(cpu: &mut Spc700, bus: &mut B, op: u8) -> bool {
    let kind = op & 0xE0;
    let form = op & 0x1F;
    match op {
        // CMP X,#imm
        0xC8 => {
            let v = cpu.fetch8(bus);
            cpu.cmp_op(cpu.x, v);
            true
        }
        // CMP X,dp
        0x3E => {
            let ea = addr::ea_dp(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.cmp_op(cpu.x, v);
            true
        }
        // CMP X,!abs
        0x1E => {
            let ea = addr::ea_abs(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.cmp_op(cpu.x, v);
            true
        }
        // CMP Y,#imm
        0xAD => {
            let v = cpu.fetch8(bus);
            cpu.cmp_op(cpu.y, v);
            true
        }
        // CMP Y,dp
        0x7E => {
            let ea = addr::ea_dp(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.cmp_op(cpu.y, v);
            true
        }
        // CMP Y,!abs
        0x5E => {
            let ea = addr::ea_abs(cpu, bus);
            let v = cpu.rd(bus, ea);
            cpu.cmp_op(cpu.y, v);
            true
        }
        _ => match kind {
            0x00 => alu_form(cpu, bus, form, Spc700::or_op),
            0x20 => alu_form(cpu, bus, form, Spc700::and_op),
            0x40 => alu_form(cpu, bus, form, Spc700::eor_op),
            0x60 => cmp_form(cpu, bus, form),
            0x80 => alu_form(cpu, bus, form, Spc700::adc_op),
            0xA0 => alu_form(cpu, bus, form, Spc700::sbc_op),
            _ => false,
        },
    }
}
