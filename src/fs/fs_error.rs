use crate::fs::fs_error::FsTestError::IoError;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum FsTestError {
    NeedDir,
    NeedFile,
    IoError(std::io::Error),
}

impl Display for FsTestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FsTestError::NeedDir => f.write_str("need directory"),
            FsTestError::NeedFile => f.write_str("need file"),
            IoError(e) => f.write_fmt(format_args!("IO error occurred : {}", e.to_string())),
        }
    }
}

impl Error for FsTestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FsTestError::NeedDir => None,
            FsTestError::NeedFile => None,
            IoError(e) => Some(e),
        }
    }
}

impl From<std::io::Error> for FsTestError {
    fn from(e: std::io::Error) -> Self {
        IoError(e)
    }
}
