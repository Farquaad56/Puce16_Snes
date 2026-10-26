//! Synthetic ROM builder for integration tests (E03-36).

use crate::common::{make_rom_with, set_vector, Map};

/// Build a LoRom 32 KiB test ROM from code blocks and interrupt vectors.
///
/// `blocks`: list of (offset relative to $00:8000, bytes) pairs.
/// `vectors`: list of (name, address) pairs for set_vector.
pub fn build_rom(blocks: &[(usize, &[u8])], vectors: &[(&str, u16)]) -> Vec<u8> {
    build_rom_with(blocks, vectors, 0, 0x00)
}

/// Build a LoRom test ROM with explicit SRAM and chipset settings.
pub fn build_rom_with(
    blocks: &[(usize, &[u8])],
    vectors: &[(&str, u16)],
    sram_log2: u8,
    chipset: u8,
) -> Vec<u8> {
    // Compute total code size needed
    let mut code_size = 0;
    for (off, data) in blocks {
        let end = off + data.len();
        if end > code_size {
            code_size = end;
        }
    }

    // Build the code buffer with zeros elsewhere
    let mut code = vec![0u8; code_size];
    for (off, data) in blocks {
        code[*off..*off + data.len()].copy_from_slice(data);
    }

    // Create the ROM via make_rom_with
    let mut rom = make_rom_with(&code, Map::LoRom, 15, sram_log2, chipset, 1);

    // Set interrupt vectors
    for (name, addr) in vectors {
        set_vector(&mut rom, Map::LoRom, name, *addr);
    }

    rom
}
