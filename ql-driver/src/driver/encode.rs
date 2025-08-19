use std::io::Write;

/// defines something that is encodable into the format
/// required for communicating with the printer
pub trait Encode {
    // encode into a buffer
    fn encode(&self, buf: impl Write) -> std::io::Result<()>;
}

// encoding slices is just writing them as is into the buffer
impl Encode for &[u8] {
    fn encode(&self, mut buf: impl Write) -> std::io::Result<()> {
        buf.write_all(&self)
    }
}

impl<const N: usize> Encode for [u8; N] {
    fn encode(&self, buf: impl Write) -> std::io::Result<()> {
        self.as_ref().encode(buf)
    }
}

#[macro_export]
macro_rules! encode_integers {
    ($integer:ty) => {
        impl crate::driver::encode::Encode for $integer {
            // const SIZE: usize = $size;
            fn encode(&self, mut buf: impl Write) -> std::io::Result<()> {
                let encoded = self.to_le_bytes();
                buf.write_all(&encoded)
            }
        }
    };
}

// default implementations for integer types
encode_integers!(u8);
encode_integers!(u16);
encode_integers!(u32);
encode_integers!(u64);
encode_integers!(i8);
encode_integers!(i16);
encode_integers!(i32);
encode_integers!(i64);
