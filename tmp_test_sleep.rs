use puce16_core::apu::spc700::{Spc700, testbus::FlatSmpBus};

fn main() {
    let mut cpu = Spc700::new();
    let mut bus = FlatSmpBus::new();
    cpu.pc = 0x0200;
    bus.poke(0x0200, &[0xEF]); // SLEEP
    
    println!("Before step: halted={}, cyc={}", cpu.halted, cpu.cyc);
    let cyc = cpu.step(&mut bus);
    println!("After step: halted={}, pc={}, returned cycles={}", cpu.halted, cpu.pc, cyc);
    
    // Second step (should return 1)
    let cyc2 = cpu.step(&mut bus);
    println!("Second step: returned cycles={}, pc={}", cyc2, cpu.pc);
}
