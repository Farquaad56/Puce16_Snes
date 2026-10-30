//! S-DD1 DMA interception: spy on DMA register writes, decompress on-the-fly during MCU reads.

use super::regs::Sdd1;

impl Sdd1 {
    /// Intercept a write to a DMA channel register ($43x2-$43x6).
    /// Assembles the address and size fields per channel.
    pub fn dma_write(&mut self, reg: u16, v: u8) {
        if !(0x4300..=0x437F).contains(&reg) {
            return;
        }
        let n = ((reg >> 4) & 7) as usize;
        match reg & 0xF {
            2 => self.dma_addr[n] = (self.dma_addr[n] & !0xFF) | v as u32,
            3 => self.dma_addr[n] = (self.dma_addr[n] & !(0xFF << 8)) | ((v as u32) << 8),
            4 => self.dma_addr[n] = (self.dma_addr[n] & !(0xFF << 16)) | ((v as u32) << 16),
            5 => self.dma_size[n] = (self.dma_size[n] & !0xFF) | v as u16,
            6 => self.dma_size[n] = (self.dma_size[n] & !(0xFF << 8)) | ((v as u16) << 8),
            _ => {} // other registers ignored
        }
    }

    /// Get the DMA channel's current address and size.
    pub fn dma_channel(&self, n: usize) -> (u32, u16) {
        (self.dma_addr[n], self.dma_size[n])
    }

    /// Read a byte from ROM through MMC mapping (no decompression).
    pub fn mmc_read(&self, addr: u32, rom: &[u8]) -> u8 {
        let mask = rom.len() - 1;
        rom[self.mmc_offset(addr) & mask]
    }

    /// MCU read with DMA interception and on-the-fly decompression.
    pub fn mcu_read(&mut self, addr: u32, rom: &[u8]) -> u8 {
        if (self.r4800 & self.r4801) != 0 {
            for n in 0..8 {
                let bit = 1 << n;
                if (self.r4800 & bit) != 0 && (self.r4801 & bit) != 0 && addr == self.dma_addr[n] {
                    // Copy mmc to avoid double borrow of self in closures
                    let mmc_copy = self.mmc;
                    let mask = rom.len() - 1;

                    if !self.dma_ready {
                        let f = move |a: u32| -> u8 {
                            let block = ((a >> 20) & 3) as usize;
                            let bank_reg = mmc_copy[block];
                            let offset = (((bank_reg & 0x0F) as u32) << 20) | (a & 0xFFFFF);
                            rom[offset as usize & mask]
                        };
                        self.dec.init(addr, &f);
                        self.dma_ready = true;
                    }

                    let byte = self.dec.read(&|a: u32| -> u8 {
                        let block = ((a >> 20) & 3) as usize;
                        let bank_reg = mmc_copy[block];
                        let offset = (((bank_reg & 0x0F) as u32) << 20) | (a & 0xFFFFF);
                        rom[offset as usize & mask]
                    });
                    self.dma_size[n] = self.dma_size[n].wrapping_sub(1);
                    if self.dma_size[n] == 0 {
                        self.dma_ready = false;
                        self.r4801 &= !bit;
                    }
                    return byte;
                }
            }
        }
        // No decompression: read plain ROM through MMC mapping
        self.mmc_read(addr, rom)
    }
}
