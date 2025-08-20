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

// pub struct PrinterSettings {
//     high_dpi: bool,
//     print_width: u32,
// }

// impl PrinterSettings {
//     pub const QL_500: PrinterSettings = PrinterSettings {
//         high_dpi: false,
//         print_width: 720,
//     };
// }

// pub struct PrintJob {
//     data: bitvec::vec::BitVec<u8, Msb0>,
//     settings: PrintSettings,
// }

// impl PrintJob {
//     pub fn rasterize_image(image: ::image::DynamicImage, settings: PrintSettings) -> PrintJob {
//         let width = std::cmp::min(image.width(), settings.print_width);
//         let height =
//             image.height() * width / image.width() * settings.high_dpi.then_some(2).unwrap_or(1);

//         let mut image = image.resize(width, height, FilterType::Lanczos3);
//         image.apply_orientation(Orientation::FlipHorizontal);

//         let mut bg = ImageBuffer::from_pixel(settings.print_width, image.height(), Rgba([255; 4]));
//         ::image::imageops::overlay(&mut bg, &image, 0, 0);

//         let mut image = ::image::imageops::grayscale(&bg);

//         // let gamma_correction = 1.0;
//         // image.pixels_mut().for_each(|x| {
//         //     x.0 = [(255.0 * (x.0[0] as f32 / 255.0).powf(1.0 / gamma_correction)) as u8]
//         // });

//         ::image::imageops::dither(&mut image, &BiLevel);

//         image.save("./test.png").unwrap();
//         unreachable!();

//         let dithered: BitVec<u8, Msb0> = image.pixels().map(|&Luma([a])| a == 0).collect();

//         PrintJob {
//             data: dithered,
//             settings,
//         }
//     }
// }

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

        let mut ship = self.printer.send_transfer(RasterTransfer)?;
        for line in 0..lines {
            let index = (line * job.width) as usize;
            let line = &job.image[index..(index + job.width as usize)];

            ship.push_bits(line);
            ship.fill_remaining(720);
            ship.send_line()?;
        }

        ship.end();

        self.printer.send_command(PrintWithFeeding)?;

        Ok(())
    }
}
