use bitvec::{order::Msb0, vec::BitVec};
use exoquant::ditherer::FloydSteinberg;
use image::{
    GenericImageView, ImageBuffer, Luma, Rgba,
    imageops::{BiLevel, FilterType},
    metadata::Orientation,
};

use crate::{
    driver::PrinterCommander,
    error::QlDriverError,
    prelude::{
        Initialize, MediaType, PrintWithFeeding, PrinterCommandMode, RasterGraphicsTransfer, Reset,
        SetCommandMode, SetMarginAmount, SetPrintInformation, StatusInfoRequest,
    },
};

pub mod driver;
pub mod error;

pub mod prelude {
    pub use crate::driver::PrinterCommander;
    pub use crate::driver::commands::*;
    pub use crate::driver::types::*;
}

pub struct PrintSettings {
    high_dpi: bool,
    print_width: u32,
}

impl PrintSettings {
    pub const QL_500: PrintSettings = PrintSettings {
        high_dpi: false,
        print_width: 720,
    };
}

pub struct PrintJob {
    data: bitvec::vec::BitVec<u8, Msb0>,

    length: u32,
    width: u32,

    settings: PrintSettings,
}

impl PrintJob {
    pub fn rasterize_image(image: image::DynamicImage, settings: PrintSettings) -> PrintJob {
        // let width = std::cmp::min(image.width(), settings.print_width);
        let width = settings.print_width;
        let height =
            image.height() * width / image.width() * settings.high_dpi.then_some(2).unwrap_or(1);

        let mut image = image.resize(width, height, FilterType::Lanczos3);
        image.apply_orientation(Orientation::FlipVertical);

        let mut bg = ImageBuffer::from_pixel(image.width(), image.height(), Rgba([255; 4]));
        image::imageops::overlay(&mut bg, &image, 0, 0);

        let mut image = image::imageops::grayscale(&bg);

        let gamma_correction = 5.14;
        image.pixels_mut().for_each(|x| {
            x.0 = [(255.0 * (x.0[0] as f32 / 255.0).powf(1.0 / gamma_correction)) as u8]
        });

        image::imageops::dither(&mut image, &BiLevel);

        let dithered: BitVec<u8, Msb0> = image.pixels().map(|&Luma([a])| dbg!(a) == 0).collect();

        PrintJob {
            data: dithered,
            length: image.height(),
            width: image.width(),
            settings,
        }
    }
}

pub struct Printer {
    printer: PrinterCommander,
}

impl Printer {
    pub fn open(path: &str) -> Result<Self, QlDriverError> {
        let printer = PrinterCommander::main(path)?;
        Ok(Self { printer })
    }

    pub fn print(&mut self, job: &PrintJob) -> Result<(), QlDriverError> {
        self.printer.send_command(Reset)?;
        self.printer.send_command(Initialize)?;
        self.printer
            .send_command(SetCommandMode::new(PrinterCommandMode::Raster))?;

        self.printer.send_command(SetPrintInformation::new(
            MediaType::Continuous,
            62,
            job.length,
        ))?;

        self.printer.send_command(SetMarginAmount::new(0))?;

        for line in 0..job.length {
            let index = (line * job.width) as usize / 8;
            let line = &job.data.as_raw_slice()[index..(index + job.width as usize / 8)];

            println!("Sending {}", line.len());

            self.printer
                .send_command(RasterGraphicsTransfer::new(line)?)?;
        }

        self.printer.send_command(PrintWithFeeding)?;

        let status = self.printer.send_command_read(StatusInfoRequest)?;
        dbg!(status);

        Ok(())
    }
}
