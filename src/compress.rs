use anyhow::{anyhow, Context, Ok};
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
    input_path: &Path,
    recursive: bool,
    delete_original: bool,
) -> anyhow::Result<()> {
    let start = Instant::now();

    // If input path is just a path to a single file, we get a vec with only that path,
    // otherwise we find all the image file paths in the directory.
    let input_file_paths: Vec<PathBuf> = if input_path.is_file() {
        vec![input_path.to_path_buf()]
    } else {
        jpg_paths(input_path, recursive)?
    };

    // Determine the output path for all files that are to be processed.
    let mut file_path_set: Vec<(PathBuf, PathBuf)> = Vec::new();
    for input_file_path in input_file_paths {
        let output_file_path = output_path(&input_file_path);
        file_path_set.push((input_file_path, output_file_path));
    }

    let bar = ProgressBar::new(file_path_set.len().try_into()?);

    // Compress the images in parallel.
    file_path_set.par_iter().for_each(|(input, output)| {
        if let Err(e) = compress(input, output) {
            eprintln!(
                "Failed to compress file '{}'. Reason: {}",
                input.display(),
                e
            );
        } else if delete_original {
            fs::remove_file(input).context("Remove file").unwrap();
        }
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
fn output_path(input_file_path: &Path) -> PathBuf {
    let mut output_file_path = input_file_path.parent().unwrap().to_path_buf();

    let extension = input_file_path.extension().unwrap_or_default();

    // Get the file stem and convert it to str.
    let file_stem = input_file_path
        .file_stem()
        .context("Failed to get file stem from path.")
        .unwrap()
        .to_str()
        .context("Convert to str")
        .unwrap();

    // Post-fix the compressed file name with '-c'. It will later be renamed to its original file name.
    let mut file_name = OsString::from(format!("{}-c", file_stem));
    file_name.push(".");
    file_name.push(extension);

    output_file_path.push(file_name);
    output_file_path
}

// Locates all the JPG files in the given directory and returns the paths.
fn jpg_paths(input_path: &Path, recursive: bool) -> anyhow::Result<Vec<PathBuf>> {
    let mut paths: Vec<PathBuf> = Vec::new();

    let entries = fs::read_dir(input_path)?;
    for entry in entries {
        let path = entry?.path();
        if recursive && path.is_dir() {
            paths.extend(jpg_paths(&path, recursive)?);
        } else {
            path.extension()
                .is_some_and(|ext| ext.to_ascii_lowercase() == "jpg")
                .then(|| {
                    paths.push(path);
                });
        }
    }
    Ok(paths)
}

// Compresses the input image and saves it to the output path.
fn compress(input_path: &PathBuf, output_path: &PathBuf) -> anyhow::Result<()> {
    // Load the image using the `image` crate
    let img = match ImageReader::open(input_path)
        .expect("Failed to open file.")
        .decode()
    {
        core::result::Result::Ok(img) => img,
        Err(e) => {
            return Err(anyhow!(
                "Failed to decode file '{}'. Reason: {}",
                input_path.display(),
                e
            ))
        }
    };
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
    copy_exif(input_path, output_path)?;
    Ok(())
}

// Copies the EXIF meta data from the input file to the output file.
fn copy_exif(input_path: &PathBuf, output_path: &PathBuf) -> anyhow::Result<()> {
    // Read data from files.
    let input_data = fs::read(input_path).unwrap();
    let output_data = fs::read(output_path).unwrap();
    let output_file = OpenOptions::new().write(true).open(output_path).unwrap();

    // Read EXIF meta data from input file.
    let in_jpeg = Jpeg::from_bytes(input_data.into()).unwrap();
    let exif_metadata = match in_jpeg.exif() {
        Some(exif) => exif,
        None => {
            return Err(anyhow!(
                "EXIF data not found in file '{}'.",
                &input_path.display().to_string()
            ))
        }
    };

    // Write EXIF meta data to output file.
    let mut out_jpeg = Jpeg::from_bytes(output_data.clone().into()).unwrap();
    out_jpeg.set_exif(exif_metadata.into());
    out_jpeg.encoder().write_to(output_file).unwrap();

    Ok(())
}
