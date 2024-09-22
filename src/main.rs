use clap::{Args, Parser, Subcommand};
use colored::{self, Colorize};
use compress::compress_image_files;
use path_clean::PathClean;
use std::env;
use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::{path::PathBuf, process};
pub mod compress;
pub mod organizer;

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
    /// Organizes image files in a directory.
    Organize(OrganizeArgs),
}

#[derive(Args)]
struct CompressArgs {
    /// The output pattern specifies how the output file name will be constructed.
    /// Example:
    /// input file name: foo.jpg |
    /// output-pattern: "X-c" |
    /// output file name: foo-c.jpg
    #[arg(long)]
    output_pattern: Option<String>,
    input_dir: PathBuf,
    output_dir: Option<PathBuf>,
}

#[derive(Args)]
struct OrganizeArgs {
    base_dir: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.commands {
        Commands::Compress(args) => {
            let output_pattern = args.output_pattern.clone().unwrap_or_default();
            let output_folder = validate_compression_args(args);

            if let Err(e) = compress_image_files(
                &args.input_dir,
                &output_folder,
                &output_pattern,
                cli.recursive,
            ) {
                eprintln!("Application error {}", e);
                process::exit(1);
            } else {
                Ok(())
            }
        }
        Commands::Organize(args) => {
            println!("Do you want to organize files in directory? {:?}\r\n[Y] Yes [N] No (default is \"N\"):", path_to_absolute_path(&args.base_dir)?);

            let input = read_input();

            if input == "Y" || input == "y" {
                if let Err(e) = organizer::organize_img_files(&args.base_dir) {
                    eprintln!("Application error {}", e);
                    process::exit(1);
                }
            }

            Ok(())
        }
    }
}

// Read user input from console.
fn read_input() -> String {
    let mut input = String::new();
    let _ = stdout().flush();
    stdin()
        .read_line(&mut input)
        .expect("Did not enter a correct string");

    if let Some('\n') = input.chars().next_back() {
        input.pop();
    }
    if let Some('\r') = input.chars().next_back() {
        input.pop();
    }
    input
}

// Convert any path to absolute path.
pub fn path_to_absolute_path(path: impl AsRef<Path>) -> anyhow::Result<PathBuf> {
    let path = path.as_ref();

    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()?.join(path)
    }
    .clean();

    Ok(absolute_path)
}

// Validate compression arguments.
fn validate_compression_args(args: &CompressArgs) -> PathBuf {
    let output_folder: PathBuf = match &args.output_dir {
        Some(val) => {
            if val.eq(&args.input_dir) && args.output_pattern.is_none() {
                eprintln!("Output pattern must be specified when output folder are equal to input folder.");
                process::exit(1);
            }
            val.to_path_buf()
        }
        None => {
            if args.output_pattern.is_none() {
                eprintln!(
                    "{}",
                    "When omitting the output folder an output pattern must be specified.".red()
                );
                process::exit(1);
            }
            args.input_dir.clone()
        }
    };
    output_folder
}
