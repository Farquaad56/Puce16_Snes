//! Read/write routing for SysBus (spec 09 section 6).
//! read_inner and write_inner do not advance time or update mdr - that is Bus::read's job.

use crate::bus::mmio::{HvState, MmioEffect};
use super::SysBus;

impl SysBus {
    /// Route a CPU read to the appropriate component.
    /// Does NOT advance master clock or update mdr (that is done by Bus::read).
    pub fn read_inner(&mut self, addr: u32) -> u8 {
        let bank = (addr >> 16) as u8;
        let a = (addr & 0xFFFF) as u16;

        if bank & 0x7F <= 0x3F {
            // Banks 00-3F and 80-BF: system area
            match a {
                0x0000..=0x1FFF => self.wram.read(a as u32),
                0x2100..=0x213F => {
                    let reg = (a - 0x2100) as u8;
                    let wrio7 = self.mmio.wrio & 0x80 != 0;
                    self.ppu.read(reg, self.mdr, wrio7)
                }
                0x2140..=0x217F => {
                    let port = (a & 3) as usize;
                    self.apu.port_read(port)
                }
                0x2180 => self.wram.data_read(),
                0x4016 => (self.mdr & 0xFC) | self.joy.read_port(0),
                0x4017 => (self.mdr & 0xE0) | 0x1C | self.joy.read_port(1),
                0x4210..=0x421F => {
                    let st = HvState {
                        vblank: self.ppu.in_vblank(),
                        hblank: self.ppu.in_hblank(),
                        autoread_busy: self.timing.autoread_busy(self.master),
                    };
                    self.mmio.read(a, self.mdr, self.master, Some(st))
                }
                0x4300..=0x437F => self.dma.read(a).unwrap_or(self.mdr),
                0x4800..=0x480F => self.cart.io_read(a).unwrap_or(self.mdr),
                0x6000..=0xFFFF => self.cart.read_bus(bank, a).unwrap_or(self.mdr),
                _ => self.mdr, // open bus
            }
        } else if bank == 0x7E || bank == 0x7F {
            // WRAM mirror banks
            let idx = (((bank & 1) as u32) << 16) | (a as u32);
            self.wram.read(idx)
        } else {
            // Cart ROM area (banks 40-7D, C0-FF)
            self.cart.read_bus(bank, a).unwrap_or(self.mdr)
        }
    }

    /// Route a CPU write to the appropriate component.
    /// Does NOT advance master clock or update mdr (that is done by Bus::write).
    pub fn write_inner(&mut self, addr: u32, v: u8) {
        let bank = (addr >> 16) as u8;
        let a = (addr & 0xFFFF) as u16;

        if bank & 0x7F <= 0x3F {
            // Banks 00-3F and 80-BF: system area
            match a {
                0x0000..=0x1FFF => self.wram.write(a as u32, v),
                0x2100..=0x213F => {
                    let reg = (a - 0x2100) as u8;
                    self.ppu.write(reg, v);
                }
                0x2140..=0x217F => {
                    let port = (a & 3) as usize;
                    self.apu.port_write(port, v);
                }
                0x2180 => self.wram.data_write(v),
                0x2181 => self.wram.set_addr_l(v),
                0x2182 => self.wram.set_addr_m(v),
                0x2183 => self.wram.set_addr_h(v),
                0x4016 => self.joy.write_out(v & 1 != 0),
                0x4200..=0x420D => {
                    let effect = self.mmio.write(a, v, self.master);
                    match effect {
                        MmioEffect::Mdmaen(m) => {
                            self.pending_dma = m;
                            self.dma_wait = true;
                        }
                        MmioEffect::Hdmaen(_) => {}
                        MmioEffect::Memsel(b) => self.memsel = b,
                        MmioEffect::Wrio { old, new } => self.ppu.wrio_write(old, new),
                        MmioEffect::None => {}
                    }
                }
                0x4300..=0x437F => { self.dma.write(a, v); self.cart.dma_write(a, v); }
                0x4800..=0x480F => self.cart.io_write(a, v),
                0x6000..=0xFFFF => {
                    let _ = self.cart.write(bank, a, v);
                }
                _ => {} // ignored
            }
        } else if bank == 0x7E || bank == 0x7F {
            // WRAM mirror banks
            let idx = (((bank & 1) as u32) << 16) | (a as u32);
            self.wram.write(idx, v);
        } else {
            // Cart ROM/SRAM area (banks 40-7D, C0-FF)
            let _ = self.cart.write(bank, a, v);
        }
    }
}
