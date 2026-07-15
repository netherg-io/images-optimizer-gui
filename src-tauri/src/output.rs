use image::io::Reader as ImageReader;
use image::{GenericImageView, ImageFormat};
use std::fs;
use std::path::Path;
use tempfile::{Builder, NamedTempFile};

use crate::types::{OperationError, OptimizationErrorCode};

pub enum WriteOutcome {
    Written(u64),
    SkippedLarger,
}

pub fn write_validated<F>(
    destination: &Path,
    expected_format: ImageFormat,
    expected_dimensions: (u32, u32),
    original_size: u64,
    skip_if_larger: bool,
    allow_overwrite: bool,
    encode: F,
) -> Result<WriteOutcome, OperationError>
where
    F: FnOnce(&Path) -> Result<(), OperationError>,
{
    let parent = destination.parent().ok_or_else(|| {
        OperationError::new(
            OptimizationErrorCode::DestinationWriteFailed,
            "Destination has no parent directory",
        )
    })?;
    fs::create_dir_all(parent).map_err(|error| {
        OperationError::io(error, OptimizationErrorCode::DestinationWriteFailed)
    })?;

    let suffix = destination
        .extension()
        .map(|extension| format!(".{}", extension.to_string_lossy()))
        .unwrap_or_default();
    let temp = Builder::new()
        .prefix(".image-optimizer-")
        .suffix(&suffix)
        .tempfile_in(parent)
        .map_err(|error| {
            OperationError::io(error, OptimizationErrorCode::DestinationWriteFailed)
        })?;

    encode(temp.path())?;
    temp.as_file().sync_all().map_err(|error| {
        OperationError::io(error, OptimizationErrorCode::DestinationWriteFailed)
    })?;

    let output_size = temp
        .as_file()
        .metadata()
        .map_err(|error| OperationError::io(error, OptimizationErrorCode::ValidationFailed))?
        .len();
    if output_size == 0 {
        return Err(OperationError::new(
            OptimizationErrorCode::ValidationFailed,
            "Codec produced an empty file",
        ));
    }

    let reader = ImageReader::open(temp.path())
        .map_err(|error| OperationError::io(error, OptimizationErrorCode::ValidationFailed))?
        .with_guessed_format()
        .map_err(|error| {
            OperationError::new(OptimizationErrorCode::ValidationFailed, error.to_string())
        })?;
    if reader.format() != Some(expected_format) {
        return Err(OperationError::new(
            OptimizationErrorCode::ValidationFailed,
            "Codec output format does not match the destination",
        ));
    }
    // ponytail: ravif is the only AVIF writer; add native decode if external writers appear.
    if expected_format != ImageFormat::Avif {
        let dimensions = reader
            .decode()
            .map_err(|error| {
                OperationError::new(OptimizationErrorCode::ValidationFailed, error.to_string())
            })?
            .dimensions();
        if dimensions != expected_dimensions {
            return Err(OperationError::new(
                OptimizationErrorCode::ValidationFailed,
                "Codec output dimensions changed unexpectedly",
            ));
        }
    }

    if skip_if_larger && output_size >= original_size {
        return Ok(WriteOutcome::SkippedLarger);
    }

    commit(temp, destination, allow_overwrite)?;
    Ok(WriteOutcome::Written(output_size))
}

