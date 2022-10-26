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

    /// MrBundleError
    #[error(transparent)]
    CargoMetadataError(#[from] cargo_metadata::Error),

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

    #[error("TOML deserialization error: {0}")]
    TomlDeError(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSerError(#[from] toml::ser::Error),

    #[error(transparent)]
    SynError(#[from] syn::Error),

    #[error(transparent)]
    IgnoreError(#[from] ignore::Error),

    #[error(transparent)]
    HandlebarsRenderError(#[from] handlebars::RenderError),

    #[error(transparent)]
    HandlebarsTemplateError(#[from] handlebars::TemplateError),

    #[error("Path was not found: {0}")]
    PathNotFound(PathBuf),

    #[error("Invalid path {0}: {1}")]
    InvalidPath(PathBuf, String),

    #[error("No app manifest (happ.yaml) was found in this directory tree")]
    AppManifestNotFound,

    #[error("App \"{0}\" already exists in this directory tree")]
    AppAlreadyExists(String),

    #[error("DNA \"{0}\" was not found in this app")]
    DnaNotFound(String),

    #[error("No DNAs were found in app \"{0}\"")]
    NoDnasFound(String),

    #[error("Malformed file {0}: ")]
    MalformedFile(PathBuf, String),

    #[error("DNA \"{0}\" already exists in app \"{1}\"")]
    DnaAlreadyExists(String, String),

    #[error("Zome \"{0}\" already exists in dna \"{1}\"")]
    ZomeAlreadyExists(String, String),

    #[error("Integrity zome \"{0}\" was not found in dna \"{1}\"")]
    IntegrityZomeNotFound(String, String),

    #[error("Coordinator zome \"{0}\" was not found in dna \"{1}\"")]
    CoordinatorZomeNotFound(String, String),

    #[error("No integrity zomes were found in dna \"{0}\"")]
    NoIntegrityZomesFound(String),

    #[error("No coordinator zomes were found in dna \"{0}\"")]
    NoCoordinatorZomesFound(String),

    #[error("No coordinator zomes were found in dna \"{0}\" for the integrity zome \"{1}\"")]
    NoCoordinatorZomesFoundForIntegrityZome(String, String),

    #[error("Invalid field type \"{0}\", here are all valid field types: \"{1}\"")]
    InvalidFieldType(String, String),

    #[error("Invalid UI framework \"{0}\", here are all valid UI frameworks: \"{1}\"")]
    InvalidUiFramework(String, String),

    #[error("Invalid string format: \"{0}\"")]
    InvalidStringFormat(String),

    #[error("Invalid index type: \"{0}\". Allowed index types: \"{1}\"")]
    InvalidIndexType(String, String),

    #[error("No entry type definitions (#[hdk_entry_defs]) were found in dna \"{0}\" for the integrity zome \"{1}\"")]
    NoEntryTypesDefFoundForIntegrityZome(String, String),

    #[error("Entry type \"{0}\" already exists in dna \"{1}\" for the integrity zome \"{2}\"")]
    EntryTypeAlreadyExists(String, String, String),

    #[error("Multiple entry type definitions (#[hdk_entry_defs]) were found in dna \"{0}\" for the integrity zome \"{1}\"")]
    MultipleEntryTypesDefsFoundForIntegrityZome(String, String),

    #[error("Entry type \"{0}\" was not found in dna \"{1}\" for the integrity zome \"{2}\"")]
    EntryTypeNotFound(String, String, String),

    #[error("Link type \"{0}\" already exists in dna \"{1}\" for the integrity zome \"{2}\"")]
    LinkTypeAlreadyExists(String, String, String),

    #[error("Invalid arguments: \"{0}\"")]
    InvalidArguments(String),


    /// anything else
    #[error("Unknown error: {0}")]
    MiscError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

/// HcBundle Result type.
pub type ScaffoldResult<T> = Result<T, ScaffoldError>;
