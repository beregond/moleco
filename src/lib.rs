pub mod layouts;
pub mod tokenize;
use crate::layouts::Picture;
use crate::tokenize::generate_mixture_tree;
use log::{debug, info};
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

pub fn generate_moleco(
    payload: String,
    base_size: u32,
    border_size_percent_points: u32,
    strict_version_check: bool,
) -> Result<Picture, String> {
    if payload.starts_with("InChI=") {
        if !payload.starts_with("InChI=1S/") && strict_version_check {
            return Err(
                "Only InChI version 1S is supported for now, you may pass flag to skip it."
                    .to_string(),
            );
        }
        Ok(generate_for_inchi(
            payload,
            base_size,
            border_size_percent_points,
        )?)
    } else if payload.starts_with("MInChI=") {
        if !payload.starts_with("MInChI=0.00.1S/") && strict_version_check {
            return Err(
                "Only MInChI version 0.00.1S is supported for now, you may pass flag to skip it."
                    .to_string(),
            );
        }
        Ok(generate_for_minchi(
            payload,
            base_size,
            border_size_percent_points,
        )?)
    } else if payload.starts_with("InChIKey=") || payload.starts_with("MInChIKey=") {
        Err("Keys are not supported. Check readme for more info.".to_string())
    } else {
        Err("No InChI or MInChI provided".to_string())
    }
}

pub fn calculate_scheme(substance: String) -> Scheme {
    let substance = match substance {
        s if s.starts_with("InChI=") => s[6..].to_string(),
        s => s,
    };
    info!("Substance: {}", substance);

    let mut hasher = Sha512::new();
    hasher.update(substance);
    let result = hasher.finalize();
    debug!(" -> Raw hash: {:?}", result);
    let mut sum: BigUint = Zero::zero();
    for i in result.iter() {
        sum <<= 8;
        let step = i.clone() as u64;
        sum += step;
    }
    info!(" -> Substance hash: {}", sum);

    let primary_hue = modulo(&sum, 360);
    let complementary_hue = primary_hue + 165 + modulo(&sum, 30);
    let first_accent_hue = primary_hue + modulo(&sum, (complementary_hue - 5) - (primary_hue + 5));
    let second_accent_hue =
        complementary_hue + modulo(&sum, (primary_hue + 355) - (complementary_hue + 5));

    // Normalization of hues, as they can go over 360
    let complementary_hue = complementary_hue % 360;
    let first_accent_hue = first_accent_hue % 360;
    let second_accent_hue = second_accent_hue % 360;

    let scheme = Scheme::new(
        primary_hue,
        first_accent_hue,
        second_accent_hue,
        complementary_hue,
    );
    info!(
        " -> Hues, primary: {}, complementary: {}, first accent: {}, second accent: {}",
        scheme.primary.hue,
        scheme.complementary.hue,
        scheme.first_accent.hue,
        scheme.second_accent.hue
    );
    scheme
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

pub fn generate_for_inchi(
    substance: String,
    base_size: u32,
    border_size_percent_points: u32,
) -> Result<Picture, String> {
    let (actual_size, actual_border_size) = check_sizes(base_size, border_size_percent_points)?;
    let scheme = calculate_scheme(substance);

    Ok(Picture::new(
        actual_size,
        actual_border_size,
        vec![scheme],
        None,
    ))
}

pub fn generate_for_minchi(
    substance: String,
    base_size: u32,
    border_size_percent_points: u32,
) -> Result<Picture, String> {
    let (actual_size, actual_border_size) = check_sizes(base_size, border_size_percent_points)?;
    let mut chunks: Vec<&str> = substance.split('/').collect();
    if chunks.len() < 4 {
        return Err("MInChI must have at least 4 parts separated by '/'.".to_string());
    }
    // Popping concentration, THEN indexing, order is flipped if you start from the end
    let concentration = chunks.pop().unwrap();
    let indexing = chunks.pop().unwrap();
    let mixture_info = Some(generate_mixture_tree(indexing, concentration)?);

    // Drop version chunk
    chunks.remove(0);

    let schemes = chunks
        .join("/")
        .split('&')
        .map(|molecule| calculate_scheme(molecule.to_string()))
        .collect();

    Ok(Picture::new(
        actual_size,
        actual_border_size,
        schemes,
        mixture_info,
    ))
}

fn check_sizes(base_size: u32, border_size_percent_points: u32) -> Result<(u32, u32), String> {
    if base_size < 16 {
        return Err("Base size must be bigger than 16 pixels.".to_string());
    }

    let mut actual_border_size =
        (base_size as f32 * border_size_percent_points as f32 / 100.0) as u32;
    if actual_border_size % 2 == 0 {
        actual_border_size += 1;
    }
    debug!("Calculated border size: {}", actual_border_size);

    let actual_size: u32;
    if base_size % 2 == 0 {
        actual_size = base_size + 1;
    } else {
        actual_size = base_size;
    }

    debug!("Calculated base size: {}", actual_size);

    Ok((actual_size, actual_border_size))
}
