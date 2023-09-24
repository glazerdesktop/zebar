import { createMemo } from 'solid-js';

import glazewmWorkspacesTemplate from './glazewm-workspaces.template.njk?raw';
import weatherTemplate from './weather.template.njk?raw';
import { createTemplateElement } from '~/shared/template-parsing';
import { ComponentConfig } from '~/shared/user-config';

export interface BarComponentProps {
  config: ComponentConfig;
}

export function BarComponent(props: BarComponentProps) {
  const template = createMemo(() => {
    switch (props.config.template) {
      case 'template.glazewm_workspaces':
        return glazewmWorkspacesTemplate;
      case 'template.weather':
        return weatherTemplate;
      default:
        return props.config.template;
    }
  });

  return createTemplateElement({
    bindings: () => ({}),
    config: () => props.config,
    defaultTemplate: () => '',
  });
}
