use std::io::Write;

use crate::driver::encode::Encode;

#[derive(Debug, Clone, Copy)]
pub struct ErrorInformation1 {
    pub no_media_when_printing: bool,
    pub end_of_media: bool,
    pub tape_cutter_jam: bool,
    pub main_unit_in_use: bool,
    pub fan_doesnt_work: bool,
}

impl ErrorInformation1 {
    const NO_MEDIA_WHEN_PRINTING: u8 = 0x01;
    const END_OF_MEDIA: u8 = 0x02;
    const TAPE_CUTTER_JAM: u8 = 0x04;
    const MAIN_UNIT_IN_USE: u8 = 0x10;
    const FAN_DOESNT_WORK: u8 = 0x80;

    pub fn from_bits(bits: u8) -> Self {
        ErrorInformation1 {
            no_media_when_printing: bits & Self::NO_MEDIA_WHEN_PRINTING != 0,
            end_of_media: bits & Self::END_OF_MEDIA != 0,
            tape_cutter_jam: bits & Self::TAPE_CUTTER_JAM != 0,
            main_unit_in_use: bits & Self::MAIN_UNIT_IN_USE != 0,
            fan_doesnt_work: bits & Self::FAN_DOESNT_WORK != 0,
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct ErrorInformation2 {
    pub transmission_error: bool,
    pub cover_opened_while_printing: bool,
    pub cannot_feed: bool,
    pub system_error: bool,
}

impl ErrorInformation2 {
    const TRANSMISSION_ERROR: u8 = 0x04;
    const COVER_OPENED_WHILE_PRINTING: u8 = 0x10;
    const CANNOT_FEED: u8 = 0x40;
    const SYSTEM_ERROR: u8 = 0x80;

    pub fn from_bits(bits: u8) -> Self {
        ErrorInformation2 {
            transmission_error: bits & Self::TRANSMISSION_ERROR != 0,
            cover_opened_while_printing: bits & Self::COVER_OPENED_WHILE_PRINTING != 0,
            cannot_feed: bits & Self::CANNOT_FEED != 0,
            system_error: bits & Self::SYSTEM_ERROR != 0,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MediaType {
    NoMedia = 0x00,
    Continuous = 0x0A,
    DieCutLabels = 0x0B,
}

impl Encode for MediaType {
    fn encode(&self, buf: impl Write) -> std::io::Result<()> {
        (*self as u8).encode(buf)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StatusType {
    ReplyToStatusRequest,
    PrintingCompleted,
    Error,
    Notification,
    PhaseChange,
}

#[derive(Debug, Clone, Copy)]
pub enum PhaseState {
    Waiting,
    Printing,
}

#[derive(Debug, Clone, Copy)]
pub struct PrinterStatus {
    pub media_width: u8,
    pub media_length: u8,
    pub media_type: MediaType,
    pub error1: ErrorInformation1,
    pub error2: ErrorInformation2,
    pub status_type: StatusType,
    pub phase_state: PhaseState,
}

#[derive(Clone, Copy)]
pub enum PrinterCommandMode {
    /// ESC/P mode (normal)
    EscpNormal = 0x00, // WARNING: THE PDF DOCUMENTATION IS BROKEN AND DOES NOT HAVE THIS VALUES
    /// Raster mode (default)
    Raster = 0x01,
    /// ESC/P mode (text) for QL-650TD
    EscpText = 0x02,
    /// P-touch Template mode for QL-580N/1050/1060N
    PtouchTemplate = 0x03,
}

impl crate::driver::encode::Encode for PrinterCommandMode {
    fn encode(&self, buf: impl Write) -> std::io::Result<()> {
        (*self as u8).encode(buf)
    }
}

#[derive(Clone, Copy)]
pub struct PrinterMode {
    /// Auto cut (QL550/560/570/580N/650TD/700/1050/1060N)
    auto_cut: bool,
}

impl Encode for PrinterMode {
    fn encode(&self, buf: impl Write) -> std::io::Result<()> {
        [(self.auto_cut as u8) << 6].encode(buf)
    }
}

#[derive(Clone, Copy)]
pub struct PrinterExpandedMode {
    /// Cut at end (Earlier version of QL-650TD firmware is not supported.)
    cut_at_end: bool,
    /// High resolution printing (QL-570/580N/700)
    high_resolution_printing: bool,
}

impl Encode for PrinterExpandedMode {
    fn encode(&self, buf: impl Write) -> std::io::Result<()> {
        [(self.cut_at_end as u8) << 4 | (self.high_resolution_printing as u8) << 6].encode(buf)
    }
}
