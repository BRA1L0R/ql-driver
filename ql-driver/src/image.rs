use bitvec::vec::BitVec;
use image::{ImageBuffer, Luma, Rgba, imageops::BiLevel};

pub struct PrintableImage {
    pub(crate) image: BitVec,
    pub(crate) width: u32,
}

pub struct ImageBuilder {
    image: ImageBuffer<Luma<u8>, Vec<u8>>,
}

impl ImageBuilder {
    pub fn open(file: &str) -> image::ImageResult<ImageBuilder> {
        let image = image::open(file)?;

        let mut buffer = ImageBuffer::from_pixel(image.width(), image.height(), Rgba([255; 4]));
        image::imageops::overlay(&mut buffer, &image, 0, 0);

        let mut image = image::imageops::grayscale(&buffer);
        image::imageops::flip_vertical_in_place(&mut image);

        Ok(ImageBuilder { image })
    }

    /// reduces the grayscale image to a dithered
    pub fn dither(&mut self) -> &mut Self {
        image::imageops::dither(&mut self.image, &BiLevel);
        self
    }

    /// get access to the inner image buffer
    pub fn inner_mut(&mut self) -> &mut ImageBuffer<Luma<u8>, Vec<u8>> {
        &mut self.image
    }

    /// applies gamma correction to the image
    pub fn gamma_correction(&mut self, gamma: f64) -> &mut Self {
        self.image
            .pixels_mut()
            .for_each(|x| x.0 = [(255.0 * (x.0[0] as f64 / 255.0).powf(1.0 / gamma)) as u8]);

        self
    }

    /// renders the image into    
    pub fn render(&self) -> PrintableImage {
        let image = self.image.pixels().map(|&Luma([x])| x < 127).collect();

        PrintableImage {
            image,
            width: self.image.width(),
        }
    }
}
