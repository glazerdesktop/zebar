/* @refresh reload */
import './index.css';
import { HashRouter, Route } from '@solidjs/router';
import { render } from 'solid-js/web';

import { WidgetConfigs } from './configs/WidgetConfigs';
import Sidebar from './configs/Sidebar';

render(
  () => (
    <HashRouter>
      <Route path="/" component={Sidebar} />
      <Route path="/widget/:path" component={WidgetConfigs} />
    </HashRouter>
  ),
  document.getElementById('root')!,
);
