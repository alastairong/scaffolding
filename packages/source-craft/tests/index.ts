import '@lit-labs/ssr/lib/render-with-global-dom-shim.js';

import test from 'tape';
import path from 'path';
import { applyPatch } from '@source-craft/fs';
import { webHapp, WebFramework } from '../dist';
import { holochainEntryTypeDefinition } from '@holochain-scaffolding/definitions';
import { dateType } from '@type-craft/date';

import { fileURLToPath } from 'url';

// @ts-ignore
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

test('generate a full blown happ', async t => {
  const happChanges = webHapp(
    {
      name: 'haha',
      dnas: [
        {
          name: 'hehe',
          zomes: [
            {
              name: 'hihi',
              entry_defs: [
                {
                  create: true,
                  update: true,
                  delete: false,
                  read: true,
                  typeDefinition: holochainEntryTypeDefinition('sample_entry', [
                    {
                      name: 'createdAt',
                      type: dateType,
                      configuration: {},
                    },
                  ]),
                },
                {
                  create: true,
                  update: false,
                  delete: false,
                  read: true,
                  typeDefinition: holochainEntryTypeDefinition('sample_entry2', [
                    {
                      name: 'createdAt',
                      type: dateType,
                      configuration: {},
                    },
                  ]),
                },
              ],
            },
            {
              name: 'hihi2',
              entry_defs: [
                {
                  create: true,
                  update: false,
                  delete: true,
                  read: false,
                  typeDefinition: holochainEntryTypeDefinition('sample_entry3', [
                    {
                      name: 'createdAt',
                      type: dateType,
                      configuration: {},
                    },
                  ]),
                },
              ],
            },
          ],
        },
        {
          name: 'hehe2',
          zomes: [
            {
              name: 'hihi',
              entry_defs: [
                {
                  create: true,
                  update: false,
                  delete: false,
                  read: true,
                  typeDefinition: holochainEntryTypeDefinition('sample_entry', [
                    {
                      name: 'createdAt',
                      type: dateType,
                      configuration: {},
                    },
                  ]),
                },
              ],
            },
          ],
        },
      ],
    },
    WebFramework.Vue,
  );

  applyPatch(__dirname + '/.fixture', happChanges);

  t.equal(1, 1);
  t.end();
});
