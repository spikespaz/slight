use std::path::Path;

use thiserror::Error;

const BRIGHTNESS_CAPABILITY_FILES: &[&str; 2] = &["brightness", "max_brightness"];
const BACKLIGHT_CAPABILITY_FILES: &[&str; 3] = &["actual_brightness", "bl_power", "type"];

pub enum Capability {
    Brightness,
    Backlight,
    /// No errors occurred, but the specified path does not look
    /// like either a brightness or backlight device.
    None,
}

#[derive(Error, Debug)]
pub enum CapabilityCheckError {
    #[error("device path does not exist: {0}")]
    NotFound(String),
    #[error("lacking permissions to read path: {0}")]
    PermissionDenied(String),
    #[error("device path is not a directory: {0}")]
    NotADirectory(String),
    #[error("unexpected IO error while {whilst}: {source}")]
    Unexpected {
        #[source]
        source: std::io::Error,
        whilst: String,
    },
}

impl CapabilityCheckError {
    fn not_found(path: &Path) -> Self {
        Self::NotFound(path.to_string_lossy().to_string())
    }

    fn permission_denied(path: &Path) -> Self {
        Self::NotFound(path.to_string_lossy().to_string())
    }

    fn not_a_directory(path: &Path) -> Self {
        Self::NotFound(path.to_string_lossy().to_string())
    }
}

impl Capability {
    fn check(path: &Path) -> Result<Self, CapabilityCheckError> {
        // do checks on the path to make sure all further errors are
        // truly unexpected, bubble error
        match path.try_exists() {
            Ok(true) => path
                .is_dir()
                .then_some(())
                .ok_or_else(|| CapabilityCheckError::not_a_directory(path)),
            Ok(false) => Err(CapabilityCheckError::not_found(path)),
            Err(e) => match e.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    Err(CapabilityCheckError::permission_denied(path))
                }
                _ => Err(CapabilityCheckError::Unexpected {
                    source: e,
                    whilst: format!("attempting to read device path: {}", path.display()),
                }),
            },
        }?;
        // now loop the entries and filter, bubble error if needed
        let files = match path.read_dir() {
            Err(_) => unreachable!(), // should have covered all bases above
            Ok(mut entries) => {
                let mut files = Vec::with_capacity(7);
                entries.try_for_each(|entry| {
                    let entry = entry.map_err(|e| CapabilityCheckError::Unexpected {
                        source: e,
                        whilst: format!("iterating device path: {}", path.display()),
                    })?;
                    // filter entries to those who are regular files with reasonable names
                    if entry.file_type().map_or(false, |f| f.is_file()) {
                        if let Ok(f) = entry.file_name().into_string() {
                            files.push(f);
                        }
                    }
                    Ok(())
                })?;
                Ok(files)
            }
        }?;
        // check if the files present match either of the predefined
        // slices of file names
        let has_brightness = BRIGHTNESS_CAPABILITY_FILES
            .map(|a| files.iter().any(|b| a == b))
            .into_iter()
            .all(|x| x);
        let has_backlight = has_brightness
            && BACKLIGHT_CAPABILITY_FILES
                .map(|a| files.iter().any(|b| a == b))
                .into_iter()
                .all(|x| x);
        // return the discovered capability
        Ok(if has_backlight {
            Capability::Backlight
        } else if has_brightness {
            Capability::Brightness
        } else {
            Capability::None
        })
    }
}
