import { PatcherFile, PatcherNodeType } from '@patcher/types';
import camelCase from 'lodash-es/camelCase';
import kebabCase from 'lodash-es/kebabCase';
import upperFirst from 'lodash-es/upperFirst';
import snakeCase from 'lodash-es/snakeCase';

export const listKebabsTs = ({moduleNamePluralTitleCase, _kebab}: {moduleNamePluralTitleCase: string; _kebab: string;}): PatcherFile => ({
  type: PatcherNodeType.File,
  content: `import { customElement } from 'lit/decorators.js';
import { List${moduleNamePluralTitleCase} } from '../elements/list${_kebab}s';

@customElement('list${_kebab}s')
class LP extends List${moduleNamePluralTitleCase} {}
`
});
    