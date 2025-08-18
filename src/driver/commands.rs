use crate::{
    driver::{Printer, types::*},
    error::PrinterError,
};

use super::encode::Encode;

macro_rules! implement_basic_command {
    ($name:tt, $data:expr) => {
        pub struct $name;

        impl Command for $name {
            fn send_command(&self, printer: &mut Printer) -> Result<(), PrinterError> {
                printer.write(&$data).map_err(Into::into)
            }
        }
    };
}

macro_rules! command_segment {
    ($self:expr, $buf:expr, $segment:ident) => {
        $buf.write(&$self.$segment.encode())
            .expect("no space in buffer");
    };

    ($self:expr, $buf:expr, $segment:expr) => {
        $buf.write(&[$segment]).expect("no space in buffer");
    };
}

macro_rules! implement_command_args {
    ($name:tt, ($($argname:ident: $argty:ty),+) => [$($segment:tt),+]) => {
        pub struct $name {
            $($argname: $argty),+
        }

        impl $name {
            pub fn new($($argname: $argty),+) -> Self {
                Self {
                    $($argname),+
                }
            }
        }

        impl Command for $name {
            fn send_command(&self, printer: &mut Printer) -> Result<(), PrinterError> {
                use std::io::Write;

                let mut buf = [0u8; 64];
                let mut cursor = std::io::Cursor::<&mut [u8]>::new(buf.as_mut());

                $(
                    command_segment!(self, cursor, $segment);
                )+

                let written = cursor.position();
                printer.write(&buf[..(written as usize)])?;

                Ok(())
            }
        }
    };
}

macro_rules! implement_commands {
    ($($name:tt => $data:expr),+) => {
        $(implement_basic_command!($name, $data);)+
    };
}

pub trait CommandResponse: Command {
    type Response;
    fn read_response(&self, printer: &mut Printer) -> Result<Self::Response, PrinterError>;
}

pub trait Command {
    fn send_command(&self, printer: &mut Printer) -> Result<(), PrinterError>;
}

implement_commands! {
    Reset => [0x00; 200],
    Invalid => [0x00],
    Initialize => [0x1b, 0x40],
    StatusInfoRequest => [0x1b, 0x69, 0x53],
    SetCompressionMode => [0x4d, 0x00],
    ZeroRasterGraphics => [0x5A],
    Print => [0x0c],
    PrintWithFeeding => [0x1A]
}

implement_command_args!(SetCommandMode, (mode: PrinterCommandMode) => [0x1B, 0x69, 0x61, mode]);
implement_command_args!(SetMarginAmount, (margin: u16) => [0x1b, 0x69, 0x64, margin]);
implement_command_args!(SetBaudRate, (baud_rate: u16) => [0x1b, 0x69, 0x42, baud_rate]);
implement_command_args!(SetPrintInformation, (status: PrinterStatus, line_count: i32) => [0x1b, 0x69, 0x7a, (0x02 | 0x04 | 0x08 | 0x40 | 0x80), status, line_count, 1, 0]);
implement_command_args!(SetMode, (printer_mode: PrinterMode) => [0x1b, 0x69, 0x4d, printer_mode]);
implement_command_args!(SetExpandedMode, (printer_mode: PrinterExpandedMode) => [0x1b, 0x69, 0x4d, printer_mode]);

impl CommandResponse for StatusInfoRequest {
    type Response = PrinterStatus;
    fn read_response(&self, printer: &mut Printer) -> Result<Self::Response, PrinterError> {
        let res = printer.read(32)?;
        assert!(res[0] == 0x80);
        assert!(res[1] == 0x20);

        let media_type = match res[11] {
            0x00 => MediaType::NoMedia,
            0x0A => MediaType::Continuous,
            0x0B => MediaType::DieCutLabels,
            _ => return Err(PrinterError::BadData("unknown media type")),
        };

        let status_type = match res[18] {
            0x00 => StatusType::ReplyToStatusRequest,
            0x01 => StatusType::PrintingCompleted,
            0x02 => StatusType::Error,
            0x05 => StatusType::Notification,
            0x06 => StatusType::PhaseChange,
            _ => return Err(PrinterError::BadData("unknown status type")),
        };

        let phase_state = match res[19] {
            0x00 => PhaseState::Waiting,
            0x01 => PhaseState::Printing,
            _ => return Err(PrinterError::BadData("unknown phase status")),
        };

        Ok(PrinterStatus {
            media_width: res[10],
            media_type,
            media_length: res[17],
            error1: ErrorInformation1::from_bits(res[8]),
            error2: ErrorInformation2::from_bits(res[9]),
            status_type,
            phase_state,
        })
    }
}
// // impl Command {}

pub struct RasterGraphicsTransfer<'a> {
    data: &'a [u8],
}

impl<'a> RasterGraphicsTransfer<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, PrinterError> {
        (data.len() < u8::MAX as usize)
            .then_some(Self { data })
            .ok_or(PrinterError::WrongDataSize)
    }
}

impl Command for RasterGraphicsTransfer<'_> {
    fn send_command(&self, printer: &mut Printer) -> Result<(), PrinterError> {
        let size: u8 = self.data.len().try_into().unwrap(); // already checked when creating the struct

        printer.write(&[0x67, 0x00, size])?;
        printer.write(self.data)?;

        Ok(())
    }
}
