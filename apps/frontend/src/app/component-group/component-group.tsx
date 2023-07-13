import defaultTemplate from './component-group.njk?raw';
import { ClockComponent } from '~/components/clock/clock-component';
import { createTemplateElement } from '~/shared/template-parsing';
import { ComponentConfig, ComponentGroupConfig } from '~/shared/user-config';

export function ComponentGroup(props: { config: ComponentGroupConfig }) {
  function getComponentType(componentConfig: ComponentConfig) {
    switch (componentConfig.type) {
      case 'clock':
        return <ClockComponent config={componentConfig} />;
      case 'cpu':
        return <p>Not implemented.</p>;
      case 'glazewm':
        return <p>Not implemented.</p>;
    }
  }

  function getBindings() {
    return {
      strings: {
        root_props: `id="${props.config.id}" class="${props.config.class_name}"`,
      },
      components: {
        components: () => props.config.components.map(getComponentType),
      },
    };
  }
  return createTemplateElement({
    bindings: getBindings,
    config: () => props.config,
    defaultTemplate: () => defaultTemplate,
  });
}
