import { z } from 'zod';

import type { Prettify } from '~/utils';
import { ProvidersConfigSchema } from './providers-config.model';
import { ElementEventsConfigSchema } from './element-events-config.model';

export const BaseElementConfigSchema = z.object({
  id: z.string(),
  class_names: z.array(z.string()).default([]),
  styles: z.string().optional(),
  providers: ProvidersConfigSchema,
  events: ElementEventsConfigSchema,
});

/** Base config for windows, groups, and components. */
export type BaseElementConfig = Prettify<
  z.infer<typeof BaseElementConfigSchema>
>;
