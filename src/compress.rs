use colored::{self, Colorize};
use image::io::Reader as ImageReader;
use img_parts::jpeg::Jpeg;
use img_parts::ImageEXIF;
use indicatif::ProgressBar;
use mozjpeg::{ColorSpace, Compress, ScanMode};
use rayon::prelude::*;
use std::ffi::OsString;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::Instant;

pub fn compress_image_files(
    input_folder_path: &Path,
    output_folder_path: &Path,
    output_pattern: &str,
    recursive: bool,
) -> anyhow::Result<()> {
    let start = Instant::now();

    let input_file_paths = jpg_paths(input_folder_path, recursive)?;

    let mut file_path_set: Vec<(PathBuf, PathBuf)> = Vec::new();
    for input_file_path in input_file_paths {
        let output_file_path = output_path(output_folder_path, &input_file_path, output_pattern);
        file_path_set.push((input_file_path, output_file_path));
    }

    let bar = ProgressBar::new(file_path_set.len().try_into()?);

    file_path_set.par_iter().for_each(|(input, output)| {
        compress(input, output).expect("Compression failed.");
        bar.inc(1)
    });

    let end = Instant::now();
    let duration = end.duration_since(start);
    println!(
        "{}{}{}",
        "Done! Compression took ".green(),
        duration.as_millis().to_string().green(),
        " ms.".green()
    );

    Ok(())
}

// Get the output path based on the input path.
fn output_path(output_folder_path: &Path, input_file_path: &Path, output_pattern: &str) -> PathBuf {
    let mut output_file_path = output_folder_path.to_path_buf();

    let extension = input_file_path.extension().unwrap_or_default();

    let file_name = input_file_path
        .file_stem()
        .expect("Failed to get file stem from path.");

    let mut file_name = OsString::from(output_pattern.replace('X', file_name.to_str().unwrap()));
    file_name.push(".");
    file_name.push(extension);

    output_file_path.push(file_name);
    output_file_path
}

// Locates all the JPG files in the given folder and returns the paths.
fn jpg_paths(folder_path: &Path, recursive: bool) -> anyhow::Result<Vec<PathBuf>> {
    // TODO: Only get jpg files that do not match the output pattern.
    let mut paths: Vec<PathBuf> = Vec::new();

    let entries = fs::read_dir(folder_path)?;
    for entry in entries {
        let path = entry?.path();
        if recursive && path.is_dir() {
            paths.extend(jpg_paths(&path, recursive)?);
        } else if let Some(ext) = path.extension() {
            if ext.to_ascii_lowercase() == "jpg" {
                paths.push(path);
            }
        }
    }
    Ok(paths)
}

// Compresses the input image and saves it to the output path.
fn compress(input_path: &PathBuf, output_path: &PathBuf) -> anyhow::Result<()> {
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
    let mut comp = comp.start_compress(Vec::new())?;

    // Write the pixel data to the compressor
    comp.write_scanlines(img.as_raw()).unwrap();

    // Finish the compression process
    let jpeg_data = comp.finish().unwrap();
    // Save the compressed image to a file
    if let Err(e) = std::fs::write(output_path, jpeg_data) {
        panic!("Could not save file to output path. {}", e);
    }
    copy_exif(input_path, output_path);

    Ok(())
}

// Copies the EXIF meta data from the input file to the output file.
fn copy_exif(input_path: &PathBuf, output_path: &PathBuf) {
    // Read data from files.
    let input_data = fs::read(input_path).unwrap();
    let output_data = fs::read(output_path).unwrap();
    let output_file = OpenOptions::new().write(true).open(output_path).unwrap();

    // Read EXIF meta data from input file.
    let in_jpeg = Jpeg::from_bytes(input_data.into()).unwrap();
    let exif_metadata = in_jpeg.exif().unwrap();

    // Write EXIF meta data to output file.
    let mut out_jpeg = Jpeg::from_bytes(output_data.clone().into()).unwrap();
    out_jpeg.set_exif(exif_metadata.into());
    out_jpeg.encoder().write_to(output_file).unwrap();
}
