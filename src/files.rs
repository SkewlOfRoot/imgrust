use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use anyhow::Context;
use chrono::NaiveDateTime;

pub enum MediaFile {
    Image(Image),
    Video(Video),
}

pub struct Image {
    pub file_path: PathBuf,
    pub output_path: PathBuf,
}
pub struct Video {
    pub file_path: PathBuf,
    pub output_path: PathBuf,
}

impl Image {
    pub fn new(file_path: &Path) -> Image {
        Image {
            file_path: file_path.to_owned(),
            output_path: output_path(file_path),
        }
    }

    /// Returns the created date of the image file if found in its EXIF data.
    ///
    /// The created date is obtained from the `DateTime` EXIF tag.
    ///
    /// If the tag is not found, `None` is returned.
    pub fn created_date(&self) -> Option<NaiveDateTime> {
        let exif_result = rexif::parse_file(&self.file_path).context("rexif").unwrap();
        let date_time = exif_result
            .entries
            .iter()
            .find(|t| t.tag == rexif::ExifTag::DateTime)
            .map(|t| &t.value);

        match date_time {
            Some(date) => {
                let d = &date.to_string();
                Some(
                    NaiveDateTime::parse_from_str(d, "%Y:%m:%d %H:%M:%S")
                        .expect("Error parsing string to date."),
                )
            }
            None => None,
        }
    }
}

impl Video {
    pub fn new(file_path: &Path) -> Video {
        Video {
            file_path: file_path.to_owned(),
            output_path: output_path(file_path),
        }
    }

    /// Extracts the creation date of the video file using ffprobe.
    ///
    /// The creation date is retrieved from the `creation_time` tag
    /// in the metadata of the video streams. The tag is expected to
    /// be in RFC 3339 format and is converted to `NaiveDateTime`.
    ///
    /// Returns `Some(NaiveDateTime)` if the creation time is found
    /// and successfully parsed; otherwise, returns `None`.
    pub fn created_date(&self) -> Option<NaiveDateTime> {
        let meta_data = ffprobe::ffprobe(&self.file_path)
            .context("ffprobe")
            .unwrap();

        for stream in meta_data.streams {
            let tags = if let Some(t) = stream.tags {
                t
            } else {
                continue;
            };

            if let Some(date_str) = tags.creation_time {
                let date = chrono::DateTime::parse_from_rfc3339(&date_str);
                match date {
                    Ok(d) => return Some(d.naive_utc()),
                    Err(e) => {
                        eprintln!(
                            "Failed to parse '{}' to NativeDateTime. Error: {}",
                            &date_str, e
                        );
                        continue;
                    }
                };
            } else {
                continue;
            };
        }
        None
    }
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
