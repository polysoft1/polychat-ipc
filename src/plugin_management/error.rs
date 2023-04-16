use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginManagerError {
    #[error("Path '{0}' does not exist")]
    NonExistent(PathBuf),
    #[error("'{0}' is a relative path")]
    RelativePath(PathBuf),
    #[error("'{0}' is not a directory")]
    NonDirectory(PathBuf),
    #[error("Path is not set")]
    NoPath,
}