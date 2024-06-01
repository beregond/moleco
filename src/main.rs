use clap::{arg, command, Parser, Subcommand};
use clap_verbosity_flag::{Verbosity, WarnLevel};
use image::{ImageBuffer, Rgba};
use log::debug;
use moleco::layouts::Picture;
use moleco::{generate_for_inchi, generate_for_minchi, generate_scheme};
use pretty_env_logger;
use prettytable::{row, Table};
use viuer::Config;

#[derive(clap::ValueEnum, Clone, Default, Debug)]
pub enum Format {
    #[default]
    Table,
    Json,
    Yaml,
    Csv,
}

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
        #[arg(default_value_t = 200, long)]
        base_size: u16,
        /// Print image to terminal.
        #[arg(long, default_value = "false")]
        print: bool,
        #[arg(long, default_value = "false")]
        /// Print image to terminal only, without saving.
        print_only: bool,
        #[arg(long, default_value = "output.png")]
        /// Output filename.
        filename: String,
        #[arg(long, default_value = "1")]
        /// Border size in percent points of base size.
        border_size: u32,
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

fn print_to_terminal(buffer: ImageBuffer<Rgba<u8>, Vec<u8>>) {
    let height = buffer.height();
    let img = image::DynamicImage::ImageRgba8(buffer);
    let conf = Config {
        // set offset
        x: 10,
        y: height as i16,
        ..Default::default()
    };
    viuer::print(&img, &conf).expect("Image printing failed.");
}

fn main() {
    let cli = Cli::parse();

    pretty_env_logger::formatted_builder()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    match &cli.command {
        Commands::Generate {
            substance,
            base_size,
            print,
            print_only,
            filename,
            border_size,
        } => {
            debug!("Print: {}", print);
            debug!("Print only: {}", print_only);

            if *base_size < 16 {
                eprintln!("Base size must be bigger than 16 pixels.");
                std::process::exit(exitcode::USAGE);
            }

            let mut actual_b_size = (*base_size as f32 * *border_size as f32 / 100.0) as u32;
            if actual_b_size % 2 == 0 {
                actual_b_size += 1;
            }
            debug!("Border size: {}", actual_b_size);
            let actual_size: u16;
            if base_size % 2 == 0 {
                actual_size = *base_size + 1;
            } else {
                actual_size = *base_size;
            }

            let picture = Picture::new(actual_size as u32, actual_b_size);

            if substance.starts_with("InChI=") {
                let buffer = generate_for_inchi(substance.to_string(), picture).generate();
                if *print || *print_only {
                    print_to_terminal(buffer.clone());
                }
                if !*print_only {
                    buffer.save(filename).unwrap();
                    println!("Image saved as {}", filename);
                }
            } else if substance.starts_with("MInChI=") {
                let buffer = generate_for_minchi(substance.to_string(), picture).generate();

                if *print || *print_only {
                    print_to_terminal(buffer.clone());
                }
                if !*print_only {
                    buffer.save(filename).unwrap();
                    println!("Image saved as {}", filename);
                }
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
