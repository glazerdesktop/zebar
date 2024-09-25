import { getCurrentWindow } from '@tauri-apps/api/window';
import { join } from '@tauri-apps/api/path';

import {
  getWindowState,
  openWindow,
  setWindowZOrder,
  showErrorDialog,
} from '~/desktop';
import { createLogger } from '~/utils';
import { createProvider, createProviderGroup } from '~/providers';
import type { ZebarContext } from './zebar-context.model';

const logger = createLogger('init-window');

/**
 * Handles initialization.
 */
export async function init(): Promise<ZebarContext> {
  try {
    const currentWindow = getCurrentWindow();

    const windowState =
      window.__ZEBAR_INITIAL_STATE ??
      (await getWindowState(currentWindow.label));

    // @ts-ignore - TODO
    return {
      launchInstance: async (configPath: string) => {
        // Ensure the config path ends with '.zebar.json'.
        const filePath = configPath.endsWith('.zebar.json')
          ? configPath
          : `${configPath}.zebar.json`;

        const absolutePath = await join(
          windowState.configPath,
          '../',
          filePath,
        );

        return openWindow(absolutePath);
      },
      createProvider,
      createProviderGroup,
      currentInstance: {
        ...windowState,
        tauri: currentWindow,
        setZOrder: zOrder => {
          return setWindowZOrder(currentWindow, zOrder);
        },
      },
      allWindows: [],
      currentMonitor: {},
      allMonitors: [],
    } as ZebarContext;
  } catch (err) {
    logger.error('Failed to initialize window:', err);

    await showErrorDialog({
      title: 'Failed to initialize window',
      error: err,
    });

    // Error during window initialization is unrecoverable, so we close
    // the window.
    getCurrentWindow().close();
    throw err;
  }
}
