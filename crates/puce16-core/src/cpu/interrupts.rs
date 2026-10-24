use crate::cpu::{Bus, Cpu};

/// Poll for pending interrupts at instruction boundary. Returns true if an interrupt was taken.
pub fn poll<B: Bus>(_cpu: &mut Cpu, _bus: &mut B) -> bool { false }

/// Reset sequence (provisional).
pub fn reset<B: Bus>(cpu: &mut Cpu, _bus: &mut B) { cpu.e = true; }
