use anyhow::{anyhow, Ok};
use chrono::NaiveDateTime;
use std::fs::{self};
use std::path::{Path, PathBuf};

pub fn organize_folder(folder_path: &PathBuf) -> anyhow::Result<()> {
    if !folder_path.is_dir() {
        return Err(anyhow!("An invalid folder was specified."));
    }
    let entries = fs::read_dir(folder_path)?;

    for entry in entries {
        let entry = entry?;

        if is_image_file(&entry.path()) {
            let date_taken = extract_image_date(&entry.path())?;
            let new_folder_name = date_taken.date().format("%Y-%m").to_string();

            let new_folder_path = folder_path.join(new_folder_name);
            if !new_folder_path.exists() {
                fs::create_dir(&new_folder_path)?;
            }

            let new_file_name = new_folder_path.join(entry.file_name());

            if let Err(e) = fs::rename(entry.path(), new_file_name) {
                eprintln!(
                    "Failed to move file '{}' to new folder '{}': {}",
                    entry.path().display(),
                    new_folder_path.display(),
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

fn extract_image_date(file_path: &Path) -> anyhow::Result<NaiveDateTime> {
    let exif_result = rexif::parse_file(file_path).unwrap();

    if let Some(val) = exif_result
        .entries
        .iter()
        .find(|t| t.tag == rexif::ExifTag::DateTime)
        .map(|t| &t.value)
    {
        let d = &val.to_string();
        Ok(NaiveDateTime::parse_from_str(d, "%Y:%m:%d %H:%M:%S")?)
    } else {
        Err(anyhow!("DateTime not found in EXIF data."))
    }
}
