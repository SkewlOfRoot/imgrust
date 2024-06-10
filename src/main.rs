use clap::{Args, Parser, Subcommand};
use colored::{self, Colorize};
use imgrust::compress_image_files;
use std::{path::PathBuf, process};

#[derive(Parser)]
#[clap(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Looks for media files recursively.
    #[arg(short, long)]
    recursive: bool,

    /// The output pattern specifies how the output file name will be constructed.
    /// Example:
    /// input file name: foo.jpg |
    /// output-pattern: "X-c" |
    /// output file name: foo-c.jpg
    #[arg(long)]
    output_pattern: Option<String>,

    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compresses the found jpg files.
    Compress(CompressArgs),
}

#[derive(Args)]
struct CompressArgs {
    input_folder: PathBuf,
    output_folder: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    match &cli.commands {
        Commands::Compress(args) => {
            let output_folder: PathBuf = match &args.output_folder {
                Some(val) => val.to_path_buf(),
                None => {
                    if cli.output_pattern.is_none() {
                        eprintln!(
                            "{}",
                            "When omitting the output folder an output pattern must be specified."
                                .red()
                        );
                        process::exit(1);
                    }
                    args.input_folder.clone()
                }
            };

            if let Err(e) = compress_image_files(
                &args.input_folder,
                &output_folder,
                &cli.output_pattern.unwrap_or_default(),
                cli.recursive,
            ) {
                eprintln!("Application error {}", e);
                process::exit(1);
            }
        }
    }
}
