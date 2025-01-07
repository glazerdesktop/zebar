import { createContext, type JSX, Resource, useContext } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';
import { listen, type Event } from '@tauri-apps/api/event';
import { createResource } from 'solid-js';
import type { Widget, WidgetConfig } from 'zebar';

type UserWidgetPacksContextState = {
  widgetConfigs: Resource<Record<string, WidgetConfig>>;
  widgetStates: Resource<Record<string, Widget>>;
  updateWidgetConfig: (
    configPath: string,
    newConfig: WidgetConfig,
  ) => Promise<void>;
  togglePreset: (configPath: string, presetName: string) => Promise<void>;
};

const UserWidgetPacksContext =
  createContext<UserWidgetPacksContextState>();

export function UserWidgetPacksProvider(props: { children: JSX.Element }) {
  const [widgetConfigs, { mutate: mutateWidgetConfigs }] = createResource(
    async () => invoke<Record<string, WidgetConfig>>('widget_configs'),
    { initialValue: {} },
  );

  const [widgetStates, { mutate: mutateWidgetStates }] = createResource(
    async () => invoke<Record<string, Widget>>('widget_states'),
    { initialValue: {} },
  );

  // Update widget states on open.
  listen('widget-opened', (event: Event<any>) => {
    mutateWidgetStates(states => ({
      ...states,
      [event.payload.id]: event.payload,
    }));
  });

  // Update widget states on close.
  listen('widget-closed', (event: Event<any>) => {
    mutateWidgetStates(states => {
      const newStates = { ...states };
      delete newStates[event.payload];
      return newStates;
    });
  });

  async function updateWidgetConfig(
    configPath: string,
    newConfig: WidgetConfig,
  ) {
    mutateWidgetConfigs(configs => ({
      ...configs,
      [configPath]: newConfig,
    }));

    await invoke<void>('update_widget_config', {
      configPath,
      newConfig,
    });
  }

  async function togglePreset(configPath: string, presetName: string) {
    const states = widgetStates();

    const configStates = Object.values(states).filter(
      state => state.configPath === configPath,
    );

    const presetStates = configStates.filter(
      // @ts-ignore - TODO
      state => state.openOptions?.preset === presetName,
    );

    if (presetStates.length > 0) {
      await invoke<void>('stop_preset', {
        configPath,
        presetName,
      });
    } else {
      await invoke<void>('start_preset', {
        configPath,
        presetName,
      });
    }
  }

  const store: UserWidgetPacksContextState = {
    widgetConfigs,
    widgetStates,
    updateWidgetConfig,
    togglePreset,
  };

  return (
    <UserWidgetPacksContext.Provider value={store}>
      {props.children}
    </UserWidgetPacksContext.Provider>
  );
}

export function useWidgetPacks() {
  const context = useContext(UserWidgetPacksContext);

  if (!context) {
    throw new Error(
      '`useWidgetPacks` must be used within a `UserWidgetPacksProvider`.',
    );
  }

  return context;
}
