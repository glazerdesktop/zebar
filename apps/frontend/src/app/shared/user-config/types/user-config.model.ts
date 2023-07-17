import { z } from 'zod';

import { BarConfigSchema } from './bar/bar-config.model';
import { GeneralConfigSchema } from './general-config.model';
import { addDelimitedKey } from './shared/add-delimited-key';
import { Prettify } from '~/shared/utils';

export const UserConfigSchema = z
  .object({
    general: GeneralConfigSchema,
    bar: BarConfigSchema.optional(),
  })
  .passthrough()
  .superRefine(addDelimitedKey('bar', BarConfigSchema));

export type UserConfig = Prettify<z.infer<typeof UserConfigSchema>>;
