use crate::cpu::table::{Am, Mn};
use crate::cpu::{Bus, Cpu};

pub(super) fn exec<B: Bus>(cpu: &mut Cpu, bus: &mut B, mn: Mn, _am: Am) -> bool {
    match mn {
        Mn::MVN | Mn::MVP => {}
        _ => return false,
    }

    // Fetch destination and source bank bytes
    let dst_bank = cpu.fetch8(bus);
    let src_bank = cpu.fetch8(bus);

    // Set data bank to destination bank
    cpu.db = dst_bank;

    // Read from source address (src_bank << 16) | x
    let src_addr = ((src_bank as u32) << 16) | (cpu.x as u32);
    let v = bus.read(src_addr);

    // Write to destination address (dst_bank << 16) | y
    let dst_addr = ((dst_bank as u32) << 16) | (cpu.y as u32);
    bus.write(dst_addr, v);

    // Two idle cycles
    bus.idle();
    bus.idle();

    // Increment or decrement x and y based on instruction
    if cpu.x8() {
        // 8-bit index mode: wrap on 0..255
        let x_byte = (cpu.x & 0xFF) as u8;
        let y_byte = (cpu.y & 0xFF) as u8;
        match mn {
            Mn::MVN => {
                cpu.x = (x_byte.wrapping_add(1)) as u16;
                cpu.y = (y_byte.wrapping_add(1)) as u16;
            }
            Mn::MVP => {
                cpu.x = (x_byte.wrapping_sub(1)) as u16;
                cpu.y = (y_byte.wrapping_sub(1)) as u16;
            }
            _ => unreachable!(),
        }
    } else {
        // 16-bit index mode: wrap on full range
        match mn {
            Mn::MVN => {
                cpu.x = cpu.x.wrapping_add(1);
                cpu.y = cpu.y.wrapping_add(1);
            }
            Mn::MVP => {
                cpu.x = cpu.x.wrapping_sub(1);
                cpu.y = cpu.y.wrapping_sub(1);
            }
            _ => unreachable!(),
        }
    }

    // Decrement A (always 16-bit)
    cpu.a = cpu.a.wrapping_sub(1);

    // If A != 0xFFFF, re-execute by decrementing PC by 3
    if cpu.a != 0xFFFF {
        cpu.pc = cpu.pc.wrapping_sub(3);
    }

    true
}
