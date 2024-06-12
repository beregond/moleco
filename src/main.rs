use clap::{arg, command, Parser, Subcommand};
use clap_verbosity_flag::{Verbosity, WarnLevel};
use image::{ImageBuffer, Rgba};
use log::{debug, info};
use moleco::{calculate_scheme, generate_moleco};
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
        base_size: u32,
        /// Print image to terminal.
        #[arg(long, default_value = "false")]
        print: bool,
        #[arg(long, default_value = "false")]
        /// Print image to terminal only, without saving.
        print_only: bool,
        #[arg(long, default_value = "moleco.png")]
        /// Output filename.
        output: String,
        #[arg(long, default_value = "1")]
        /// Border size in percent points of base size.
        border_size: u32,
        #[arg(long, default_value = "false")]
        /// Allow to skip MInChI version check (currently only 0.00.1S is supported).
        skip_minchi_version_check: bool,
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
        // set offset in terminal
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
            output,
            border_size,
            skip_minchi_version_check,
        } => {
            let picture = generate_moleco(
                substance.to_string(),
                base_size.clone(),
                border_size.clone(),
                !skip_minchi_version_check,
            );
            match picture {
                Ok(mut picture) => {
                    let buffer = picture.generate();
                    if *print || *print_only {
                        print_to_terminal(buffer.clone());
                    }
                    if !*print_only {
                        buffer.save(output).unwrap();
                        info!("Image saved as {}", output);
                    }
                }
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(exitcode::USAGE);
                }
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
                "No InChI provided, only payload starting with 'InChI=' is supported for calculation. Error source: {}",
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
                let palette = calculate_scheme(substance.to_string());
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
                let palette = calculate_scheme(substance.to_string());
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
                let palette = calculate_scheme(substance.to_string());
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
                let palette = calculate_scheme(substance.to_string());
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
