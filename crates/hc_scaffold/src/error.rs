use std::path::PathBuf;

use holochain_util::ffs;
use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum ScaffoldError {
    /// std::io::Error
    #[error("IO error: {0}")]
    StdIoError(#[from] std::io::Error),

    #[error("ffs::IoError: {0}")]
    FfsIoError(#[from] ffs::IoError),

    /// MrBundleError
    #[error(transparent)]
    MrBundleError(#[from] mr_bundle::error::MrBundleError),

    /*
    /// DnaError
    #[error("DNA error: {0}")]
    DnaError(#[from] holochain_types::dna::DnaError),

    /// SerializedBytesError
    #[error("Internal serialization error: {0}")]
    SerializedBytesError(#[from] SerializedBytesError),
    */
    /// serde_yaml::Error
    #[error("YAML serialization error: {0}")]
    SerdeYamlError(#[from] serde_yaml::Error),

    #[error("Path was not found: {0}")]
    PathNotFound(PathBuf),

    #[error("No app manifest (happ.yaml) was found in this directory tree")]
    AppManifestNotFound,

    #[error("App \"{0}\" already exists in this directory tree")]
    AppAlreadyExists(String),

    #[error("DNA \"{0}\" was not found for this app")]
    DnaNotFound(String),

    #[error("No DNAs were found for this app")]
    NoDnasFound,

    #[error("DNA \"{0}\" already exists for app \"{1}\"")]
    DnaAlreadyExists(String, String),

    #[error("Zome \"{0}\" already exists for app \"{1}\" and dna \"{2}\"")]
    ZomeAlreadyExists(String, String, String),

    /// anything else
    #[error("Unknown error: {0}")]
    MiscError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

/// HcBundle Result type.
pub type ScaffoldResult<T> = Result<T, ScaffoldError>;
