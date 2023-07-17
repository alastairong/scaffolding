use build_fs_tree::file;
use dirs::home_dir;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::error::{ScaffoldError, ScaffoldResult};
use crate::file_tree::*;
use crate::versions::holochain_nix_version;

pub fn flake_nix() -> FileTree {
    let holochain_nix_version = holochain_nix_version();
    file!(format!(
        r#"{{
  description = "Template for Holochain app development";

  inputs = {{
    holochain-nix-versions.url  = "github:holochain/holochain?dir=versions/{holochain_nix_version}";
    # TODO: next line is temporary. Only necessary for pinning this specific version of 0.3.x
    holochain-nix-versions.inputs.holochain.url = "github:holochain/holochain/holochain-0.3.0-beta-dev.9";

    holochain-flake = {{
      url = "github:holochain/holochain";
      inputs.versions.follows = "holochain-nix-versions";
    }};

    nixpkgs.follows = "holochain-flake/nixpkgs";
    flake-parts.follows = "holochain-flake/flake-parts";
  }};

  outputs = inputs @ {{ flake-parts, holochain-flake, ... }}:
    flake-parts.lib.mkFlake
      {{
        inherit inputs;
      }}
      {{
        systems = builtins.attrNames holochain-flake.devShells;
        perSystem =
          {{ config
          , pkgs
          , system
          , ...
          }}: {{
            devShells.default = pkgs.mkShell {{
              inputsFrom = [ holochain-flake.devShells.${{system}}.holonix ];
              packages = [ pkgs.nodejs-18_x ];
            }};
          }};
      }};
}}"#
    ))
}

pub fn setup_nix_developer_environment(dir: &PathBuf) -> ScaffoldResult<()> {
    if cfg!(target_os = "windows") {
        return Err(ScaffoldError::NixSetupError(
            "Windows doesn't support nix".to_string(),
        ));
    }

    println!("Setting up nix development environment...");

    add_extra_experimental_features()?;

    let output = Command::new("nix")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .current_dir(dir)
        .args(["flake", "update"])
        .output()?;

    if !output.status.success() {
        return Err(ScaffoldError::NixSetupError("".to_string()))?;
    }

    Ok(())
}

const EXTRA_EXPERIMENTAL_FEATURES_LINE: &'static str =
    "extra-experimental-features = flakes nix-command";

pub fn add_extra_experimental_features() -> ScaffoldResult<()> {
    let config_path = home_dir().ok_or(ScaffoldError::NixSetupError(
        "Config dir doesn't exist".to_string(),
    ))?;

    let nix_conf_dir = config_path.join(".config").join("nix");
    fs::create_dir_all(&nix_conf_dir)?;

    let nix_conf_path = nix_conf_dir.join("nix.conf");
    if let Ok(contents) = fs::read_to_string(&nix_conf_path) {
        if contents.contains(EXTRA_EXPERIMENTAL_FEATURES_LINE) {
            return Ok(());
        }
    }

    if let Ok(mut file) = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(nix_conf_path)
    {
        file.write_all(EXTRA_EXPERIMENTAL_FEATURES_LINE.as_bytes())?;
    } else {
        println!("Warning: could not write extra-experimental-features to nix.conf");
    }
    Ok(())
}
