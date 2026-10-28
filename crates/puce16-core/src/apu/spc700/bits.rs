use super::addr;
use super::{SmpBus, Spc700};

/// SET1 dp.n - set bit n in direct page byte
fn set1<B: SmpBus>(cpu: &mut Spc700, bus: &mut B, bit: u8) {
    let b = cpu.fetch8(bus);
    let ea = cpu.dp() | (b as u16);
    let v = cpu.rd(bus, ea);
    cpu.wr(bus, ea, v | (1 << bit));
}

/// CLR1 dp.n - clear bit n in direct page byte
fn clr1<B: SmpBus>(cpu: &mut Spc700, bus: &mut B, bit: u8) {
    let b = cpu.fetch8(bus);
    let ea = cpu.dp() | (b as u16);
    let v = cpu.rd(bus, ea);
    cpu.wr(bus, ea, v & !(1 << bit));
}

/// BBS dp.n,rel - branch if bit n set in direct page byte
fn bbs<B: SmpBus>(cpu: &mut Spc700, bus: &mut B, bit: u8) {
    let b = cpu.fetch8(bus);
    let rel = cpu.fetch8(bus);
    let ea = cpu.dp() | (b as u16);
    let v = cpu.rd(bus, ea);
    if (v >> bit) & 1 != 0 {
        cpu.branch(rel);
    }
}

/// BBC dp.n,rel - branch if bit n clear in direct page byte
fn bbc<B: SmpBus>(cpu: &mut Spc700, bus: &mut B, bit: u8) {
    let b = cpu.fetch8(bus);
    let rel = cpu.fetch8(bus);
    let ea = cpu.dp() | (b as u16);
    let v = cpu.rd(bus, ea);
    if (v >> bit) & 1 == 0 {
        cpu.branch(rel);
    }
}

/// TSET1 !abs - test and set bits in absolute address
fn tset1<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    let ea = addr::ea_abs(cpu, bus);
    let t = cpu.rd(bus, ea);
    let r = cpu.a.wrapping_sub(t);
    cpu.set_nz(r);
    cpu.wr(bus, ea, t | cpu.a);
}

/// TCLR1 !abs - test and clear bits in absolute address
fn tclr1<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    let ea = addr::ea_abs(cpu, bus);
    let t = cpu.rd(bus, ea);
    let r = cpu.a.wrapping_sub(t);
    cpu.set_nz(r);
    cpu.wr(bus, ea, t & !cpu.a);
}

/// OR1 C,m.b - OR carry with bit in memory (absolute)
fn or1<B: SmpBus>(cpu: &mut Spc700, bus: &mut B, invert: bool) {
    let v = cpu.fetch16(bus);
    let ea = v & 0x1FFF;
    let bit = (v >> 13) as u8;
    let m = (cpu.rd(bus, ea) >> bit) & 1 != 0;
    if invert {
        cpu.set_flag(super::F_C, cpu.flag(super::F_C) || !m);
    } else {
        cpu.set_flag(super::F_C, cpu.flag(super::F_C) || m);
    }
}

/// AND1 C,m.b - AND carry with bit in memory (absolute)
fn and1<B: SmpBus>(cpu: &mut Spc700, bus: &mut B, invert: bool) {
    let v = cpu.fetch16(bus);
    let ea = v & 0x1FFF;
    let bit = (v >> 13) as u8;
    let m = (cpu.rd(bus, ea) >> bit) & 1 != 0;
    if invert {
        cpu.set_flag(super::F_C, cpu.flag(super::F_C) && !m);
    } else {
        cpu.set_flag(super::F_C, cpu.flag(super::F_C) && m);
    }
}

/// EOR1 C,m.b - XOR carry with bit in memory (absolute)
fn eor1<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    let v = cpu.fetch16(bus);
    let ea = v & 0x1FFF;
    let bit = (v >> 13) as u8;
    let m = (cpu.rd(bus, ea) >> bit) & 1 != 0;
    cpu.set_flag(super::F_C, cpu.flag(super::F_C) ^ m);
}

/// MOV1 C,m.b - move bit from memory to carry (absolute)
fn mov1_to_c<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    let v = cpu.fetch16(bus);
    let ea = v & 0x1FFF;
    let bit = (v >> 13) as u8;
    let m = (cpu.rd(bus, ea) >> bit) & 1 != 0;
    cpu.set_flag(super::F_C, m);
}

/// MOV1 m.b,C - move carry to bit in memory (absolute)
fn mov1_to_mem<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    let v = cpu.fetch16(bus);
    let ea = v & 0x1FFF;
    let bit = (v >> 13) as u8;
    let t = cpu.rd(bus, ea);
    if cpu.flag(super::F_C) {
        cpu.wr(bus, ea, t | (1 << bit));
    } else {
        cpu.wr(bus, ea, t & !(1 << bit));
    }
}

/// NOT1 m.b - invert bit in memory (absolute)
fn not1<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    let v = cpu.fetch16(bus);
    let ea = v & 0x1FFF;
    let bit = (v >> 13) as u8;
    let t = cpu.rd(bus, ea);
    cpu.wr(bus, ea, t ^ (1 << bit));
}

pub(super) fn exec<B: SmpBus>(cpu: &mut Spc700, bus: &mut B, op: u8) -> bool {
    // SET1 dp.n (opcodes 02, 22, 42, 62, 82, A2, C2, E2)
    if op & 0x1F == 0x02 {
        let bit = (op >> 5) as u8;
        set1(cpu, bus, bit);
        return true;
    }
    // CLR1 dp.n (opcodes 12, 32, 52, 72, 92, B2, D2, F2)
    if op & 0x1F == 0x12 {
        let bit = (op >> 5) as u8;
        clr1(cpu, bus, bit);
        return true;
    }
    // BBS dp.n,rel (opcodes 03, 23, 43, 63, 83, A3, C3, E3)
    if op & 0x1F == 0x03 {
        let bit = (op >> 5) as u8;
        bbs(cpu, bus, bit);
        return true;
    }
    // BBC dp.n,rel (opcodes 13, 33, 53, 73, 93, B3, D3, F3)
    if op & 0x1F == 0x13 {
        let bit = (op >> 5) as u8;
        bbc(cpu, bus, bit);
        return true;
    }
    // TSET1 !abs (opcode 0E)
    if op == 0x0E {
        tset1(cpu, bus);
        return true;
    }
    // TCLR1 !abs (opcode 4E)
    if op == 0x4E {
        tclr1(cpu, bus);
        return true;
    }
    // OR1 C,m.b (opcode 0A)
    if op == 0x0A {
        or1(cpu, bus, false);
        return true;
    }
    // OR1 C,/m.b (opcode 2A)
    if op == 0x2A {
        or1(cpu, bus, true);
        return true;
    }
    // AND1 C,m.b (opcode 4A)
    if op == 0x4A {
        and1(cpu, bus, false);
        return true;
    }
    // AND1 C,/m.b (opcode 6A)
    if op == 0x6A {
        and1(cpu, bus, true);
        return true;
    }
    // EOR1 C,m.b (opcode 8A)
    if op == 0x8A {
        eor1(cpu, bus);
        return true;
    }
    // MOV1 C,m.b (opcode AA)
    if op == 0xAA {
        mov1_to_c(cpu, bus);
        return true;
    }
    // MOV1 m.b,C (opcode CA)
    if op == 0xCA {
        mov1_to_mem(cpu, bus);
        return true;
    }
    // NOT1 m.b (opcode EA)
    if op == 0xEA {
        not1(cpu, bus);
        return true;
    }
    false
}
