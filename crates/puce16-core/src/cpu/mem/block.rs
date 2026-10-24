use crate::cpu::table::{Am, Mn};
use crate::cpu::{Bus, Cpu};
pub(super) fn exec<B: Bus>(_cpu: &mut Cpu, _bus: &mut B, _mn: Mn, _am: Am) -> bool { false }
