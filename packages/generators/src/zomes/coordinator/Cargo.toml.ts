import { ScFile, ScNodeType } from '@source-craft/types';
import { getCoordinatorCrateName } from '../utils';

export const coordinatorZomeCargoToml = (
  zomeName: string,
  coordinatorCrateName: string,
  integrityCrateName: string,
  author: string,
  hdkVersion: string,
): ScFile => ({
  type: ScNodeType.File,
  content: `[package]
edition = "2021"
name = "${coordinatorCrateName}"
version = "0.0.1"

[lib]
crate-type = ["cdylib", "rlib"]
name = "${coordinatorCrateName}"

[dependencies]
serde = "=1.0.136"
chrono = { version = "0.4.22", default-features = false, features = ["clock", "std", "oldtime", "serde"], optional = true }
derive_more = "0"
${integrityCrateName} = { path = "../../integrity_zomes/${zomeName}" }

hdk = "${hdkVersion}"
`,
});
