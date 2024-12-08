use super::files::{Image, MediaFile, Video};
use anyhow::{anyhow, Context, Ok};
use colored::{self, Colorize};
use image::io::Reader as ImageReader;
use img_parts::jpeg::Jpeg;
use img_parts::ImageEXIF;
use indicatif::ProgressBar;
use mozjpeg::{ColorSpace, Compress, ScanMode};
use rayon::prelude::*;
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
    let input_file_paths: Vec<MediaFile> = if input_path.is_file() {
        load_media_file(input_path).into_iter().collect()
    } else {
        load_media_files(input_path, recursive)?
    };

    let bar = ProgressBar::new(input_file_paths.len().try_into()?);

    // Compress the images in parallel.
    input_file_paths.par_iter().for_each(|media_file| {
        match media_file {
            MediaFile::Image(image) => {
                if let Err(e) = compress_image_file(image) {
                    eprintln!(
                        "Failed to compress file '{}'. Reason: {}",
                        image.file_path.display(),
                        e
                    );
                } else if delete_original {
                    fs::remove_file(&image.file_path)
                        .context("Remove file")
                        .unwrap();
                }
            }
            MediaFile::Video(video) => {}
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

// Locates all the supported media files in the given directory and returns the paths.
fn load_media_files(input_path: &Path, recursive: bool) -> anyhow::Result<Vec<MediaFile>> {
    let mut paths: Vec<MediaFile> = Vec::new();

    let entries = fs::read_dir(input_path)?;
    for entry in entries {
        let path = entry?.path();
        if recursive && path.is_dir() {
            paths.extend(load_media_files(&path, recursive)?);
        } else if let Some(path) = load_media_file(&path) {
            paths.push(path)
        }
    }
    Ok(paths)
}

/// Attempts to load a media file from the given path. If the file is a supported
/// media file type (jpg, mp4), it is returned as a `MediaFile` enum variant.
/// Otherwise `None` is returned.
fn load_media_file(path: &Path) -> Option<MediaFile> {
    match path.extension()?.to_ascii_lowercase().to_str().unwrap() {
        "jpg" => Some(MediaFile::Image(Image::new(path))),
        "mp4" => Some(MediaFile::Video(Video::new(path))),
        _ => None,
    }
}

// Compresses the input image and saves it to the output path.
fn compress_image_file(image_file: &Image) -> anyhow::Result<()> {
    // Load the image using the `image` crate
    let img = match ImageReader::open(&image_file.file_path)
        .expect("Failed to open file.")
        .decode()
    {
        core::result::Result::Ok(img) => img,
        Err(e) => return Err(anyhow!("Failed to decode. Reason: {}", e)),
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
    if let Err(e) = std::fs::write(&image_file.output_path, jpeg_data) {
        panic!("Could not save file to output path. {}", e);
    }
    copy_exif(&image_file.file_path, &image_file.output_path)?;
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
        None => return Err(anyhow!("EXIF data not found in file.")),
    };

    // Write EXIF meta data to output file.
    let mut out_jpeg = Jpeg::from_bytes(output_data.clone().into()).unwrap();
    out_jpeg.set_exif(exif_metadata.into());
    out_jpeg.encoder().write_to(output_file).unwrap();

    Ok(())
}
