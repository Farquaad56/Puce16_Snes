pub mod spc700;
pub mod dsp;
pub mod timer;
pub mod ipl;
pub mod io;

use crate::consts::{APU_RESONATOR_HZ, MASTER_HZ_NTSC};
use crate::region::Region;
use crate::savestate::{State, StateWriter, StateReader, StateError};
use self::spc700::Spc700;
use self::io::ApuIo;

/// Complete APU with SPC700 CPU and I/O interface.
pub struct Apu {
    spc: Spc700,
    io: ApuIo,
    acc: i64,
    last_master: u64,
    master_hz: u64,
}

impl Apu {
    /// Fresh APU with NTSC timing and IPL-loaded SPC700.
    pub fn new() -> Apu {
        let mut apu = Apu {
            spc: Spc700::new(),
            io: ApuIo::new(),
            acc: 0,
            last_master: 0,
            master_hz: MASTER_HZ_NTSC,
        };
        // Set PC to IPL reset vector: (IPL_ROM[63] << 8) | IPL_ROM[62]
        apu.spc.pc = ((self::ipl::IPL_ROM[63] as u16) << 8) | (self::ipl::IPL_ROM[62] as u16);
        apu
    }

    /// Reset APU: reload IPL, clear ports and accumulators.
    pub fn reset(&mut self) {
        self.io.reset();
        let mut spc = Spc700::new();
        spc.pc = ((self::ipl::IPL_ROM[63] as u16) << 8) | (self::ipl::IPL_ROM[62] as u16);
        self.spc = spc;
        self.acc = 0;
        self.last_master = 0;
    }

    /// Set region timing (NTSC or PAL).
    pub fn set_region(&mut self, r: Region) {
        self.master_hz = r.master_hz();
    }

    /// Read from port $2140-$2143 (mirrored every 4). Returns byte written by SPC700.
    pub fn port_read(&self, i: usize) -> u8 {
        self.io.port_out[i & 3]
    }

    /// Write to port $2140-$2143 (mirrored every 4). Sends byte to SPC700.
    pub fn port_write(&mut self, i: usize, v: u8) {
        self.io.port_in[i & 3] = v;
    }

    /// Advance APU clock to catch up with master clock (spec 07d section 4).
    pub fn step_to(&mut self, master: u64) {
        if master <= self.last_master {
            return;
        }
        self.acc += (master - self.last_master) as i64 * APU_RESONATOR_HZ as i64;
        self.last_master = master;
        while self.acc > 0 {
            let c = self.spc.step(&mut self.io);
            self.acc -= c as i64 * 24 * self.master_hz as i64;
        }
    }

    /// Debug/test access to bytes written by CPU destined for SPC700.
    pub fn cpu_to_apu(&self, i: usize) -> u8 {
        self.io.port_in[i & 3]
    }

    /// Debug/test: set a byte that the CPU will read from this port.
    pub fn debug_set_from_apu(&mut self, i: usize, v: u8) {
        self.io.port_out[i & 3] = v;
    }

    /// Take accumulated audio samples (pairs of L,R).
    pub fn take_audio(&mut self) -> Vec<i16> {
        self.io.dsp.take_audio()
    }

    /// Debug: write bytes to SPC700 RAM without ticking.
    pub fn debug_poke(&mut self, addr: u16, bytes: &[u8]) {
        for (i, &b) in bytes.iter().enumerate() {
            let a = addr.wrapping_add(i as u16) as usize;
            self.io.ram[a] = b;
        }
    }

    /// Debug: read byte from SPC700 RAM.
    pub fn debug_peek(&self, addr: u16) -> u8 {
        self.io.ram[addr as usize]
    }

    /// Debug: set SPC700 program counter.
    pub fn debug_set_pc(&mut self, pc: u16) {
        self.spc.pc = pc;
    }

    /// Debug: get SPC700 program counter.
    pub fn debug_pc(&self) -> u16 {
        self.spc.pc
    }

    /// Debug: write to DSP register directly.
    pub fn debug_dsp_write(&mut self, reg: u8, v: u8) {
        self.io.dsp.write(reg, v);
    }

    /// Debug: read from DSP register directly.
    pub fn debug_dsp_read(&mut self, reg: u8) -> u8 {
        self.io.dsp.read(reg)
    }

    /// SMP register snapshot for debugger: (a, x, y, sp, pc, psw).
    pub fn smp_regs(&self) -> (u8, u8, u8, u8, u16, u8) {
        (self.spc.a, self.spc.x, self.spc.y, self.spc.sp, self.spc.pc, self.spc.psw)
    }

    /// Hash APU state into the given hasher.
    pub fn hash_into(&self, h: &mut crate::hash::Fnv64) {
        // SPC700 registers
        h.write_u8(self.spc.a);
        h.write_u8(self.spc.x);
        h.write_u8(self.spc.y);
        h.write_u8(self.spc.sp);
        h.write_u16(self.spc.pc);
        h.write_u8(self.spc.psw);
        if self.spc.halted {
            h.write_u8(1);
        } else {
            h.write_u8(0);
        }
        // APU state
        h.write_u64(self.acc as u64);
        h.write_u64(self.last_master);
        // I/O state
        self.io.hash_into(h);
    }
}

