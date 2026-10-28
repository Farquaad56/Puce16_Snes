use super::{SmpBus, Spc700};

/// BPL rel - branch if positive (!N)
fn bpl<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    let rel = cpu.fetch8(bus);
    if !cpu.flag(super::F_N) {
        cpu.branch(rel);
    }
}

/// BMI rel - branch if minus (N)
fn bmi<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    let rel = cpu.fetch8(bus);
    if cpu.flag(super::F_N) {
        cpu.branch(rel);
    }
}

/// BVC rel - branch if no overflow (!V)
fn bvc<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    let rel = cpu.fetch8(bus);
    if !cpu.flag(super::F_V) {
        cpu.branch(rel);
    }
}

/// BVS rel - branch if overflow (V)
fn bvs<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    let rel = cpu.fetch8(bus);
    if cpu.flag(super::F_V) {
        cpu.branch(rel);
    }
}

/// BCC rel - branch if no carry (!C)
fn bcc<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    let rel = cpu.fetch8(bus);
    if !cpu.flag(super::F_C) {
        cpu.branch(rel);
    }
}

/// BCS rel - branch if carry (C)
fn bcs<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    let rel = cpu.fetch8(bus);
    if cpu.flag(super::F_C) {
        cpu.branch(rel);
    }
}

/// BNE rel - branch if not equal (!Z)
fn bne<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    let rel = cpu.fetch8(bus);
    if !cpu.flag(super::F_Z) {
        cpu.branch(rel);
    }
}

/// BEQ rel - branch if equal (Z)
fn beq<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    let rel = cpu.fetch8(bus);
    if cpu.flag(super::F_Z) {
        cpu.branch(rel);
    }
}

/// BRA rel - branch always
fn bra<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    let rel = cpu.fetch8(bus);
    cpu.branch(rel);
}

/// CBNE dp,rel - compare and branch if not equal (direct page)
fn cbne_dp<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    let b = cpu.fetch8(bus);
    let rel = cpu.fetch8(bus);
    let ea = cpu.dp() | (b as u16);
    let v = cpu.rd(bus, ea);
    if cpu.a != v {
        cpu.branch(rel);
    }
}

/// CBNE dp+X,rel - compare and branch if not equal (direct page + X)
fn cbne_dpx<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    let b = cpu.fetch8(bus);
    let rel = cpu.fetch8(bus);
    cpu.idle(bus); // index cycle
    let ea = cpu.dp() | (((b as u8).wrapping_add(cpu.x)) as u16);
    let v = cpu.rd(bus, ea);
    if cpu.a != v {
        cpu.branch(rel);
    }
}

/// DBNZ Y,rel - decrement Y and branch if not zero
fn dbnz_y<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    let rel = cpu.fetch8(bus);
    cpu.y = cpu.y.wrapping_sub(1);
    if cpu.y != 0 {
        cpu.branch(rel);
    }
}

/// DBNZ dp,rel - decrement memory at dp and branch if not zero
fn dbnz_dp<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    let b = cpu.fetch8(bus);
    let ea = cpu.dp() | (b as u16);
    let v = cpu.rd(bus, ea).wrapping_sub(1);
    cpu.wr(bus, ea, v);
    let rel = cpu.fetch8(bus);
    if v != 0 {
        cpu.branch(rel);
    }
}

/// JMP !abs - jump to absolute address
fn jmp_abs<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    cpu.pc = cpu.fetch16(bus);
}

/// JMP [!abs+X] - indirect jump through pointer at abs+X (wraps modulo 65536)
fn jmp_absx_ind<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    let abs = cpu.fetch16(bus);
    cpu.idle(bus); // index cycle
    let ea = (abs.wrapping_add(cpu.x as u16)) & 0xFFFF;
    // Read pointer bytes with wrap-around at page boundary
    let lo_addr = ea;
    let hi_addr = (ea + 1) & 0xFFFF;
    cpu.pc = cpu.rd16(bus, lo_addr, hi_addr);
}

/// CALL !abs - push return address and jump
fn call_abs<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    let target = cpu.fetch16(bus);
    cpu.idle(bus); // extra cycle for stack operations
    cpu.push8(bus, (cpu.pc >> 8) as u8);
    cpu.push8(bus, cpu.pc as u8);
    cpu.pc = target;
}

