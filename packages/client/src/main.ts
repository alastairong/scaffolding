import { WebHappDefinitionBuilder } from '@holochain-scaffolding/elements';
import { createApp } from 'vue';
import App from './App.vue';
import '@ui5/webcomponents/dist/Tree.js';
import '@material/mwc-textfield';
import '@material/mwc-button';
import '@material/mwc-icon-button';
import '@material/mwc-fab';
import '@material/mwc-list';
import '@material/mwc-dialog';
import '@material/mwc-select';
import '@ui5/webcomponents/dist/Card.js';


customElements.define('webhapp-definition-builder', WebHappDefinitionBuilder);

createApp(App).mount('#app');
