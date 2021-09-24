import { DnaDefinition, FileChanges, FileChangesType, HappDefinition } from '../../types';
import { generateDna, generateDnaYaml } from '../dna';
import { generateNixFile } from '../nix';

import cargoToml from '../happ/Cargo.toml';

import happYaml from './happ.yaml';
import { generateRootPackageJson } from '../npm';
import { generateTryorama } from '../tryorama';
import { generateGithubWorkfows } from '../github';
import gitignore from './gitignore';

export function generateHapp(happ: HappDefinition): FileChanges[] {
  return [
    ...generateNixFile(),
    ...generateHappWorkdir(happ),
    ...generateWorkspaceCargoToml(happ),
    ...generateDnas(happ),
    ...generateRootPackageJson(happ),
    ...generateGithubWorkfows(happ),
    ...generateGitIgnore(),
    {
      type: FileChangesType.InDir,
      dirName: 'tests',
      changes: generateTryorama(happ),
    },
  ];
}

export function generateHappYaml(happ: HappDefinition): FileChanges[] {
  return [
    {
      type: FileChangesType.Create,
      fileName: 'happ.yaml',
      content: happYaml(happ),
    },
  ];
}

export function generateGitIgnore(): FileChanges[] {
  return [
    {
      type: FileChangesType.Create,
      fileName: '.gitignore',
      content: gitignore(),
    },
  ];
}

export function generateHappWorkdir(happ: HappDefinition): FileChanges[] {
  return [
    {
      type: FileChangesType.InDir,
      dirName: 'workdir',
      changes: generateHappYaml(happ),
    },
  ];
}

function generateDnas(happ: HappDefinition): FileChanges[] {
  if (happ.dnas.length === 1)
    return [
      {
        type: FileChangesType.InDir,
        dirName: 'dna',
        changes: generateDna(happ.dnas[0], '../'),
      },
    ];

  return [
    {
      type: FileChangesType.InDir,
      dirName: 'dnas',
      changes: happ.dnas.map(dna => ({
        type: FileChangesType.InDir,
        dirName: dna.name,
        changes: generateDna(dna, '../../'),
      })),
    },
  ];
}

export function generateWorkspaceCargoToml(happ: HappDefinition): FileChanges[] {
  return [
    {
      type: FileChangesType.Create,
      fileName: 'Cargo.toml',
      content: cargoToml(happ),
    },
  ];
}
