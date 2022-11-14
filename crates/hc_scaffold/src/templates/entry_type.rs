use std::{collections::BTreeMap, ffi::OsString, path::PathBuf};

use holochain_types::prelude::ZomeManifest;
use serde::Serialize;

use crate::{
    error::ScaffoldResult,
    file_tree::FileTree,
    scaffold::entry_type::{
        crud::Crud,
        definitions::{Cardinality, DependsOn, EntryDefinition},
        DependsOnItself,
    },
};

use super::{build_handlebars, render_template_file_tree_and_merge_with_existing};

#[derive(Serialize)]
pub struct ScaffoldEntryTypeData {
    dna_role_id: String,
    coordinator_zome_manifest: ZomeManifest,
    entry_type: EntryDefinition,
    crud: Crud,
    link_from_original_to_each_update: bool,
}
pub fn scaffold_entry_type_templates(
    mut app_file_tree: FileTree,
    template_file_tree: &FileTree,
    dna_role_id: &String,
    coordinator_zome: &ZomeManifest,
    entry_type: &EntryDefinition,
    crud: &Crud,
    link_from_original_to_each_update: bool,
) -> ScaffoldResult<FileTree> {
    let data = ScaffoldEntryTypeData {
        dna_role_id: dna_role_id.clone(),
        coordinator_zome_manifest: coordinator_zome.clone(),
        entry_type: entry_type.clone(),
        crud: crud.clone(),
        link_from_original_to_each_update: link_from_original_to_each_update.clone(),
    };

    let h = build_handlebars(&template_file_tree)?;

    let field_types_path = PathBuf::from("entry-type");
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
