//! 5A22 MMIO registers ($4200-$421F) - write side only (reads are E03-15).

use crate::hash::Fnv64;

pub const MUL_CLOCKS: u64 = 48;     // UNKNOWN - to confirm
pub const DIV_CLOCKS: u64 = 96;     // UNKNOWN - to confirm

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MmioEffect {
    None,
    Mdmaen(u8),
    Hdmaen(u8),
    Memsel(bool),
    Wrio { old: u8, new: u8 },
}

#[derive(Clone, Copy, Debug, Default)]
pub struct HvState {
    pub vblank: bool,
    pub hblank: bool,
    pub autoread_busy: bool,
}

struct MathPending {
    done_at: u64,
    rddiv: u16,
    rdmpy: u16,
}

pub struct Mmio {
    pub nmitimen: u8,
    pub wrio: u8,
    pub htime: u16,
    pub vtime: u16,
    pub hdmaen: u8,
    pub memsel: bool,
    pub rdnmi_flag: bool,
    pub timeup_flag: bool,
    pub irq_line: bool,
    pub joy: [u16; 4],
    wrmpya: u8,
    wrdiv: u16,
    rddiv: u16,
    rdmpy: u16,
    math_pending: Option<MathPending>,
}

impl Mmio {
    pub fn new() -> Mmio {
        Mmio {
            nmitimen: 0,
            wrio: 0xFF,
            htime: 0x1FF,
            vtime: 0x1FF,
            hdmaen: 0,
            memsel: false,
            rdnmi_flag: false,
            timeup_flag: false,
            irq_line: false,
            joy: [0; 4],
            wrmpya: 0xFF,
            wrdiv: 0xFFFF,
            rddiv: 0,
            rdmpy: 0,
            math_pending: None,
        }
    }

    pub fn reset(&mut self) {
        self.nmitimen = 0;
        self.hdmaen = 0;
        self.memsel = false;
        self.rdnmi_flag = false;
        self.timeup_flag = false;
        self.irq_line = false;
        self.math_pending = None;
    }

    fn settle(&mut self, master: u64) {
        if let Some(p) = &self.math_pending {
            if master >= p.done_at {
                self.rddiv = p.rddiv;
                self.rdmpy = p.rdmpy;
                self.math_pending = None;
            }
        }
    }

    pub fn write(&mut self, reg: u16, v: u8, master: u64) -> MmioEffect {
        match reg {
            0x4200 => {
                self.nmitimen = v;
                if (v >> 4) & 3 == 0 {
                    self.timeup_flag = false;
                    self.irq_line = false;
                }
                MmioEffect::None
            }
            0x4201 => {
                let old = self.wrio;
                self.wrio = v;
                MmioEffect::Wrio { old, new: v }
            }
            0x4202 => {
                self.wrmpya = v;
                MmioEffect::None
            }
            0x4203 => {
                self.settle(master);
                let result = (self.wrmpya as u16) * (v as u16);
                self.math_pending = Some(MathPending {
                    done_at: master + MUL_CLOCKS,
                    rddiv: self.rddiv,
                    rdmpy: result,
                });
                MmioEffect::None
            }
            0x4206 => {
                self.settle(master);
                let divisor = v as u16;
                let (q, r) = if divisor != 0 {
                    (self.wrdiv / divisor, self.wrdiv % divisor)
                } else {
                    (0xFFFF, self.wrdiv)
                };
                self.math_pending = Some(MathPending {
                    done_at: master + DIV_CLOCKS,
                    rddiv: q,
                    rdmpy: r,
                });
                MmioEffect::None
            }
            0x4204 => {
                self.wrdiv = (self.wrdiv & 0xFF00) | v as u16;
                MmioEffect::None
            }
            0x4205 => {
                self.wrdiv = (self.wrdiv & 0x00FF) | ((v as u16) << 8);
                MmioEffect::None
            }
            0x4207 => {
                self.htime = (self.htime & 0x100) | v as u16;
                MmioEffect::None
            }
            0x4208 => {
                self.htime = (self.htime & 0x0FF) | ((v as u16 & 1) << 8);
                MmioEffect::None
            }
            0x4209 => {
                self.vtime = (self.vtime & 0x100) | v as u16;
                MmioEffect::None
            }
            0x420A => {
                self.vtime = (self.vtime & 0x0FF) | ((v as u16 & 1) << 8);
                MmioEffect::None
            }
            0x420B => MmioEffect::Mdmaen(v),
            0x420C => {
                self.hdmaen = v;
                MmioEffect::Hdmaen(v)
            }
            0x420D => {
                self.memsel = (v & 1) != 0;
                MmioEffect::Memsel(self.memsel)
            }
            _ => MmioEffect::None,
        }
    }

