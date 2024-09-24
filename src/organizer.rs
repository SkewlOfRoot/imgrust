use anyhow::{anyhow, Context};
use chrono::NaiveDateTime;
use std::fs::{self};
use std::path::{Path, PathBuf};
use std::result::Result::Ok;

pub fn organize_img_files(base_dir: &PathBuf) -> anyhow::Result<()> {
    if !base_dir.is_dir() {
        return Err(anyhow!("An invalid directory was specified."));
    }
    let entries = fs::read_dir(base_dir)?;

    for entry in entries {
        let entry = entry?;
        let file_path = entry.path();
        let file_name = entry.file_name();

        let created_date = match get_media_file(&file_path) {
            Some(m) => match m {
                MediaFile::Image(img) => img.created_date(),
                MediaFile::Video(vid) => vid.created_date(),
            },
            None => continue,
        };

        let created_date = if let Some(date) = created_date {
            date
        } else {
            eprintln!("DateTime not found in EXIF data.");
            continue;
        };

        let new_dir_name = created_date.date().format("%Y-%m").to_string();
        let new_dir_path = base_dir.join(new_dir_name);
        if !new_dir_path.exists() {
            fs::create_dir(&new_dir_path)?;
        }

        let new_file_path = new_dir_path.join(file_name);

        if let Err(e) = fs::rename(&file_path, &new_file_path) {
            eprintln!(
                "Failed to move file '{}' to new directory '{}': {}",
                file_path.display(),
                new_dir_path.display(),
                e
            );
        }
    }
    Ok(())
}

fn get_media_file(file_path: &Path) -> Option<MediaFile> {
    if file_path.is_dir() {
        return None;
    }

    let extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default();

    match extension {
        "jpg" | "jpeg" | "png" => Some(MediaFile::Image(Image::new(file_path))),
        "mp4" => Some(MediaFile::Video(Video::new(file_path))),
        _ => None,
    }
}

enum MediaFile {
    Image(Image),
    Video(Video),
}

struct Image {
    file_path: PathBuf,
}
struct Video {
    file_path: PathBuf,
}

impl Image {
    fn new(file_path: &Path) -> Image {
        Image {
            file_path: file_path.to_owned(),
        }
    }

    fn created_date(&self) -> Option<NaiveDateTime> {
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
    fn new(file_path: &Path) -> Video {
        Video {
            file_path: file_path.to_owned(),
        }
    }

    fn created_date(&self) -> Option<NaiveDateTime> {
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
