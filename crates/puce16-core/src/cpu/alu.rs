mod adc;
mod sbc;
mod logic;
mod cmp;
mod incdec;
mod shift;
mod tsbtrb;

use crate::cpu::table::{Am, Mn};
use crate::cpu::{Bus, Cpu};

pub(crate) fn exec<B: Bus>(cpu: &mut Cpu, bus: &mut B, mn: Mn, am: Am) -> bool {
    adc::exec(cpu, bus, mn, am) || sbc::exec(cpu, bus, mn, am)
        || logic::exec(cpu, bus, mn, am) || cmp::exec(cpu, bus, mn, am)
        || incdec::exec(cpu, bus, mn, am) || shift::exec(cpu, bus, mn, am)
        || tsbtrb::exec(cpu, bus, mn, am)
}
