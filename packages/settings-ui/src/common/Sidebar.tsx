import { Button, cn, ResizablePanel, Separator } from '@glzr/components';
import { createSignal } from 'solid-js';
import { SidebarItem } from './SidebarItem';
import {
  IconBuildingStore,
  IconChevronsLeft,
} from '@tabler/icons-solidjs';

export function Sidebar() {
  const [isCollapsed, setIsCollapsed] = createSignal(false);

  return (
    <ResizablePanel
      minSize={0.1}
      maxSize={0.2}
      collapsible
      onCollapse={size => setIsCollapsed(size === 0)}
      onExpand={() => setIsCollapsed(false)}
      class={cn(
        isCollapsed() &&
          'min-w-[50px] transition-all duration-300 ease-in-out',
      )}
    >
      <div class="flex justify-between items-center">
        <img
          src="/resources/logo-128x128.png"
          alt="Zebar logo"
          class={cn(
            'w-8 h-8 m-2 ml-4 transition-all',
            isCollapsed() && 'w-5 h-5',
          )}
        />

        <Button
          onClick={() => setIsCollapsed(!isCollapsed())}
          class={cn(
            'mr-2 p-2 text-muted-foreground',
            isCollapsed() && 'hidden',
          )}
          variant="ghost"
        >
          <IconChevronsLeft class="size-4" />
        </Button>
      </div>

      <Separator />

      <SidebarItem
        isCollapsed={isCollapsed()}
        title="Community"
        label="128"
        icon={<IconBuildingStore class="size-6" />}
        variant="ghost"
      />
    </ResizablePanel>
  );
}
