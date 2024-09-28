import { z } from 'zod';

import {
  createBaseProvider,
  type Provider,
} from '../create-base-provider';
import { onProviderEmit } from '~/desktop';

export interface CpuProviderConfig {
  type: 'cpu';

  /**
   * How often this provider refreshes in milliseconds.
   */
  refreshInterval?: number;
}

const cpuProviderConfigSchema = z.object({
  type: z.literal('cpu'),
  refreshInterval: z.coerce.number().default(5 * 1000),
});

export type CpuProvider = Provider<CpuProviderConfig, CpuOutput>;

export interface CpuOutput {
  frequency: number;
  usage: number;
  logicalCoreCount: number;
  physicalCoreCount: number;
  vendor: string;
}

export function createCpuProvider(config: CpuProviderConfig): CpuProvider {
  const mergedConfig = cpuProviderConfigSchema.parse(config);

  return createBaseProvider(mergedConfig, async queue => {
    return onProviderEmit<CpuOutput>(mergedConfig, ({ result }) => {
      if ('error' in result) {
        queue.error(result.error);
      } else {
        queue.output(result.output);
      }
    });
  });
}
