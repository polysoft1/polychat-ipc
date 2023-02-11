use std::path::Path;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessManagerError<'a> {
    #[error("Path '{0}' does not exist")]
    NonExistent(&'a Path),
    #[error("'{0}' is a relative path")]
    RelativePath(&'a Path),
    #[error("'{0}' is not a directory")]
    NonDirectory(&'a Path)
}