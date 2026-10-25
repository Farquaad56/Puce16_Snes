use puce16_core::cpu::Cpu;
use puce16_core::cpu::testbus::FlatBus;

fn setup_cpu() -> Cpu {
    let mut cpu = Cpu::new();
    cpu.e = false; // native mode
    cpu.p = 0x00;  // clear all flags (M=0, X=0)
    cpu.pb = 0;
    cpu.pc = 0x8000;
    cpu
}

#[test]
fn e02_30_mvn_one_byte() {
    let mut cpu = setup_cpu();
    let mut bus = FlatBus::new();
    
    // Set up: A=1 (copy 2 bytes), X=0x1000, Y=0x2000
    cpu.a = 0x0001;
    cpu.x = 0x1000;
    cpu.y = 0x2000;
    
    // Source data at bank 2, addr 0x1000
    bus.poke(0x021000, &[0xAA, 0xBB]);
    
    // MVN instruction: opcode 0x54, dst_bank=0x01, src_bank=0x02
    bus.poke(0x008000, &[0x54, 0x01, 0x02]);
    
    cpu.step(&mut bus);
    
    // After first byte transfer: A should be 0 (decremented from 1)
    assert_eq!(cpu.a, 0x0000);
    // X and Y incremented
    assert_eq!(cpu.x, 0x1001);
    assert_eq!(cpu.y, 0x2001);
    // DB set to destination bank
    assert_eq!(cpu.db, 0x01);
    // PC rewound for re-execution (A != 0xFFFF)
    assert_eq!(cpu.pc, 0x8000);
    // Data transferred
    assert_eq!(bus.peek(0x012000), 0xAA);
}

#[test]
fn e02_30_mvn_finishes() {
    let mut cpu = setup_cpu();
    let mut bus = FlatBus::new();
    
    // Set up: A=1 (copy 2 bytes), X=0x1000, Y=0x2000
    cpu.a = 0x0001;
    cpu.x = 0x1000;
    cpu.y = 0x2000;
    
    // Source data at bank 2, addr 0x1000
    bus.poke(0x021000, &[0xAA, 0xBB]);
    
    // MVN instruction: opcode 0x54, dst_bank=0x01, src_bank=0x02
    bus.poke(0x008000, &[0x54, 0x01, 0x02]);
    
    // First step: copy first byte
    cpu.step(&mut bus);
    
    // Second step: copy second byte and finish (A becomes 0xFFFF)
    cpu.step(&mut bus);
    
    // After completion: A should be 0xFFFF
    assert_eq!(cpu.a, 0xFFFF);
    // PC advanced past instruction
    assert_eq!(cpu.pc, 0x8003);
    // Both bytes transferred
    assert_eq!(bus.peek(0x012000), 0xAA);
    assert_eq!(bus.peek(0x012001), 0xBB);
}

#[test]
fn e02_30_mvn_log() {
    let mut cpu = setup_cpu();
    let mut bus = FlatBus::new();
    
    // Set up: A=1 (copy 2 bytes), X=0x1000, Y=0x2000
    cpu.a = 0x0001;
    cpu.x = 0x1000;
    cpu.y = 0x2000;
    
    // Source data at bank 2, addr 0x1000
    bus.poke(0x021000, &[0xAA, 0xBB]);
    
    // MVN instruction: opcode 0x54, dst_bank=0x01, src_bank=0x02
    bus.poke(0x008000, &[0x54, 0x01, 0x02]);
    
    cpu.step(&mut bus);
    
    // Verify cycle log: opcode fetch + 2 operand fetches + read + write + 2 idle = 7 cycles
    assert_eq!(bus.log.len(), 7);
    
    // Check the sequence of operations
    let expected = vec![
        ('R', 0x008000, 0),   // opcode fetch
        ('R', 0x008001, 0),   // dst bank fetch
        ('R', 0x008002, 0),   // src bank fetch
        ('R', 0x021000, 0xAA),// read from source
        ('W', 0x012000, 0xAA),// write to destination
        ('I', 0, 0),          // idle
        ('I', 0, 0),          // idle
    ];
    
    for (i, exp) in expected.iter().enumerate() {
        let cycle = &bus.log[i];
        match exp.0 {
            'R' => assert_eq!(cycle.kind, puce16_core::cpu::testbus::CycleKind::Read),
            'W' => assert_eq!(cycle.kind, puce16_core::cpu::testbus::CycleKind::Write),
            'I' => assert_eq!(cycle.kind, puce16_core::cpu::testbus::CycleKind::Idle),
            _ => panic!("unknown cycle type"),
        }
        if exp.0 != 'I' {
            assert_eq!(cycle.addr, exp.1);
        }
    }
}

