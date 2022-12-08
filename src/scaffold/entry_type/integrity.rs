use convert_case::{Case, Casing};
use prettyplease::unparse;
use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use std::{ffi::OsString, path::PathBuf};

use crate::error::{ScaffoldError, ScaffoldResult};
use crate::file_tree::insert_file;
use crate::scaffold::dna::DnaFileTree;
use crate::scaffold::zome::coordinator::find_extern_function_in_zomes;
use crate::scaffold::zome::utils::get_coordinator_zomes_for_integrity;
use crate::{
    file_tree::{find_map_rust_files, map_file, map_rust_files},
    scaffold::zome::ZomeFileTree,
};

use super::crud::Crud;
use super::definitions::{EntryDefinition, EntryTypeReference};

pub fn render_entry_definition_struct(entry_def: &EntryDefinition) -> ScaffoldResult<TokenStream> {
    let name: syn::Expr = syn::parse_str(entry_def.name.to_case(Case::Pascal).as_str())?;

    let fields: Vec<TokenStream> = entry_def
        .fields
        .iter()
        .map(|field_def| {
            let name: syn::Expr =
                syn::parse_str(field_def.field_name.to_case(Case::Snake).as_str())?;
            let rust_type = field_def.rust_type();
            Ok(quote! {  #name: #rust_type })
        })
        .collect::<ScaffoldResult<Vec<TokenStream>>>()?;
    Ok(quote! {

      pub struct #name {
        #(pub #fields),*
      }

    })
}

pub fn render_entry_definition_file(entry_def: &EntryDefinition, crud: &Crud) -> ScaffoldResult<syn::File> {
    let entry_def_token_stream = render_entry_definition_struct(entry_def)?;
    let name_pascal: syn::Expr = syn::parse_str(entry_def.name.to_case(Case::Pascal).as_str())?;
    let name_snake: syn::Expr = syn::parse_str(entry_def.name.to_case(Case::Snake).as_str())?;
    let plural_name_title = pluralizer::pluralize(entry_def.name.as_str(), 2, false).to_case(Case::Title);

    let type_definitions: Vec<TokenStream> = entry_def
        .fields
        .iter()
        .filter_map(|field_def| field_def.field_type.rust_type_definition())
        .collect();

    let validate_update_fn = format_ident!("validate_update_{}", entry_def.name.to_case(Case::Snake));
    let new_post_arg = format_ident!("new_{}", entry_def.name.to_case(Case::Snake));

    let updated_invalid_reason = format!("{} cannot be updated", plural_name_title);

    let validate_update_result: TokenStream = match crud.update {
        true => quote! {
            // TODO: add the appropriate validation rules
            Ok(ValidateCallbackResult::Valid)
        },
        false => quote! {
             Ok(ValidateCallbackResult::Invalid(String::from(#updated_invalid_reason)))
        },
    };
    let validate_update: TokenStream = quote! {
        pub fn #validate_update_fn(#new_post_arg: #name_pascal) -> ExternResult<ValidateCallbackResult> {
            #validate_update_result
        }
    };

    let validate_delete_fn = format_ident!("validate_delete_{}", entry_def.name.to_case(Case::Snake));
    let deleted_post_arg = format_ident!("deleted_{}", entry_def.name.to_case(Case::Snake));
    
    let deleted_invalid_reason = format!("{} cannot be deleted", plural_name_title);

    let validate_delete_result: TokenStream = match crud.update {
        true => quote! {
            // TODO: add the appropriate validation rules
            Ok(ValidateCallbackResult::Valid)
        },
        false => quote! {
             Ok(ValidateCallbackResult::Invalid(String::from(#deleted_invalid_reason)))
        },
    };
    let validate_delete: TokenStream = quote! {
        pub fn #validate_delete_fn(#deleted_post_arg: #name_pascal) -> ExternResult<ValidateCallbackResult> {
            #validate_delete_result
        }
    };
    
    let validate_create_fn = format_ident!("validate_create_{}", entry_def.name.to_case(Case::Snake));

    let token_stream = quote! {
      use hdi::prelude::*;

      #(#type_definitions)*

      #[hdk_entry_helper]
      #[derive(Clone)]
      #entry_def_token_stream

      pub fn #validate_create_fn(#name_snake: #name_pascal) -> ExternResult<ValidateCallbackResult> {
          // TODO: add the appropriate validation rules
          Ok(ValidateCallbackResult::Valid)
      }

      #validate_update
        
      #validate_delete
    };

    let file = syn::parse_file(token_stream.to_string().as_str())?;

    Ok(file)
}

fn find_ending_match_expr<'a>(e: &'a mut syn::Expr) -> Option<&'a mut syn::ExprMatch> {
    match e {
        syn::Expr::Match(expr_match) => Some(expr_match),
        syn::Expr::Block(expr_block) => {
if let Some(e) =            expr_block.block.stmts.last_mut(){
                 match e {syn::Stmt::Expr(syn::Expr::Match(e_m))=> Some(e_m), _=> None}

                
            }else {None}
        },
        _ => None
    }
}

pub fn add_entry_type_to_integrity_zome(
    zome_file_tree: ZomeFileTree,
    entry_def: &EntryDefinition,
    crud: &Crud
) -> ScaffoldResult<ZomeFileTree> {
    let dna_manifest_path = zome_file_tree.dna_file_tree.dna_manifest_path.clone();
    let dna_manifest = zome_file_tree.dna_file_tree.dna_manifest.clone();
    let zome_manifest = zome_file_tree.zome_manifest.clone();

    let snake_entry_def_name = entry_def.name.to_case(Case::Snake);
    let entry_def_file = render_entry_definition_file(entry_def, crud)?;

    let entry_types = get_all_entry_types(&zome_file_tree)?;

    // 1. Create an ENTRY_DEF_NAME.rs in "src/", with the entry definition struct
    let crate_src_path = zome_file_tree.zome_crate_path.join("src");

    let entry_def_path = crate_src_path.join(format!("{}.rs", snake_entry_def_name));

    let mut file_tree = zome_file_tree.dna_file_tree.file_tree();

    insert_file(&mut file_tree, &entry_def_path, &unparse(&entry_def_file))?;

    // 2. Add this file as a module in the entry point for the crate

    let lib_rs_path = crate_src_path.join("lib.rs");

    map_file(&mut file_tree, &lib_rs_path, |s| {
        format!(
            r#"pub mod {};
pub use {}::*;

{}"#,
            snake_entry_def_name, snake_entry_def_name, s
        )
    })?;

    let pascal_entry_def_name = entry_def.name.to_case(Case::Pascal);

    let v: Vec<OsString> = crate_src_path
        .clone()
        .iter()
        .map(|s| s.to_os_string())
        .collect();
    // 3. Find the #[hdk_entry_defs] macro
    // 3.1 Import the new struct
    // 3.2 Add a variant for the new entry def with the struct as its payload
    map_rust_files(
        file_tree
            .path_mut(&mut v.iter())
            .ok_or(ScaffoldError::PathNotFound(crate_src_path.clone()))?,
        |file_path, mut file| {
            let mut found = false;

            // If there are no entry types definitions in this zome, first add the empty enum
            if entry_types.is_none() && file_path == PathBuf::from("lib.rs") {
                file.items.push(syn::parse_str::<syn::Item>(
                    "#[hdk_entry_defs]
                     #[unit_enum(UnitEntryTypes)]
                      pub enum EntryTypes {}
                        ",
                )?);

                for item in &mut file.items {
                    if let syn::Item::Fn(item_fn) = item {
                        if item_fn.sig.ident.to_string().eq(&String::from("validate")) {
                            for stmt in &mut item_fn.block.stmts {
                                if let syn::Stmt::Expr(syn::Expr::Match(match_expr)) = stmt {
                                    if let syn::Expr::Try(try_expr) = &mut *match_expr.expr {
                                        if let syn::Expr::MethodCall(call) = &mut *try_expr.expr {
                                            if call.method.to_string().eq(&String::from("to_type"))
                                            {
                                                if let Some(turbofish) = &mut call.turbofish {
                                                    if let Some(first_arg) =
                                                        turbofish.args.first_mut()
                                                    {
                                                        *first_arg =
                                                            syn::GenericMethodArgument::Type(
                                                                syn::parse_str::<syn::Type>(
                                                                    "EntryTypes",
                                                                )?,
                                                            );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            file.items = file
                .items
                .into_iter()
                .map(|mut i| {
                    if let syn::Item::Enum(mut item_enum) = i.clone() {
                        if item_enum
                            .attrs
                            .iter()
                            .any(|a| a.path.segments.iter().any(|s| s.ident.eq("hdk_entry_defs")))
                        {
                            if item_enum
                                .variants
                                .iter()
                                .any(|v| v.ident.to_string().eq(&pascal_entry_def_name))
                            {
                                return Err(ScaffoldError::EntryTypeAlreadyExists(
                                    pascal_entry_def_name.clone(),
                                    dna_manifest.name(),
                                    zome_file_tree.zome_manifest.name.0.to_string(),
                                ));
                            }

                            found = true;
                            let new_variant = syn::parse_str::<syn::Variant>(
                                format!("{}({})", pascal_entry_def_name, pascal_entry_def_name)
                                    .as_str(),
                            )
                            .unwrap();
                            item_enum.variants.push(new_variant);
                            return Ok(syn::Item::Enum(item_enum));
                        }
                    }
                    if let syn::Item::Fn(item_fn) = &mut i {
                        if item_fn.sig.ident.to_string().eq(&String::from("validate")) {
                            for stmt in &mut item_fn.block.stmts {
                                if let syn::Stmt::Expr(syn::Expr::Match(match_expr)) = stmt {
                                    if let syn::Expr::Try(try_expr) = &mut *match_expr.expr {
                                        if let syn::Expr::MethodCall(call) = &mut *try_expr.expr {
                                            if call.method.to_string().eq(&String::from("to_type"))
                                            {
                                                for arm in &mut match_expr.arms {
                                                    if let syn::Pat::TupleStruct(pat_tuple_struct) =
                                                        &mut arm.pat
                                                    {
                                                        if let Some(path_segment) =
                                                            pat_tuple_struct.path.segments.last()
                                                        {
                                                            let path_segment_str = path_segment
                                                                .ident
                                                                .to_string();
                                                                
                                                            if path_segment_str.eq(&String::from("StoreEntry"))
                                                            {
                                                                if let Some(op_entry_match_expr) = find_ending_match_expr(&mut *arm.body)
                                                                {
                                                                    for op_entry_arm in
                                                                        &mut op_entry_match_expr
                                                                            .arms
                                                                    {
                                                                       if let syn::Pat::Struct(pat_struct) = &mut op_entry_arm.pat {
                                                                            if let Some(ps) = pat_struct.path.segments.last() {
                                                                                if ps.ident.to_string().eq(&String::from("CreateEntry")) || ps.ident.to_string().eq(&String::from("UpdateEntry")) {
                                                                                    
                                                                                    // Add new entry type to match arm
                                                                                    if let Some(_) = find_ending_match_expr( &mut *op_entry_arm.body) {} else {
                                                                                        // Change empty invalid to match on entry_type
                                                                                        *op_entry_arm.body = syn::parse_str::<syn::Expr>("match entry_type {}")?;
                                                                                    }
                                                                                    
                                                                                    // Add new entry type to match arm
                                                                                    if let Some(entry_type_match) = find_ending_match_expr(&mut *op_entry_arm.body) {                                                                                        
                                                                                        let new_arm: syn::Arm = syn::parse_str(
                                                                                            format!("EntryTypes::{}({}) => {}::validate_create_{}({}),", 
                                                                                                entry_def.name.to_case(Case::Title), 
                                                                                                entry_def.name.to_case(Case::Snake), 
                                                                                                entry_def.name.to_case(Case::Snake), 
                                                                                                entry_def.name.to_case(Case::Snake), 
                                                                                                entry_def.name.to_case(Case::Snake)).as_str()
                                                                                        )?;
                                                                                        entry_type_match.arms.push(new_arm);
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            } else if path_segment_str.eq(&String::from("RegisterUpdate")) {
                                                                if let Some(op_entry_match_expr) = find_ending_match_expr(&mut *arm.body)
                                                                {
                                                                    for op_entry_arm in
                                                                        &mut op_entry_match_expr
                                                                            .arms
                                                                    {
                                                                       if let syn::Pat::Struct(pat_struct) = &mut op_entry_arm.pat {
                                                                            if let Some(ps) = pat_struct.path.segments.last() {
                                                                                if ps.ident.to_string().eq(&String::from("Entry")) {                                                                              
                                                                                    // Add new entry type to match arm
                                                                                    if let Some(_) = find_ending_match_expr( &mut *op_entry_arm.body) {} else {
                                                                                        // Change empty invalid to match on entry_type
                                                                                        *op_entry_arm.body = syn::parse_str::<syn::Expr>("match new_entry_type {}")?;
                                                                                    }
                                                                                    
                                                                                    // Add new entry type to match arm
                                                                                    if let Some(entry_type_match) = find_ending_match_expr(&mut *op_entry_arm.body) {                                                                                        
                                                                                        let new_arm: syn::Arm = syn::parse_str(
                                                                                            format!("EntryTypes::{}({}) => {}::validate_update_{}({}),", 
                                                                                                entry_def.name.to_case(Case::Title), 
                                                                                                entry_def.name.to_case(Case::Snake), 
                                                                                                entry_def.name.to_case(Case::Snake), 
                                                                                                entry_def.name.to_case(Case::Snake), 
                                                                                                entry_def.name.to_case(Case::Snake)).as_str()
                                                                                        )?;
                                                                                        entry_type_match.arms.push(new_arm);
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            } else if path_segment_str.eq(&String::from("RegisterDelete")) {
                                                                if let Some(op_entry_match_expr) = find_ending_match_expr(&mut *arm.body)
                                                                {
                                                                    for op_entry_arm in
                                                                        &mut op_entry_match_expr
                                                                            .arms
                                                                    {
                                                                       if let syn::Pat::Struct(pat_struct) = &mut op_entry_arm.pat {
                                                                            if let Some(ps) = pat_struct.path.segments.last() {
                                                                                if ps.ident.to_string().eq(&String::from("Entry")) {                                                                              
                                                                                    // Add new entry type to match arm
                                                                                    if let Some(_) = find_ending_match_expr( &mut *op_entry_arm.body) {} else {
                                                                                        // Change empty invalid to match on entry_type
                                                                                        *op_entry_arm.body = syn::parse_str::<syn::Expr>("match original_entry_type {}")?;
                                                                                    }
                                                                                    
                                                                                    // Add new entry type to match arm
                                                                                    if let Some(entry_type_match) = find_ending_match_expr(&mut *op_entry_arm.body) {                                                                                        
                                                                                        let new_arm: syn::Arm = syn::parse_str(
                                                                                            format!("EntryTypes::{}({}) => {}::validate_delete_{}({}),", 
                                                                                                entry_def.name.to_case(Case::Title), 
                                                                                                entry_def.name.to_case(Case::Snake), 
                                                                                                entry_def.name.to_case(Case::Snake), 
                                                                                                entry_def.name.to_case(Case::Snake), 
                                                                                                entry_def.name.to_case(Case::Snake)).as_str()
                                                                                        )?;
                                                                                        entry_type_match.arms.push(new_arm);
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    Ok(i)
                })
                .collect::<ScaffoldResult<Vec<syn::Item>>>()?;

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

    let dna_file_tree = DnaFileTree::from_dna_manifest_path(file_tree, &dna_manifest_path)?;
    let zome_file_tree = ZomeFileTree::from_zome_manifest(dna_file_tree, zome_manifest)?;

    Ok(zome_file_tree)
}

pub fn get_all_entry_types(
    zome_file_tree: &ZomeFileTree,
) -> ScaffoldResult<Option<Vec<EntryTypeReference>>> {
    let crate_src_path = zome_file_tree.zome_crate_path.join("src");
    let crate_src_path_iter: Vec<OsString> =
        crate_src_path.iter().map(|s| s.to_os_string()).collect();
    let entry_defs_instances = find_map_rust_files(
        zome_file_tree
            .dna_file_tree
            .file_tree_ref()
            .path(&mut crate_src_path_iter.iter())
            .ok_or(ScaffoldError::PathNotFound(crate_src_path.clone()))?,
        &|_file_path, rust_file| {
            rust_file.items.iter().find_map(|i| {
                if let syn::Item::Enum(item_enum) = i.clone() {
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
        0 => Ok(None),
        1 => {
            let entry_def_enum = entry_defs_instances.values().next().unwrap();

            let variants: Vec<String> = entry_def_enum
                .clone()
                .variants
                .into_iter()
                .map(|v| v.ident.to_string())
                .collect();

            let coordinators_for_zome = get_coordinator_zomes_for_integrity(
                &zome_file_tree.dna_file_tree.dna_manifest,
                &zome_file_tree.zome_manifest.name.0.to_string(),
            );

            let mut entry_types: Vec<EntryTypeReference> = Vec::new();

            let entry_hash_type: syn::Type = syn::parse_str("EntryHash")?;

            for v in variants {
                let referenced_by_entry_hash = match find_extern_function_in_zomes(
                    &zome_file_tree.dna_file_tree,
                    &coordinators_for_zome,
                    &format!("read_{}", v),
                )? {
                    Some((_z, item_fn)) => {
                        match item_fn
                            .sig
                            .inputs
                            .first()
                            .expect("Extern function must have one argument")
                        {
                            syn::FnArg::Typed(typed) => entry_hash_type.eq(&(*typed.ty)),
                            _ => false,
                        }
                    }
                    None => false,
                };
                entry_types.push(EntryTypeReference {
                    entry_type: v,
                    reference_entry_hash: referenced_by_entry_hash,
                });
            }

            Ok(Some(entry_types))
        }
        _ => Err(ScaffoldError::MultipleEntryTypesDefsFoundForIntegrityZome(
            zome_file_tree.dna_file_tree.dna_manifest.name(),
            zome_file_tree.zome_manifest.name.0.to_string(),
        )),
    }
}
