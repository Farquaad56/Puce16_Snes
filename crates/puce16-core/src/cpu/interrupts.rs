use crate::cpu::{Bus, Cpu, FLAG_D, FLAG_I};

/// Poll for pending interrupts at instruction boundary. Returns true if an interrupt was taken.
pub fn poll<B: Bus>(cpu: &mut Cpu, bus: &mut B) -> bool {
    // 1. If halted (STP): idle and return true
    if cpu.halted {
        bus.idle();
        return true;
    }

    // 2. Check NMI/IRQ lines
    let nmi = bus.nmi_pending();
    let irq = bus.irq_asserted() && !cpu.flag(FLAG_I);

    // 3. If waiting (WAI)
    if cpu.waiting {
        if nmi || bus.irq_asserted() {
            cpu.waiting = false;
        } else {
            bus.idle();
            return true;
        }
    }

    // 4. NMI has priority over IRQ
    if nmi {
        service(cpu, bus, true);
        return true;
    }

    // 5. IRQ
    if irq {
        service(cpu, bus, false);
        return true;
    }

    // 6. No interrupt
    false
}

/// Service an interrupt: push PC and P, read vector, jump to handler.
fn service<B: Bus>(cpu: &mut Cpu, bus: &mut B, nmi: bool) {
    // Phantom opcode fetch (ignored)
    let _ = bus.read(cpu.pc24());
    // IO cycle
    bus.idle();

    if !cpu.e {
        // Native mode: push PB, PCH, PCL, P
        cpu.push8(bus, cpu.pb);
        cpu.push8(bus, (cpu.pc >> 8) as u8);
        cpu.push8(bus, (cpu.pc & 0xFF) as u8);
        cpu.push8(bus, cpu.p);
    } else {
        // Emulation mode: push PCH, PCL, P (with B=0 forced)
        cpu.push8(bus, (cpu.pc >> 8) as u8);
        cpu.push8(bus, (cpu.pc & 0xFF) as u8);
        let p_pushed = (cpu.p | 0x20) & !0x10; // B=0 in emulation
        cpu.push8(bus, p_pushed);
    }

    // Read vector address based on mode and interrupt type
    let vec_addr: u32;
    if nmi {
        vec_addr = if cpu.e { 0xFFFA } else { 0xFFEA };
    } else {
        vec_addr = if cpu.e { 0xFFFE } else { 0xFFEE };
    }

    let lo = bus.read(vec_addr);
    let hi = bus.read(vec_addr + 1);

    // Set flags: I=1, D=0, PB=0
    cpu.set_flag(FLAG_I, true);
    cpu.set_flag(FLAG_D, false);
    cpu.pb = 0;
    cpu.pc = ((hi as u16) << 8) | (lo as u16);
}

/// Reset sequence.
pub fn reset<B: Bus>(cpu: &mut Cpu, bus: &mut B) {
    // Set to emulation mode with default flags
    cpu.e = true;
    cpu.p = 0x34; // M=1, X=1, I=1 (D=0 and rest 0)
    cpu.s = 0x01FF;
    cpu.db = 0;
    cpu.pb = 0;
    cpu.d = 0;
    cpu.x &= 0xFF;
    cpu.y &= 0xFF;
    cpu.halted = false;
    cpu.waiting = false;

    // RESET sequence: 5 internal cycles + 2 vector reads = 7 cycles (master-clock alignment is handled by bus timing)
    bus.idle();
    bus.idle();
    bus.idle();
    bus.idle();
    bus.idle();

    // Read reset vector from $FFFC/$FFFD
    let lo = bus.read(0xFFFC);
    let hi = bus.read(0xFFFD);
    cpu.pc = ((hi as u16) << 8) | (lo as u16);
}
