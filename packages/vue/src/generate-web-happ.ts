import { findByPath, ScDirectory, ScFile } from '@source-craft/types';
import { HappDefinition } from '@holochain-scaffolding/definitions';
import { webHapp, generateTsTypesForHapp } from '@holochain-scaffolding/generators';
import { happVocabulary, renderersImports, happTsGenerators } from '@holochain-scaffolding/vocabulary';
import generateVueApp from './app';
import { addComponentsForEntryDef } from './add-components';
import { addNpmDependency } from '@source-craft/npm';

export function generateVueWebHapp(happDefinition: HappDefinition): ScDirectory {
  let vueApp = generateVueApp({
    happName: happDefinition.name,
    appContent: '<!-- TODO: put here the content of your app -->',
    appSubcomponents: '// TODO: Add here the appropriate subcomponents'
  });
  const typesDir = generateTsTypesForHapp(happDefinition);

  (vueApp.children['src'] as ScDirectory).children['types'] = typesDir;

  for (const dna of happDefinition.dnas) {
    for (const zome of dna.zomes) {
      for (const entryDef of zome.entry_defs) {
        vueApp = addComponentsForEntryDef(
          vueApp,
          happVocabulary,
          happTsGenerators,
          renderersImports,
          entryDef.typeDefinition,
          dna.name,
          zome.name,
        );
      }
    }
  }

  const packageJson = findByPath(vueApp, 'package.json') as ScFile;
  packageJson.content = addNpmDependency(packageJson, '@material/mwc-button', '^0.25.3').content;
  packageJson.content = addNpmDependency(packageJson, '@material/mwc-circular-progress', '^0.25.3').content;

  return webHapp(happDefinition, vueApp);
}
