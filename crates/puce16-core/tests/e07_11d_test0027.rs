// E07-11d: gilyon cputest-full test 0027 = `adc ($F7,x)` with E=1, DL != 0.
// Undocumented behaviour: the dp+D+X sum does not wrap, but the +1 for the pointer
// high byte wraps inside the page ($02FF -> $0200).

use puce16_core::cpu::testbus::FlatBus;
use puce16_core::cpu::Cpu;

fn run_adc_x_ind(dp: u8, x: u16, d: u16, setup: impl Fn(&mut FlatBus)) -> Cpu {
    let mut cpu = Cpu::new();
    cpu.e = true;
    cpu.p = 0x31; // M, X, C (emulation mode)
    cpu.a = 0x1112;
    cpu.x = x;
    cpu.y = 0x0078;
    cpu.d = d;
    cpu.db = 0x7F;
    cpu.pb = 0x00;
    cpu.pc = 0x8000;
    cpu.s = 0x01EF;
    let mut bus = FlatBus::new();
    bus.poke(0x8000, &[0x61, dp]); // ADC (dp,X)
    setup(&mut bus);
    cpu.step(&mut bus);
    cpu
}

#[test]
fn e07_11d_test0027_regression() {
    let cpu = run_adc_x_ind(0xF7, 0x00EE, 0x011A, |bus| {
        bus.poke(0x7F1234, &[0xED]);
        bus.poke(0x0002FF, &[0x34]); // pointer low
        bus.poke(0x000200, &[0x12]); // pointer high (page wrap of the +1)
        bus.poke(0x000300, &[0x55]); // would be used without the wrap
    });
    assert_eq!(cpu.a, 0x1100, "A");
    assert_eq!(cpu.x, 0x00EE, "X");
    assert_eq!(cpu.y, 0x0078, "Y");
    assert_eq!(cpu.p, 0x33, "P");
    assert_eq!(cpu.s, 0x01EF, "S");
}

/// Test 0026 neighbour: ($F6,x), sum $02FE, pointer at $02FE/$02FF, no page crossing.
#[test]
fn e07_11d_test0026_no_wrap_needed() {
    let cpu = run_adc_x_ind(0xF6, 0x00EE, 0x011A, |bus| {
        bus.poke(0x7F1234, &[0xED]);
        bus.poke(0x0002FE, &[0x34, 0x12]);
    });
    assert_eq!(cpu.a, 0x1100, "A");
    assert_eq!(cpu.p, 0x33, "P");
}