    pub fn read(&mut self, reg: u16, mdr: u8, master: u64, st: Option<HvState>) -> u8 {
        self.settle(master);
        let state = st.unwrap_or_default();
        match reg {
            0x4210 => {
                let v = (mdr & 0x70) | 0x02 | ((self.rdnmi_flag as u8) << 7);
                self.rdnmi_flag = false;
                v
            }
            0x4211 => {
                let v = (mdr & 0x7F) | ((self.timeup_flag as u8) << 7);
                self.timeup_flag = false;
                self.irq_line = false;
                v
            }
            0x4212 => {
                (mdr & 0x3E)
                    | ((state.vblank as u8) << 7)
                    | ((state.hblank as u8) << 6)
                    | (state.autoread_busy as u8)
            }
            0x4213 => self.wrio,
            0x4214 => (self.rddiv & 0xFF) as u8,
            0x4215 => ((self.rddiv >> 8) & 0xFF) as u8,
            0x4216 => (self.rdmpy & 0xFF) as u8,
            0x4217 => ((self.rdmpy >> 8) & 0xFF) as u8,
            r if (0x4218..=0x421F).contains(&r) => {
                let i = ((r - 0x4218) / 2) as usize;
                if r % 2 == 0 {
                    (self.joy[i] & 0xFF) as u8
                } else {
                    ((self.joy[i] >> 8) & 0xFF) as u8
                }
            }
            _ => mdr,
        }
    }

    pub fn hash_into(&self, h: &mut Fnv64) {
        h.write_u8(self.nmitimen);
        h.write_u8(self.wrio);
        h.write_u16(self.htime);
        h.write_u16(self.vtime);
        h.write_u8(self.hdmaen);
        if self.memsel {
            h.write_u8(1);
        } else {
            h.write_u8(0);
        }
        if self.rdnmi_flag {
            h.write_u8(1);
        } else {
            h.write_u8(0);
        }
        if self.timeup_flag {
            h.write_u8(1);
        } else {
            h.write_u8(0);
        }
        if self.irq_line {
            h.write_u8(1);
        } else {
            h.write_u8(0);
        }
        for j in &self.joy {
            h.write_u16(*j);
        }
        h.write_u8(self.wrmpya);
        h.write_u16(self.wrdiv);
        h.write_u16(self.rddiv);
        h.write_u16(self.rdmpy);
        match &self.math_pending {
            None => h.write_u8(0),
            Some(p) => {
                h.write_u8(1);
                h.write_u64(p.done_at);
                h.write_u16(p.rddiv);
                h.write_u16(p.rdmpy);
            }
        }
    }
}

impl crate::savestate::State for Mmio {
    fn save(&self, w: &mut crate::savestate::StateWriter) {
        // Save all fields in declaration order. All are covered by hash_into.
        w.u8(self.nmitimen);
        w.u8(self.wrio);
        w.u16(self.htime);
        w.u16(self.vtime);
        w.u8(self.hdmaen);
        w.bool(self.memsel);
        w.bool(self.rdnmi_flag);
        w.bool(self.timeup_flag);
        w.bool(self.irq_line);
        w.slice_u16(&self.joy);
        w.u8(self.wrmpya);
        w.u16(self.wrdiv);
        w.u16(self.rddiv);
        w.u16(self.rdmpy);
        match &self.math_pending {
            None => w.u8(0),
            Some(p) => {
                w.u8(1);
                w.u64(p.done_at);
                w.u16(p.rddiv);
                w.u16(p.rdmpy);
            }
        }
    }

