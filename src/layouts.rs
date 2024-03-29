use crate::common::{Picture, Scheme};
use image::{ImageBuffer, Rgba};

pub struct Rhombus {
    size: u32,
    scheme: Scheme,
}

impl Picture for Rhombus {
    fn new(size: u32, scheme: Scheme) -> Self {
        Self { size, scheme }
    }

    fn generate(&self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let mut buffer = ImageBuffer::new(self.size, self.size);
        let primary = self.scheme.primary.srgb;
        let first_accent = self.scheme.first_accent.srgb;
        let second_accent = self.scheme.second_accent.srgb;
        let complementary = self.scheme.complementary.srgb;
        for (x, y, pixel) in buffer.enumerate_pixels_mut() {
            if x < 100 && y < 100 {
                *pixel = Rgba([
                    primary.red as u8,
                    primary.green as u8,
                    primary.blue as u8,
                    255u8,
                ])
            } else if x < 100 && y > 100 {
                *pixel = Rgba([
                    first_accent.red as u8,
                    first_accent.green as u8,
                    first_accent.blue as u8,
                    255u8,
                ])
            } else if x > 100 && y < 100 {
                *pixel = Rgba([
                    second_accent.red as u8,
                    second_accent.green as u8,
                    second_accent.blue as u8,
                    255u8,
                ])
            } else if x > 100 && y > 100 {
                *pixel = Rgba([
                    complementary.red as u8,
                    complementary.green as u8,
                    complementary.blue as u8,
                    255u8,
                ])
            } else {
                *pixel = Rgba([0u8, 0u8, 0u8, 255u8])
            }
        }
        buffer
    }

    fn get_height(&self) -> u32 {
        self.size.clone()
    }
}
