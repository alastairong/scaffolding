import { TypeDefinition } from '@type-craft/vocabulary';
import { TypeElementsImportDeclarations } from '@type-craft/web-components';
import { TypescriptTypeGenerator } from '@type-craft/typescript';
import { RustTypeGenerator } from '@type-craft/rust';

export const type: TypeDefinition<string, {}> = {
  name: 'AgentPubKey',
  description: 'The identifier of an Agent in Holochain',

  sample: () => 'uhCAkr6pGIyV6_lr2MbT_Siw0DXZInPa0cgA9B9Sq1NtokBr0IiM2',
};

export const tsGenerator: TypescriptTypeGenerator = {
  imports: [],
  defineType: 'export type AgentPubKeyB64 = string;',
  referenceType: 'AgentPubKeyB64',
};

export function rustGenerator(hdkVersion: string): RustTypeGenerator {
  return {
    imports: [
      {
        crateName: 'hdk',
        importDeclaration: `use hdk::prelude::holo_hash::AgentPubKeyB64;`,
        version: hdkVersion,
      },
    ],
    defineType: '',
    referenceType: 'AgentPubKeyB64',
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
