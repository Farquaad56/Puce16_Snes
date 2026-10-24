mod branch;
mod jump;
mod call;
mod sys;

use crate::cpu::table::{Am, Mn};
use crate::cpu::{Bus, Cpu};

pub(crate) fn exec<B: Bus>(cpu: &mut Cpu, bus: &mut B, mn: Mn, am: Am) -> bool {
    branch::exec(cpu, bus, mn, am) || jump::exec(cpu, bus, mn, am)
        || call::exec(cpu, bus, mn, am) || sys::exec(cpu, bus, mn, am)
}
