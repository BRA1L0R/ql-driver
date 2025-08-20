use bitvec::{order::Msb0, vec::BitVec};
use exoquant::ditherer::FloydSteinberg;
use image::{GenericImageView, Rgba, imageops::FilterType, metadata::Orientation};

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
        // let image = image::imageops::resize(&image, width, height, FilterType::Lanczos3);

        // let white = Rgba([255; 4]);
        // let white_bg = image::ImageBuffer::from_pixel(image.width(), image.height(), white);

        // image::imageops::dither

        use exoquant::*;

        const PALETTE: [Color; 2] = [
            Color {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            },
            Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
        ];

        let ditherer = FloydSteinberg::vanilla();
        let cs = SimpleColorSpace::default();
        let remapper = exoquant::Remapper::new(&PALETTE, &cs, &ditherer);

        fn gamma_correct(input: u8, gamma: f64) -> u8 {
            ((input as f64 / 255.0).powf(1.0 / gamma) * 255.0) as u8
        }

        fn map_gamma(gamma: f64) -> impl Fn((u32, u32, Rgba<u8>)) -> Color {
            move |(_, _, Rgba([r, g, b, a]))| {
                Color::new(
                    gamma_correct(r, gamma),
                    gamma_correct(g, gamma),
                    gamma_correct(b, gamma),
                    a,
                )
            }
        }

        let gamma = 5.14;
        let image_iterator = image.pixels().map(map_gamma(gamma));

        let image_iterator = Box::new(image_iterator);
        let dithered: BitVec<u8, Msb0> = remapper
            .remap_iter(image_iterator, image.width() as usize)
            .map(|a| a == 0)
            .collect();

        PrintJob {
            data: dithered,
            width: image.width(),
            length: image.height(),
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
