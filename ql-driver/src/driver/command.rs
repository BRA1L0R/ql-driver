use crate::{driver::Printer, error::QlDriverError};

#[macro_export]
macro_rules! implement_basic_command {
    ($name:tt, $data:expr) => {
        pub struct $name;

        impl Command for $name {
            fn send_command(&self, printer: &mut Printer) -> Result<(), QlDriverError> {
                printer.write(&$data).map_err(Into::into)
            }
        }
    };
}

#[macro_export]
macro_rules! command_segment {
    ($self:expr, $buf:expr, $segment:ident) => {
        $self
            .$segment
            .encode(&mut $buf)
            .expect("no space in buffer");
    };

    ($self:expr, $buf:expr, $segment:expr) => {
        $buf.write_all(&[$segment]).expect("no space in buffer");
    };
}

#[macro_export]
macro_rules! implement_command_args {
    ($name:tt, ($($argname:ident: $argty:ty),+) => [$($segment:tt),+]) => {
        pub struct $name {
            $($argname: $argty),+
        }

        impl $name {
            pub fn new($($argname: $argty),+) -> Self {
                Self { $($argname),+ }
            }
        }

        impl Command for $name {
            fn send_command(&self, printer: &mut Printer) -> Result<(), QlDriverError> {
                use std::io::Write;
                use crate::driver::encode::Encode;

                let mut buf = [0u8; 64];
                let mut cursor = std::io::Cursor::<&mut [u8]>::new(buf.as_mut());

                $(command_segment!(self, cursor, $segment);)+

                let written = cursor.position();
                printer.write(&buf[..(written as usize)])?;

                Ok(())
            }
        }
    };
}
pub trait CommandResponse: Command {
    type Response;
    fn read_response(&self, printer: &mut Printer) -> Result<Self::Response, QlDriverError>;
}

pub trait Command {
    fn send_command(&self, printer: &mut Printer) -> Result<(), QlDriverError>;
}
