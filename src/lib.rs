use std::error::Error;
use std::fs::{self};
use std::path::PathBuf;

use image::io::Reader as ImageReader;
use mozjpeg::{ColorSpace, Compress, ScanMode};

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

fn compress(input_path: PathBuf, output_path: PathBuf) {
    // Load the image using the `image` crate
    let img = ImageReader::open(&input_path).unwrap().decode().unwrap();

    // Convert the image to RGB format
    let img = img.to_rgb8();

    // Prepare for compression
    let mut comp = Compress::new(ColorSpace::JCS_RGB);
    comp.set_scan_optimization_mode(ScanMode::AllComponentsTogether);

    // Set the quality of the output JPEG (0 to 100)
    comp.set_quality(75.0);

    // Set the image dimensions
    comp.set_size(img.width() as usize, img.height() as usize);

    // Begin the compression process
    let mut comp = comp.start_compress(Vec::new()).unwrap();

    // Write the pixel data to the compressor
    comp.write_scanlines(img.as_raw()).unwrap();

    // Finish the compression process
    let jpeg_data = comp.finish().unwrap();

    // Save the compressed image to a file
    std::fs::write(output_path, jpeg_data).unwrap();

    println!("Image compression complete.");
}
