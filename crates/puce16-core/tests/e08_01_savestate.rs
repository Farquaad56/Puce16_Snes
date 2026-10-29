use puce16_core::savestate::*;

#[test]
fn e08_01_little_endian_bytes() {
    let mut w = StateWriter::new();
    w.u16(0x1234);
    w.u32(0x01020304);
    w.u64(0x0102030405060708);
    let data = w.finish();
    assert_eq!(data, [
        0x34, 0x12,
        0x04, 0x03, 0x02, 0x01,
        8, 7, 6, 5, 4, 3, 2, 1
    ]);
}

#[test]
fn e08_01_signed_and_bool() {
    let mut w = StateWriter::new();
    w.i16(-2);
    w.i32(-3);
    w.i64(-4);
    w.bool(true);
    w.bool(false);
    let data = w.finish();
    assert_eq!(data, [
        0xFE, 0xFF,
        0xFD, 0xFF, 0xFF, 0xFF,
        0xFC, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        1, 0
    ]);

    let mut r = StateReader::new(&data);
    assert_eq!(r.i16().unwrap(), -2);
    assert_eq!(r.i32().unwrap(), -3);
    assert_eq!(r.i64().unwrap(), -4);
    assert_eq!(r.bool().unwrap(), true);
    assert_eq!(r.bool().unwrap(), false);
}

#[test]
fn e08_01_roundtrip_all() {
    let mut w = StateWriter::new();
    w.u8(42);
    w.u16(0x1234);
    w.u32(0xDEADBEEF);
    w.u64(0x0102030405060708);
    w.i16(-100);
    w.i32(-200);
    w.i64(-300);
    w.bool(true);
    w.bool(false);
    w.bytes(&[1, 2, 3]);
    w.slice_u16(&[0x1111, 0x2222]);
    w.slice_u32(&[7, 8]);
    w.vec_u8(&[9, 9]);
    w.vec_u16(&[5]);
    let data = w.finish();

    let mut r = StateReader::new(&data);
    assert_eq!(r.u8().unwrap(), 42);
    assert_eq!(r.u16().unwrap(), 0x1234);
    assert_eq!(r.u32().unwrap(), 0xDEADBEEF);
    assert_eq!(r.u64().unwrap(), 0x0102030405060708);
    assert_eq!(r.i16().unwrap(), -100);
    assert_eq!(r.i32().unwrap(), -200);
    assert_eq!(r.i64().unwrap(), -300);
    assert_eq!(r.bool().unwrap(), true);
    assert_eq!(r.bool().unwrap(), false);

    let mut bytes = [0u8; 3];
    r.bytes_into(&mut bytes).unwrap();
    assert_eq!(bytes, [1, 2, 3]);

    let mut slice16 = [0u16; 2];
    r.slice_u16_into(&mut slice16).unwrap();
    assert_eq!(slice16, [0x1111, 0x2222]);

    let mut slice32 = [0u32; 2];
    r.slice_u32_into(&mut slice32).unwrap();
    assert_eq!(slice32, [7, 8]);

    assert_eq!(r.vec_u8().unwrap(), vec![9, 9]);
    assert_eq!(r.vec_u16().unwrap(), vec![5]);
    r.finish().unwrap();
}

#[test]
fn e08_01_slice_layouts() {
    let mut w = StateWriter::new();
    w.slice_u16(&[0x0102]);
    assert_eq!(w.finish(), [2, 1]);

    let mut w = StateWriter::new();
    w.slice_u32(&[1]);
    assert_eq!(w.finish(), [1, 0, 0, 0]);

    let mut w = StateWriter::new();
    w.vec_u8(&[7, 8]);
    assert_eq!(w.finish(), [2, 0, 0, 0, 7, 8]);

    let mut w = StateWriter::new();
    w.vec_u16(&[0x0A0B]);
    assert_eq!(w.finish(), [1, 0, 0, 0, 0x0B, 0x0A]);
}

#[test]
fn e08_01_truncated_scalars() {
    let mut r = StateReader::new(&[1]);
    assert_eq!(r.u16(), Err(StateError::Truncated));

    let mut r = StateReader::new(&[]);
    assert_eq!(r.u8(), Err(StateError::Truncated));

    let mut r = StateReader::new(&[0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(r.u64(), Err(StateError::Truncated));
}

#[test]
fn e08_01_truncated_bytes_into() {
    let mut r = StateReader::new(&[1, 2, 3]);
    let mut buf = [0u8; 4];
    assert_eq!(r.bytes_into(&mut buf), Err(StateError::Truncated));

    let mut r = StateReader::new(&[1, 2, 3]);
    let mut buf = [0u16; 2];
    assert_eq!(r.slice_u16_into(&mut buf), Err(StateError::Truncated));
}

#[test]
fn e08_01_vec_huge_length() {
    let mut r = StateReader::new(&[0xFF, 0xFF, 0xFF, 0xFF, 1]);
    assert_eq!(r.vec_u8(), Err(StateError::Truncated));
}

#[test]
fn e08_01_bool_invalid() {
    let mut r = StateReader::new(&[2]);
    assert_eq!(r.bool(), Err(StateError::Mismatch("bool")));
}

#[test]
fn e08_01_finish_trailing() {
    let mut w = StateWriter::new();
    w.u8(1);
    w.u8(2);
    let data = w.finish();

    let mut r = StateReader::new(&data);
    r.u8().unwrap();
    assert_eq!(r.finish(), Err(StateError::Mismatch("trailing data")));

    let mut r = StateReader::new(&data);
    r.u8().unwrap();
    r.u8().unwrap();
    r.finish().unwrap();
}

#[test]
fn e08_01_display() {
    assert_eq!(format!("{}", StateError::Truncated), "state truncated");
    assert_eq!(format!("{}", StateError::Version(3)), "unsupported state version 3");
    assert_eq!(format!("{}", StateError::Mismatch("region")), "state mismatch: region");
}

#[test]
fn e08_01_trait_object_free_roundtrip() {
    #[derive(PartialEq, Debug)]
    struct P { a: u16, b: bool }

    impl State for P {
        fn save(&self, w: &mut StateWriter) {
            w.u16(self.a);
            w.bool(self.b);
        }
        fn load(&mut self, r: &mut StateReader) -> Result<(), StateError> {
            self.a = r.u16()?;
            self.b = r.bool()?;
            Ok(())
        }
    }

    let p = P { a: 0x4242, b: true };
    let mut w = StateWriter::new();
    p.save(&mut w);
    let data = w.finish();

    let mut q = P { a: 0, b: false };
    let mut r = StateReader::new(&data);
    q.load(&mut r).unwrap();
    assert_eq!(p, q);
}

#[test]
fn e08_01_version_constant() {
    assert_eq!(STATE_VERSION, 1);
}
