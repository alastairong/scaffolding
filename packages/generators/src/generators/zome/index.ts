import { DnaDefinition, FileChanges, FileChangesType, ZomeDefinition } from '../../types';
import { camelToSnakeCase } from '../utils';

import cargoToml from './Cargo.toml';
import libRs from './lib.rs';

export function generateZomeCargoToml(zomeName: string, author: string, hdkVersion = '0.0.107'): FileChanges[] {
  return [
    {
      type: FileChangesType.Create,
      fileName: `Cargo.toml`,
      content: cargoToml({
        zomeName: camelToSnakeCase(zomeName),
        author,
        hdkVersion,
      }),
    },
  ];
}

export function generateZomeCode(zomeName: string): FileChanges[] {
  return [
    {
      type: FileChangesType.InDir,
      dirName: 'src',
      changes: [
        {
          type: FileChangesType.Create,
          fileName: `lib.rs`,
          content: libRs(),
        },
      ],
    },
  ];
}

export function generateZome(zome: ZomeDefinition): FileChanges[] {
  return [...generateZomeCargoToml(zome.name, '<AUTHOR>'), ...generateZomeCode(zome.name)];
}
