import { Accessor, createEffect, on, onCleanup, onMount } from 'solid-js';

import { useLogger } from '../logging';
import { mount } from '../utils/mount';
import { runTemplateEngine } from './run-template-engine';

export interface CreateTemplateElementArgs {
  id: Accessor<string>;
  className: Accessor<string>;
  variables: Accessor<Record<string, unknown>>;
  commands: Accessor<Record<string, (...args: unknown[]) => unknown>>;
  template: Accessor<string>;
  slots: Accessor<Record<string, string>>;
}

export function createTemplateElement(args: CreateTemplateElementArgs) {
  const logger = useLogger(`.${args.className()}#${args.id()}`);

  // Create element with ID.
  const element = document.createElement('div');
  element.id = args.id();

  createEffect(
    on(
      () => args.variables(),
      () => {
        console.log('ranaaa');
        // Compile template with template engine.
        const newElement = createRootElement();
        newElement.innerHTML = runTemplateEngine(
          args.template(),
          args.slots(),
          args.variables(),
        );
        console.log('newElement', newElement);

        // TODO: Is it actually necessary to use `createRoot` around the mounted
        // elemented ? Is`onCleanup` called corrrectly when it's omitted?
        const oldElement = document.getElementById(args.id());
        oldElement?.replaceWith(newElement);
        // const dispose = mount(oldElement, newElement);

        // onCleanup(() => dispose());
      },
    ),
  );

  function createRootElement() {
    const element = document.createElement('div');
    element.id = args.id();
    element.className = args.className();
    return element;
  }

  onMount(() => logger.debug('Mounted'));
  onCleanup(() => logger.debug('Cleanup'));

  return element;
}
