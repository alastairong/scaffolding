import { addNpmDependency } from '@source-craft/npm';
import { findByPath, ScDirectory, ScFile, ScNodeType } from '@source-craft/types';
import { getAllImports, VocabularyElementsImports } from '@type-craft/elements-imports';
import { VocabularyTypescriptGenerators } from '@type-craft/typescript';
import { getAllChildrenTypes, TypeDefinition, Vocabulary } from '@type-craft/vocabulary';
import { camelCase, flatten, upperFirst } from 'lodash-es';

import { generateTypeDetailVueComponent } from './detail-type-component';
import { generateCreateTypeVueComponent } from './create-type-component';

const titleCase = (str: string) => upperFirst(camelCase(str));

export function addComponentsForEntryDef(
  vueApp: ScDirectory,
  vocabulary: Vocabulary,
  typescriptGenerators: VocabularyTypescriptGenerators,
  renderersImports: VocabularyElementsImports,
  type: TypeDefinition<any, any>,
  dnaName: string,
  zomeName: string,
): ScDirectory {
  const componentsDir = findByPath(vueApp, 'src/components') as ScDirectory;

  let dnaComponentsDir = findByPath(componentsDir, dnaName) as ScDirectory;

  if (!dnaComponentsDir) {
    dnaComponentsDir = {
      type: ScNodeType.Directory,
      children: {},
    };
    componentsDir.children[dnaName] = dnaComponentsDir;
  }

  let zomeComponentsDir = findByPath(dnaComponentsDir, zomeName) as ScDirectory;

  if (!zomeComponentsDir) {
    zomeComponentsDir = {
      type: ScNodeType.Directory,
      children: {},
    };
    dnaComponentsDir.children[zomeName] = zomeComponentsDir;
  }

  const createComponentFile = generateCreateTypeVueComponent(
    typescriptGenerators,
    renderersImports,
    type,
    dnaName,
    zomeName,
  );

  const detailComponentFile = generateTypeDetailVueComponent(
    typescriptGenerators,
    renderersImports,
    type,
    dnaName,
    zomeName,
  );

  zomeComponentsDir.children[`Create${titleCase(type.name)}.vue`] = createComponentFile;
  zomeComponentsDir.children[`${titleCase(type.name)}Detail.vue`] = detailComponentFile;

  const packageJson = findByPath(vueApp, 'package.json') as ScFile;

  const allTypes = getAllChildrenTypes(vocabulary, type);

  const allRenderers = allTypes.map(t => renderersImports[t]).filter(r => !!r);
  const allImports = flatten(allRenderers.map(r => getAllImports(r)));

  for (const i of allImports) {
    packageJson.content = addNpmDependency(packageJson, i.packageName, i.version).content;
  }

  return vueApp;
}
