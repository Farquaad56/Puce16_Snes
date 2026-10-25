use super::MapMode;

/// Offset into the (power-of-two) ROM BEFORE masking. None = the cartridge does not answer with ROM here.
pub fn rom_offset(map: MapMode, bank: u8, addr: u16) -> Option<usize> {
    match map {
        MapMode::LoRom => lorom_rom(bank, addr),
        MapMode::HiRom => hirom_rom(bank, addr),
        MapMode::ExHiRom => exhirom_rom(bank, addr),
    }
}

/// Offset into SRAM (already reduced modulo sram_len). None if not an SRAM address or sram_len == 0.
pub fn sram_offset(map: MapMode, bank: u8, addr: u16, sram_len: usize) -> Option<usize> {
    if sram_len == 0 {
        return None;
    }
    match map {
        MapMode::LoRom => lorom_sram(bank, addr, sram_len),
        MapMode::HiRom | MapMode::ExHiRom => hirom_sram(bank, addr, sram_len),
    }
}

fn lorom_rom(bank: u8, addr: u16) -> Option<usize> {
    let high = (bank >= 0x00 && bank <= 0x7D) || (bank >= 0x80 && bank <= 0xFF);
    let low_mirror = (bank >= 0x40 && bank <= 0x7D) || (bank >= 0xC0 && bank <= 0xFF);

    if addr >= 0x8000 && high {
        Some((((bank & 0x7F) as usize) << 15) | (addr & 0x7FFF) as usize)
    } else if addr < 0x8000 && low_mirror {
        // Mirror of upper half: same formula with addr masked to lower 15 bits
        Some((((bank & 0x7F) as usize) << 15) | (addr & 0x7FFF) as usize)
    } else {
        None
    }
}

fn hirom_rom(bank: u8, addr: u16) -> Option<usize> {
    let full_bank = (bank >= 0xC0 && bank <= 0xFF) || (bank >= 0x40 && bank <= 0x7D);
    let high_half_only = (bank >= 0x00 && bank <= 0x3F) || (bank >= 0x80 && bank <= 0xBF);

    if full_bank {
        Some((((bank & 0x3F) as usize) << 16) | addr as usize)
    } else if high_half_only && addr >= 0x8000 {
        Some((((bank & 0x3F) as usize) << 16) | addr as usize)
    } else {
        None
    }
}

fn exhirom_rom(bank: u8, addr: u16) -> Option<usize> {
    let full_bank = (bank >= 0xC0 && bank <= 0xFF) || (bank >= 0x40 && bank <= 0x7D);
    let high_half_only = (bank >= 0x00 && bank <= 0x3F) || (bank >= 0x80 && bank <= 0xBF);

    if full_bank || (high_half_only && addr >= 0x8000) {
        let upper = if bank & 0x80 == 0 { 0x40_0000 } else { 0 };
        Some(upper | (((bank & 0x3F) as usize) << 16) | addr as usize)
    } else {
        None
    }
}

fn lorom_sram(bank: u8, addr: u16, sram_len: usize) -> Option<usize> {
    let in_bank = (bank >= 0x70 && bank <= 0x7D) || (bank >= 0xF0 && bank <= 0xFF);
    if in_bank && addr < 0x8000 {
        Some(((((bank & 0x0F) as usize) << 15) | addr as usize) % sram_len)
    } else {
        None
    }
}

fn hirom_sram(bank: u8, addr: u16, sram_len: usize) -> Option<usize> {
    let in_bank = (bank >= 0x20 && bank <= 0x3F) || (bank >= 0xA0 && bank <= 0xBF);
    if in_bank && addr >= 0x6000 && addr <= 0x7FFF {
        Some(((((bank & 0x1F) as usize) << 13) | ((addr - 0x6000) as usize)) % sram_len)
    } else {
        None
    }
}

