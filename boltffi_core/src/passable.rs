use crate::types::FfiBuf;
use crate::types::FfiSpan;
use crate::wire::{WireDecode, WireEncode};

pub unsafe trait Passable: Sized {
    type In;
    type Out;
    unsafe fn unpack(input: Self::In) -> Self;
    fn pack(self) -> Self::Out;
}

macro_rules! impl_passable_scalar {
    ($($ty:ty),*) => {
        $(
            unsafe impl Passable for $ty {
                type In = $ty;
                type Out = $ty;
                unsafe fn unpack(input: $ty) -> Self { input }
                fn pack(self) -> $ty { self }
            }
        )*
    };
}

impl_passable_scalar!(i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, bool, usize, isize);

unsafe impl Passable for String {
    type In = FfiSpan;
    type Out = FfiBuf<u8>;

    unsafe fn unpack(input: FfiSpan) -> Self {
        let bytes = unsafe { input.as_bytes() };
        core::str::from_utf8(bytes)
            .expect("invalid UTF-8 in FfiSpan")
            .to_string()
    }

    fn pack(self) -> FfiBuf<u8> {
        FfiBuf::from_vec(self.into_bytes())
    }
}

pub unsafe trait WirePassable: WireEncode + WireDecode + Sized {}

unsafe impl<T: WirePassable> Passable for T {
    type In = FfiSpan;
    type Out = FfiBuf<u8>;

    unsafe fn unpack(input: FfiSpan) -> Self {
        let bytes = unsafe { input.as_bytes() };
        crate::wire::decode(bytes).expect("wire decode failed in Passable::unpack")
    }

    fn pack(self) -> FfiBuf<u8> {
        FfiBuf::wire_encode(&self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primitive_roundtrip() {
        let value: i32 = 42;
        let packed = value.pack();
        let unpacked = unsafe { i32::unpack(packed) };
        assert_eq!(unpacked, 42);
    }

    #[test]
    fn bool_roundtrip() {
        assert!(unsafe { bool::unpack(true.pack()) });
        assert!(!unsafe { bool::unpack(false.pack()) });
    }

    #[test]
    fn string_pack() {
        let value = String::from("hello");
        let buf = value.pack();
        assert_eq!(buf.len(), 5);
    }

    #[test]
    fn string_roundtrip() {
        let original = String::from("hello world");
        let bytes = original.as_bytes();
        let span = FfiSpan {
            ptr: bytes.as_ptr(),
            len: bytes.len(),
        };
        let recovered = unsafe { String::unpack(span) };
        assert_eq!(recovered, "hello world");
    }
}
