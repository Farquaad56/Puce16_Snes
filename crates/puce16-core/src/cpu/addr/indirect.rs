// Indirect addressing modes: DpInd, DpIndLong, DpXInd, DpIndY, DpIndLongY, SrIndY

use super::super::{Bus, Cpu};
use crate::cpu::table::Am;
use crate::cpu::addr::Ea;

pub(super) fn ea<B: Bus>(cpu: &mut Cpu, bus: &mut B, am: Am, store: bool) -> Ea {
    unimplemented!("E02-xx");
}
