import type { MonitorSelection } from './monitor-selection';
import type { ReserveSpaceConfig } from './reserve-space-config';

export type WidgetPlacement = {
  anchor:
    | 'top_left'
    | 'top_center'
    | 'top_right'
    | 'center'
    | 'bottom_left'
    | 'bottom_center'
    | 'bottom_right';
  offsetX: string;
  offsetY: string;
  width: string;
  height: string;
  monitorSelection: MonitorSelection;
  reserveSpace: ReserveSpaceConfig;
};