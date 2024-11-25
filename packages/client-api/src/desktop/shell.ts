import { Command } from '@tauri-apps/plugin-shell';

export async function shellExec(cmd: string, args?: string | string[]) {
  return Command.create(cmd, args).execute();
}
