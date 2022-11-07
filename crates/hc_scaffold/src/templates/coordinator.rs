use std::{ffi::OsString, path::PathBuf};

use holochain_types::prelude::ZomeManifest;
use serde::Serialize;

use crate::{
    definitions::EntryDefinition, error::ScaffoldResult, file_tree::FileTree,
    scaffold::entry_type::DependsOnItself,
};

use super::{build_handlebars, render_template_file_tree_and_merge_with_existing};

#[derive(Serialize)]
pub struct ScaffoldCoordinatorZomeData {
    dna_role_id: String,
    zome_manifest: ZomeManifest,
}

pub fn scaffold_coordinator_zome_templates(
    mut app_file_tree: FileTree,
    template_file_tree: &FileTree,
    dna_role_id: &String,
    zome_manifest: &ZomeManifest,
) -> ScaffoldResult<FileTree> {
    let data = ScaffoldCoordinatorZomeData {
        dna_role_id: dna_role_id.clone(),
        zome_manifest: zome_manifest.clone(),
    };

    let h = build_handlebars(&template_file_tree)?;

    let field_types_path = PathBuf::from("coordinator-zome");
    let v: Vec<OsString> = field_types_path.iter().map(|s| s.to_os_string()).collect();

    if let Some(web_app_template) = template_file_tree.path(&mut v.iter()) {
        app_file_tree = render_template_file_tree_and_merge_with_existing(
            app_file_tree,
            &h,
            web_app_template,
            &data,
        )?;
    }

    Ok(app_file_tree)
}
