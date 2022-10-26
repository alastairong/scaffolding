use handlebars::Handlebars;
use include_dir::{include_dir, Dir};

use crate::{
    error::ScaffoldResult,
    file_tree::FileTree,
    templates::{register_case_helpers, scaffold_dir},
};

use super::{AddEntryTypeComponentsData, ScaffoldWebAppData};

static VANILLA_WEB_APP: Dir<'static> =
    include_dir!("$CARGO_MANIFEST_DIR/templates/uis/vanilla/web-app");

static CREATE_ENTRY_COMPONENT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/uis/vanilla/create-entry-type.hbs"
));

pub fn scaffold_vanilla_web_app(data: &ScaffoldWebAppData) -> ScaffoldResult<FileTree> {
    scaffold_dir(&VANILLA_WEB_APP, data)
}

pub fn create_entry_component(data: &AddEntryTypeComponentsData) -> ScaffoldResult<String> {
    let h = Handlebars::new();
    let h = register_case_helpers(h);

    let s = h.render_template(CREATE_ENTRY_COMPONENT, data)?;

    Ok(s)
}
