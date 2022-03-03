import { TypeDefinition } from '@type-craft/vocabulary';
import { TypeElementsImportDeclarations } from '@type-craft/web-components';
import { TypescriptTypeGenerator } from '@type-craft/typescript';
import { RustTypeGenerator } from '@type-craft/rust';

export const type: TypeDefinition<string, {}> = {
  name: 'HeaderHash',
  description: 'A hash of a Holochain header',

  sample: () => 'uhCkkr6pGIyV6_lr2MbT_Siw0DXZInPa0cgA9B9Sq1NtokBr0IiM2',
};

export const tsGenerator: TypescriptTypeGenerator = {
  imports: [],
  defineType: 'export type HeaderHashB64 = string;',
  referenceType: 'HeaderHashB64',
};

export function rustGenerator(hdkVersion: string): RustTypeGenerator {
  return {
    imports: [
      {
        crateName: 'hdk',
        importDeclaration: `use hdk::prelude::holo_hash::HeaderHashB64;`,
        version: hdkVersion,
      },
    ],
    defineType: '',
    referenceType: 'HeaderHashB64',
  };
}

export const elementsImports: TypeElementsImportDeclarations = {
  detail: {
    sideEffectImport: {
      importDeclaration: `import '@holochain-open-dev/utils/copiable-hash';`,
      packageName: '@holochain-open-dev/utils',
      version: '^0.0.1',
    },
    tagName: 'copiable-hash',
  },
};