    fn load(&mut self, r: &mut crate::savestate::StateReader) -> Result<(), crate::savestate::StateError> {
        // Load all fields in declaration order.
        self.nmitimen = r.u8()?;
        self.wrio = r.u8()?;
        self.htime = r.u16()?;
        self.vtime = r.u16()?;
        self.hdmaen = r.u8()?;
        self.memsel = r.bool()?;
        self.rdnmi_flag = r.bool()?;
        self.timeup_flag = r.bool()?;
        self.irq_line = r.bool()?;
        r.slice_u16_into(&mut self.joy)?;
        self.wrmpya = r.u8()?;
        self.wrdiv = r.u16()?;
        self.rddiv = r.u16()?;
        self.rdmpy = r.u16()?;
        let idx = r.u8()?;
        match idx {
            0 => self.math_pending = None,
            1 => {
                let done_at = r.u64()?;
                let rddiv = r.u16()?;
                let rdmpy = r.u16()?;
                self.math_pending = Some(MathPending { done_at, rddiv, rdmpy });
            }
            _ => return Err(crate::savestate::StateError::Mismatch("math_pending")),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::savestate::{State, StateWriter, StateReader};
    use crate::hash::Fnv64;

    fn mmio_hash(mmio: &Mmio) -> u64 {
        let mut h = Fnv64::new();
        mmio.hash_into(&mut h);
        h.finish()
    }

    #[test]
    fn e08_04_mmio_roundtrip() {
        // Set every saved field to a distinct non-zero value
        let mut mmio = Mmio::new();
        mmio.nmitimen = 0x12;
        mmio.wrio = 0x34;
        mmio.htime = 0x5678;
        mmio.vtime = 0x9ABC;
        mmio.hdmaen = 0xDE;
        mmio.memsel = true;
        mmio.rdnmi_flag = true;
        mmio.timeup_flag = true;
        mmio.irq_line = true;
        mmio.joy = [0x1111, 0x2222, 0x3333, 0x4444];
        mmio.wrmpya = 0xFF;
        mmio.wrdiv = 0xFFFF;
        mmio.rddiv = 0xABCD;
        mmio.rdmpy = 0xEF01;
        mmio.math_pending = Some(MathPending { done_at: 987654321, rddiv: 0x1234, rdmpy: 0x5678 });

        // Save
        let mut writer = StateWriter::new();
        mmio.save(&mut writer);
        let data = writer.finish();

        // Load into fresh instance
        let mut loaded = Mmio::new();
        let mut reader = StateReader::new(&data);
        loaded.load(&mut reader).unwrap();

        // Verify bytes match on re-save
        let mut writer2 = StateWriter::new();
        loaded.save(&mut writer2);
        let data2 = writer2.finish();
        assert_eq!(data, data2, "re-saved bytes must match original");

        // Verify hash matches
        assert_eq!(mmio_hash(&mmio), mmio_hash(&loaded), "hashes must match after roundtrip");
    }

    #[test]
    fn e08_04_mmio_truncated() {
        let mut mmio = Mmio::new();
        mmio.nmitimen = 0x12;
        mmio.wrio = 0x34;
        mmio.htime = 0x5678;
        mmio.vtime = 0x9ABC;
        mmio.hdmaen = 0xDE;
        mmio.memsel = true;
        mmio.rdnmi_flag = true;
        mmio.timeup_flag = true;
        mmio.irq_line = true;
        mmio.joy = [0x1111, 0x2222, 0x3333, 0x4444];
        mmio.wrmpya = 0xFF;
        mmio.wrdiv = 0xFFFF;
        mmio.rddiv = 0xABCD;
        mmio.rdmpy = 0xEF01;
        mmio.math_pending = Some(MathPending { done_at: 987654321, rddiv: 0x1234, rdmpy: 0x5678 });

        let mut writer = StateWriter::new();
        mmio.save(&mut writer);
        let data = writer.finish();

        // Test truncation at various points
        for n in [0, data.len() / 2, data.len() - 1] {
            let mut loaded = Mmio::new();
            let mut reader = StateReader::new(&data[..n]);
            let result = loaded.load(&mut reader);
            assert!(matches!(result, Err(crate::savestate::StateError::Truncated)), 
                "truncation at {} should fail with Truncated", n);
        }
    }
}
