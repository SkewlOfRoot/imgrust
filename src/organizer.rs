use super::files::{Image, MediaFile, Video};
use anyhow::anyhow;
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