impl State for Apu {
    fn save(&self, w: &mut StateWriter) {
        // Save all fields in declaration order.
        // not saved: master_hz (derived from region via set_region())
        self.spc.save(w);
        self.io.save(w);
        w.i64(self.acc);
        w.u64(self.last_master);
    }

    fn load(&mut self, r: &mut StateReader) -> Result<(), StateError> {
        // Load all fields in declaration order.
        self.spc.load(r)?;
        self.io.load(r)?;
        self.acc = r.i64()?;
        self.last_master = r.u64()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::savestate::{State, StateWriter, StateReader};
    use crate::hash::Fnv64;

    fn apu_hash(apu: &Apu) -> u64 {
        let mut h = Fnv64::new();
        apu.hash_into(&mut h);
        h.finish()
    }

    #[test]
    fn e08_11_apu_roundtrip() {
        // Set up Apu with non-trivial state.
        let mut apu = Apu::new();
        apu.acc = 1234567;
        apu.last_master = 9876543;

        // Save.
        let mut writer = StateWriter::new();
        apu.save(&mut writer);
        let data = writer.finish();

        // Load into fresh instance.
        let mut loaded = Apu::new();
        let mut reader = StateReader::new(&data);
        loaded.load(&mut reader).unwrap();

        // Verify bytes match on re-save.
        let mut writer2 = StateWriter::new();
        loaded.save(&mut writer2);
        let data2 = writer2.finish();
        assert_eq!(data, data2, "re-saved bytes must match original");

        // Verify hash matches.
        assert_eq!(apu_hash(&apu), apu_hash(&loaded), "hashes must match after roundtrip");
    }

    #[test]
    fn e08_11_apu_behaves_identically() {
        // Run to 100k, save/restore, then both advance to 300k.
        let mut apu = Apu::new();
        apu.step_to(100_000);

        // Save state.
        let mut writer = StateWriter::new();
        apu.save(&mut writer);
        let data = writer.finish();

        // Load into fresh instance.
        let mut loaded = Apu::new();
        let mut reader = StateReader::new(&data);
        loaded.load(&mut reader).unwrap();

        // Both advance to 300k.
        apu.step_to(300_000);
        loaded.step_to(300_000);

        assert_eq!(apu_hash(&apu), apu_hash(&loaded), "hashes must match after advancing");
        for i in 0..4 {
            assert_eq!(apu.port_read(i), loaded.port_read(i), "port {} must match", i);
        }
    }

    #[test]
    fn e08_11_ipl_handshake_survives() {
        // Run to IPL handshake, save/restore, verify port values.
        let mut apu = Apu::new();
        apu.step_to(100_000);

        // Save state.
        let mut writer = StateWriter::new();
        apu.save(&mut writer);
        let data = writer.finish();

        // Load into fresh instance.
        let mut loaded = Apu::new();
        let mut reader = StateReader::new(&data);
        loaded.load(&mut reader).unwrap();

        assert_eq!(loaded.port_read(0), 0xAA, "port 0 must be 0xAA after restore");
        assert_eq!(loaded.port_read(1), 0xBB, "port 1 must be 0xBB after restore");
    }

    #[test]
    fn e08_11_truncated_apu() {
        let mut apu = Apu::new();
        apu.acc = 1234567;
        apu.last_master = 9876543;

        let mut writer = StateWriter::new();
        apu.save(&mut writer);
        let data = writer.finish();

        // Test truncation at various points.
        for n in [0, data.len() / 2, data.len() - 1] {
            let mut loaded = Apu::new();
            let mut reader = StateReader::new(&data[..n]);
            let result = loaded.load(&mut reader);
            assert!(matches!(result, Err(StateError::Truncated)),
                "truncation at {} should fail with Truncated", n);
        }
    }

    #[test]
    fn e08_11_truncated_each() {
        // Combined truncation test for both ApuIo and Apu.
        let mut io = ApuIo::new();
        for i in 0..65536 {
            io.ram[i] = (i & 0xFF) as u8;
        }

        let mut writer = StateWriter::new();
        io.save(&mut writer);
        let data = writer.finish();

        for n in [0, data.len() / 2, data.len() - 1] {
            let mut loaded = ApuIo::new();
            let mut reader = StateReader::new(&data[..n]);
            assert!(matches!(loaded.load(&mut reader), Err(StateError::Truncated)),
                "io truncation at {} should fail", n);
        }

        let mut apu = Apu::new();
        apu.acc = 1234567;
        apu.last_master = 9876543;

        let mut writer2 = StateWriter::new();
        apu.save(&mut writer2);
        let data2 = writer2.finish();

        for n in [0, data2.len() / 2, data2.len() - 1] {
            let mut loaded = Apu::new();
            let mut reader = StateReader::new(&data2[..n]);
            assert!(matches!(loaded.load(&mut reader), Err(StateError::Truncated)),
                "apu truncation at {} should fail", n);
        }
    }
}
