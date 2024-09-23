use clap::{Args, Parser, Subcommand};
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
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Will compress any detected jpg files. Compressed files will be postfixed with a '-c'. For example, 'hello.jpg' will become 'hello-c.jpg'.
    Compress(CompressArgs),
    /// Organizes image files into directories based on the image files EXIF 'date taken' data. For example, an image with a 'date taken' on 2024-09-10 will be placed in a directory called '2024-09'.
    Organize(OrganizeArgs),
}

#[derive(Args)]
struct CompressArgs {
    /// Looks for media files recursively within the specified input directory.
    #[arg(short, long)]
    recursive: bool,
    /// If set, the original uncompressed source file will be deleted. Only the compressed version will remain.
    #[arg(short, long)]
    delete_original: bool,
    input_dir: PathBuf,
}

#[derive(Args)]
struct OrganizeArgs {
    base_dir: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.commands {
        Commands::Compress(args) => {
            if let Err(e) =
                compress_image_files(&args.input_dir, args.recursive, args.delete_original)
            {
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
fn path_to_absolute_path(path: impl AsRef<Path>) -> anyhow::Result<PathBuf> {
    let path = path.as_ref();

    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()?.join(path)
    }
    .clean();

    Ok(absolute_path)
}
