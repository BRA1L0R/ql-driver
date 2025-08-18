pub trait Encode<const SIZE: usize> {
    // const SIZE: usize;
    fn encode(self) -> [u8; SIZE];
}

impl<const SIZE: usize> Encode<SIZE> for [u8; SIZE] {
    fn encode(self) -> [u8; SIZE] {
        self
    }
}

#[macro_export]
macro_rules! encode_integers {
    ($integer:ty, $size:expr) => {
        impl crate::driver::encode::Encode<$size> for $integer {
            // const SIZE: usize = $size;
            fn encode(self) -> [u8; $size] {
                self.to_le_bytes()
            }
        }
    };
}

encode_integers!(u8, 1);
encode_integers!(u16, 2);
encode_integers!(u32, 4);
encode_integers!(u64, 8);
encode_integers!(i8, 1);
encode_integers!(i16, 2);
encode_integers!(i32, 4);
encode_integers!(i64, 8);

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
