import { DateTime } from 'luxon';
import { Owner, onCleanup, runWithOwner } from 'solid-js';
import { createStore } from 'solid-js/store';

import { DateProviderConfig } from '~/user-config';

export interface DateVariables {
  /**
   * Current date/time as a JavaScript `Date` object. Uses `new Date()` under
   * the hood.
   **/
  new: Date;

  /**
   * Current date/time as milliseconds since epoch. Uses `Date.now()` under the
   * hood.
   **/
  now: number;

  /**
   * Current date/time as an ISO-8601 string (eg.
   * `2017-04-22T20:47:05.335-04:00`). Uses `date.toISOString()` under the hood.
   **/
  iso: string;
}

export async function createDateProvider(
  config: DateProviderConfig,
  owner: Owner,
) {
  const [dateVariables, setDateVariables] =
    createStore<DateVariables>(getDateVariables());

  const interval = setInterval(
    () => setDateVariables(getDateVariables()),
    config.refresh_interval_ms,
  );

  runWithOwner(owner, () => {
    onCleanup(() => clearInterval(interval));
  });

  function getDateVariables() {
    const date = new Date();

    return {
      new: date,
      now: date.getTime(),
      iso: date.toISOString(),
    };
  }

  function toFormat(now: number, format: string) {
    const dateTime = DateTime.fromMillis(now);

    if (config.timezone) {
      dateTime.setZone(config.timezone);
    }

    return dateTime.toFormat(format, { locale: config.locale });
  }

  return {
    get new() {
      return dateVariables.new;
    },
    get now() {
      return dateVariables.now;
    },
    get iso() {
      return dateVariables.iso;
    },
    toFormat,
  };
}
