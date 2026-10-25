use crate::cpu::table::{Am, Mn};
use crate::cpu::{Bus, Cpu};

pub(super) fn exec<B: Bus>(cpu: &mut Cpu, bus: &mut B, mn: Mn, am: Am) -> bool {
    match (mn, am) {
        (Mn::JMP, Am::Abs) => {
            let target = cpu.fetch16(bus);
            cpu.pc = target;
            true
        }
        (Mn::JML, Am::Long) => {
            let v = cpu.fetch24(bus);
            cpu.pb = (v >> 16) as u8;
            cpu.pc = v as u16;
            true
        }
        (Mn::JMP, Am::AbsInd) => {
            let a = cpu.fetch16(bus);
            // Read pointer from bank 0, with 16-bit wrap on high byte address
            let lo = bus.read(a as u32);
            let hi_addr = (a.wrapping_add(1)) as u32;
            let hi = bus.read(hi_addr);
            cpu.pc = ((hi as u16) << 8) | (lo as u16);
            true
        }
        (Mn::JML, Am::AbsIndLong) => {
            let a = cpu.fetch16(bus);
            // Read 3-byte pointer from bank 0, each address masked to 16 bits
            let lo = bus.read(a as u32);
            let mid_addr = (a.wrapping_add(1)) as u32;
            let mid = bus.read(mid_addr);
            let hi_addr = (a.wrapping_add(2)) as u32;
            let bank = bus.read(hi_addr);
            cpu.pc = ((mid as u16) << 8) | (lo as u16);
            cpu.pb = bank;
            true
        }
        (Mn::JMP, Am::AbsXInd) => {
            let a = cpu.fetch16(bus);
            bus.idle(); // extra cycle for indexed indirect
            let p = (a.wrapping_add(cpu.x)) & 0xFFFF;
            let addr_lo = ((cpu.pb as u32) << 16) | (p as u32);
            let lo = bus.read(addr_lo);
            let hi_addr = ((cpu.pb as u32) << 16) | (((p + 1) & 0xFFFF) as u32);
            let hi = bus.read(hi_addr);
            cpu.pc = ((hi as u16) << 8) | (lo as u16);
            true
        }
        _ => false,
    }
}
