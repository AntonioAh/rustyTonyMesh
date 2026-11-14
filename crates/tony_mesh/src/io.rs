mod read_mesh;
mod write_mesh;

use thiserror::Error;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FileType {
    Obj,
    Off,
}
#[derive(Debug, Error)]
pub enum IoError {

    #[error("file not found")]
    FileNotFound,

    #[error("extension not found")]
    ExtensionNotFound,

    #[error("method yet to implement")]
    YetToImplement,

    #[error("wrong file format")]
    WrongFileFormat,

    #[error("not triangular mesh")]
    NotTriangleMesh,

    #[error("can't read file format: {0}")]
    Generic(#[from] std::io::Error),

}

use std::path::{Path, PathBuf};

static FORMATS: [(&str, FileType); 2] = [("obj", FileType::Obj), ("off", FileType::Off)];

static DIRS: [&str; 3] = ["data/", "../data", "../../data"];

pub fn get_path(file_name: &str) -> Result<PathBuf, IoError>{
    DIRS.iter().map(|d| Path::new(d).join(file_name))
        .find(|p| p.is_file())
        .ok_or(IoError::FileNotFound)
}
pub fn get_file_type(path: &Path) -> Result<FileType, IoError> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase)
        .ok_or(IoError::ExtensionNotFound)?;

    FORMATS
        .iter()
        .find(|(e, _)| *e == ext)
        .map(|(_, t)| *t)
        .ok_or(IoError::ExtensionNotFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file_type(name: &str) -> Result<FileType, IoError> {
        get_file_type(Path::new(name))
    }

    #[test]
    fn known_extensions() {
        assert_eq!(file_type("bunny.off").unwrap(), FileType::Off);
        assert_eq!(file_type("bunny.obj").unwrap(), FileType::Obj);
    }

    #[test]
    fn extension_is_case_insensitive() {
        assert_eq!(file_type("bunny.OFF").unwrap(), FileType::Off);
        assert_eq!(file_type("bunny.Obj").unwrap(), FileType::Obj);
    }

    #[test]
    fn only_the_last_extension_counts() {
        assert_eq!(file_type("bunny.v2.off").unwrap(), FileType::Off);
        assert!(matches!(file_type("bunny.off.bak"), Err(IoError::ExtensionNotFound)));
    }

    #[test]
    fn directories_in_the_path_are_ignored() {
        assert_eq!(file_type("data/meshes.v1/bunny.off").unwrap(), FileType::Off);
        assert_eq!(file_type("../../data/bunny.obj").unwrap(), FileType::Obj);
    }

    #[test]
    fn unknown_extension_is_rejected() {
        assert!(matches!(file_type("bunny.ply"), Err(IoError::ExtensionNotFound)));
        assert!(matches!(file_type("bunny.txt"), Err(IoError::ExtensionNotFound)));
    }

    #[test]
    fn missing_extension_is_rejected() {
        assert!(matches!(file_type("bunny"), Err(IoError::ExtensionNotFound)));
        assert!(matches!(file_type("data/bunny"), Err(IoError::ExtensionNotFound)));
        assert!(matches!(file_type(""), Err(IoError::ExtensionNotFound)));
    }

    #[test]
    fn hidden_file_without_extension_is_rejected() {
        // For ".off" the whole name is the file stem, so there is no extension.
        assert!(matches!(file_type(".off"), Err(IoError::ExtensionNotFound)));
    }

    #[test]
    fn trailing_dot_is_rejected() {
        assert!(matches!(file_type("bunny."), Err(IoError::ExtensionNotFound)));
    }
}
