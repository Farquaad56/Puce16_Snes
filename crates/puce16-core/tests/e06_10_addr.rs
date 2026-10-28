use puce16_core::apu::spc700::{addr, Spc700, testbus::FlatSmpBus};

fn make_cpu() -> Spc700 {
    let mut cpu = Spc700::new();
    cpu.pc = 0x0200;
    cpu
}

#[test]
fn e06_10_ea_dp() {
    let mut cpu = make_cpu();
    let mut bus = FlatSmpBus::new();
    bus.poke(0x0200, &[0x34]);
    let addr_result = addr::ea_dp(&mut cpu, &mut bus);
    assert_eq!(addr_result, 0x0034);
    assert_eq!(cpu.pc, 0x0201);
    assert_eq!(bus.log.len(), 1);

    // With P flag set (page 1)
    let mut cpu2 = make_cpu();
    cpu2.psw |= 0x20; // F_P
    let mut bus2 = FlatSmpBus::new();
    bus2.poke(0x0200, &[0x34]);
    let addr_result2 = addr::ea_dp(&mut cpu2, &mut bus2);
    assert_eq!(addr_result2, 0x0134);
}

#[test]
fn e06_10_ea_dpx_wrap() {
    let mut cpu = make_cpu();
    cpu.x = 3;
    let mut bus = FlatSmpBus::new();
    bus.poke(0x0200, &[0xFE]);
    let addr_result = addr::ea_dpx(&mut cpu, &mut bus);
    assert_eq!(addr_result, 0x0001); // (0xFE + 3) & 0xFF = 0x01
    assert_eq!(bus.log.len(), 2);

    // With P flag set
    let mut cpu2 = make_cpu();
    cpu2.x = 3;
    cpu2.psw |= 0x20;
    let mut bus2 = FlatSmpBus::new();
    bus2.poke(0x0200, &[0xFE]);
    let addr_result2 = addr::ea_dpx(&mut cpu2, &mut bus2);
    assert_eq!(addr_result2, 0x0101);
}

#[test]
fn e06_10_ea_dpy_wrap() {
    let mut cpu = make_cpu();
    cpu.y = 0x10;
    let mut bus = FlatSmpBus::new();
    bus.poke(0x0200, &[0xF8]);
    let addr_result = addr::ea_dpy(&mut cpu, &mut bus);
    assert_eq!(addr_result, 0x0008); // (0xF8 + 0x10) & 0xFF = 0x08
}

#[test]
fn e06_10_ea_abs() {
    let mut cpu = make_cpu();
    let mut bus = FlatSmpBus::new();
    bus.poke(0x0200, &[0x34, 0x12]);
    let addr_result = addr::ea_abs(&mut cpu, &mut bus);
    assert_eq!(addr_result, 0x1234);
    assert_eq!(cpu.pc, 0x0202);
    assert_eq!(bus.log.len(), 2);
}

#[test]
fn e06_10_ea_absx_wrap() {
    let mut cpu = make_cpu();
    cpu.x = 2;
    let mut bus = FlatSmpBus::new();
    bus.poke(0x0200, &[0xFF, 0xFF]);
    let addr_result = addr::ea_absx(&mut cpu, &mut bus);
    assert_eq!(addr_result, 0x0001); // (0xFFFF + 2) & 0xFFFF = 0x0001
    assert_eq!(bus.log.len(), 3);
}

#[test]
fn e06_10_ea_absy() {
    let mut cpu = make_cpu();
    cpu.y = 5;
    let mut bus = FlatSmpBus::new();
    bus.poke(0x0200, &[0x00, 0x20]);
    let addr_result = addr::ea_absy(&mut cpu, &mut bus);
    assert_eq!(addr_result, 0x2005);
}

#[test]
fn e06_10_ea_x_y() {
    // ea_x
    let mut cpu = make_cpu();
    cpu.x = 7;
    let pc_before = cpu.pc;
    let mut bus = FlatSmpBus::new();
    let addr_result = addr::ea_x(&mut cpu, &mut bus);
    assert_eq!(addr_result, 0x0007);
    assert_eq!(cpu.pc, pc_before); // PC unchanged
    assert_eq!(bus.log.len(), 1);

    // ea_x with P flag
    let mut cpu2 = make_cpu();
    cpu2.x = 7;
    cpu2.psw |= 0x20;
    let mut bus2 = FlatSmpBus::new();
    let addr_result2 = addr::ea_x(&mut cpu2, &mut bus2);
    assert_eq!(addr_result2, 0x0107);

    // ea_y
    let mut cpu3 = make_cpu();
    cpu3.y = 9;
    let mut bus3 = FlatSmpBus::new();
    let addr_result3 = addr::ea_y(&mut cpu3, &mut bus3);
    assert_eq!(addr_result3, 0x0009);
}

#[test]
fn e06_10_ea_dpx_ind() {
    let mut cpu = make_cpu();
    cpu.x = 2;
    let mut bus = FlatSmpBus::new();
    bus.poke(0x0200, &[0x10]);
    // Pointer at dp|((0x10+2)&0xFF) = 0x0012: low=0x00, high=0x30 -> address 0x3000
    bus.poke(0x0012, &[0x00, 0x30]);
    let addr_result = addr::ea_dpx_ind(&mut cpu, &mut bus);
    assert_eq!(addr_result, 0x3000);
    // Log: fetch8 (read), idle, rd16 (2 reads) = 4 entries
    assert_eq!(bus.log.len(), 4);
}

#[test]
fn e06_10_ea_dpx_ind_wrap() {
    let mut cpu = make_cpu();
    cpu.x = 1;
    let mut bus = FlatSmpBus::new();
    bus.poke(0x0200, &[0xFE]);
    // Pointer at dp|((0xFE+1)&0xFF) = 0x00FF: low=0x34, high read from wrap to 0x0000 = 0x12
    bus.poke(0x00FF, &[0x34]);
    bus.poke(0x0000, &[0x12]);
    let addr_result = addr::ea_dpx_ind(&mut cpu, &mut bus);
    assert_eq!(addr_result, 0x1234);
}

#[test]
fn e06_10_ea_dp_ind_y() {
    let mut cpu = make_cpu();
    cpu.y = 0x20;
    let mut bus = FlatSmpBus::new();
    bus.poke(0x0200, &[0x20]);
    // Pointer at dp|0x20: low=0xF0, high=0x10 -> pointer 0x10F0; + Y (0x20) = 0x1110
    bus.poke(0x0020, &[0xF0, 0x10]);
    let addr_result = addr::ea_dp_ind_y(&mut cpu, &mut bus);
    assert_eq!(addr_result, 0x1110);
    // Log: fetch8 (read), rd16 (2 reads), idle = 4 entries
    assert_eq!(bus.log.len(), 4);
}

#[test]
fn e06_10_ea_dp_ind_y_wrap() {
    let mut cpu = make_cpu();
    cpu.y = 2;
    let mut bus = FlatSmpBus::new();
    bus.poke(0x0200, &[0xFF]);
    // Pointer at dp|0xFF: low=0xFF, high read from wrap to 0x0000 = 0x12 -> pointer 0x12FF; + Y (2) = 0x1301
    bus.poke(0x00FF, &[0xFF]);
    bus.poke(0x0000, &[0x12]);
    let addr_result = addr::ea_dp_ind_y(&mut cpu, &mut bus);
    assert_eq!(addr_result, 0x1301);
}
