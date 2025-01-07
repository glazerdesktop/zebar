import {
  Button,
  cn,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@glzr/components';
import { useParams } from '@solidjs/router';
import { IconChevronDown } from '@tabler/icons-solidjs';
import { invoke } from '@tauri-apps/api/core';
import { listen, type Event } from '@tauri-apps/api/event';
import {
  createEffect,
  createMemo,
  createResource,
  createSignal,
  For,
  on,
  Show,
} from 'solid-js';
import { Widget, WidgetConfig } from 'zebar';

import { WidgetConfigSidebar } from './WidgetConfigSidebar';
import { WidgetConfigForm } from './WidgetConfigForm';

export function WidgetPacks() {
  const params = useParams();

  const [configs, { mutate: mutateWidgetConfigs }] = createResource(
    async () => invoke<Record<string, WidgetConfig>>('widget_configs'),
    { initialValue: {} },
  );

  const [widgetStates, { mutate: mutateWidgetStates }] = createResource(
    async () => invoke<Record<string, Widget>>('widget_states'),
    { initialValue: {} },
  );

  const [selectedConfigPath, setSelectedConfigPath] = createSignal<
    string | null
  >(params.path ? atob(params.path) : null);

  const [selectedPreset, setSelectedPreset] = createSignal<string | null>(
    null,
  );

  const selectedConfig = createMemo(() => configs()[selectedConfigPath()]);

  const presetNames = createMemo(() =>
    (selectedConfig()?.presets ?? []).map(preset => preset.name),
  );

  // Widget states for the selected config.
  const selectedConfigStates = createMemo(() => {
    const configPath = selectedConfigPath();
    return Object.values(widgetStates()).filter(
      state => state.configPath === configPath,
    );
  });

  // Widget states for the selected preset.
  const selectedPresetStates = createMemo(() => {
    const preset = selectedPreset();
    return selectedConfigStates().filter(
      // @ts-ignore - TODO
      state => state.openOptions?.preset === preset,
    );
  });

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

  // Update selected config path when params change. This occurs when
  // "Edit" is selected from the system tray menu.
  createEffect(
    on(
      () => params.path,
      () => {
        setSelectedConfigPath(params.path ? atob(params.path) : null);
      },
    ),
  );

  // Select the first config alphabetically on initial load.
  createEffect(
    on(
      () => configs(),
      () => {
        if (!selectedConfigPath()) {
          setSelectedConfigPath(Object.keys(configs()).sort()[0] ?? null);
        }
      },
    ),
  );

  // Initialize the selected preset when a config is selected.
  createEffect(
    on(
      () => selectedConfigPath(),
      () => {
        if (selectedConfig()) {
          setSelectedPreset(selectedConfig().presets[0]?.name ?? null);
          document.querySelector('#form-container').scrollTo(0, 0);
        }
      },
    ),
  );

  // Ensure selected preset is valid when presets change.
  createEffect(
    on(
      () => presetNames(),
      () => {
        if (
          !selectedPreset() ||
          !presetNames().includes(selectedPreset())
        ) {
          setSelectedPreset(presetNames()[0] ?? null);
        }
      },
    ),
  );

  async function onConfigChange(
    configPath: string,
    newConfig: WidgetConfig,
  ) {
    // Update the state with the new config values.
    mutateWidgetConfigs(configs => ({
      ...configs,
      [configPath]: newConfig,
    }));

    // Send updated config values to backend.
    await invoke<void>('update_widget_config', {
      configPath,
      newConfig,
    });
  }

  async function togglePreset(configPath: string, presetName: string) {
    if (selectedPresetStates().length > 0) {
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

  return (
    <div class="flex h-screen bg-background">
      {/* Sidebar. */}
      <WidgetConfigSidebar
        configs={configs()}
        widgetStates={widgetStates()}
        selectedConfig={selectedConfig()}
        selectedConfigPath={selectedConfigPath()}
        onSelect={setSelectedConfigPath}
      />

      {/* Main content. */}
      <Show
        when={selectedConfig()}
        fallback={<WidgetSettingsEmptyState />}
      >
        {config => (
          <main class="flex-1 grid grid-rows-[1fr_auto] overflow-hidden">
            <div id="form-container" class="container p-4 overflow-y-auto">
              <h1 class="text-2xl font-bold mb-1">
                {selectedConfigPath().split(/[/\\]/).at(-1)}
              </h1>

              <p class="bg-muted text-xs font-mono rounded-sm mb-6 p-1 text-muted-foreground inline-block">
                {selectedConfigPath()}
              </p>

              <WidgetConfigForm
                config={config()}
                configPath={selectedConfigPath()}
                onChange={config =>
                  onConfigChange(selectedConfigPath(), config)
                }
              />
            </div>

            {/* Action bar. */}
            <div class="flex items-center justify-end border-t p-4">
              <div class="flex items-center">
                <span class="text-sm font-normal text-muted-foreground mr-2">
                  {selectedConfigStates().length} open (
                  {selectedPresetStates().length} for preset)
                </span>

                <Button
                  class="rounded-r-none self-end"
                  disabled={presetNames().length === 0}
                  onClick={() =>
                    togglePreset(selectedConfigPath(), selectedPreset())
                  }
                >
                  <Show when={selectedPreset()} fallback="No presets">
                    {selectedPresetStates().length === 0
                      ? `Open ${selectedPreset()}`
                      : `Close ${selectedPreset()}`}
                  </Show>
                </Button>

                <DropdownMenu>
                  <DropdownMenuTrigger>
                    <Button
                      class="rounded-l-none border-l-0 px-2"
                      disabled={presetNames().length === 0}
                    >
                      <IconChevronDown class="size-3" />
                    </Button>
                  </DropdownMenuTrigger>

                  <DropdownMenuContent>
                    <For each={presetNames()}>
                      {presetName => (
                        <DropdownMenuItem
                          onClick={() => setSelectedPreset(presetName)}
                          class={cn({
                            'bg-accent text-accent-foreground':
                              presetName === selectedPreset(),
                          })}
                        >
                          {presetName}
                        </DropdownMenuItem>
                      )}
                    </For>
                  </DropdownMenuContent>
                </DropdownMenu>
              </div>
            </div>
          </main>
        )}
      </Show>
    </div>
  );
}

function WidgetSettingsEmptyState() {
  return <p class="p-4">No config selected.</p>;
}
