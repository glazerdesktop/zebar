/* @refresh reload */
import './index.css';
import { HashRouter, Route } from '@solidjs/router';
import { render } from 'solid-js/web';

import { AppLayout, UserWidgetPacksProvider } from './common';
import { WidgetPacks } from './configs';

render(
  () => (
    <UserWidgetPacksProvider>
      <AppLayout>
        <HashRouter>
          <Route path="/" component={WidgetPacks} />
          <Route path="/widget/:path" component={WidgetPacks} />
        </HashRouter>
      </AppLayout>
    </UserWidgetPacksProvider>
  ),
  document.getElementById('root')!,
);
