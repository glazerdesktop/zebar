import type { Provider } from '../create-base-provider';

export interface FocusedWindowProviderConfig {
    type: 'focused-window';
}

export interface FocusedWindowOutput {
    title: string;
    icon: string;
}

export type FocusedWindowProvider = Provider<FocusedWindowProviderConfig, FocusedWindowOutput>;
