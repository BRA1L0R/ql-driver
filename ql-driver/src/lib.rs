use crate::{
    driver::{PrinterCommander, command::Ship, transfer::RasterTransfer},
    error::QlDriverError,
    image::PrintableImage,
    prelude::{
        Initialize, PrintWithFeeding, PrinterCommandMode, Reset, SetCommandMode, SetMarginAmount,
        SetPrintInformation, StatusInfoRequest,
    },
};

pub mod driver;
pub mod error;
pub mod image;
// pub mod image;

pub mod prelude {
    pub use crate::driver::PrinterCommander;
    pub use crate::driver::commands::*;
    pub use crate::driver::types::*;
}

pub struct Printer {
    // print_buffer: BitVec<u8, Msb0>,
    printer: PrinterCommander,
}

impl Printer {
    pub fn open(path: &str) -> Result<Self, QlDriverError> {
        let mut printer = PrinterCommander::main(path)?;

        printer.send_command(Reset)?;
        printer.send_command(Initialize)?;

        Ok(Self {
            printer,
            // print_buffer: BitVec::new(),
        })
    }

    pub fn print_image(&mut self, job: &PrintableImage) -> Result<(), QlDriverError> {
        self.printer
            .send_command(SetCommandMode::new(PrinterCommandMode::Raster))?;

        let status = self.printer.send_command_read(StatusInfoRequest)?;

        let lines = job.image.len() as u32 / job.width;

        self.printer.send_command(SetPrintInformation::new(
            status.media_type,
            status.media_width,
            status.media_length,
            lines,
        ))?;

        self.printer.send_command(SetMarginAmount::new(0))?;

        let mut ship = self.printer.send_transfer(RasterTransfer::new(90))?;
        for line in 0..lines {
            let index = (line * job.width) as usize;
            let line = &job.image[index..(index + job.width as usize)];

            ship.push_line(line);
        }

        ship.send()?;

        self.printer.send_command(PrintWithFeeding)?;

        Ok(())
    }
}
