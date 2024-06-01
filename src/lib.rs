pub mod layouts;
pub mod tokenize;
use crate::layouts::Picture;
use log::{info, trace};
use num_bigint::BigUint;
use num_traits::Zero;
use palette::{FromColor, Hsv, Srgb};
use sha2::{Digest, Sha512};

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

pub fn modulo(divident: &BigUint, divisor: u32) -> u32 {
    let rest = divident % BigUint::from(divisor);
    let mut result: u32 = 0;
    // Since rest if always below 360, much below u32::MAX, we can safely convert it this way. I think.
    for i in rest.iter_u32_digits() {
        result += i;
    }
    result
}

pub fn generate_scheme(substance: String) -> Scheme {
    let mut hasher = Sha512::new();
    hasher.update(substance);
    let result = hasher.finalize();
    let mut sum: BigUint = Zero::zero();
    for i in result.iter() {
        sum <<= 8;
        let step = i.clone() as u64;
        sum += step;
    }
    trace!("Substance hash: {}", sum);
    let primary_hue = modulo(&sum, 360);
    let complementary_hue = primary_hue + 165 + modulo(&sum, 30);
    let first_accent_hue = primary_hue + modulo(&sum, (complementary_hue - 5) - (primary_hue + 5));
    let second_accent_hue =
        complementary_hue + modulo(&sum, (primary_hue + 355) - (complementary_hue + 5));
    // Normalization of hues
    let complementary_hue = complementary_hue % 360;
    let first_accent_hue = first_accent_hue % 360;
    let second_accent_hue = second_accent_hue % 360;

    let scheme = Scheme::new(
        primary_hue,
        first_accent_hue,
        second_accent_hue,
        complementary_hue,
    );
    info!("Primary hue: {}", scheme.primary.hue);
    info!("Complementary hue: {}", scheme.complementary.hue);
    info!("First accent hue: {}", scheme.first_accent.hue);
    info!("Second accent hue: {}", scheme.second_accent.hue);
    scheme
}

pub fn generate_for_inchi(substance: String, mut picture: Picture) -> Picture {
    let scheme = generate_scheme(substance[6..].to_string());

    picture.add_scheme(scheme);
    picture
}

pub fn generate_for_minchi(substance: String, mut picture: Picture) -> Picture {
    let mut chunks: Vec<&str> = substance.split('/').collect();
    if chunks.len() < 4 {
        eprintln!("MInChI must have at least 4 parts separated by '/'.");
        std::process::exit(exitcode::USAGE);
    }
    // Popping from the end, order is flipped
    let concentration = chunks.pop().unwrap();
    let indexing = chunks.pop().unwrap();

    chunks.remove(0);
    let structure = chunks.join("/");
    let molecules: Vec<&str> = structure.split('&').collect();
    for molecule in molecules {
        let scheme = generate_scheme(molecule.to_string());
        picture.add_scheme(scheme);
    }

    picture.add_ic_info(indexing.to_string(), concentration.to_string());
    picture
}
