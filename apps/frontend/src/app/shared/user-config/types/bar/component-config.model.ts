import { z } from 'zod';

import { ClockComponentConfigSchema } from './components/clock-component-config.model';
import { CpuComponentConfigSchema } from './components/cpu-component-config.model';
import { GlazeWMComponentConfigSchema } from './components/glazewm-component-config.model';
import { WeatherComponentConfigSchema } from './components/weather-component-config.model';

export const ComponentConfigSchema = z.discriminatedUnion('type', [
  ClockComponentConfigSchema,
  CpuComponentConfigSchema,
  GlazeWMComponentConfigSchema,
  WeatherComponentConfigSchema,
]);

export type ComponentConfig = z.infer<typeof ComponentConfigSchema>;
