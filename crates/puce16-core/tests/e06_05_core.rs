use puce16_core::apu::spc700::{Spc700, SmpBus, F_C, F_P, F_Z, F_N};

struct TB {
    ram: Vec<u8>,
}

impl TB {
    fn new() -> Self {
        TB { ram: vec![0u8; 65536] }
    }
}

impl SmpBus for TB {
    fn read(&mut self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }
    fn write(&mut self, addr: u16, v: u8) {
        self.ram[addr as usize] = v;
    }
    fn idle(&mut self) {}
}

#[test]
fn e06_05_new_defaults() {
    let cpu = Spc700::new();
    assert_eq!(cpu.a, 0);
    assert_eq!(cpu.x, 0);
    assert_eq!(cpu.y, 0);
    assert_eq!(cpu.sp, 0);
    assert_eq!(cpu.pc, 0);
    assert_eq!(cpu.psw, 0x02);
    assert!(!cpu.halted);
}

#[test]
fn e06_05_fetch_order_and_wrap() {
    let mut bus = TB::new();
    let mut cpu = Spc700::new();
    // fetch16 little-endian: low byte first at pc, high at pc+1
    cpu.pc = 0x0300;
    bus.ram[0x0300 as usize] = 0x34;
    bus.ram[0x0301 as usize] = 0x12;
    let v = cpu.fetch16(&mut bus);
    assert_eq!(v, 0x1234);
    assert_eq!(cpu.pc, 0x0302);
    // fetch8 at 0xFFFF wraps pc to 0x0000
    cpu.pc = 0xFFFF;
    bus.ram[0xFFFF as usize] = 0xAA;
    let b = cpu.fetch8(&mut bus);
    assert_eq!(b, 0xAA);
    assert_eq!(cpu.pc, 0x0000);
}

#[test]
fn e06_05_push_pull() {
    let mut bus = TB::new();
    let mut cpu = Spc700::new();
    // push at sp=0xEF writes to 0x01EF, then sp becomes 0xEE
    cpu.sp = 0xEF;
    cpu.push8(&mut bus, 0xAB);
    assert_eq!(bus.ram[0x01EF as usize], 0xAB);
    assert_eq!(cpu.sp, 0xEE);
    // pull increments sp to 0xEF then reads from 0x01EF
    let v = cpu.pull8(&mut bus);
    assert_eq!(v, 0xAB);
    assert_eq!(cpu.sp, 0xEF);
    // wrap: push at sp=0x00 writes to 0x0100, then sp becomes 0xFF
    cpu.sp = 0x00;
    cpu.push8(&mut bus, 0xCD);
    assert_eq!(bus.ram[0x0100 as usize], 0xCD);
    assert_eq!(cpu.sp, 0xFF);
}

#[test]
fn e06_05_dp_base() {
    let mut cpu = Spc700::new();
    // P flag clear -> dp base is 0x0000
    cpu.psw = 0x00;
    assert_eq!(cpu.dp(), 0x0000);
    // P flag set -> dp base is 0x0100
    cpu.set_flag(F_P, true);
    assert_eq!(cpu.dp(), 0x0100);
}

#[test]
fn e06_05_flags_and_nz() {
    let mut cpu = Spc700::new();
    // start with psw=0x02 (Z set)
    assert_eq!(cpu.psw, 0x02);
    // set C flag
    cpu.set_flag(F_C, true);
    assert_eq!(cpu.psw, 0x03);
    // clear C flag
    cpu.set_flag(F_C, false);
    assert_eq!(cpu.psw, 0x02);
    // set_nz with high bit -> N=1, Z=0
    cpu.set_nz(0x80);
    assert!(cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
    // set_nz with zero -> Z=1, N=0
    cpu.set_nz(0);
    assert!(cpu.flag(F_Z));
    assert!(!cpu.flag(F_N));
    // set_nz with 0x7F -> N=0, Z=0
    cpu.set_nz(0x7F);
    assert!(!cpu.flag(F_N));
    assert!(!cpu.flag(F_Z));
}

#[test]
fn e06_05_ya_pair() {
    let mut cpu = Spc700::new();
    cpu.y = 0x12;
    cpu.a = 0x34;
    assert_eq!(cpu.ya(), 0x1234);
    cpu.set_ya(0xABCD);
    assert_eq!(cpu.y, 0xAB);
    assert_eq!(cpu.a, 0xCD);
}

#[test]
fn e06_05_rd16_order() {
    let mut bus = TB::new();
    let mut cpu = Spc700::new();
    // rd16 reads low byte first at lo_addr, then high at hi_addr
    bus.ram[0x10 as usize] = 0x34;
    bus.ram[0x11 as usize] = 0x12;
    let v = cpu.rd16(&mut bus, 0x10, 0x11);
    assert_eq!(v, 0x1234);
    // wrap case: lo at 0xFF, hi at 0x00
    bus.ram[0xFF as usize] = 0x78;
    bus.ram[0x00 as usize] = 0x56;
    let v2 = cpu.rd16(&mut bus, 0xFF, 0x00);
    assert_eq!(v2, 0x5678);
}

#[test]
fn e06_05_branch_forward_backward() {
    let _bus = TB::new();
    let mut cpu = Spc700::new();
    // forward branch: pc=0x0210, rel=+5 -> 0x0215
    cpu.pc = 0x0210;
    cpu.branch(0x05);
    assert_eq!(cpu.pc, 0x0215);
    // backward branch: pc=0x0215, rel=-5 (0xFB) -> 0x0210
    cpu.branch(0xFB);
    assert_eq!(cpu.pc, 0x0210);
    // wrap: pc=0x0000, rel=-1 (0xFF) -> 0xFFFF
    cpu.pc = 0x0000;
    cpu.branch(0xFF);
    assert_eq!(cpu.pc, 0xFFFF);
}

#[test]
fn e06_05_reset_vector() {
    let mut bus = TB::new();
    let mut cpu = Spc700::new();
    // reset vector at $FFFE/$FFFF points to $FFC0
    bus.ram[0xFFFE as usize] = 0xC0;
    bus.ram[0xFFFF as usize] = 0xFF;
    cpu.reset(&mut bus);
    assert_eq!(cpu.pc, 0xFFC0);
    assert_eq!(cpu.a, 0);
    assert_eq!(cpu.x, 0);
    assert_eq!(cpu.y, 0);
    assert_eq!(cpu.sp, 0);
    assert_eq!(cpu.psw, 0x02);
    assert!(!cpu.halted);
}

#[test]
fn e06_05_halted_step() {
    let mut bus = TB::new();
    let mut cpu = Spc700::new();
    cpu.halted = true;
    cpu.pc = 0x1234;
    let cycles = cpu.step(&mut bus);
    assert_eq!(cycles, 1);
    assert_eq!(cpu.pc, 0x1234); // pc unchanged when halted
}

#[test]
#[should_panic(expected = "unimplemented opcode 00")]
fn e06_05_unimplemented_panics() {
    let mut bus = TB::new();
    let mut cpu = Spc700::new();
    // opcode 0x00 (NOP) is not implemented yet at this stage
    cpu.pc = 0x0200;
    bus.ram[0x0200 as usize] = 0x00;
    cpu.step(&mut bus);
}
