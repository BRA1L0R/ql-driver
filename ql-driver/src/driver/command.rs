use crate::{driver::PrinterLink, error::QlError, prelude::*};

mod macros;
/// bit-optimized low level commands that transfer large data to the printer such as images.
mod transfer;

use macros::*;
pub use transfer::*;

/// Implemented if the [`Command`] expects a reply.
/// In this case, use [`crate::PrinterCommander::send_command_read`] instead, as it will wait on the serial connection for a reply.
///
/// Each Command has a unique `Response` associated to it
/// that [`CommandResponse::read_response`] is responsible to decode
pub trait CommandResponse: Command {
    type Response;

    fn read_response(&self, printer: &mut PrinterLink) -> Result<Self::Response, QlError>;
}

/// Represent a command that can be sent over a [`PrinterLink`]
pub trait Command {
    fn send_command(&self, printer: &mut PrinterLink) -> Result<(), QlError>;
}

implement_basic_command!(Reset, [0x00; 200]);
implement_basic_command!(Invalid, [0x00]);
implement_basic_command!(Initialize, [0x1b, 0x40]);
implement_basic_command!(StatusInfoRequest, [0x1b, 0x69, 0x53]);
implement_basic_command!(SetCompressionMode, [0x4d, 0x00]);
implement_basic_command!(ZeroRasterGraphics, [0x5A]);
implement_basic_command!(Print, [0x0c]);
implement_basic_command!(PrintWithFeeding, [0x1A]);

implement_command_args!(SetCommandMode, (mode: PrinterCommandMode) => [0x1B, 0x69, 0x61, mode]);
implement_command_args!(SetMarginAmount, (margin: u16) => [0x1b, 0x69, 0x64, margin]);
implement_command_args!(SetBaudRate, (baud_rate: u16) => [0x1b, 0x69, 0x42, baud_rate]);
implement_command_args!(
    SetPrintInformation,
    (media_type: MediaType, paper_width: u8, paper_length: u8, raster_lines: u32) => [0x1b, 0x69, 0x7a, (0x02 | 0x04 | 0x08 | 0x40 | 0x80), media_type, paper_width, paper_length, raster_lines, 1, 0]
);
implement_command_args!(SetMode, (printer_mode: PrinterMode) => [0x1b, 0x69, 0x4d, printer_mode]);
implement_command_args!(SetExpandedMode, (printer_mode: PrinterExpandedMode) => [0x1b, 0x69, 0x4d, printer_mode]);

impl CommandResponse for StatusInfoRequest {
    type Response = PrinterStatus;
    fn read_response(&self, printer: &mut PrinterLink) -> Result<Self::Response, QlError> {
        let res = printer.read(32)?;
        assert!(res[0] == 0x80);
        assert!(res[1] == 0x20);

        let media_type = match res[11] {
            0x00 => MediaType::NoMedia,
            0x0A => MediaType::Continuous,
            0x0B => MediaType::DieCutLabels,
            _ => return Err(QlError::BadData("unknown media type")),
        };

        let status_type = match res[18] {
            0x00 => StatusType::ReplyToStatusRequest,
            0x01 => StatusType::PrintingCompleted,
            0x02 => StatusType::Error,
            0x05 => StatusType::Notification,
            0x06 => StatusType::PhaseChange,
            _ => return Err(QlError::BadData("unknown status type")),
        };

        let phase_state = match res[19] {
            0x00 => PhaseState::Waiting,
            0x01 => PhaseState::Printing,
            _ => return Err(QlError::BadData("unknown phase status")),
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
