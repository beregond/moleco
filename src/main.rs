use clap::{arg, command, Parser, Subcommand};
use clap_verbosity_flag::{Verbosity, WarnLevel};
use dialoguer::Confirm;
use image::{ImageBuffer, Rgba};
use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;
use log::{debug, error, info};
use moleco::{calculate_scheme, generate_moleco};
use num::integer::gcd;
use pretty_env_logger;
use prettytable::{row, Table};
use std::fs;
use std::io::{BufRead, BufReader};
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
        /// Output filename. Only PNG format is supported.
        output_file: String,
        #[arg(long, default_value = "false")]
        /// When output file exists, overwrite it without asking.
        overwrite: bool,
        #[arg(long, default_value = "1")]
        /// Border size in percent points of base size.
        border_size: u32,
        #[arg(long, default_value = "false")]
        /// Skip version check.
        skip_version_check: bool,
    },
    /// Calculate and print color scheme without generating image.
    Calculate {
        /// Substances to calculate. Providing input file has precedence over this.
        substances: Vec<String>,
        #[arg(long, value_enum, default_value_t)]
        format: Format,
        #[arg(long)]
        /// Read substances from file, one per line.
        input_file: Option<String>,
        #[arg(long)]
        /// Output file. (Doesn't work for table format, but you can redirect output.)
        output_file: Option<String>,
        #[arg(long, default_value = "false")]
        /// Skip version check (calculate anyway) and totally skip lines that do not start with InChI.
        skip_errors: bool,
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
            output_file,
            overwrite,
            border_size,
            skip_version_check,
        } => {
            let picture = generate_moleco(
                substance.to_string(),
                base_size.clone(),
                border_size.clone(),
                !skip_version_check,
            );
            match picture {
                Ok(mut picture) => {
                    if !*print_only && !output_file.ends_with(".png") {
                        error!("Only PNG format is supported.");
                        std::process::exit(exitcode::USAGE);
                    }

                    let buffer = picture.generate();
                    let width = buffer.width();
                    let height = buffer.height();
                    info!("Image size: {}x{}", width, height);
                    let divisor = gcd(width, height);
                    info!(
                        "Image aspect ratio: {}:{}",
                        width / divisor,
                        height / divisor
                    );
                    if *print || *print_only {
                        print_to_terminal(buffer.clone());
                    }
                    if !*print_only {
                        if !output_file.ends_with(".png") {
                            error!("Only PNG format is supported.");
                            std::process::exit(exitcode::USAGE);
                        }
                        if file_exists(output_file) && !overwrite {
                            if !Confirm::new()
                                .with_prompt(format!(
                                    "File \"{}\" already exists, overwrite?",
                                    output_file
                                ))
                                .interact()
                                .unwrap()
                            {
                                std::process::exit(exitcode::OK);
                            }
                        }
                        buffer.save(output_file).unwrap();
                        let image_path = std::path::Path::new(output_file);
                        let mut metadata = Metadata::new();
                        metadata.set_tag(ExifTag::ImageDescription(substance.to_string()));
                        metadata.write_to_file(&image_path).unwrap();
                        info!("Image saved as {}", output_file);
                    }
                }
                Err(e) => {
                    error!("{}", e);
                    std::process::exit(exitcode::USAGE);
                }
            }
        }
        Commands::Calculate {
            substances,
            format,
            input_file,
            output_file,
            skip_errors,
        } => {
            if let Some(path) = output_file {
                match format {
                    Format::Table => {
                        error!("Output file is not supported for table format.");
                        std::process::exit(exitcode::USAGE);
                    }
                    _ => {}
                }

                if file_exists(path) {
                    if !Confirm::new()
                        .with_prompt(format!("File \"{}\" already exists, overwrite?", path))
                        .interact()
                        .unwrap()
                    {
                        std::process::exit(exitcode::OK);
                    }
                }
            }
            let mut writer =
                DataWriter::new(format.clone(), output_file.clone(), skip_errors.clone());
            match input_file {
                Some(path) => {
                    if !file_exists(path) {
                        error!("File \"{}\" does not exist", path);
                        std::process::exit(exitcode::USAGE);
                    }

                    debug!("Reading from file {:?}", path);

                    if is_file_empty(path) {
                        error!("File \"{}\" is empty", path);
                        std::process::exit(exitcode::USAGE);
                    }

                    let file = std::fs::File::open(path);
                    if file.is_err() {
                        error!("Error reading file \"{}\"", path);
                        std::process::exit(exitcode::USAGE);
                    }
                    let file = file.unwrap();
                    let reader = BufReader::new(file);

                    debug!("Output generation started");
                    for line in reader.lines() {
                        if let Ok(substance) = line {
                            if let Err(message) = writer.write(substance) {
                                error!("{}", message);
                                std::process::exit(exitcode::USAGE);
                            }
                        }
                    }
                }
                None => {
                    if substances.is_empty() {
                        error!("No substances provided");
                        std::process::exit(exitcode::USAGE);
                    } else {
                        debug!("Output generation started");
                        for substance in substances {
                            if let Err(message) = writer.write(substance.clone()) {
                                error!("{}", message);
                                std::process::exit(exitcode::USAGE);
                            }
                        }
                    }
                }
            }
            writer.flush();
        }
    }
}

