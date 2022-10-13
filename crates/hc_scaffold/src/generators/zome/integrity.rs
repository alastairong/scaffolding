use std::{ffi::OsString, path::PathBuf};

use holochain_scaffolding_utils::FileTree;
use holochain_types::prelude::{DnaManifest, DnaManifestCurrentBuilder, ZomeManifest};

use crate::error::{ScaffoldError, ScaffoldResult};

use super::utils::zome_wasm_location;

pub fn add_integrity_zome_to_manifest(
    mut app_file_tree: FileTree,
    dna_manifest_path: &PathBuf,
    zome_name: String,
) -> ScaffoldResult<FileTree> {
    let v: Vec<OsString> = dna_manifest_path.iter().map(|s| s.to_os_string()).collect();
    let dna_manifest: DnaManifest = serde_yaml::from_str(
        app_file_tree
            .path(&mut v.iter())
            .ok_or(ScaffoldError::PathNotFound(dna_manifest_path.clone()))?
            .file_content()
            .ok_or(ScaffoldError::PathNotFound(dna_manifest_path.clone()))?,
    )?;

    let zome_wasm_location = zome_wasm_location(dna_manifest_path, &zome_name);

    let mut integrity_manifest = dna_manifest.integrity_manifest();

    integrity_manifest.zomes.push(ZomeManifest {
        dependencies: None,
        hash: None,
        name: zome_name.into(),
        location: zome_wasm_location,
    });

    let new_manifest: DnaManifest = DnaManifestCurrentBuilder::default()
        .coordinator(dna_manifest.coordinator_manifest())
        .integrity(integrity_manifest)
        .name(dna_manifest.name())
        .build()
        .unwrap()
        .into();

    let v: Vec<OsString> = dna_manifest_path.iter().map(|s| s.to_os_string()).collect();

    *app_file_tree
        .path_mut(&mut v.iter())
        .ok_or(ScaffoldError::PathNotFound(dna_manifest_path.clone()))?
        .file_content_mut()
        .ok_or(ScaffoldError::PathNotFound(dna_manifest_path.clone()))? =
        serde_yaml::to_string(&new_manifest)?;

    Ok(app_file_tree)
}
