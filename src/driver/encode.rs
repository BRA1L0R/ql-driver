use std::io::Write;

pub trait Encode {
    // const SIZE: usize;
    fn encode(&self, buf: impl Write) -> std::io::Result<()>;
}

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

encode_integers!(u8);
encode_integers!(u16);
encode_integers!(u32);
encode_integers!(u64);
encode_integers!(i8);
encode_integers!(i16);
encode_integers!(i32);
encode_integers!(i64);

// #[macro_export]
// macro_rules! encode_enum {
//     ($enum:ident, $repr:ty, $size:expr) => {
//         impl crate::driver::encode::Encode<$size> for $enum {
//             // const SIZE: usize = $repr::SIZE;
//             fn encode(self) -> [u8; $size] {
//                 (self as $repr).encode()
//             }
//         }
//     };
// }
