#![warn(missing_docs)]

use crate::{driver::PrinterCommander, image::PrintableImage, prelude::*};

/// Low level primitives to communicate with the printer
pub mod driver;
/// Builder pattern structures to convert any image format to printable bitmaps
pub mod image;
// pub mod image;

mod error;
pub use error::*;

/// Importable prelude for often reused structures
pub mod prelude {
    pub use crate::driver::command::*;
    pub use crate::driver::types::*;
}

/// High-level interface for communicating with the printer
pub struct Printer {
    // print_buffer: BitVec<u8, Msb0>,
    printer: PrinterCommander,
}

impl Printer {
    /// Open a char device at the specified `path`
    pub fn open(path: &str) -> Result<Self, QlError> {
        let mut printer = PrinterCommander::main(path)?;

        printer.send_command(Reset)?;
        printer.send_command(Initialize)?;

        Ok(Self {
            printer,
            // print_buffer: BitVec::new(),
        })
    }

    pub fn print_image(&mut self, job: &PrintableImage) -> Result<(), QlError> {
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
