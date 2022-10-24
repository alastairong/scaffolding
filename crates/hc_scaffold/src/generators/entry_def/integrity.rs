use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
};

use crate::{
    definitions::{EntryDefinition, FieldRepresentation, FieldType},
    file_tree::{find_files, find_map_rust_files, map_rust_files, FileTree},
};
use build_fs_tree::file;
use convert_case::{Case, Casing};
use holochain_types::prelude::{AppManifest, DnaManifest};
use prettyplease::unparse;
use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    error::{ScaffoldError, ScaffoldResult},
    generators::zome::utils::zome_manifest_path,
};

pub fn render_entry_definition_file(entry_def: &EntryDefinition) -> ScaffoldResult<syn::File> {
    let type_definitions: Vec<TokenStream> = entry_def
        .fields
        .values()
        .filter_map(|field_type| match field_type.representation.clone() {
            FieldRepresentation::Visible(widget) => widget.rust_type_definition(),
            _ => None,
        })
        .collect();

    let name: syn::Expr = syn::parse_str(entry_def.name.to_case(Case::Title).as_str())?;

    let fields: Vec<TokenStream> = entry_def
        .fields
        .iter()
        .map(|(key, value)| {
            let name: syn::Expr = syn::parse_str(key.to_case(Case::Snake).as_str())?;
            let rust_type = value.rust_type();
            Ok(quote! {  #name: #rust_type })
        })
        .collect::<ScaffoldResult<Vec<TokenStream>>>()?;

    let token_stream = quote! {
      use hdi::prelude::*;

      #(#type_definitions)*

      #[hdk_entry_helper]
      #[derive(Clone)]
      pub struct #name {
        #(#fields),*
      }
    };

    let file = syn::parse_file(token_stream.to_string().as_str())?;

    Ok(file)
}

pub fn add_entry_def_to_integrity_zome(
    mut app_file_tree: FileTree,
    app_manifest: &AppManifest,
    dna_manifest: &DnaManifest,
    integrity_zome_name: &String,
    entry_def: &EntryDefinition,
) -> ScaffoldResult<FileTree> {
    let integrity_zome = match dna_manifest {
        DnaManifest::V1(v1) => v1
            .integrity
            .zomes
            .clone()
            .into_iter()
            .find(|z| z.name.0.eq(integrity_zome_name)),
    }
    .ok_or(ScaffoldError::IntegrityZomeNotFound(
        integrity_zome_name.clone(),
        dna_manifest.name(),
    ))?;

    let mut manifest_path = zome_manifest_path(&app_file_tree, &integrity_zome)?.ok_or(
        ScaffoldError::IntegrityZomeNotFound(integrity_zome_name.clone(), dna_manifest.name()),
    )?;

    manifest_path.pop();

    let snake_entry_def_name = entry_def.name.to_case(Case::Snake);
    let entry_def_file = render_entry_definition_file(entry_def)?;

    // 1. Create an ENTRY_DEF_NAME.rs in "src/", with the entry definition struct
    let crate_src_path = manifest_path.join("src");
    let crate_src_path_iter: Vec<OsString> =
        crate_src_path.iter().map(|s| s.to_os_string()).collect();
    app_file_tree
        .path_mut(&mut crate_src_path_iter.iter())
        .ok_or(ScaffoldError::PathNotFound(crate_src_path.clone()))?
        .dir_content_mut()
        .ok_or(ScaffoldError::PathNotFound(crate_src_path.clone()))?
        .insert(
            OsString::from(format!("{}.rs", snake_entry_def_name.clone())),
            file!(unparse(&entry_def_file)),
        );

    // 2. Add this file as a module in the entry point for the crate

    let lib_rs_path = crate_src_path.join("lib.rs");
    let v: Vec<OsString> = lib_rs_path.iter().map(|s| s.to_os_string()).collect();
    app_file_tree
        .path_mut(&mut v.iter())
        .ok_or(ScaffoldError::PathNotFound(lib_rs_path.clone()))?
        .file_content_mut()
        .ok_or(ScaffoldError::PathNotFound(lib_rs_path.clone()))?
        .insert_str(
            0,
            format!(
                r#"pub mod {};
pub use {}::*;

"#,
                snake_entry_def_name, snake_entry_def_name,
            )
            .as_str(),
        );

    let title_entry_def_name = entry_def.name.to_case(Case::Title);

    // 3. Find the #[hdk_entry_defs] macro
    // 3.1 Import the new struct
    // 3.2 Add a variant for the new entry def with the struct as its payload
    map_rust_files(
        app_file_tree
            .path_mut(&mut crate_src_path_iter.iter())
            .ok_or(ScaffoldError::PathNotFound(crate_src_path.clone()))?,
        |file_path, mut file| {
            let mut found = false;

            file.items =
                file.items
                    .into_iter()
                    .map(|i| {
                        if let syn::Item::Enum(mut item_enum) = i.clone() {
                            if item_enum.attrs.iter().any(|a| {
                                a.path.segments.iter().any(|s| s.ident.eq("hdk_entry_defs"))
                            }) {
                                found = true;
                                let new_variant = syn::parse_str::<syn::Variant>(
                                    format!("{}({})", title_entry_def_name, title_entry_def_name)
                                        .as_str(),
                                )
                                .unwrap();
                                item_enum.variants.push(new_variant);
                                return syn::Item::Enum(item_enum);
                            }
                        }

                        i
                    })
                    .collect();

            // If the file is lib.rs, we already have the entry struct imported so no need to import it again
            if found && file_path.file_name() != Some(OsString::from("lib.rs").as_os_str()) {
                file.items
                    .insert(0, syn::parse_str::<syn::Item>("use crate::*;").unwrap());
            }

            Ok(file)
        },
    )
    .map_err(|e| match e {
        ScaffoldError::MalformedFile(path, error) => {
            ScaffoldError::MalformedFile(crate_src_path.join(&path), error)
        }
        _ => e,
    })?;

    Ok(app_file_tree)
}

pub fn get_all_entry_types(
    app_file_tree: &FileTree,
    app_manifest: &AppManifest,
    dna_manifest: &DnaManifest,
    integrity_zome_name: &String,
) -> ScaffoldResult<Vec<String>> {
    let integrity_zome = match dna_manifest {
        DnaManifest::V1(v1) => v1
            .integrity
            .zomes
            .clone()
            .into_iter()
            .find(|z| z.name.0.eq(integrity_zome_name)),
    }
    .ok_or(ScaffoldError::IntegrityZomeNotFound(
        integrity_zome_name.clone(),
        dna_manifest.name(),
    ))?;

    let mut manifest_path = zome_manifest_path(&app_file_tree, &integrity_zome)?.ok_or(
        ScaffoldError::IntegrityZomeNotFound(integrity_zome_name.clone(), dna_manifest.name()),
    )?;
    manifest_path.pop();

    let crate_src_path = manifest_path.join("src");
    let crate_src_path_iter: Vec<OsString> =
        crate_src_path.iter().map(|s| s.to_os_string()).collect();
    let entry_defs_instances = find_map_rust_files(
        app_file_tree
            .path(&mut crate_src_path_iter.iter())
            .ok_or(ScaffoldError::PathNotFound(crate_src_path.clone()))?,
        &|file_path, rust_file| {
            rust_file.items.iter().find_map(|i| {
                if let syn::Item::Enum(mut item_enum) = i.clone() {
                    if item_enum
                        .attrs
                        .iter()
                        .any(|a| a.path.segments.iter().any(|s| s.ident.eq("hdk_entry_defs")))
                    {
                        return Some(item_enum.clone());
                    }
                }

                None
            })
        },
    );

    match entry_defs_instances.len() {
        0 => Ok(vec![]),
        1 => {
            let entry_def_enum = entry_defs_instances.values().next().unwrap();

            let variants: Vec<String> = entry_def_enum
                .clone()
                .variants
                .into_iter()
                .map(|v| v.ident.to_string())
                .collect();

            Ok(variants)
        }
        _ => Err(ScaffoldError::MultipleEntryDefsFoundForIntegrityZome(
            dna_manifest.name(),
            integrity_zome_name.clone(),
        )),
    }
}
