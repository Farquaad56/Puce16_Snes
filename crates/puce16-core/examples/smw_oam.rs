// Debug helper (E07-14): print the SMW OAM mirror (WRAM $0200..$041F) after N frames.
use puce16_core::{joypad::*, Snes};

fn main() {
    let rom = std::fs::read(std::env::args().nth(1).expect("rom")).unwrap();
    let frames: u64 = std::env::args().nth(2).unwrap().parse().unwrap();
    let mut snes = Snes::new(&rom, None).unwrap();
    let script: [(u64, u64, u16); 7] = [
        (700, 705, BTN_START), (800, 805, BTN_A), (900, 905, BTN_A), (1000, 1005, BTN_A),
        (1100, 1105, BTN_A), (1200, 1205, BTN_A), (1300, 1305, BTN_A),
    ];
    for f in 0..frames {
        let b = script.iter().filter(|(a, z, _)| f >= *a && f <= *z).fold(0u16, |m, s| m | s.2);
        snes.set_joypad(0, b);
        snes.run_frame();
    }
    let oam = snes.peek_oam();
    let mirror_diff = (0..544u32).filter(|&k| oam[k as usize] != snes.peek_wram(0x200 + k)).count();
    println!("PPU OAM vs WRAM mirror: {mirror_diff} bytes differ");
    for i in 0..128u32 {
        let x = oam[(i * 4) as usize];
        let y = oam[(i * 4 + 1) as usize];
        let t = oam[(i * 4 + 2) as usize];
        let a = oam[(i * 4 + 3) as usize];
        let hi = oam[512 + (i / 4) as usize];
        let bits = (hi >> ((i % 4) * 2)) & 3;
        // Only sprites that can touch the top rows via wrap (y >= 0xE0) or y < 0x08.
        if y >= 0xE0 || y < 0x08 {
            println!("#{i:3} x={x:3} xhi={} large={} y={y:3} tile={t:02X} attr={a:02X}", bits & 1, bits >> 1);
        }
    }
}
