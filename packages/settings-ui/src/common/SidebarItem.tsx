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

export function SidebarItem(props: SidebarItemProps) {
  return (
    <div
      data-collapsed={props.isCollapsed}
      class="group flex flex-col gap-4 py-2 data-[collapsed=true]:py-2"
    >
      <nav class="grid gap-1 px-2 group-[[data-collapsed=true]]:justify-center group-[[data-collapsed=true]]:px-2">
        <Show
          when={props.isCollapsed}
          fallback={
            <a
              href="#"
              class={cn(
                buttonVariants({
                  variant: props.variant,
                  size: 'sm',
                  class: 'text-sm',
                }),
                props.variant === 'default' &&
                  'dark:bg-muted dark:text-white dark:hover:bg-muted dark:hover:text-white',
                'justify-start min-w-0',
              )}
            >
              <div class="mr-2">{props.icon}</div>
              <div class="truncate">{props.children}</div>
            </a>
          }
        >
          <Tooltip openDelay={0} closeDelay={0} placement="right">
            <TooltipTrigger
              as="a"
              href="#"
              class={cn(
                buttonVariants({
                  variant: props.variant,
                  size: 'icon',
                }),
                'h-9 w-9',
                props.variant === 'default' &&
                  'dark:bg-muted dark:text-muted-foreground dark:hover:bg-muted dark:hover:text-white',
              )}
            >
              {props.icon}
              <span class="sr-only">{props.tooltip}</span>
            </TooltipTrigger>
            <TooltipContent class="flex items-center gap-4">
              {props.tooltip}
            </TooltipContent>
          </Tooltip>
        </Show>
      </nav>
    </div>
  );
}
