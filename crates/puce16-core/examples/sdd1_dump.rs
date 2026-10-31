// Debug helper (E09-12): decompress S-DD1 streams listed on stdin with the Puce16 decompressor.
// usage: sdd1_dump <rom> <out.bin> < entries   (entry: addr_hex size_hex m0 m1 m2 m3)
use puce16_core::coprocessor::sdd1::{Decompressor, Sdd1};
use std::io::BufRead;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let raw = std::fs::read(&a[1]).unwrap();
    let mut rom = raw.clone();
    let n = rom.len().next_power_of_two();
    rom.resize(n, 0);
    let mask = rom.len() - 1;
    let mut out = Vec::new();
    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap();
        let t: Vec<&str> = line.split_whitespace().collect();
        if t.len() < 6 { continue; }
        let addr = u32::from_str_radix(t[0], 16).unwrap();
        let size = usize::from_str_radix(t[1], 16).unwrap();
        let size = if size == 0 { 65536 } else { size };
        let mut mmc = [0u8; 4];
        for i in 0..4 { mmc[i] = t[2 + i].parse::<u8>().unwrap(); }
        let mut s = Sdd1::new();
        for i in 0..4 { s.io_write(0x4804 + i as u16, mmc[i]); }
        let rom_ref = &rom;
        let f = move |a: u32| -> u8 {
            let block = ((a >> 20) & 3) as usize;
            let offset = (((mmc[block] & 0x0F) as u32) << 20) | (a & 0xFFFFF);
            rom_ref[offset as usize & mask]
        };
        let mut d = Decompressor::new();
        d.init(addr, &f);
        for _ in 0..size { out.push(d.read(&f)); }
    }
    std::fs::write(&a[2], out).unwrap();
}