fn commit(
    temp: NamedTempFile,
    destination: &Path,
    allow_overwrite: bool,
) -> Result<(), OperationError> {
    if !destination.exists() {
        return temp
            .persist_noclobber(destination)
            .map(|_| ())
            .map_err(|error| {
                OperationError::io(error.error, OptimizationErrorCode::DestinationWriteFailed)
            });
    }
    if !allow_overwrite {
        return Err(OperationError::new(
            OptimizationErrorCode::DestinationConflict,
            "Destination already exists",
        ));
    }

    let (temp_file, temp_path) = temp.keep().map_err(|error| {
        OperationError::io(error.error, OptimizationErrorCode::DestinationWriteFailed)
    })?;
    drop(temp_file);

    let backup = Builder::new()
        .prefix(".image-optimizer-backup-")
        .tempfile_in(destination.parent().unwrap_or_else(|| Path::new(".")))
        .map_err(|error| {
            OperationError::io(error, OptimizationErrorCode::DestinationWriteFailed)
        })?;
    let (backup_file, backup_path) = backup.keep().map_err(|error| {
        OperationError::io(error.error, OptimizationErrorCode::DestinationWriteFailed)
    })?;
    drop(backup_file);
    fs::remove_file(&backup_path).map_err(|error| {
        OperationError::io(error, OptimizationErrorCode::DestinationWriteFailed)
    })?;

    fs::rename(destination, &backup_path).map_err(|error| {
        let _ = fs::remove_file(&temp_path);
        OperationError::io(error, OptimizationErrorCode::DestinationWriteFailed)
    })?;

    if let Err(error) = fs::rename(&temp_path, destination) {
        let rollback = fs::rename(&backup_path, destination);
        let _ = fs::remove_file(&temp_path);
        return Err(OperationError::new(
            OptimizationErrorCode::DestinationWriteFailed,
            if rollback.is_ok() {
                format!("Failed to replace destination; original restored: {error}")
            } else {
                format!(
                    "Failed to replace destination and restore original; backup kept at {}",
                    backup_path.display()
                )
            },
        ));
    }

    fs::remove_file(&backup_path).map_err(|error| {
        OperationError::io(error, OptimizationErrorCode::DestinationWriteFailed)
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoder_failure_preserves_existing_file_and_cleans_temp() {
        let dir = tempfile::tempdir().unwrap();
        let destination = dir.path().join("image.png");
        fs::write(&destination, b"original").unwrap();

        let result = write_validated(
            &destination,
            ImageFormat::Png,
            (1, 1),
            100,
            false,
            true,
            |path| {
                fs::write(path, b"partial").unwrap();
                Err(OperationError::new(
                    OptimizationErrorCode::EncodeFailed,
                    "expected failure",
                ))
            },
        );

        assert!(result.is_err());
        assert_eq!(fs::read(&destination).unwrap(), b"original");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn failure_before_temp_creation_does_not_touch_parent_file() {
        let dir = tempfile::tempdir().unwrap();
        let parent_file = dir.path().join("not-a-directory");
        fs::write(&parent_file, b"original").unwrap();

        let result = write_validated(
            &parent_file.join("image.png"),
            ImageFormat::Png,
            (1, 1),
            100,
            false,
            false,
            |_| panic!("encoder must not run"),
        );

        assert!(result.is_err());
        assert_eq!(fs::read(parent_file).unwrap(), b"original");
    }

    #[test]
    fn validated_output_replaces_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let destination = dir.path().join("image.png");
        fs::write(&destination, b"original").unwrap();

        let result = write_validated(
            &destination,
            ImageFormat::Png,
            (1, 1),
            1,
            false,
            true,
            |path| {
                image::DynamicImage::new_rgb8(1, 1)
                    .save_with_format(path, ImageFormat::Png)
                    .map_err(|error| {
                        OperationError::new(OptimizationErrorCode::EncodeFailed, error.to_string())
                    })
            },
        );

        assert!(matches!(result, Ok(WriteOutcome::Written(_))));
        assert_eq!(image::image_dimensions(&destination).unwrap(), (1, 1));
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn validated_avif_output_is_committed() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.png");
        let destination = dir.path().join("image.avif");
        image::DynamicImage::new_rgb8(1, 1)
            .save_with_format(&source, ImageFormat::Png)
            .unwrap();

        let result = write_validated(
            &destination,
            ImageFormat::Avif,
            (1, 1),
            u64::MAX,
            false,
            false,
            |path| crate::image_ops::encode_avif(&source, path, 80.0, 4),
        );

        assert!(matches!(result, Ok(WriteOutcome::Written(_))));
        assert!(destination.exists());
    }
}
