use crate::generators::{self, dna::scaffold_dna};
use build_fs_tree::{Build, MergeableFileSystemTree};
use holochain_scaffolding_utils::load_directory_into_memory;
use holochain_types::{prelude::AppManifest, web_app::WebAppManifest};
use holochain_util::ffs;
use mr_bundle::{Bundle, Location, Manifest};
use std::{path::PathBuf, process::Command};
use structopt::StructOpt;

/// The list of subcommands for `hc sandbox`
#[derive(Debug, StructOpt)]
#[structopt(setting = structopt::clap::AppSettings::InferSubcommands)]
pub enum HcScaffold {
    /// Scaffold a new web app
    WebApp {
        /// Name of the app to scaffold
        name: String,

        /// [OPTIONAL] Description of the app to scaffold
        description: Option<String>,
    },
    Dna {
        /// Name of the DNA being scaffolded
        name: String,
    },
    Pack(Pack),
}

#[derive(Debug, StructOpt)]
#[structopt(setting = structopt::clap::AppSettings::InferSubcommands)]
pub struct HcScaffoldDna {
    /// Name of the DNA being scaffolded
    name: String,
}

/// The list of subcommands for `hc sandbox`
#[derive(Debug, StructOpt)]
#[structopt(setting = structopt::clap::AppSettings::InferSubcommands)]
pub enum Pack {
    /// Scaffold a new web app
    WebApp {
        /// The path to the working directory containing a `web-happ.yaml` manifest
        path: PathBuf,
    },
    App {
        /// The path to the working directory containing a `happ.yaml` manifest
        path: PathBuf,
    },
}

impl HcScaffold {
    pub async fn run(self) -> anyhow::Result<()> {
        match self {
            HcScaffold::WebApp { name, description } => {
                generators::web_app::scaffold_web_app(name.clone(), description)?;

                if cfg!(target_os = "windows") {
                    return Err(anyhow::anyhow!("Windows doesn't support nix"));
                } else {
                    Command::new("nix-shell")
                        .current_dir(std::env::current_dir()?.join(name))
                        .args(["-I", "nixpkgs=https://github.com/NixOS/nixpkgs/archive/nixos-21.11.tar.gz", "-p", "niv", "--run", "niv init && niv drop nixpkgs && niv drop niv && niv add -b main holochain/holonix"])
                        .output()?;
                };
            }
            HcScaffold::Dna { name } => {
                let current_dir = std::env::current_dir()?;

                let app_file_tree = load_directory_into_memory(&current_dir)?;
                let file_tree = scaffold_dna(app_file_tree, name)?;

                let file_tree = MergeableFileSystemTree::<String, String>::from(file_tree);

                file_tree.build(&".".into())?;
            }
            HcScaffold::Pack(Pack::WebApp { path }) => web_app_pack_all_bundled(path).await?,
            HcScaffold::Pack(Pack::App { path }) => app_pack_all_bundled(path).await?,
        }

        Ok(())
    }
}

async fn web_app_pack_all_bundled(web_app_bundle_path: PathBuf) -> anyhow::Result<()> {
    let web_app_bundle_path = ffs::canonicalize(web_app_bundle_path).await?;

    let f = std::fs::File::open(web_app_bundle_path.join(WebAppManifest::path()))?;

    let manifest: WebAppManifest = serde_yaml::from_reader(f)?;

    let location = manifest.happ_bundle_location();

    if let Location::Bundled(mut bundled_location) = location {
        bundled_location.pop();
        bundled_location = PathBuf::new()
            .join(web_app_bundle_path.clone())
            .join(bundled_location);

        app_pack_all_bundled(bundled_location).await?;
    }

    holochain_cli_bundle::HcWebAppBundle::Pack {
        path: web_app_bundle_path,
        output: None,
    }
    .run()
    .await?;

    Ok(())
}

async fn app_pack_all_bundled(app_bundle_path: PathBuf) -> anyhow::Result<()> {
    let app_bundle_path = ffs::canonicalize(app_bundle_path).await?;

    let f = std::fs::File::open(app_bundle_path.join(AppManifest::path()))?;

    let manifest: AppManifest = serde_yaml::from_reader(f)?;

    for app_role in manifest.app_roles() {
        if let Some(Location::Bundled(mut bundled_location)) = app_role.dna.location {
            bundled_location.pop();

            holochain_cli_bundle::HcDnaBundle::Pack {
                path: bundled_location,
                output: None,
            }
            .run()
            .await?;
        }
    }

    holochain_cli_bundle::HcAppBundle::Pack {
        path: app_bundle_path,
        output: None,
    }
    .run()
    .await?;

    Ok(())
}
