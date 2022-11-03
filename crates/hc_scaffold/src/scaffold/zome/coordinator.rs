use std::{collections::BTreeMap, ffi::OsString, path::PathBuf};

use crate::{
    file_tree::{
        dna_file_tree::DnaFileTree, find_map_rust_files, insert_file, zome_file_tree::ZomeFileTree,
        FileTree,
    },
    generators::dna::utils::read_dna_manifest,
};
use dialoguer::{theme::ColorfulTheme, Select};
use holochain_types::prelude::{
    DnaManifest, DnaManifestCurrentBuilder, ZomeDependency, ZomeManifest, ZomeName,
};

use crate::error::{ScaffoldError, ScaffoldResult};

use super::utils::zome_wasm_location;

pub fn initial_cargo_toml(zome_name: &String, dependencies: &Option<Vec<String>>) -> String {
    let deps = match dependencies {
        Some(d) => d
            .into_iter()
            .map(|d| format!(r#"{} = {{ workspace = true }}"#, d))
            .collect::<Vec<String>>()
            .join("\n"),
        None => String::from(""),
    };

    format!(
        r#"[package]
name = "{}"
version = "0.0.1"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]
name = "{}"

[dependencies]
hdk = {{ workspace = true }}
serde = {{ workspace = true }}

{} 
"#,
        zome_name, zome_name, deps
    )
}

pub fn initial_lib_rs() -> String {
    format!(
        r#"use hdk::prelude::*;

#[hdk_extern]
pub fn init(_: ()) -> ExternResult<InitCallbackResult> {{
  Ok(InitCallbackResult::Pass)
}}
"#
    )
}

fn choose_extern_function(
    functions_by_zome: &BTreeMap<String, Vec<String>>,
    prompt: &String,
) -> ScaffoldResult<(String, String)> {
    let all_functions: Vec<(String, String)> = functions_by_zome
        .iter()
        .map(|(z, fns)| {
            fns.iter()
                .map(|f| (z.clone(), f.clone()))
                .collect::<Vec<(String, String)>>()
        })
        .flatten()
        .collect();
    let all_fns_str: Vec<String> = all_functions
        .iter()
        .map(|(z, f)| format!(r#""{}", in zome "{}""#, f, z))
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt.as_str())
        .default(0)
        .items(&all_fns_str[..])
        .interact()?;

    Ok(all_functions[selection].clone())
}

pub fn find_extern_function_or_choose(
    app_file_tree: &FileTree,
    dna_manifest: &DnaManifest,
    coordinator_zomes: &Vec<ZomeManifest>,
    fn_name_to_find: &String,
    prompt: &String,
) -> ScaffoldResult<(ZomeManifest, String)> {
    let mut functions_by_zome: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for coordinator_zome in coordinator_zomes {
        let all_extern_functions =
            find_all_extern_functions(&app_file_tree, dna_manifest, coordinator_zome)?;

        if all_extern_functions.contains(&fn_name_to_find) {
            return Ok((coordinator_zome.clone(), fn_name_to_find.clone()));
        }

        functions_by_zome.insert(coordinator_zome.name.to_string(), all_extern_functions);
    }

    let (zome_name, fn_name) = choose_extern_function(&functions_by_zome, &prompt)?;

    let chosen_zome = coordinator_zomes
        .iter()
        .find(|z| z.name.to_string().eq(&zome_name));

    match chosen_zome {
        Some(z) => Ok((z.clone(), fn_name)),
        None => Err(ScaffoldError::CoordinatorZomeNotFound(
            zome_name.clone(),
            dna_manifest.name(),
        )),
    }
}

pub fn find_all_extern_functions(
    app_file_tree: &FileTree,
    dna_manifest: &DnaManifest,
    coordinator_zome: &ZomeManifest,
) -> ScaffoldResult<Vec<String>> {
    let mut manifest_path = zome_manifest_path(&app_file_tree, &coordinator_zome)?.ok_or(
        ScaffoldError::CoordinatorZomeNotFound(
            coordinator_zome.name.0.to_string(),
            dna_manifest.name(),
        ),
    )?;

    manifest_path.pop();

    let crate_src_path = manifest_path.join("src");
    let crate_src_path_iter: Vec<OsString> =
        crate_src_path.iter().map(|s| s.to_os_string()).collect();
    let hdk_extern_instances = find_map_rust_files(
        app_file_tree
            .path(&mut crate_src_path_iter.iter())
            .ok_or(ScaffoldError::PathNotFound(crate_src_path.clone()))?,
        &|_file_path, rust_file| {
            rust_file.items.iter().find_map(|i| {
                if let syn::Item::Fn(item_fn) = i.clone() {
                    if item_fn
                        .attrs
                        .iter()
                        .any(|a| a.path.segments.iter().any(|s| s.ident.eq("hdk_extern")))
                    {
                        return Some(item_fn.sig.ident.to_string());
                    }
                }

                None
            })
        },
    );

    Ok(hdk_extern_instances.values().cloned().collect())
}
