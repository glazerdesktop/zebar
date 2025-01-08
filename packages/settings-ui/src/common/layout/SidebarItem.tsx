import {
  buttonVariants,
  cn,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@glzr/components';
import { type JSX, Show } from 'solid-js';

export interface SidebarItemProps {
  isCollapsed: boolean;
  icon: JSX.Element;
  tooltip: string;
  children: JSX.Element;
  variant: 'default' | 'ghost';
}

/**
 * Sidebar item component.
 *
 * Shows only the icon and a tooltip on hover when collapsed.
 */
export function SidebarItem(props: SidebarItemProps) {
  const ExpandedButton = () => (
    <a
      href="#"
      class={cn(
        buttonVariants({ variant: props.variant }),
        'flex justify-start min-w-0 m-2 pl-2',
      )}
    >
      <div class="mr-2">{props.icon}</div>
      {props.children}
    </a>
  );

  const CollapsedButton = () => (
    <Tooltip openDelay={0} closeDelay={0} placement="right">
      <TooltipTrigger
        as="a"
        href="#"
        class={cn(
          buttonVariants({
            variant: props.variant,
            size: 'icon',
          }),
          'flex m-2',
        )}
      >
        {props.icon}
        <span class="sr-only">{props.tooltip}</span>
      </TooltipTrigger>

      <TooltipContent>{props.tooltip}</TooltipContent>
    </Tooltip>
  );

  return (
    <Show
      when={props.isCollapsed && props.tooltip}
      fallback={<ExpandedButton />}
    >
      <CollapsedButton />
    </Show>
  );
}
