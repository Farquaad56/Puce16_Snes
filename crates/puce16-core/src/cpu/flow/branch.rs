use crate::cpu::table::{Am, Mn};
use crate::cpu::{Bus, Cpu, FLAG_C, FLAG_N, FLAG_V, FLAG_Z};

pub(super) fn exec<B: Bus>(cpu: &mut Cpu, bus: &mut B, mn: Mn, am: Am) -> bool {
    match (mn, am) {
        (Mn::BCC | Mn::BCS | Mn::BEQ | Mn::BNE | Mn::BMI | Mn::BPL | Mn::BVC | Mn::BVS | Mn::BRA, Am::Rel) => {
            branch_rel(cpu, bus, mn);
            true
        }
        (Mn::BRL, Am::RelL) => {
            branch_long(cpu, bus);
            true
        }
        _ => false,
    }
}

fn condition_met(cpu: &Cpu, mn: Mn) -> bool {
    match mn {
        Mn::BCC => !cpu.flag(FLAG_C),
        Mn::BCS => cpu.flag(FLAG_C),
        Mn::BEQ => cpu.flag(FLAG_Z),
        Mn::BNE => !cpu.flag(FLAG_Z),
        Mn::BMI => cpu.flag(FLAG_N),
        Mn::BPL => !cpu.flag(FLAG_N),
        Mn::BVC => !cpu.flag(FLAG_V),
        Mn::BVS => cpu.flag(FLAG_V),
        Mn::BRA => true,
        _ => false,
    }
}

fn branch_rel<B: Bus>(cpu: &mut Cpu, bus: &mut B, mn: Mn) {
    let off = cpu.fetch8(bus) as i8;
    if !condition_met(cpu, mn) {
        return; // not taken: 2 cycles (opcode + operand)
    }
    let target = cpu.pc.wrapping_add(off as i16 as u16);
    bus.idle(); // branch taken cycle
    if cpu.e && ((cpu.pc & 0xFF00) != (target & 0xFF00)) {
        bus.idle(); // page crossing penalty in emulation mode
    }
    cpu.pc = target;
}

fn branch_long<B: Bus>(cpu: &mut Cpu, bus: &mut B) {
    let lo = cpu.fetch8(bus);
    let hi = cpu.fetch8(bus);
    let off16 = ((hi as i16) << 8) | (lo as i16);
    bus.idle(); // branch taken cycle
    cpu.pc = cpu.pc.wrapping_add(off16 as u16);
}
