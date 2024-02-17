import {
  type Accessor,
  type Owner,
  createEffect,
  runWithOwner,
} from 'solid-js';
import { createStore } from 'solid-js/store';
import { message } from '@tauri-apps/plugin-dialog';

import {
  getStyleBuilder,
  getParsedElementConfig,
  getChildConfigs,
  type GlobalConfig,
  TemplatePropertyError,
} from './user-config';
import { getElementProviders } from './providers';
import type { ElementContext } from './element-context.model';
import { ElementType } from './element-type.model';
import { createLogger, type PickPartial } from './utils';

const logger = createLogger('init-element');

export interface InitElementArgs {
  id: string;
  type: ElementType;
  rawConfig: unknown;
  globalConfig: GlobalConfig;
  args: Record<string, string>;
  env: Record<string, string>;
  ancestorProviders: Accessor<Record<string, unknown>>[];
  owner: Owner;
}

export async function initElement(
  args: InitElementArgs,
): Promise<ElementContext> {
  try {
    const styleBuilder = getStyleBuilder(args.owner);
    const childConfigs = getChildConfigs(args.rawConfig);

    // Create partial element context; `providers` and `parsedConfig` are set later.
    const [elementContext, setElementContext] = createStore<
      PickPartial<ElementContext, 'parsedConfig' | 'providers'>
    >({
      id: args.id,
      type: args.type,
      rawConfig: args.rawConfig,
      globalConfig: args.globalConfig,
      args: args.args,
      env: args.env,
      initChildElement,
      providers: undefined,
      parsedConfig: undefined,
    });

    const { element, merged } = await getElementProviders(
      elementContext,
      args.ancestorProviders,
      args.owner,
    );

    setElementContext('providers', merged);

    const parsedConfig = getParsedElementConfig(
      elementContext as PickPartial<ElementContext, 'parsedConfig'>,
      args.owner,
    );

    // Since `parsedConfig` and `providers` are set after initializing providers
    // and parsing the element config, they are initially unavailable on 'self'
    // provider.
    setElementContext('parsedConfig', parsedConfig);

    runWithOwner(args.owner, () => {
      createEffect(() => {
        if (parsedConfig.styles) {
          styleBuilder.setElementStyles(
            parsedConfig.id,
            parsedConfig.styles,
          );
        }
      });
    });

    async function initChildElement(id: string) {
      const childConfig = childConfigs.find(
        childConfig => childConfig.id === id,
      );

      // Check whether an element with the given ID exists in the config.
      if (!childConfig) {
        return null;
      }

      return initElement({
        id,
        type: childConfig.type,
        rawConfig: childConfig.config,
        globalConfig: args.globalConfig,
        args: args.args,
        env: args.env,
        ancestorProviders: [...(args.ancestorProviders ?? []), element],
        owner: args.owner,
      });
    }

    return elementContext as ElementContext;
  } catch (err) {
    logger.error(
      'Error initializing window:',
      //@ts-ignore
      Object.keys(err),
      err instanceof TemplatePropertyError,
      err,
    );
    message((err as Error).message, {
      title: 'Encountered an error!',
      type: 'error',
    });
    throw err;
  }
}
