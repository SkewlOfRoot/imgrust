use anyhow::{anyhow, Ok};
use chrono::NaiveDateTime;
use std::fs::{self};
use std::path::{Path, PathBuf};

pub fn organize_img_files(base_dir: &PathBuf) -> anyhow::Result<()> {
    if !base_dir.is_dir() {
        return Err(anyhow!("An invalid directory was specified."));
    }
    let entries = fs::read_dir(base_dir)?;

    for entry in entries {
        let entry = entry?;
        let file_path = entry.path();
        let file_name = entry.file_name();

        if is_image_file(&file_path) {
            let date_taken = if let Some(date) = extract_image_date(&file_path) {
                date
            } else {
                eprintln!("DateTime not found in EXIF data.");
                continue;
            };

            let new_dir_name = date_taken.date().format("%Y-%m").to_string();

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
    }

    Ok(())
}

fn is_image_file(file_path: &Path) -> bool {
    if file_path.is_dir() {
        return false;
    }

    return file_path.extension().unwrap_or_default() == "jpg"
        || file_path.extension().unwrap_or_default() == "jpeg"
        || file_path.extension().unwrap_or_default() == "png";
}

fn extract_image_date(file_path: &Path) -> Option<NaiveDateTime> {
    let exif_result = rexif::parse_file(file_path).unwrap();

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
