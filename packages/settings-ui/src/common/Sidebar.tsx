import { cn, ResizablePanel, Separator } from '@glzr/components';
import { createSignal } from 'solid-js';
import { Nav } from './Nav';

export function Sidebar() {
  const [isCollapsed, setIsCollapsed] = createSignal(false);

  return (
    <ResizablePanel
      minSize={0.1}
      maxSize={0.2}
      collapsible
      onCollapse={e => setIsCollapsed(e === 0)}
      onExpand={() => setIsCollapsed(false)}
      class={cn(
        isCollapsed() &&
          'min-w-[50px] transition-all duration-300 ease-in-out',
      )}
    >
      <img
        src="/resources/logo-128x128.png"
        alt="Zebar logo"
        class="w-8 h-8 m-2"
      />
      <Nav
        isCollapsed={isCollapsed()}
        links={[
          {
            title: 'Inbox',
            label: '128',
            icon: (
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="size-4"
                viewBox="0 0 24 24"
              >
                <g
                  fill="none"
                  stroke="currentColor"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                >
                  <path d="M4 6a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2z" />
                  <path d="M4 13h3l3 3h4l3-3h3" />
                </g>
                <title>Inbox</title>
              </svg>
            ),
            variant: 'default',
          },
          {
            title: 'Drafts',
            label: '9',
            icon: (
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="size-4"
                viewBox="0 0 24 24"
              >
                <g
                  fill="none"
                  stroke="currentColor"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                >
                  <path d="M14 3v4a1 1 0 0 0 1 1h4" />
                  <path d="M17 21H7a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h7l5 5v11a2 2 0 0 1-2 2" />
                </g>
                <title>Drafts</title>
              </svg>
            ),
            variant: 'ghost',
          },
          {
            title: 'Sent',
            label: '',
            icon: (
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="size-4"
                viewBox="0 0 24 24"
              >
                <path
                  fill="none"
                  stroke="currentColor"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M10 14L21 3m0 0l-6.5 18a.55.55 0 0 1-1 0L10 14l-7-3.5a.55.55 0 0 1 0-1z"
                />
                <title>Sent</title>
              </svg>
            ),
            variant: 'ghost',
          },
          {
            title: 'Trash',
            label: '23',
            icon: (
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="size-4"
                viewBox="0 0 24 24"
              >
                <path
                  fill="none"
                  stroke="currentColor"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M4 7h16m-10 4v6m4-6v6M5 7l1 12a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2l1-12M9 7V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v3"
                />
                <title>Trash</title>
              </svg>
            ),
            variant: 'ghost',
          },
          {
            title: 'Archive',
            label: '',
            icon: (
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="size-4"
                viewBox="0 0 24 24"
              >
                <path
                  fill="none"
                  stroke="currentColor"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M3 6a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v0a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2m2 2v10a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8m-9 4h4"
                />
                <title>Archive</title>
              </svg>
            ),
            variant: 'ghost',
          },
        ]}
      />
      <Separator />
      <Nav
        isCollapsed={isCollapsed()}
        links={[
          {
            title: 'Social',
            label: '972',
            icon: (
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="size-4"
                viewBox="0 0 24 24"
              >
                <path
                  fill="none"
                  stroke="currentColor"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M5 7a4 4 0 1 0 8 0a4 4 0 1 0-8 0M3 21v-2a4 4 0 0 1 4-4h4a4 4 0 0 1 4 4v2m1-17.87a4 4 0 0 1 0 7.75M21 21v-2a4 4 0 0 0-3-3.85"
                />
                <title>Social</title>
              </svg>
            ),
            variant: 'ghost',
          },
          {
            title: 'Updates',
            label: '342',
            icon: (
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="size-4"
                viewBox="0 0 24 24"
              >
                <path
                  fill="none"
                  stroke="currentColor"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M20 11A8.1 8.1 0 0 0 4.5 9M4 5v4h4m-4 4a8.1 8.1 0 0 0 15.5 2m.5 4v-4h-4m-4-6v3m0 3h.01"
                />
                <title>Updates</title>
              </svg>
            ),
            variant: 'ghost',
          },
          {
            title: 'Forums',
            label: '128',
            icon: (
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="size-4"
                viewBox="0 0 24 24"
              >
                <path
                  fill="none"
                  stroke="currentColor"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="m21 14l-3-3h-7a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1h9a1 1 0 0 1 1 1zm-7 1v2a1 1 0 0 1-1 1H6l-3 3V11a1 1 0 0 1 1-1h2"
                />
                <title>Forums</title>
              </svg>
            ),
            variant: 'ghost',
          },
          {
            title: 'Shopping',
            label: '8',
            icon: (
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="size-4"
                viewBox="0 0 24 24"
              >
                <g
                  fill="none"
                  stroke="currentColor"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                >
                  <path d="M4 19a2 2 0 1 0 4 0a2 2 0 1 0-4 0m11 0a2 2 0 1 0 4 0a2 2 0 1 0-4 0" />
                  <path d="M17 17H6V3H4" />
                  <path d="m6 5l14 1l-1 7H6" />
                </g>
                <title>Shopping</title>
              </svg>
            ),
            variant: 'ghost',
          },
          {
            title: 'Promotions',
            label: '21',
            icon: (
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="size-4"
                viewBox="0 0 24 24"
              >
                <path
                  fill="none"
                  stroke="currentColor"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M3 6a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v0a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2m2 2v10a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8m-9 4h4"
                />
                <title>Promotions</title>
              </svg>
            ),
            variant: 'ghost',
          },
        ]}
      />
    </ResizablePanel>
  );
}
