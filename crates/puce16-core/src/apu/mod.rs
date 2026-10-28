pub mod spc700;
pub mod dsp;
pub mod timer;
pub mod ipl;
pub mod io;

use crate::consts::{APU_RESONATOR_HZ, MASTER_HZ_NTSC};
use crate::region::Region;
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
        for &b in bytes {
            let a = addr as usize;
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
