import { ZomeDefinition } from '@holochain-scaffolding/definitions';
import { PatcherFile, PatcherNodeType } from '@patcher/types';
import { mergeStrings, titleCase } from '../utils';

export const libRs = (zomeDefinition: ZomeDefinition): PatcherFile => ({
  type: PatcherNodeType.File,
  content: `use hdk::prelude::*;
${mergeStrings(
  zomeDefinition.entry_defs.map(
    entry_def => `
mod ${entry_def.name};`,
  ),
)}
${mergeStrings(
  zomeDefinition.entry_defs.map(
    entry_def => `
use ${entry_def.name}::${titleCase(entry_def.name)};`,
  ),
)}

entry_defs![${mergeStrings(
    zomeDefinition.entry_defs.map(
      entry_def => `
  ${titleCase(entry_def.name)}::entry_def()`,
    ),
    ',',
  )}
];

`,
});
