use crate::cpu::table::{Am, Mn};
use crate::cpu::{Bus, Cpu};

pub(super) fn exec<B: Bus>(cpu: &mut Cpu, bus: &mut B, mn: Mn, am: Am) -> bool {
    match (mn, am) {
        // JSR Abs - old stack, push PC-1 high then low
        (Mn::JSR, Am::Abs) => {
            let target = cpu.fetch16(bus);
            bus.idle();
            let pc_push = cpu.pc.wrapping_sub(1);
            cpu.push8(bus, ((pc_push >> 8) & 0xFF) as u8);
            cpu.push8(bus, (pc_push & 0xFF) as u8);
            cpu.pc = target;
        }
        // JSR AbsXInd - new stack, push PC before reading aah
        (Mn::JSR, Am::AbsXInd) => {
            let aal = cpu.fetch8(bus);
            cpu.push8n(bus, ((cpu.pc >> 8) & 0xFF) as u8);
            cpu.push8n(bus, (cpu.pc & 0xFF) as u8);
            let aah = cpu.fetch8(bus);
            bus.idle();
            let p = (aah as u16).wrapping_mul(256).wrapping_add(aal as u16).wrapping_add(cpu.x);
            let addr_lo = ((cpu.pb as u32) << 16) | (p as u32);
            let lo = bus.read(addr_lo);
            let hi_addr = ((cpu.pb as u32) << 16) | (p.wrapping_add(1) as u32);
            let hi = bus.read(hi_addr);
            cpu.pc = ((hi as u16) << 8) | (lo as u16);
            cpu.fix_stack_e();
        }
        // JSL Long - new stack, push PB then PC-1 high/low
        (Mn::JSL, Am::Long) => {
            let aal = cpu.fetch8(bus);
            let aah = cpu.fetch8(bus);
            cpu.push8n(bus, cpu.pb);
            bus.idle();
            let bank = cpu.fetch8(bus);
            let pc_push = cpu.pc.wrapping_sub(1);
            cpu.push8n(bus, ((pc_push >> 8) & 0xFF) as u8);
            cpu.push8n(bus, (pc_push & 0xFF) as u8);
            cpu.pb = bank;
            cpu.pc = ((aah as u16) << 8) | (aal as u16);
            cpu.fix_stack_e();
        }
        // RTS - old stack, pull low then high, add 1
        (Mn::RTS, Am::Imp) => {
            bus.idle();
            bus.idle();
            let lo = cpu.pull8(bus);
            let hi = cpu.pull8(bus);
            bus.idle();
            let addr = ((hi as u16) << 8) | (lo as u16);
            cpu.pc = addr.wrapping_add(1);
        }
        // RTL - new stack, pull low/high/bank, restore PB and PC+1
        (Mn::RTL, Am::Imp) => {
            bus.idle();
            bus.idle();
            let lo = cpu.pull8n(bus);
            let hi = cpu.pull8n(bus);
            let bank = cpu.pull8n(bus);
            cpu.pb = bank;
            let addr = ((hi as u16) << 8) | (lo as u16);
            cpu.pc = addr.wrapping_add(1);
            cpu.fix_stack_e();
        }
        _ => return false,
    }
    true
}
