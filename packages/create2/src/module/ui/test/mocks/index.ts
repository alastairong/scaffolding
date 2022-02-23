import { ScNodeType, ScDirectory } from '@source-craft/types'; 

import { indexJs } from './indexJs';
import { moduleNamePluralMockJs } from './moduleNamePluralMockJs';  

export default ({moduleNamePluralTitleCase, moduleNamePlural, moduleNameSnakeCase, moduleName}: {moduleNamePluralTitleCase: string; moduleNamePlural: string; moduleNameSnakeCase: string; moduleName: string;}): ScDirectory => ({
  type: ScNodeType.Directory,
  children: {
  'index.js': indexJs({moduleNamePluralTitleCase, moduleNamePlural}),
  [`${moduleNamePlural}.mock.js`]: moduleNamePluralMockJs({moduleNameSnakeCase, moduleNamePluralTitleCase, moduleName})
  }
})