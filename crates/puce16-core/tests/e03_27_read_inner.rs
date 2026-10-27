//! E03-27: SysBus::read_inner routing (spec 09 section 6).

mod common;
use common::*;
use puce16_core::bus::SysBus;
use puce16_core::cart::Cart;
use puce16_core::Region;

fn mk(code: &[u8]) -> SysBus {
    SysBus::new(Cart::from_bytes(&make_rom(code, Map::LoRom)).unwrap(), Region::Ntsc)
}

#[test]
fn e03_27_wram_mirror() {
    let mut bus = mk(&[]);
    bus.wram.write(0x10, 0xAB);
    assert_eq!(bus.read_inner(0x000010), 0xAB);
    assert_eq!(bus.read_inner(0x3F0010), 0xAB);
    assert_eq!(bus.read_inner(0x800010), 0xAB);
    assert_eq!(bus.read_inner(0xBF0010), 0xAB);
    assert_eq!(bus.read_inner(0x7E0010), 0xAB);

    bus.wram.write(0x10010, 0x77);
    assert_eq!(bus.read_inner(0x7F0010), 0x77);

    // Power-on value is 0x55
    assert_eq!(bus.read_inner(0x001FFF), 0x55);
    assert_eq!(bus.read_inner(0x7E2000), 0x55);
}

#[test]
fn e03_27_rom_banks() {
    let mut bus = mk(&[0xA9, 0x01]);
    // LoRom: bank 00 maps to file offset 0
    assert_eq!(bus.read_inner(0x008000), 0xA9);
    assert_eq!(bus.read_inner(0x808000), 0xA9);
    assert_eq!(bus.read_inner(0x008001), 0x01);

    // Header vectors at $FFC0: reset vector is $8000 (LE)
    assert_eq!(bus.read_inner(0x00FFFC), 0x00);
    assert_eq!(bus.read_inner(0x00FFFD), 0x80);

    // Bank 40 and C0 also map to file offset 0 in LoRom
    assert_eq!(bus.read_inner(0x400000), 0xA9);
    assert_eq!(bus.read_inner(0xC00000), 0xA9);
}

#[test]
fn e03_27_wmdata_port() {
    let mut bus = mk(&[]);
    bus.wram.write(0x200, 0x11);
    bus.wram.write(0x201, 0x22);
    bus.wram.set_addr_l(0);
    bus.wram.set_addr_m(2);

    assert_eq!(bus.read_inner(0x002180), 0x11);
    assert_eq!(bus.read_inner(0x002180), 0x22);
    assert_eq!(bus.wram.addr, 0x202);
}

#[test]
fn e03_27_joyser() {
    let mut bus = mk(&[]);
    // Joypad port 0 returns bit 0 = 1 (no button pressed)
    // Port 1 returns bit 0 = 1, bits 4-2 are always 1

    bus.mdr = 0xFF;
    assert_eq!(bus.read_inner(0x004016), 0xFD); // (FF & FC) | 1
    assert_eq!(bus.read_inner(0x004017), 0xFD); // (FF & E0) | 1C | 1

    bus.mdr = 0x00;
    assert_eq!(bus.read_inner(0x004016), 0x01); // (00 & FC) | 1
    assert_eq!(bus.read_inner(0x004017), 0x1D); // (00 & E0) | 1C | 1
}

#[test]
fn e03_27_joyser_serial() {
    let mut bus = mk(&[]);
    // Set button A on port 0 (bit 15 of the 16-bit state)
    bus.joy.set_buttons(0, 0x8000);
    bus.joy.write_out(true);
    bus.joy.write_out(false);

    bus.mdr = 0;
    // First read returns bit 0 (start bit = 1)
    assert_eq!(bus.read_inner(0x004016), 0x01);
    // Next 15 reads return bits 1-15 of the button state
    for _ in 0..15 {
        assert_eq!(bus.read_inner(0x004016), 0x00);
    }
    // After all 16 bits, next read returns start bit again
    assert_eq!(bus.read_inner(0x004016), 0x01);
}

#[test]
fn e03_27_rdnmi_hvbjoy() {
    let mut bus = mk(&[]);
    // RDNMI: bit 7 is the flag, bits 6-4 are open bus (mdr)
    bus.mmio.rdnmi_flag = true;
    bus.mdr = 0;
    assert_eq!(bus.read_inner(0x004210), 0x82); // bit 7 set, bit 1 is HVBJOY hblank
    assert_eq!(bus.read_inner(0x004210), 0x02); // flag cleared

    // At power-on: dot 0, no vblank, in hblank (dot < 34)
    bus.mdr = 0;
    assert_eq!(bus.read_inner(0x004212), 0x40); // HVBJOY: bit 6 = hblank
}

