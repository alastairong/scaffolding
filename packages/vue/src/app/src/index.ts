import { ScNodeType, ScDirectory } from '@source-craft/types'; 

import { appVue } from './appVue';
import assets from './assets';
import components from './components';
import { envDTs } from './envDTs';
import { mainTs } from './mainTs';  

export default ({happName, appContent, appSubcomponents}: {happName: string; appContent: string; appSubcomponents: string;}): ScDirectory => ({
  type: ScNodeType.Directory,
  children: {
  'App.vue': appVue({happName, appContent, appSubcomponents}),
  'assets': assets(),
  'components': components(),
  'env.d.ts': envDTs(),
  'main.ts': mainTs()
  }
})