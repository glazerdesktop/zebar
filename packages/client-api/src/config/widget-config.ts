import type { WidgetPreset } from './widget-preset';
import type { WidgetPrivileges } from './widget-privileges';

export type WidgetConfig = {
  htmlPath: string;
  zOrder: 'normal' | 'top_most' | 'bottom_most';
  shownInTaskbar: boolean;
  focused: boolean;
  resizable: boolean;
  transparent: boolean;
  privileges: WidgetPrivileges;
  presets: WidgetPreset[];
};
