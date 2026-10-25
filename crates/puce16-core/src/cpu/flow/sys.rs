use crate::cpu::table::{Am, Mn};
use crate::cpu::{Bus, Cpu, FLAG_I, FLAG_D};

pub(super) fn exec<B: Bus>(cpu: &mut Cpu, bus: &mut B, mn: Mn, am: Am) -> bool {
    match (mn, am) {
        // BRK - software interrupt with signature byte
        (Mn::BRK, Am::Brk) => {
            let _sig = cpu.fetch8(bus);
            if !cpu.e {
                cpu.push8n(bus, cpu.pb);
            }
            cpu.push8(bus, ((cpu.pc >> 8) & 0xFF) as u8);
            cpu.push8(bus, (cpu.pc & 0xFF) as u8);
            let p_pushed = if cpu.e { cpu.p | 0x30 } else { cpu.p };
            cpu.push8(bus, p_pushed);
            let vec = if cpu.e { 0xFFFE } else { 0xFFE6 };
            let lo = bus.read(vec as u32);
            let hi = bus.read((vec + 1) as u32);
            cpu.set_flag(FLAG_I, true);
            cpu.set_flag(FLAG_D, false);
            cpu.pb = 0;
            cpu.pc = ((hi as u16) << 8) | (lo as u16);
        }
        // COP - coprocessor instruction with signature byte
        (Mn::COP, Am::Cop) => {
            let _sig = cpu.fetch8(bus);
            if !cpu.e {
                cpu.push8n(bus, cpu.pb);
            }
            cpu.push8(bus, ((cpu.pc >> 8) & 0xFF) as u8);
            cpu.push8(bus, (cpu.pc & 0xFF) as u8);
            let p_pushed = if cpu.e { cpu.p | 0x30 } else { cpu.p };
            cpu.push8(bus, p_pushed);
            let vec = if cpu.e { 0xFFF4 } else { 0xFFE4 };
            let lo = bus.read(vec as u32);
            let hi = bus.read((vec + 1) as u32);
            cpu.set_flag(FLAG_I, true);
            cpu.set_flag(FLAG_D, false);
            cpu.pb = 0;
            cpu.pc = ((hi as u16) << 8) | (lo as u16);
        }
        // RTI - return from interrupt
        (Mn::RTI, Am::Imp) => {
            bus.idle();
            bus.idle();
            let p = cpu.pull8(bus);
            cpu.set_p(p);
            let lo = cpu.pull8(bus);
            let hi = cpu.pull8(bus);
            if !cpu.e {
                let bank = cpu.pull8n(bus);
                cpu.pb = bank;
            }
            cpu.pc = ((hi as u16) << 8) | (lo as u16);
        }
        // WAI - wait for interrupt
        (Mn::WAI, Am::Imp) => {
            bus.idle();
            bus.idle();
            cpu.waiting = true;
        }
        // STP - stop CPU until reset
        (Mn::STP, Am::Imp) => {
            bus.idle();
            bus.idle();
            cpu.halted = true;
        }
        // WDM - wait for data match (signature byte ignored)
        (Mn::WDM, Am::Wdm) => {
            bus.idle();
            cpu.pc = cpu.pc.wrapping_add(1);
        }
        _ => return false,
    }
    true
}
