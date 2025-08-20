use crate::{
    command_segment,
    driver::{
        PrinterLink,
        command::{Command, CommandResponse},
    },
    error::QlDriverError,
    implement_basic_command, implement_command_args,
    prelude::*,
};

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
    fn read_response(&self, printer: &mut PrinterLink) -> Result<Self::Response, QlDriverError> {
        let res = printer.read(32)?;
        assert!(res[0] == 0x80);
        assert!(res[1] == 0x20);

        let media_type = match res[11] {
            0x00 => MediaType::NoMedia,
            0x0A => MediaType::Continuous,
            0x0B => MediaType::DieCutLabels,
            _ => return Err(QlDriverError::BadData("unknown media type")),
        };

        let status_type = match res[18] {
            0x00 => StatusType::ReplyToStatusRequest,
            0x01 => StatusType::PrintingCompleted,
            0x02 => StatusType::Error,
            0x05 => StatusType::Notification,
            0x06 => StatusType::PhaseChange,
            _ => return Err(QlDriverError::BadData("unknown status type")),
        };

        let phase_state = match res[19] {
            0x00 => PhaseState::Waiting,
            0x01 => PhaseState::Printing,
            _ => return Err(QlDriverError::BadData("unknown phase status")),
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

pub struct RasterGraphicsTransfer<'a> {
    data: &'a [u8],
}

impl<'a> RasterGraphicsTransfer<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, QlDriverError> {
        (data.len() < u8::MAX as usize)
            .then_some(Self { data })
            .ok_or(QlDriverError::WrongDataSize)
    }
}

impl Command for RasterGraphicsTransfer<'_> {
    fn send_command(&self, printer: &mut PrinterLink) -> Result<(), QlDriverError> {
        let size: u8 = self.data.len().try_into().unwrap(); // already checked when creating the struct

        printer.write(&[0x67, 0x00, size])?;
        printer.write(self.data)?;

        Ok(())
    }
}
