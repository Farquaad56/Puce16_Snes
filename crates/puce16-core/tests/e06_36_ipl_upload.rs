use puce16_core::apu::Apu;

fn run(apu: &mut Apu, master: &mut u64, clocks: u64) {
    *master += clocks;
    apu.step_to(*master);
}

fn wait_port(apu: &mut Apu, master: &mut u64, port: usize, val: u8) -> bool {
    for _ in 0..200 {
        run(apu, master, 2000);
        if apu.port_read(port) == val {
            return true;
        }
    }
    false
}

#[test]
fn e06_36_handshake() {
    let mut a = Apu::new();
    run(&mut a, &mut 0, 100_000);
    assert_eq!(a.port_read(0), 0xAA);
    assert_eq!(a.port_read(1), 0xBB);
}

#[test]
fn e06_36_upload_and_run() {
    let mut a = Apu::new();
    let mut master: u64 = 0;

    // Step 1: handshake
    run(&mut a, &mut master, 100_000);
    assert_eq!(a.port_read(0), 0xAA);
    assert_eq!(a.port_read(1), 0xBB);

    // Step 2: send destination address and start transfer command
    let prog = [0xE8, 0x42, 0xC4, 0xF4, 0x2F, 0xFE];
    a.port_write(2, 0x00); // dest low byte (0x0200)
    a.port_write(3, 0x02); // dest high byte
    a.port_write(1, 1);    // non-zero to indicate data follows
    a.port_write(0, 0xCC); // start transfer command
    assert!(wait_port(&mut a, &mut master, 0, 0xCC), "IPL did not echo CC");

    // Step 3: send each byte with counter
    for (i, &byte) in prog.iter().enumerate() {
        let counter = i as u8;
        a.port_write(1, byte);
        a.port_write(0, counter);
        assert!(wait_port(&mut a, &mut master, 0, counter), "IPL did not echo counter {}", i);
    }

    // Step 4: launch the program at 0x0200
    a.port_write(2, 0x00); // entry low byte
    a.port_write(3, 0x02); // entry high byte
    a.port_write(1, 0);    // zero indicates launch command
    let n = prog.len() as u8;
    a.port_write(0, n + 1); // (n+1) to trigger execution

    // Wait for program output: it writes $42 to port 0 via register $F4
    assert!(wait_port(&mut a, &mut master, 0, 0x42), "Program did not write $42 to port 0");

    // Verify program was uploaded correctly
    assert_eq!(a.debug_peek(0x0200), 0xE8);
    assert_eq!(a.debug_peek(0x0205), 0xFE);
}

#[test]
fn e06_36_upload_two_blocks() {
    let mut a = Apu::new();
    let mut master: u64 = 0;

    // Handshake
    run(&mut a, &mut master, 100_000);
    assert_eq!(a.port_read(0), 0xAA);
    assert_eq!(a.port_read(1), 0xBB);

    // Block A: upload program to 0x0200 (but don't launch yet)
    let prog = [0xE8, 0x42, 0xC4, 0xF4, 0x2F, 0xFE];
    a.port_write(2, 0x00);
    a.port_write(3, 0x02);
    a.port_write(1, 1);
    a.port_write(0, 0xCC);
    assert!(wait_port(&mut a, &mut master, 0, 0xCC));

    for (i, &byte) in prog.iter().enumerate() {
        let counter = i as u8;
        a.port_write(1, byte);
        a.port_write(0, counter);
        assert!(wait_port(&mut a, &mut master, 0, counter));
    }

    // Block B: upload data to 0x0300
    let data = [0xAA, 0xBB, 0xCC];
    a.port_write(2, 0x00); // dest low (0x0300)
    a.port_write(3, 0x03); // dest high
    a.port_write(1, 1);
    let n = prog.len() as u8;
    a.port_write(0, n + 1); // signal end of block A and start of B
    assert!(wait_port(&mut a, &mut master, 0, n + 1));

    for (i, &byte) in data.iter().enumerate() {
        let counter = i as u8;
        a.port_write(1, byte);
        a.port_write(0, counter);
        assert!(wait_port(&mut a, &mut master, 0, counter));
    }

    // Launch the program at 0x0200
    a.port_write(2, 0x00);
    a.port_write(3, 0x02);
    a.port_write(1, 0);
    let data_n = data.len() as u8;
    a.port_write(0, data_n + 1);

    assert!(wait_port(&mut a, &mut master, 0, 0x42));

    // Verify both blocks were uploaded correctly
    assert_eq!(a.debug_peek(0x0300), 0xAA);
    assert_eq!(a.debug_peek(0x0301), 0xBB);
    assert_eq!(a.debug_peek(0x0302), 0xCC);
    assert_eq!(a.debug_peek(0x0200), 0xE8);
}

#[test]
fn e06_36_ipl_does_not_start_before_cc() {
    let mut a = Apu::new();
    let mut master: u64 = 0;

    // Handshake
    run(&mut a, &mut master, 100_000);
    assert_eq!(a.port_read(0), 0xAA);
    assert_eq!(a.port_read(1), 0xBB);

    // Don't send $CC - just step more time
    run(&mut a, &mut master, 100_000);

    // IPL should still be waiting at handshake
    assert_eq!(a.port_read(0), 0xAA);
    assert_eq!(a.debug_peek(0x0200), 0);
}
