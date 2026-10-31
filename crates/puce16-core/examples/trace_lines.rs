// Debug helper (E09-12): trace PPU register changes (with scanline/dot) over one frame after loading a state.
// usage: trace_lines <rom> <state.raw> <skip_frames> <trace_frames>
use puce16_core::Snes;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let rom = std::fs::read(&a[1]).unwrap();
    let st = std::fs::read(&a[2]).unwrap();
    let skip: u64 = a[3].parse().unwrap();
    let count: u64 = a[4].parse().unwrap();
    let mut snes = Snes::new(&rom, None).unwrap();
    snes.load_state(&st).unwrap();
    for _ in 0..skip { snes.run_frame(); }
    let mut last = String::new();
    let start = snes.frame_count();
    let mut instr = 0u64;
    while snes.frame_count() < start + count {
        snes.step_instruction();
        instr += 1;
        let r = snes.debug_ppu_regs().unwrap();
        let key = format!("mode={} tm={:02X} ts={:02X} obj={} hofs={:?} vofs={:?} map={:?} chr={:?} fb={} br={} m={:02X}/{:02X}/{}",
            r.bg_mode, r.tm, r.ts, r.obj_size, r.hofs, r.vofs, r.map_base, r.chr_base, r.forced_blank as u8, r.brightness, r.math_region, r.math_mask, r.math_sub as u8);
        if key != last {
            let d = snes.debug();
            println!("F{} v={:3} dot={:3} pc={:02X}:{:04X} {}", snes.frame_count() - start, r.v, r.dot, d.pb, d.pc, key);
            last = key;
        }
    }
    println!("instructions {instr}");
}