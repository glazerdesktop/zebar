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
  const [sizes, setSizes] = createSignal<number[]>([0.2, 0.8]);

  return (
    <>
      <Resizable sizes={sizes()} onSizesChange={setSizes}>
        <Sidebar />
        <ResizableHandle withHandle />
        <ResizablePanel>{props.children}</ResizablePanel>
      </Resizable>
    </>
  );
}
