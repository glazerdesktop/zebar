import { createContext, type JSX, Resource, useContext } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';
import { listen, type Event } from '@tauri-apps/api/event';
import { createResource } from 'solid-js';
import type { Widget, WidgetConfig } from 'zebar';

const installedPacksMock = [
  {
    id: 'system-monitor',
    name: 'System Monitor',
    author: 'Zebar Team',
    description: 'CPU, memory, and disk usage widgets',
    version: '1.0.0',
    widgets: [
      { id: 'cpu-usage', name: 'CPU Usage' },
      { id: 'memory-usage', name: 'Memory Usage' },
      { id: 'disk-space', name: 'Disk Space' },
    ],
  },
  {
    id: 'weather-widgets',
    name: 'Weather Pack',
    author: 'Weather Team',
    description: 'Current weather and forecast widgets',
    version: '2.1.0',
    widgets: [
      { id: 'current-weather', name: 'Current Weather' },
      { id: 'forecast', name: 'Weekly Forecast' },
    ],
  },
];

const localPacksMock = [
  {
    id: 'my-custom-widgets',
    name: 'My Custom Widgets',
    description: 'Personal collection of widgets',
    version: '0.1.0',
    widgets: [{ id: 'todo-list', name: 'Todo List' }],
  },
];

type WidgetPack = {
  id: string;
  name: string;
  description: string;
  version: string;
  widgets: { id: string; name: string }[];
};

type UserWidgetPacksContextState = {
  installedPacks: Resource<WidgetPack[]>;
  localPacks: Resource<WidgetPack[]>;
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
  // TODO: Fetch installed packs from the backend.
  const [installedPacks] = createResource(async () => installedPacksMock);

  // TODO: Fetch local packs from the backend.
  const [localPacks] = createResource(async () => localPacksMock);

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
    installedPacks,
    localPacks,
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
