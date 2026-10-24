mod transfer;
mod flags;

use crate::cpu::table::{Am, Mn};
use crate::cpu::{Bus, Cpu};

pub(crate) fn exec<B: Bus>(cpu: &mut Cpu, bus: &mut B, mn: Mn, am: Am) -> bool {
    transfer::exec(cpu, bus, mn, am) || flags::exec(cpu, bus, mn, am)
}
