use std::error::Error;
use std::fs::{self};
use std::path::PathBuf;

pub struct CommandArgs {
    pub input_path: String,
    pub output_path: String,
}

impl CommandArgs {
    pub fn new(args: &[String]) -> Result<CommandArgs, &str> {
        if args.len() < 3 {
            return Err("not enough arguments.");
        }
        Ok(CommandArgs {
            input_path: args[1].clone(),
            output_path: args[2].clone(),
        })
    }
}

pub fn compress_image_files(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let paths = get_jpg_paths(input_path)?;
    Ok(())
}

fn get_jpg_paths(folder_path: &str) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut jpeg_paths: Vec<PathBuf> = Vec::new();

    let entries = fs::read_dir(folder_path)?;
    for entry in entries {
        let path = entry?.path();
        if let Some(ext) = path.extension() {
            if ext.to_ascii_lowercase() == "jpg" {
                jpeg_paths.push(path);
            }
        }
    }
    Ok(jpeg_paths)
}