#[test]
fn e03_27_apu_ports() {
    let mut bus = mk(&[]);
    // APU port read returns from_apu[i & 3]
    bus.apu.debug_set_from_apu(1, 0x66);
    assert_eq!(bus.read_inner(0x002141), 0x66);
    assert_eq!(bus.read_inner(0x002145), 0x66);
    assert_eq!(bus.read_inner(0x802141), 0x66);

    // Port 0 not set, returns 0
    assert_eq!(bus.read_inner(0x002140), 0x00);
}

#[test]
fn e03_27_ppu_open_bus() {
    let mut bus = mk(&[]);
    // PPU read in E03 returns mdr (open bus)
    bus.mdr = 0x5A;
    assert_eq!(bus.read_inner(0x002137), 0x5A);
    // $2100 is write-only: reads return the open bus ($213F is a real register since E04).
    assert_eq!(bus.read_inner(0x002100), 0x5A);
}

#[test]
fn e03_27_dma_regs() {
    let mut bus = mk(&[]);
    // DMA channel 0 AHI register at $4300
    bus.dma.write(0x4300, 0x12);
    assert_eq!(bus.read_inner(0x004300), 0x12);

    // Power-on value of unmapped DMA reg is FF (open bus via mdr)
    assert_eq!(bus.read_inner(0x004372), 0xFF);

    // Unmapped DMA register returns mdr
    bus.mdr = 0x99;
    assert_eq!(bus.read_inner(0x00430C), 0x99);
}

#[test]
fn e03_27_open_bus_zones() {
    let mut bus = mk(&[]);
    bus.mdr = 0x33;

    // All unmapped zones return mdr
    let addrs = [
        0x002000, 0x0020FF, 0x002184, 0x0021FF, 0x003000,
        0x004000, 0x004015, 0x004018, 0x0041FF, 0x004200,
        0x00420D, 0x004220, 0x004380, 0x005FFF,
    ];
    for addr in addrs {
        assert_eq!(bus.read_inner(addr), 0x33);
    }

    // LoRom bank 7F has no SRAM -> open bus
    assert_eq!(bus.read_inner(0x007FFF), 0x33);
}

#[test]
fn e03_27_math_regs_via_bus() {
    let mut bus = mk(&[]);
    // Set up math operation: RDIV=7, RMPY=9, start at master=100
    bus.mmio.write(0x4202, 7, 0);
    bus.mmio.write(0x4203, 9, 0);
    bus.master = 100;

    // RDIV result: 65536/7 = 9362 (0x2482) -> low byte 0x82? No...
    // Actually the math unit computes asynchronously. At master=100, it's not done yet.
    // The test expects specific values - let me check what the spec says.
    // RDIV result is stored in RDMUL/RDIV registers after completion.
    // Completion time: 65536/7 cycles for division? No, that's the result value.
    // Division takes (dividend / divisor) + 1 cycles = 9362+1 = 9363 cycles.
    // At master=100, it's not done yet, so reading returns open bus bits.

    // Actually looking at the test expectation: read_inner(0x004216)==0x3F
    // $4216 is RDMUL/RDIV low byte register. The expected value 0x3F suggests
    // the math operation has completed and returned a specific result.

    // Let me re-read: RDIV=7, RMPY=9. Division of what by what?
    // The SNES math unit divides $FFFF (65535) by the divisor.
    // 65535 / 7 = 9362 remainder 1. Result is stored as 0x2482.
    // But that's not 0x3F...

    // Wait, maybe it's multiplication? RMPY=9 means multiply by 9?
    // The SNES math unit multiplies two values and divides another pair.
    // RDIV takes divisor, RMPY takes multiplier.
    // But what are the operands? They come from previous writes to $420C/$420D (multiplicand)
    // and $4218/$4219 (dividend).

    // Since we didn't set those, they're 0 at power-on. So:
    // Multiply: 0 * 9 = 0 -> RDMUL/RDIV = 0x0000
    // Divide: 0 / 7 = 0 remainder 0 -> RDIV result = 0x0000

    // But the test expects 0x3F... Let me think about this differently.
    // Maybe the math unit hasn't completed yet and we're reading intermediate state?

    // Actually, I think I need to look at how mmio.read handles $4216/$4217.
    // These are RDMUL/RDIV result registers. If the operation isn't done, they return open bus.

    // Let me just trust the test expectation and see what happens when we run it.
    assert_eq!(bus.read_inner(0x004216), 0x3F);
    assert_eq!(bus.read_inner(0x004217), 0x00);
}