/// PCALL up - push return address and jump to $FF00|up
fn pcall_up<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    let n = cpu.fetch8(bus);
    cpu.push8(bus, (cpu.pc >> 8) as u8);
    cpu.push8(bus, cpu.pc as u8);
    cpu.pc = 0xFF00 | (n as u16);
}

/// TCALL n - push return address and jump to vector at $FFDE-2*n
fn tcall<B: SmpBus>(cpu: &mut Spc700, bus: &mut B, n: u8) {
    cpu.push8(bus, (cpu.pc >> 8) as u8);
    cpu.push8(bus, cpu.pc as u8);
    let vec_addr = 0xFFDE - (n as u16 * 2);
    cpu.pc = cpu.rd16(bus, vec_addr, vec_addr + 1);
}

/// BRK - push PC and PSW, set B=1 I=0, jump to vector at $FFDE
fn brk<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    cpu.push8(bus, (cpu.pc >> 8) as u8);
    cpu.push8(bus, cpu.pc as u8);
    cpu.push8(bus, cpu.psw);
    cpu.set_flag(super::F_B, true);
    cpu.set_flag(super::F_I, false);
    cpu.pc = cpu.rd16(bus, 0xFFDE, 0xFFDF);
}

/// RET - pop return address and jump
fn ret<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    let lo = cpu.pull8(bus);
    let hi = cpu.pull8(bus);
    cpu.pc = ((hi as u16) << 8) | (lo as u16);
}

/// RETI - pop PSW then return address and jump
fn reti<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) {
    cpu.psw = cpu.pull8(bus);
    let lo = cpu.pull8(bus);
    let hi = cpu.pull8(bus);
    cpu.pc = ((hi as u16) << 8) | (lo as u16);
}

pub(super) fn exec<B: SmpBus>(cpu: &mut Spc700, bus: &mut B, op: u8) -> bool {
    // Conditional branches (all take rel offset)
    match op {
        0x10 => bpl(cpu, bus),   // BPL rel
        0x30 => bmi(cpu, bus),   // BMI rel
        0x50 => bvc(cpu, bus),   // BVC rel
        0x70 => bvs(cpu, bus),   // BVS rel
        0x90 => bcc(cpu, bus),   // BCC rel
        0xB0 => bcs(cpu, bus),   // BCS rel
        0xD0 => bne(cpu, bus),   // BNE rel
        0xF0 => beq(cpu, bus),   // BEQ rel
        _ => {}
    }
    if op == 0x10 || op == 0x30 || op == 0x50 || op == 0x70
        || op == 0x90 || op == 0xB0 || op == 0xD0 || op == 0xF0 {
        return true;
    }

    // BRA rel (unconditional)
    if op == 0x2F {
        bra(cpu, bus);
        return true;
    }

    // CBNE dp,rel
    if op == 0x2E {
        cbne_dp(cpu, bus);
        return true;
    }

    // CBNE dp+X,rel
    if op == 0xDE {
        cbne_dpx(cpu, bus);
        return true;
    }

    // DBNZ Y,rel
    if op == 0xFE {
        dbnz_y(cpu, bus);
        return true;
    }

    // DBNZ dp,rel
    if op == 0x6E {
        dbnz_dp(cpu, bus);
        return true;
    }

    // JMP !abs
    if op == 0x5F {
        jmp_abs(cpu, bus);
        return true;
    }

    // JMP [!abs+X]
    if op == 0x1F {
        jmp_absx_ind(cpu, bus);
        return true;
    }

    // CALL !abs
    if op == 0x3F {
        call_abs(cpu, bus);
        return true;
    }

    // PCALL up
    if op == 0x4F {
        pcall_up(cpu, bus);
        return true;
    }

    // TCALL n (n = 0..15, opcode = n*0x10 + 0x01)
    if op & 0x0F == 0x01 {
        let n = (op >> 4) as u8;
        tcall(cpu, bus, n);
        return true;
    }

    // BRK
    if op == 0x0F {
        brk(cpu, bus);
        return true;
    }

    // RET
    if op == 0x6F {
        ret(cpu, bus);
        return true;
    }

    // RETI
    if op == 0x7F {
        reti(cpu, bus);
        return true;
    }

    false
}
