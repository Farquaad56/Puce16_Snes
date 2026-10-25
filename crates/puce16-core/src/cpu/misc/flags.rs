use crate::cpu::table::{Am, Mn};
use crate::cpu::{Bus, Cpu, FLAG_C, FLAG_D, FLAG_I, FLAG_V};

pub(super) fn exec<B: Bus>(cpu: &mut Cpu, bus: &mut B, mn: Mn, _am: Am) -> bool {
    match mn {
        // CLC: Clear Carry Flag - 2 cycles
        Mn::CLC => {
            cpu.set_flag(FLAG_C, false);
        }
        // SEC: Set Carry Flag - 2 cycles
        Mn::SEC => {
            cpu.set_flag(FLAG_C, true);
        }
        // CLI: Clear Interrupt Disable - 2 cycles
        Mn::CLI => {
            cpu.set_flag(FLAG_I, false);
        }
        // SEI: Set Interrupt Disable - 2 cycles
        Mn::SEI => {
            cpu.set_flag(FLAG_I, true);
        }
        // CLD: Clear Decimal Mode - 2 cycles
        Mn::CLD => {
            cpu.set_flag(FLAG_D, false);
        }
        // SED: Set Decimal Mode - 2 cycles
        Mn::SED => {
            cpu.set_flag(FLAG_D, true);
        }
        // CLV: Clear Overflow Flag - 2 cycles
        Mn::CLV => {
            cpu.set_flag(FLAG_V, false);
        }
        // REP: Reset Processor Flags - 3 cycles
        Mn::REP => {
            let m = cpu.fetch8(bus);
            cpu.set_p(cpu.p & !m);
        }
        // SEP: Set Processor Flags - 3 cycles
        Mn::SEP => {
            let m = cpu.fetch8(bus);
            cpu.set_p(cpu.p | m);
        }
        // XCE: Exchange Carry and Emulation Mode - 2 cycles
        Mn::XCE => {
            let old_e = cpu.e;
            cpu.e = cpu.flag(FLAG_C);
            cpu.set_flag(FLAG_C, old_e);
            if cpu.e {
                cpu.set_p(cpu.p); // forces M=X=1 and clears high bytes of X/Y
                cpu.s = 0x0100 | (cpu.s & 0xFF);
            }
        }
        // XBA: Exchange Accumulator Bytes - 3 cycles
        Mn::XBA => {
            bus.idle();
            let a = cpu.a;
            cpu.a = ((a << 8) | (a >> 8)) as u16;
            cpu.set_nz8(cpu.a as u8);
        }
        // NOP: No Operation - 2 cycles
        Mn::NOP => {}
        _ => return false,
    }
    bus.idle();
    true
}
