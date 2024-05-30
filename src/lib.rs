use indicatif::ProgressBar;
use rayon::prelude::*;
use std::error::Error;
use std::fs::{self};
use std::path::{Path, PathBuf};
use std::time::Instant;

use image::io::Reader as ImageReader;
use mozjpeg::{ColorSpace, Compress, ScanMode};

pub fn compress_image_files(
    input_folder_path: &PathBuf,
    output_folder_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    let input_paths = get_jpg_paths(input_folder_path)?;

    let mut names: Vec<(PathBuf, PathBuf)> = Vec::new();
    for input_path in input_paths {
        let output_path = get_output_path(output_folder_path, &input_path);
        names.push((input_path, output_path));
    }

    let bar = ProgressBar::new(names.len().try_into()?);

    names.par_iter().for_each(|x| {
        compress(&x.0, &x.1);
        bar.inc(1)
    });

    let end = Instant::now();
    let duration = end.duration_since(start);
    println!("Done! Compression took {} ms.", duration.as_millis());

    Ok(())
}

// Get the output path based on the input path.
fn get_output_path(output_folder_path: &Path, input_path: &Path) -> PathBuf {
    let mut output_file_path = PathBuf::from(output_folder_path);

    let file_name = input_path
        .file_name()
        .expect("Failed to get file name from path.");

    output_file_path.push(file_name);
    output_file_path
}

// Locates all the JPG files in the given folder and returns the paths.
fn get_jpg_paths(folder_path: &PathBuf) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut jpg_paths: Vec<PathBuf> = Vec::new();

    let entries = fs::read_dir(folder_path)?;
    for entry in entries {
        let path = entry?.path();
        if let Some(ext) = path.extension() {
            if ext.to_ascii_lowercase() == "jpg" {
                jpg_paths.push(path);
            }
        }
    }
    Ok(jpg_paths)
}

// Compresses the input image and saves it to the output path.
fn compress(input_path: &PathBuf, output_path: &PathBuf) {
    // Load the image using the `image` crate
    let img = ImageReader::open(input_path).unwrap().decode().unwrap();
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
    if let Err(e) = std::fs::write(output_path, jpeg_data) {
        panic!("Could not save file to output path. {}", e);
    }
}