struct DataWriter {
    skip_errors: bool,
    actual_writer: Box<dyn Writer>,
}
impl DataWriter {
    fn new(format: Format, output_file: Option<String>, skip_errors: bool) -> Self {
        DataWriter {
            skip_errors,
            actual_writer: match format {
                Format::Table => {
                    if output_file.is_some() {
                        unreachable!();
                    }
                    Box::new(TableWriter::new())
                }
                Format::Json => Box::new(JsonWriter::new(output_file)),
                Format::Yaml => Box::new(YamlWriter::new(output_file)),
                Format::Csv => match output_file {
                    Some(path) => Box::new(CsvFileWriter::new(path)),
                    None => Box::new(CsvStdoutWriter::new()),
                },
            },
        }
    }
    fn write(&mut self, substance: String) -> Result<(), String> {
        if !substance.starts_with("InChI=1S/") && !self.skip_errors {
            return Err(format!(
                "Only InChI version 1S is supported for now, you may pass flag to skip it. Error received: {}",
                substance
            ));
        }

        if substance.starts_with("InChI=") {
            self.actual_writer.write(substance);
        } else {
            if !self.skip_errors {
                return Err(format!(
                    "No InChI provided, only payload starting with 'InChI=' is supported for calculation. Error source: {}",
                    substance
                ));
            }
        }
        Ok(())
    }
    fn flush(&mut self) {
        self.actual_writer.flush();
    }
}

trait Writer {
    fn write(&mut self, substance: String);
    fn flush(&mut self);
}

struct TableWriter {
    table: Table,
}
impl TableWriter {
    fn new() -> Self {
        let mut table = Table::new();
        table.add_row(row![
            "Substance",
            "Primary hue",
            "First accent hue",
            "Second accent hue",
            "Complementary hue"
        ]);
        TableWriter { table }
    }
}

impl Writer for TableWriter {
    fn write(&mut self, substance: String) {
        let palette = calculate_scheme(substance.to_string());
        self.table.add_row(row![
            substance,
            palette.primary.hue,
            palette.first_accent.hue,
            palette.second_accent.hue,
            palette.complementary.hue
        ]);
    }
    fn flush(&mut self) {
        self.table.printstd();
    }
}
struct JsonWriter {
    path: Option<String>,
    doc_root: serde_json::Map<String, serde_json::Value>,
}

impl JsonWriter {
    fn new(path: Option<String>) -> Self {
        let doc_root = serde_json::Map::new();
        JsonWriter { path, doc_root }
    }
}

impl Writer for JsonWriter {
    fn write(&mut self, substance: String) {
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
        self.doc_root
            .insert(substance.to_string(), serde_json::Value::Object(sub_json));
    }
    fn flush(&mut self) {
        match self.path {
            Some(ref path) => {
                let file = fs::File::create(path).unwrap();
                serde_json::to_writer_pretty(file, &self.doc_root).unwrap();
                info!("Image saved as {}", path);
            }
            None => {
                println!("{}", serde_json::to_string_pretty(&self.doc_root).unwrap());
            }
        }
    }
}

struct YamlWriter {
    path: Option<String>,
    doc_root: serde_yaml::Mapping,
}

impl YamlWriter {
    fn new(path: Option<String>) -> Self {
        let doc_root = serde_yaml::Mapping::new();
        YamlWriter { path, doc_root }
    }
}

impl Writer for YamlWriter {
    fn write(&mut self, substance: String) {
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
        self.doc_root.insert(
            serde_yaml::Value::String(substance.to_string()),
            serde_yaml::Value::Mapping(sub_yaml),
        );
    }
    fn flush(&mut self) {
        match self.path {
            Some(ref path) => {
                let file = fs::File::create(path).unwrap();
                serde_yaml::to_writer(file, &self.doc_root).unwrap();
                info!("Image saved as {}", path);
            }
            None => {
                println!("{}", serde_yaml::to_string(&self.doc_root).unwrap());
            }
        }
    }
}
struct CsvStdoutWriter {
    output: csv::Writer<std::io::Stdout>,
}

impl CsvStdoutWriter {
    fn new() -> Self {
        let mut output = csv::Writer::from_writer(std::io::stdout());
        output
            .write_record(&[
                "Substance",
                "Primary hue",
                "First accent hue",
                "Second accent hue",
                "Complementary hue",
            ])
            .unwrap();
        CsvStdoutWriter { output }
    }
}

impl Writer for CsvStdoutWriter {
    fn write(&mut self, substance: String) {
        let palette = calculate_scheme(substance.to_string());
        self.output
            .write_record(&[
                &substance,
                &palette.primary.hue.to_string(),
                &palette.first_accent.hue.to_string(),
                &palette.second_accent.hue.to_string(),
                &palette.complementary.hue.to_string(),
            ])
            .unwrap();
    }
    fn flush(&mut self) {
        self.output.flush().unwrap();
    }
}

struct CsvFileWriter {
    output: csv::Writer<fs::File>,
}

impl CsvFileWriter {
    fn new(path: String) -> Self {
        let file = fs::File::create(path).unwrap();
        let mut output = csv::Writer::from_writer(file);
        output
            .write_record(&[
                "Substance",
                "Primary hue",
                "First accent hue",
                "Second accent hue",
                "Complementary hue",
            ])
            .unwrap();
        CsvFileWriter { output }
    }
}

impl Writer for CsvFileWriter {
    fn write(&mut self, substance: String) {
        let palette = calculate_scheme(substance.to_string());
        self.output
            .write_record(&[
                &substance,
                &palette.primary.hue.to_string(),
                &palette.first_accent.hue.to_string(),
                &palette.second_accent.hue.to_string(),
                &palette.complementary.hue.to_string(),
            ])
            .unwrap();
    }
    fn flush(&mut self) {
        self.output.flush().unwrap();
    }
}

fn file_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}

fn is_file_empty(file_path: &str) -> bool {
    if let Ok(metadata) = fs::metadata(file_path) {
        return metadata.len() == 0;
    }
    false
}
