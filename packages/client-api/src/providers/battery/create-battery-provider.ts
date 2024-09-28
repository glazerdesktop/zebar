import { z } from 'zod';

import {
  createBaseProvider,
  type Provider,
} from '../create-base-provider';
import { onProviderEmit } from '~/desktop';

export interface BatteryProviderConfig {
  type: 'battery';

  /**
   * How often this provider refreshes in milliseconds.
   */
  refreshInterval?: number;
}

const batteryProviderConfigSchema = z.object({
  type: z.literal('battery'),
  refreshInterval: z.coerce.number().default(60 * 1000),
});

export type BatteryProvider = Provider<
  BatteryProviderConfig,
  BatteryOutput
>;

export interface BatteryOutput {
  chargePercent: number;
  cycleCount: number;
  healthPercent: number;
  powerConsumption: number;
  state: 'discharging' | 'charging' | 'full' | 'empty' | 'unknown';
  isCharging: boolean;
  timeTillEmpty: number | null;
  timeTillFull: number | null;
  voltage: number | null;
}

export function createBatteryProvider(
  config: BatteryProviderConfig,
): BatteryProvider {
  const mergedConfig = batteryProviderConfigSchema.parse(config);

  return createBaseProvider(mergedConfig, async queue => {
    return onProviderEmit<BatteryOutput>(mergedConfig, ({ result }) => {
      if ('error' in result) {
        queue.error(result.error);
      } else {
        queue.output(result.output);
      }
    });
  });
}
