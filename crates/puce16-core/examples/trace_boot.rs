// Debug helper (E09-12): print the first N executed instructions (pc, registers) of a ROM.
use puce16_core::Snes;

fn main() {
    let rom = std::fs::read(std::env::args().nth(1).expect("rom")).unwrap();
    let skip: u64 = std::env::args().nth(2).unwrap().parse().unwrap();
    let count: u64 = std::env::args().nth(3).unwrap().parse().unwrap();
    let mut snes = Snes::new(&rom, None).unwrap();
    for i in 0..(skip + count) {
        let d = snes.debug();
        if i >= skip {
            println!("{i:6} {:02X}:{:04X} a={:04X} x={:04X} y={:04X} s={:04X} d={:04X} db={:02X} p={:02X} e={}",
                d.pb, d.pc, d.a, d.x, d.y, d.s, d.d, d.db, d.p, d.e as u8);
        }
        snes.step_instruction();
    }
}