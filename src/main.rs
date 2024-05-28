use std::{path::PathBuf, process};

use clap::{Args, Parser, Subcommand};

use imgrust::compress_image_files;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
                args.input_folder.as_ref().expect(""),
                args.output_folder.as_ref().expect(""), // args.input_folder.as_ref().expect("").as_str(),
                                                        // args.output_folder.as_ref().expect("").as_str(),
            ) {
                eprintln!("Application error {}", e);
                process::exit(1);
            }
        }
    }
}
