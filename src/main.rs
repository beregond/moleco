mod common;
mod layouts;

use crate::common::{Picture, Scheme};
use crate::layouts::Rhombus;
use clap::{arg, command, Parser, Subcommand};
use clap_verbosity_flag::{Verbosity, WarnLevel};
use log::{debug, info, trace};
use num_bigint::BigUint;
use num_traits::Zero;
use pretty_env_logger;
use prettytable::{row, Table};
use sha2::{Digest, Sha512};
use viuer::Config;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[command(flatten)]
    verbose: Verbosity<WarnLevel>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate color scheme image for a given substance.
    Generate {
        substance: String,
        #[arg(long, default_value = "false")]
        print: bool,
        #[arg(default_value_t = 200, long)]
        size: u16,
        #[arg(long, value_enum, default_value_t)]
        layout: Layout,
        #[arg(long, default_value = "true")]
        print_only: bool,
        #[arg(long, default_value = "output.png")]
        filename: String,
    },
    /// Calculate and print color scheme without generating image.
    Calculate {
        substances: Vec<String>,
        #[arg(long, value_enum, default_value_t)]
        format: Format,
        #[arg(long)]
        /// Read substances from file, one per line. Works if no arguments are passed.
        from_file: Option<String>,
    },
}

#[derive(clap::ValueEnum, Clone, Default, Debug)]
enum Layout {
    #[default]
    Quinqunx,
    Rhombus,
    Stripe,
}

