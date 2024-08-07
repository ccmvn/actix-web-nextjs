use std::io;

use actix_web::ResponseError;
use tracing::error;

/// Custom error type for SPA service
#[derive(thiserror::Error, Debug)]
pub enum SpaError {
    #[error("File system error: {0}")]
    FileSystemError(#[from] io::Error),
    #[error("Failed to serve file: {0}")]
    ServeFileError(#[from] actix_web::Error),
    #[error("Build manifest not found")]
    BuildManifestNotFound,
    #[error("Path conversion error")]
    PathConversionError,
    #[error("Glob pattern error: {0}")]
    GlobPatternError(#[from] glob::PatternError),
}

impl ResponseError for SpaError {}

impl From<anyhow::Error> for SpaError {
    fn from(error: anyhow::Error) -> Self {
        error!("An unexpected error occurred: {:?}", error);
        SpaError::ServeFileError(actix_web::Error::from(Box::<dyn std::error::Error>::from(error)))
    }
}