#[test]
fn e02_30_mvp_decrements() {
    let mut cpu = setup_cpu();
    let mut bus = FlatBus::new();
    
    // Set up: A=0 (copy 1 byte), X=0x1001, Y=0x2001
    cpu.a = 0x0000;
    cpu.x = 0x1001;
    cpu.y = 0x2001;
    
    // Source data at bank 2, addr 0x1001
    bus.poke(0x021001, &[0x5A]);
    
    // MVP instruction: opcode 0x44, dst_bank=0x01, src_bank=0x02
    bus.poke(0x008000, &[0x44, 0x01, 0x02]);
    
    cpu.step(&mut bus);
    
    // After transfer: A should be 0xFFFF (decremented from 0)
    assert_eq!(cpu.a, 0xFFFF);
    // X and Y decremented
    assert_eq!(cpu.x, 0x1000);
    assert_eq!(cpu.y, 0x2000);
    // PC advanced past instruction (A == 0xFFFF)
    assert_eq!(cpu.pc, 0x8003);
    // Data transferred
    assert_eq!(bus.peek(0x012001), 0x5A);
}

#[test]
fn e02_30_mvn_8bit_index() {
    let mut cpu = setup_cpu();
    let mut bus = FlatBus::new();
    
    // Set up: A=0 (copy 1 byte), X=0xFF, Y=0x10, M=0, X=1 (8-bit index mode)
    cpu.a = 0x0000;
    cpu.x = 0x00FF;
    cpu.y = 0x0010;
    cpu.p = 0x10; // Set X flag for 8-bit index mode
    
    // Source data at bank 0, addr 0xFF
    bus.poke(0x0000FF, &[0xCC]);
    
    // MVN instruction: opcode 0x54, dst_bank=0x00, src_bank=0x00
    bus.poke(0x008000, &[0x54, 0x00, 0x00]);
    
    cpu.step(&mut bus);
    
    // After transfer: X should wrap from 0xFF to 0x00 (8-bit mode)
    assert_eq!(cpu.x, 0x0000);
    // Y incremented normally
    assert_eq!(cpu.y, 0x0011);
}

#[test]
fn e02_30_mvn_a_wrap() {
    let mut cpu = setup_cpu();
    let mut bus = FlatBus::new();
    
    // Set up: A=0 (copy 1 byte), X=0x1000, Y=0x2000
    cpu.a = 0x0000;
    cpu.x = 0x1000;
    cpu.y = 0x2000;
    
    // Source data at bank 2, addr 0x1000
    bus.poke(0x021000, &[0xDD]);
    
    // MVN instruction: opcode 0x54, dst_bank=0x01, src_bank=0x02
    bus.poke(0x008000, &[0x54, 0x01, 0x02]);
    
    cpu.step(&mut bus);
    
    // After transfer: A should be 0xFFFF (decremented from 0)
    assert_eq!(cpu.a, 0xFFFF);
    // PC advanced past instruction (A == 0xFFFF)
    assert_eq!(cpu.pc, 0x8003);
    // Data transferred
    assert_eq!(bus.peek(0x012000), 0xDD);
}