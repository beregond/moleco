use palette::{FromColor, Hsv, Srgb};

pub struct Color {
    pub hue: u32,
    pub srgb: Srgb<u8>,
}

pub struct Scheme {
    pub primary: Color,
    pub first_accent: Color,
    pub second_accent: Color,
    pub complementary: Color,
}

fn to_color(hue: u32) -> Srgb<u8> {
    Srgb::from_color(Hsv::new(hue as f32, 0.7, 0.9)).into_format()
}

impl Scheme {
    pub fn new(primary: u32, first_accent: u32, second_accent: u32, complementary: u32) -> Self {
        Self {
            primary: Color {
                hue: primary,
                srgb: to_color(primary),
            },
            first_accent: Color {
                hue: first_accent,
                srgb: to_color(first_accent),
            },
            second_accent: Color {
                hue: second_accent,
                srgb: to_color(second_accent),
            },
            complementary: Color {
                hue: complementary,
                srgb: to_color(complementary),
            },
        }
    }
}
