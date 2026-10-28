use super::{Spc700, SmpBus};

/// Direct page: P | b
pub fn ea_dp<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) -> u16 {
    let b = cpu.fetch8(bus);
    cpu.dp() | (b as u16)
}

/// Direct page + X: P | ((b + X) & 0xFF), with idle cycle
pub fn ea_dpx<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) -> u16 {
    let b = cpu.fetch8(bus);
    cpu.idle(bus);
    cpu.dp() | ((b.wrapping_add(cpu.x)) as u16)
}

/// Direct page + Y: P | ((b + Y) & 0xFF), with idle cycle
pub fn ea_dpy<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) -> u16 {
    let b = cpu.fetch8(bus);
    cpu.idle(bus);
    cpu.dp() | ((b.wrapping_add(cpu.y)) as u16)
}

/// Absolute: fetch 16-bit address
pub fn ea_abs<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) -> u16 {
    cpu.fetch16(bus)
}

/// Absolute + X: (abs + X) & 0xFFFF, with idle cycle
pub fn ea_absx<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) -> u16 {
    let abs = cpu.fetch16(bus);
    cpu.idle(bus);
    (abs as i32 + cpu.x as i32) as u16
}

/// Absolute + Y: (abs + Y) & 0xFFFF, with idle cycle
pub fn ea_absy<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) -> u16 {
    let abs = cpu.fetch16(bus);
    cpu.idle(bus);
    (abs as i32 + cpu.y as i32) as u16
}

/// Indirect X: P | X, with idle cycle
pub fn ea_x<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) -> u16 {
    cpu.idle(bus);
    cpu.dp() | (cpu.x as u16)
}

/// Indirect Y: P | Y, with idle cycle
pub fn ea_y<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) -> u16 {
    cpu.idle(bus);
    cpu.dp() | (cpu.y as u16)
}

/// Indirect via DP+X: read pointer at P|((b+X)&0xFF), with idle cycle before pointer read
pub fn ea_dpx_ind<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) -> u16 {
    let b = cpu.fetch8(bus);
    cpu.idle(bus);
    let lo_addr = cpu.dp() | ((b.wrapping_add(cpu.x)) as u16);
    let hi_addr = cpu.dp() | ((b.wrapping_add(cpu.x).wrapping_add(1)) as u16);
    cpu.rd16(bus, lo_addr, hi_addr)
}

/// Indirect via DP + Y: read pointer at P|b, add Y to result, with idle cycle after pointer read
pub fn ea_dp_ind_y<B: SmpBus>(cpu: &mut Spc700, bus: &mut B) -> u16 {
    let b = cpu.fetch8(bus);
    let lo_addr = cpu.dp() | (b as u16);
    let hi_addr = cpu.dp() | ((b.wrapping_add(1)) as u16);
    cpu.idle(bus);
    let ptr = cpu.rd16(bus, lo_addr, hi_addr);
    (ptr as i32 + cpu.y as i32) as u16
}

