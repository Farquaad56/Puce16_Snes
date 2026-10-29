pub const STATE_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StateError {
    Truncated,
    Version(u32),
    Mismatch(&'static str),
}

impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateError::Truncated => write!(f, "state truncated"),
            StateError::Version(n) => write!(f, "unsupported state version {}", n),
            StateError::Mismatch(what) => write!(f, "state mismatch: {}", what),
        }
    }
}

impl std::error::Error for StateError {}

pub struct StateWriter {
    buf: Vec<u8>,
}

impl StateWriter {
    pub fn new() -> StateWriter {
        StateWriter { buf: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn finish(self) -> Vec<u8> {
        self.buf
    }

    pub fn u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    pub fn u16(&mut self, v: u16) {
        let bytes = v.to_le_bytes();
        self.buf.extend_from_slice(&bytes);
    }

    pub fn u32(&mut self, v: u32) {
        let bytes = v.to_le_bytes();
        self.buf.extend_from_slice(&bytes);
    }

    pub fn u64(&mut self, v: u64) {
        let bytes = v.to_le_bytes();
        self.buf.extend_from_slice(&bytes);
    }

    pub fn i16(&mut self, v: i16) {
        let bytes = v.to_le_bytes();
        self.buf.extend_from_slice(&bytes);
    }

    pub fn i32(&mut self, v: i32) {
        let bytes = v.to_le_bytes();
        self.buf.extend_from_slice(&bytes);
    }

    pub fn i64(&mut self, v: i64) {
        let bytes = v.to_le_bytes();
        self.buf.extend_from_slice(&bytes);
    }

    pub fn bool(&mut self, v: bool) {
        if v {
            self.buf.push(1);
        } else {
            self.buf.push(0);
        }
    }

    pub fn bytes(&mut self, v: &[u8]) {
        self.buf.extend_from_slice(v);
    }

    pub fn slice_u16(&mut self, v: &[u16]) {
        for &val in v {
            let bytes = val.to_le_bytes();
            self.buf.extend_from_slice(&bytes);
        }
    }

    pub fn slice_u32(&mut self, v: &[u32]) {
        for &val in v {
            let bytes = val.to_le_bytes();
            self.buf.extend_from_slice(&bytes);
        }
    }

    pub fn vec_u8(&mut self, v: &[u8]) {
        self.u32(v.len() as u32);
        self.bytes(v);
    }

    pub fn vec_u16(&mut self, v: &[u16]) {
        self.u32(v.len() as u32);
        self.slice_u16(v);
    }
}

pub struct StateReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> StateReader<'a> {
    pub fn new(data: &'a [u8]) -> StateReader<'a> {
        StateReader { data, pos: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    fn check(&mut self, needed: usize) -> Result<(), StateError> {
        if self.remaining() < needed {
            Err(StateError::Truncated)
        } else {
            Ok(())
        }
    }

    pub fn u8(&mut self) -> Result<u8, StateError> {
        self.check(1)?;
        let v = self.data[self.pos];
        self.pos += 1;
        Ok(v)
    }

    pub fn u16(&mut self) -> Result<u16, StateError> {
        self.check(2)?;
        let bytes = [self.data[self.pos], self.data[self.pos + 1]];
        self.pos += 2;
        Ok(u16::from_le_bytes(bytes))
    }

    pub fn u32(&mut self) -> Result<u32, StateError> {
        self.check(4)?;
        let bytes = [
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ];
        self.pos += 4;
        Ok(u32::from_le_bytes(bytes))
    }

    pub fn u64(&mut self) -> Result<u64, StateError> {
        self.check(8)?;
        let mut bytes = [0u8; 8];
        for i in 0..8 {
            bytes[i] = self.data[self.pos + i];
        }
        self.pos += 8;
        Ok(u64::from_le_bytes(bytes))
    }

    pub fn i16(&mut self) -> Result<i16, StateError> {
        let v = self.u16()?;
        Ok(v as i16)
    }

    pub fn i32(&mut self) -> Result<i32, StateError> {
        let v = self.u32()?;
        Ok(v as i32)
    }

    pub fn i64(&mut self) -> Result<i64, StateError> {
        let v = self.u64()?;
        Ok(v as i64)
    }

    pub fn bool(&mut self) -> Result<bool, StateError> {
        let v = self.u8()?;
        if v > 1 {
            return Err(StateError::Mismatch("bool"));
        }
        Ok(v == 1)
    }

    pub fn bytes_into(&mut self, out: &mut [u8]) -> Result<(), StateError> {
        self.check(out.len())?;
        for i in 0..out.len() {
            out[i] = self.data[self.pos + i];
        }
        self.pos += out.len();
        Ok(())
    }

    pub fn slice_u16_into(&mut self, out: &mut [u16]) -> Result<(), StateError> {
        for slot in out.iter_mut() {
            *slot = self.u16()?;
        }
        Ok(())
    }

    pub fn slice_u32_into(&mut self, out: &mut [u32]) -> Result<(), StateError> {
        for slot in out.iter_mut() {
            *slot = self.u32()?;
        }
        Ok(())
    }

    pub fn vec_u8(&mut self) -> Result<Vec<u8>, StateError> {
        let len = self.u32()? as usize;
        if len > self.remaining() {
            return Err(StateError::Truncated);
        }
        let mut v = Vec::with_capacity(len);
        for _ in 0..len {
            v.push(self.u8()?);
        }
        Ok(v)
    }

    pub fn vec_u16(&mut self) -> Result<Vec<u16>, StateError> {
        let len = self.u32()? as usize;
        if len * 2 > self.remaining() {
            return Err(StateError::Truncated);
        }
        let mut v = Vec::with_capacity(len);
        for _ in 0..len {
            v.push(self.u16()?);
        }
        Ok(v)
    }

    pub fn finish(self) -> Result<(), StateError> {
        if self.remaining() > 0 {
            Err(StateError::Mismatch("trailing data"))
        } else {
            Ok(())
        }
    }
}

pub trait State {
    fn save(&self, w: &mut StateWriter);
    fn load(&mut self, r: &mut StateReader) -> Result<(), StateError>;
}
