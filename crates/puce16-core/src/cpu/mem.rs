mod load;
mod store;
mod push;
mod pull;
mod block;

use crate::cpu::table::{Am, Mn};
use crate::cpu::{Bus, Cpu};

pub(crate) fn exec<B: Bus>(cpu: &mut Cpu, bus: &mut B, mn: Mn, am: Am) -> bool {
    load::exec(cpu, bus, mn, am) || store::exec(cpu, bus, mn, am)
        || push::exec(cpu, bus, mn, am) || pull::exec(cpu, bus, mn, am)
        || block::exec(cpu, bus, mn, am)
}
