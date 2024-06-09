use std::{path::PathBuf, process};

use clap::{Args, Parser, Subcommand};

use imgrust::compress_image_files;

#[derive(Parser)]
#[clap(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Looks for media files recursively.
    #[arg(short, long)]
    recursive: bool,

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
    input_folder: Option<PathBuf>,
    output_folder: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();
    match &cli.commands {
        Commands::Compress(args) => {
            if let Err(e) = compress_image_files(
                args.input_folder
                    .as_ref()
                    .expect("Failed to parse to path."),
                args.output_folder
                    .as_ref()
                    .expect("Failed to parse to path."),
                cli.recursive,
            ) {
                eprintln!("Application error {}", e);
                process::exit(1);
            }
        }
    }
}
