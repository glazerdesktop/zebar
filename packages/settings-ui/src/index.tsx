/* @refresh reload */
import './index.css';
import { HashRouter, Route } from '@solidjs/router';
import { render } from 'solid-js/web';

import { WidgetPacks } from './configs/WidgetPacks';
import Sidebar from './configs/Sidebar';

render(
  () => (
    <HashRouter>
      <Route path="/" component={Sidebar} />
      <Route path="/widget/:path" component={WidgetPacks} />
    </HashRouter>
  ),
  document.getElementById('root')!,
);
