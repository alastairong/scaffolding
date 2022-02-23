import { ScFile, ScNodeType } from '@source-craft/types'; 

export const envDTs = (): ScFile => ({
  type: ScNodeType.File,
  content: `/// <reference types="vite/client" />

declare module '*.vue' {
  import { DefineComponent } from 'vue'
  // eslint-disable-next-line @typescript-eslint/no-explicit-any, @typescript-eslint/ban-types
  const component: DefineComponent<{}, {}, any>
  export default component
}
`
});
    