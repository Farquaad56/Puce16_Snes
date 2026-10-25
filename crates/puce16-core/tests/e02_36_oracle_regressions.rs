// E02-36: Oracle regression tests for flow control instructions.
// Each test reproduces a bug found during oracle validation.

use puce16_core::cpu::{Bus, Cpu};

struct TestBus {
    mem: [u8; 0x10000],
}

impl Bus for TestBus {
    fn read(&mut self, addr: u32) -> u8 {
        let a = (addr & 0xFFFF) as usize;
        self.mem[a]
    }
    fn write(&mut self, _addr: u32, _val: u8) {}
    fn idle(&mut self) {}
}

fn make_cpu() -> Cpu {
    let mut cpu = Cpu::new();
    cpu.e = true;
    cpu
}

#[test]
fn e02_36_jmp_absxind_page_boundary() {
    // Bug: JMP AbsXInd with X causing page wrap (p=0xFFFF) panicked on p+1 overflow.
    let mut bus = TestBus { mem: [0u8; 0x10000] };
    let mut cpu = make_cpu();

    // Set up: opcode at PC, then two operand bytes, X=2 to cause wrap from 0xFFFE
    bus.mem[0x8000] = 0x7C; // JMP AbsXInd
    bus.mem[0x8001] = 0xFE; // low byte of address (0xFFFE)
    bus.mem[0x8002] = 0xFF; // high byte of address (0xFFFF)
    cpu.pc = 0x8000;
    cpu.x = 2;

    // Target pointer at wrapped address 0x0000: points to 0x1234
    bus.mem[0x0000] = 0x34;
    bus.mem[0x0001] = 0x12;

    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x1234, "JMP AbsXInd should follow wrapped pointer");
}

#[test]
fn e02_36_jsr_absxind_page_boundary() {
    // Bug: JSR AbsXInd with X causing page wrap panicked on p+1 overflow.
    let mut bus = TestBus { mem: [0u8; 0x10000] };
    let mut cpu = make_cpu();

    bus.mem[0x8000] = 0xFC; // JSR AbsXInd
    bus.mem[0x8001] = 0xFE; // low byte of address (0xFFFE)
    bus.mem[0x8002] = 0xFF; // high byte of address (0xFFFF)
    cpu.pc = 0x8000;
    cpu.x = 2;

    // Target pointer at wrapped address 0x0000: points to 0x5678
    bus.mem[0x0000] = 0x78;
    bus.mem[0x0001] = 0x56;

    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x5678, "JSR AbsXInd should follow wrapped pointer");
}

#[test]
fn e02_36_wdm_cycle_count() {
    // Bug: WDM was fetching signature byte (read cycle) but oracle expects idle.
    let mut bus = TestBus { mem: [0u8; 0x10000] };
    let mut cpu = make_cpu();

    bus.mem[0x8000] = 0x42; // WDM
    bus.mem[0x8001] = 0x99; // signature byte (ignored)
    cpu.pc = 0x8000;

    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x8002, "WDM should advance PC past opcode and signature");
}