#[derive(clap::ValueEnum, Clone, Default, Debug)]
enum Format {
    #[default]
    Table,
    Json,
    Yaml,
    Csv,
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

fn generate_scheme(substance: String) -> Scheme {
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

    Scheme::new(
        primary_hue,
        first_accent_hue,
        second_accent_hue,
        complementary_hue,
    )
}

fn generate_for_inchi(substance: String, layout: &Layout, size: u16) -> Rhombus {
    let scheme = generate_scheme(substance[6..].to_string());
    info!("Primary hue: {}", scheme.primary.hue);
    info!("Complementary hue: {}", scheme.complementary.hue);
    info!("Second accent hue: {}", scheme.second_accent.hue);
    info!("First accent hue: {}", scheme.first_accent.hue);

    Rhombus::new(size as u32, scheme)
}

fn print_to_terminal(picture: &Rhombus) {
    let imgbuf = picture.generate();
    let img = image::DynamicImage::ImageRgba8(imgbuf);
    let conf = Config {
        // set offset
        x: 10,
        y: picture.get_height() as i16,
        ..Default::default()
    };
    viuer::print(&img, &conf).expect("Image printing failed.");
}

fn generate_for_minchi(substance: String, layout: &Layout, size: u16) {
    println!("MInChI is not supported yet.");
}

fn main() {
    let cli = Cli::parse();

    pretty_env_logger::formatted_builder()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    match &cli.command {
        Commands::Generate {
            substance,
            layout,
            size,
            print,
            print_only,
            filename,
        } => {
            debug!("Layout: {:?}", layout);
            debug!("Print: {}", print);
            if substance.starts_with("InChI=") {
                let pic = generate_for_inchi(substance.to_string(), layout, size.clone());
                if *print || *print_only {
                    print_to_terminal(&pic);
                }
                if !*print_only {
                    let imgbuf = pic.generate();
                    imgbuf.save(filename).unwrap();
                    println!("Image saved as {}", filename);
                }
            } else if substance.starts_with("MInChI=") {
                generate_for_minchi(substance.to_string(), layout, size.clone());
            } else if substance.starts_with("InChIKey=") || substance.starts_with("MInChIKey=") {
                eprintln!("Keys are not supported. Check readme for more info.");
                std::process::exit(exitcode::USAGE);
            } else {
                eprintln!("No InChI or MInChI provided");
                std::process::exit(exitcode::USAGE);
            }
        }
        Commands::Calculate {
            substances,
            format,
            from_file,
        } => {
            if substances.is_empty() {
                match from_file {
                    Some(file) => {
                        debug!("Reading from file {:?}", from_file);
                        let file = std::fs::read_to_string(file).unwrap();
                        let substances: Vec<String> = file.lines().map(|x| x.to_string()).collect();
                        if substances.is_empty() {
                            eprintln!("No substances provided in file");
                            std::process::exit(exitcode::USAGE);
                        }
                        handle_generate(substances, format);
                    }
                    None => {
                        eprintln!("No substances provided");
                        std::process::exit(exitcode::USAGE);
                    }
                }
            } else {
                handle_generate(substances.clone(), format);
            }
        }
    }
}

fn handle_generate(substances: Vec<String>, format: &Format) {
    debug!("Validation started");
    for substance in &substances {
        if !substance.starts_with("InChI=") {
            eprintln!(
                "No InChI provided, only InChI is supported for calculation. Error source: {}",
                substance
            );
            std::process::exit(exitcode::USAGE);
        }
    }

    debug!("Output generation started");
    match format {
        Format::Table => {
            let mut table = Table::new();
            table.add_row(row![
                "Substance",
                "Primary hue",
                "First accent hue",
                "Second accent hue",
                "Complementary hue"
            ]);
            for substance in substances {
                let palette = generate_scheme(substance.to_string());
                table.add_row(row![
                    substance,
                    palette.primary.hue,
                    palette.first_accent.hue,
                    palette.second_accent.hue,
                    palette.complementary.hue
                ]);
            }
            table.printstd();
        }
        Format::Json => {
            let mut json = serde_json::Map::new();
            for substance in substances {
                let palette = generate_scheme(substance.to_string());
                let mut sub_json = serde_json::Map::new();
                sub_json.insert("primary".to_string(), palette.primary.hue.into());
                sub_json.insert("first_accent".to_string(), palette.first_accent.hue.into());
                sub_json.insert(
                    "second_accent".to_string(),
                    palette.second_accent.hue.into(),
                );
                sub_json.insert(
                    "complementary".to_string(),
                    palette.complementary.hue.into(),
                );
                json.insert(substance.to_string(), serde_json::Value::Object(sub_json));
            }
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        Format::Yaml => {
            let mut yaml = serde_yaml::Mapping::new();
            for substance in substances {
                let palette = generate_scheme(substance.to_string());
                let mut sub_yaml = serde_yaml::Mapping::new();
                sub_yaml.insert(
                    serde_yaml::Value::String("primary".to_string()),
                    serde_yaml::Value::Number(serde_yaml::Number::from(palette.primary.hue)),
                );
                sub_yaml.insert(
                    serde_yaml::Value::String("first_accent".to_string()),
                    serde_yaml::Value::Number(serde_yaml::Number::from(palette.first_accent.hue)),
                );
                sub_yaml.insert(
                    serde_yaml::Value::String("second_accent".to_string()),
                    serde_yaml::Value::Number(serde_yaml::Number::from(palette.second_accent.hue)),
                );
                sub_yaml.insert(
                    serde_yaml::Value::String("complementary".to_string()),
                    serde_yaml::Value::Number(serde_yaml::Number::from(palette.complementary.hue)),
                );
                yaml.insert(
                    serde_yaml::Value::String(substance.to_string()),
                    serde_yaml::Value::Mapping(sub_yaml),
                );
            }
            println!("{}", serde_yaml::to_string(&yaml).unwrap());
        }
        Format::Csv => {
            let mut wtr = csv::Writer::from_writer(std::io::stdout());
            wtr.write_record(&[
                "Substance",
                "Primary hue",
                "First accent hue",
                "Second accent hue",
                "Complementary hue",
            ])
            .unwrap();
            for substance in substances {
                let palette = generate_scheme(substance.to_string());
                wtr.write_record(&[
                    &substance,
                    &palette.primary.hue.to_string(),
                    &palette.first_accent.hue.to_string(),
                    &palette.second_accent.hue.to_string(),
                    &palette.complementary.hue.to_string(),
                ])
                .unwrap();
            }
            wtr.flush().unwrap();
        }
    }
}
