import {
  Resizable,
  ResizableHandle,
  ResizablePanel,
} from '@glzr/components';
import { createSignal, type JSX } from 'solid-js';
import { Sidebar } from './Sidebar';

export interface AppLayoutProps {
  children: JSX.Element;
}

export function AppLayout(props: AppLayoutProps) {
  const [sizes, setSizes] = createSignal<number[]>([]);

  return (
    <>
      <Resizable sizes={sizes()} onSizesChange={setSizes}>
        <Sidebar />
        <ResizableHandle withHandle />
        <ResizablePanel minSize={0.3}>{props.children}</ResizablePanel>
      </Resizable>
    </>
  );
}
